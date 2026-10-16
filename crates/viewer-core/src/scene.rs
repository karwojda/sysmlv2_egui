//! A drawn Diagram as renderer-neutral primitives. Renderers (egui, SVG)
//! paint these items in order and add no notation of their own, so every
//! renderer produces the same picture.

use std::collections::BTreeMap;

use crate::model::QualifiedName;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn center_x(&self) -> f32 {
        self.x + self.width / 2.0
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.x <= self.right() && point.y >= self.y && point.y <= self.bottom()
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    None,
    Background,
    /// The stroke color, for solid markers such as a composition diamond.
    Stroke,
    /// A color a Style gave.
    Color(Rgb),
}

/// An sRGB color, written `#rrggbb`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// Reads `#rrggbb` (either case).
    pub fn parse(text: &str) -> Option<Self> {
        let hex = text.strip_prefix('#').filter(|h| h.len() == 6 && h.is_ascii())?;
        let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        Some(Rgb(channel(0)?, channel(2)?, channel(4)?))
    }

    /// Whether black text on it would be harder to read than white: its
    /// relative luminance is below the point where both contrast equally.
    pub fn is_dark(self) -> bool {
        let linear = |c: u8| {
            let c = c as f32 / 255.0;
            if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
        };
        let luminance = 0.2126 * linear(self.0) + 0.7152 * linear(self.1) + 0.0722 * linear(self.2);
        luminance < 0.179
    }
}

impl std::fmt::Display for Rgb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

impl serde::Serialize for Rgb {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Rgb {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Rgb::parse(&text).ok_or_else(|| serde::de::Error::custom(format!("`{text}` is not a #rrggbb color")))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Font {
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Rect {
        rect: Rect,
        fill: Fill,
        /// Zero for square corners (definitions); rounded for usages.
        corner_radius: f32,
        /// A dashed outline, for reference parts.
        dashed: bool,
    },
    Polyline {
        points: Vec<Point>,
        dashed: bool,
    },
    Polygon {
        points: Vec<Point>,
        fill: Fill,
    },
    Circle {
        center: Point,
        radius: f32,
        fill: Fill,
    },
    /// `position` is the text baseline at the anchor.
    Text {
        position: Point,
        text: String,
        anchor: TextAnchor,
        font: Font,
        /// Normally [`Fill::Stroke`]; [`Fill::Background`] inside a dark
        /// Style fill.
        fill: Fill,
    },
}

impl Shape {
    /// The area the shape covers. Text is estimated from its length, as
    /// the layout does, so it doesn't depend on installed fonts.
    pub fn extent(&self) -> Rect {
        crate::notation::extent(self)
    }
}

/// What a scene item depicts, for hit-testing and selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Depicts {
    Element(QualifiedName),
    Specialization {
        specific: QualifiedName,
        general: QualifiedName,
    },
    /// A usage defined by a definition (`:`).
    Typing {
        usage: QualifiedName,
        definition: QualifiedName,
    },
    /// A part usage owned by `owner`, drawn to the definition typing it.
    Composition {
        owner: QualifiedName,
        usage: QualifiedName,
    },
    /// A connection or interface usage, by its qualified name.
    Connection(QualifiedName),
    /// A part in an Interconnection Diagram: `key` is its feature path from
    /// the root (`rearAxleAssembly.differential`), which keys its Layout
    /// entries; `element` is the part usage. Two usages of one definition
    /// can draw the same inner usage, so the element alone isn't unique.
    Part { key: QualifiedName, element: QualifiedName },
    /// A port drawn on a box's border in an Interconnection Diagram: the
    /// box's element (the part usage, or the root for the Frame), the port,
    /// and `key`, the port's feature path from the root (`engine.fuelIn`,
    /// or `power` on the Frame), which keys its Layout entry.
    Port { part: QualifiedName, port: QualifiedName, key: String },
}

impl Depicts {
    /// A stable key for an edge, under which a Layout stores its route.
    /// Nodes have none: they are keyed by their element.
    pub fn edge_key(&self) -> Option<String> {
        match self {
            Depicts::Element(_) | Depicts::Part { .. } | Depicts::Port { .. } => None,
            Depicts::Specialization { specific, general } => Some(format!("specialization {specific} {general}")),
            Depicts::Typing { usage, definition } => Some(format!("defined-by {usage} {definition}")),
            Depicts::Composition { owner, usage } => Some(format!("composition {owner} {usage}")),
            Depicts::Connection(connection) => Some(format!("connection {connection}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub depicts: Depicts,
    pub shape: Shape,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    /// Top-left of the page; negative when something is drawn left of or
    /// above the nodes, such as a label beside an edge.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Painted in order; later items are on top.
    pub items: Vec<Item>,
    /// For nodes positioned inside another box (Interconnection Diagrams),
    /// that box's top-left corner, by node key: their Layout positions are
    /// relative to it. Nodes not listed are positioned on the page.
    pub origins: BTreeMap<QualifiedName, Point>,
}

impl Depicts {
    /// The key a node is drawn and laid out under: its element in a General
    /// Diagram, its feature path for a part in an Interconnection Diagram.
    pub fn node_key(&self) -> Option<&QualifiedName> {
        match self {
            Depicts::Element(e) => Some(e),
            Depicts::Part { key, .. } => Some(key),
            _ => None,
        }
    }
}

impl Scene {
    /// The rect of the node drawn under `key` (see [`Depicts::node_key`]).
    pub fn node_bounds(&self, key: &QualifiedName) -> Option<Rect> {
        self.items.iter().find_map(|item| match (item.depicts.node_key(), &item.shape) {
            (Some(k), Shape::Rect { rect, .. }) if k == key => Some(*rect),
            _ => None,
        })
    }

    /// The rect of the first node drawn for `element`, whatever its key.
    pub fn element_bounds(&self, element: &QualifiedName) -> Option<Rect> {
        self.items.iter().find_map(|item| match (&item.depicts, &item.shape) {
            (Depicts::Element(e) | Depicts::Part { element: e, .. }, Shape::Rect { rect, .. }) if e == element => {
                Some(*rect)
            }
            _ => None,
        })
    }

    /// The element of the node drawn under `key`.
    pub fn element_of(&self, key: &QualifiedName) -> Option<&QualifiedName> {
        self.items.iter().find_map(|item| match &item.depicts {
            Depicts::Element(e) if e == key => Some(e),
            Depicts::Part { key: k, element } if k == key => Some(element),
            _ => None,
        })
    }

    /// The key of the node that is topmost at `point`, if any.
    pub fn element_at(&self, point: Point) -> Option<&QualifiedName> {
        self.items.iter().rev().find_map(|item| match (item.depicts.node_key(), &item.shape) {
            (Some(key), Shape::Rect { rect, .. }) if rect.contains(point) => Some(key),
            _ => None,
        })
    }

    /// The port whose square is topmost at `point`, as (box element, port,
    /// port key).
    pub fn port_at(&self, point: Point) -> Option<(&QualifiedName, &QualifiedName, &str)> {
        self.items.iter().rev().find_map(|item| match (&item.depicts, &item.shape) {
            (Depicts::Port { part, port, key }, Shape::Rect { rect, .. }) if rect.contains(point) => {
                Some((part, port, key.as_str()))
            }
            _ => None,
        })
    }

    /// The key of every node in the Diagram, in paint order.
    pub fn elements(&self) -> Vec<&QualifiedName> {
        let mut keys = Vec::new();
        for item in &self.items {
            if let Some(key) = item.depicts.node_key()
                && !keys.contains(&key)
            {
                keys.push(key);
            }
        }
        keys
    }

    /// Every edge with the path of its line, in paint order. A path's inner
    /// points are the edge's bends.
    pub fn edges(&self) -> Vec<(&Depicts, &[Point])> {
        let mut edges: Vec<(&Depicts, &[Point])> = Vec::new();
        for item in &self.items {
            if let (Some(_), Shape::Polyline { points, .. }) = (item.depicts.edge_key(), &item.shape)
                && !edges.iter().any(|(d, _)| *d == &item.depicts)
            {
                edges.push((&item.depicts, points));
            }
        }
        edges
    }

    /// The items drawn for `depicts`, in paint order.
    pub fn items_depicting<'a>(&'a self, depicts: &'a Depicts) -> impl Iterator<Item = &'a Shape> {
        self.items
            .iter()
            .filter(move |item| &item.depicts == depicts)
            .map(|item| &item.shape)
    }
}
