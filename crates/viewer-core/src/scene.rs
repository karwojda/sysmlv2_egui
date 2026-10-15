//! A drawn Diagram as renderer-neutral primitives. Renderers (egui, SVG)
//! paint these items in order and add no notation of their own, so every
//! renderer produces the same picture.

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
    },
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
}

impl Depicts {
    /// A stable key for an edge, under which a Layout stores its route.
    /// Nodes have none: they are keyed by their element.
    pub fn edge_key(&self) -> Option<String> {
        match self {
            Depicts::Element(_) => None,
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
}

impl Scene {
    /// The frame of the node drawn for `element`, if it is in the Diagram.
    pub fn node_bounds(&self, element: &QualifiedName) -> Option<Rect> {
        self.items.iter().find_map(|item| match (&item.depicts, &item.shape) {
            (Depicts::Element(e), Shape::Rect { rect, .. }) if e == element => Some(*rect),
            _ => None,
        })
    }

    /// The element whose node is topmost at `point`, if any.
    pub fn element_at(&self, point: Point) -> Option<&QualifiedName> {
        self.items.iter().rev().find_map(|item| match (&item.depicts, &item.shape) {
            (Depicts::Element(e), Shape::Rect { rect, .. }) if rect.contains(point) => Some(e),
            _ => None,
        })
    }

    /// Every element that has a node in the Diagram, in paint order.
    pub fn elements(&self) -> Vec<&QualifiedName> {
        let mut elements = Vec::new();
        for item in &self.items {
            if let Depicts::Element(e) = &item.depicts
                && !elements.contains(&e)
            {
                elements.push(e);
            }
        }
        elements
    }

    /// The items drawn for `depicts`, in paint order.
    pub fn items_depicting<'a>(&'a self, depicts: &'a Depicts) -> impl Iterator<Item = &'a Shape> {
        self.items
            .iter()
            .filter(move |item| &item.depicts == depicts)
            .map(|item| &item.shape)
    }
}
