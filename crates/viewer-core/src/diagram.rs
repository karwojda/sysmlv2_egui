//! Building a Diagram's scene from the Model.

use std::collections::BTreeMap;

use crate::diagram_file::Layout;
use crate::layout;
use crate::model::{Element, ElementKind, Model, QualifiedName};
use crate::notation;
use crate::scene::{Depicts, Item, Point, Rect, Scene};

pub(crate) const MARGIN: f32 = 20.0;
/// Space kept around anything drawn past the nodes' margin.
const EDGE_PADDING: f32 = 4.0;
/// Space between saved nodes and newly placed ones below them.
const NEW_NODES_GAP: f32 = 60.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagramKind {
    General,
    Interconnection,
}

impl DiagramKind {
    /// How the kind reads in a sentence, e.g. "an Interconnection Diagram".
    pub fn description(self) -> &'static str {
        match self {
            DiagramKind::General => {
                "a General Diagram (it must be a package, definition or requirement)"
            }
            DiagramKind::Interconnection => {
                "an Interconnection Diagram (it must be a part or part def)"
            }
        }
    }

    /// Whether a Diagram of this kind can be rooted at `element`: a General
    /// Diagram shows the members of a package or definition.
    pub fn accepts_root(self, element: &Element) -> bool {
        match self {
            DiagramKind::General => {
                element.kind == ElementKind::Package
                    || element.kind.is_definition()
                    || element.kind == ElementKind::RequirementUsage
            }
            DiagramKind::Interconnection => {
                element.kind == ElementKind::PartDefinition
                    || element.kind == ElementKind::PartUsage
            }
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DiagramError {
    #[error("root element `{0}` does not exist in the Model")]
    RootNotFound(QualifiedName),
    #[error("`{root}` can't be the root of {kind}")]
    UnsuitableRoot {
        root: QualifiedName,
        kind: &'static str,
    },
    #[error(transparent)]
    View(#[from] crate::view::ViewError),
}

/// Where a Diagram's content comes from: a Diagram Kind and root element
/// the user chose (a Tool-defined Diagram), or a `view` in the Model (a
/// Model-defined Diagram), whose kind and elements are read from the Model
/// each time it is drawn.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum DiagramSource {
    Tool {
        kind: DiagramKind,
        root: QualifiedName,
    },
    View {
        view: QualifiedName,
    },
}

impl DiagramSource {
    /// Draws the Diagram with the user's `layout`.
    pub fn build_scene(&self, model: &Model, layout: &Layout) -> Result<Scene, DiagramError> {
        match self {
            DiagramSource::Tool { kind, root } => {
                build_scene_with_layout(model, *kind, root, layout)
            }
            DiagramSource::View { view } => Ok(crate::view::build_scene(model, view, layout)?),
        }
    }

    /// Whether a Layout key still names something in the Model; see
    /// [`layout_key_exists`].
    pub fn layout_key_exists(&self, model: &Model, key: &QualifiedName) -> bool {
        match self {
            DiagramSource::Tool { kind, root } => layout_key_exists(model, *kind, root, key),
            DiagramSource::View { view } => crate::view::layout_key_exists(model, view, key),
        }
    }

    /// The Diagram Kind it is drawn as, if it can be drawn at all.
    pub fn kind(&self, model: &Model) -> Option<DiagramKind> {
        match self {
            DiagramSource::Tool { kind, .. } => Some(*kind),
            DiagramSource::View { view } => crate::view::kind(model, view).ok(),
        }
    }

    /// The element whose name a Diagram drawn from it goes by: the root, or
    /// the view.
    pub fn named_after(&self) -> &QualifiedName {
        match self {
            DiagramSource::Tool { root, .. } => root,
            DiagramSource::View { view } => view,
        }
    }
}

/// Whether a Layout key of a Diagram of `kind` rooted at `root` still names
/// something in the Model: an element (General), or a feature path from the
/// root (Interconnection).
pub fn layout_key_exists(
    model: &Model,
    kind: DiagramKind,
    root: &QualifiedName,
    key: &QualifiedName,
) -> bool {
    match kind {
        DiagramKind::General => model.element(key).is_some(),
        DiagramKind::Interconnection => {
            crate::interconnection::path_exists(model, root, key.as_str())
        }
    }
}

/// Builds the scene for a Diagram of `kind` rooted at `root`, arranged
/// automatically.
pub fn build_scene(
    model: &Model,
    kind: DiagramKind,
    root: &QualifiedName,
) -> Result<Scene, DiagramError> {
    build_scene_with_layout(model, kind, root, &Layout::default())
}

/// Builds the scene for a Diagram with the user's `layout` applied: saved
/// nodes keep their place and size, the rest are arranged automatically
/// below them, hidden elements are left out and collapsed ones drawn
/// without compartments. Entries for elements not in the Diagram are
/// ignored.
pub fn build_scene_with_layout(
    model: &Model,
    kind: DiagramKind,
    root: &QualifiedName,
    layout: &Layout,
) -> Result<Scene, DiagramError> {
    let Some(element) = model.element(root) else {
        return Err(DiagramError::RootNotFound(root.clone()));
    };
    if !kind.accepts_root(element) {
        return Err(DiagramError::UnsuitableRoot {
            root: root.clone(),
            kind: kind.description(),
        });
    }
    match kind {
        DiagramKind::General => {
            // A requirement is drawn as the top of its tree, its nested
            // requirements below it; its subject is on its header already.
            let itself = (element.kind == ElementKind::RequirementUsage).then_some(element);
            let nodes: Vec<&Element> = itself
                .into_iter()
                .chain(model.owned_members(root).filter(|e| is_node_in_general(e)))
                .filter(|e| element.subject.as_ref() != Some(&e.qualified_name))
                .collect();
            let connections = model
                .owned_members(root)
                .filter(|e| e.kind.is_connection())
                .collect();
            Ok(general(model, nodes, connections, layout))
        }
        DiagramKind::Interconnection => crate::interconnection::build(model, root, layout),
    }
}

/// Definitions, and part, attribute and port usages, are nodes. Connections
/// are edges; Unsupported Elements are never drawn.
pub(crate) fn is_node_in_general(element: &Element) -> bool {
    element.kind.is_definition()
        || matches!(
            element.kind,
            ElementKind::PartUsage
                | ElementKind::AttributeUsage
                | ElementKind::PortUsage
                | ElementKind::RequirementUsage
                | ElementKind::ConstraintUsage
                | ElementKind::AnalysisUsage
        )
}

/// Builds a General Diagram of exactly `elements` (those it can draw as
/// nodes), as a Model-defined Diagram does, with the user's `layout`.
/// Connections owned by the nodes or their owners are drawn between them.
pub fn build_general_scene(model: &Model, elements: &[QualifiedName], layout: &Layout) -> Scene {
    let nodes: Vec<&Element> = elements
        .iter()
        .filter_map(|name| model.element(name))
        .filter(|e| is_node_in_general(e))
        .collect();
    let owners: std::collections::BTreeSet<&QualifiedName> = nodes
        .iter()
        .flat_map(|e| std::iter::once(&e.qualified_name).chain(&e.owner))
        .collect();
    let connections = owners
        .into_iter()
        .flat_map(|owner| model.owned_members(owner))
        .filter(|e| e.kind.is_connection())
        .collect();
    general(model, nodes, connections, layout)
}

enum Edge<'a> {
    Specialization {
        specific: usize,
        general: usize,
    },
    Typing {
        usage: usize,
        definition: usize,
    },
    Composition {
        owner: usize,
        part_type: usize,
        usage: &'a Element,
    },
    /// A usage drawn below the usage or definition that owns it, as in a
    /// parts tree.
    Ownership {
        owner: usize,
        usage: usize,
    },
    Connection {
        a: usize,
        b: usize,
        connection: &'a Element,
    },
    /// An analysis, to the part its subject is bound to.
    Subject {
        analysis: usize,
        part: usize,
    },
    /// A Satisfaction Claim, from the satisfier to the requirement.
    Satisfy {
        satisfier: usize,
        requirement: usize,
        claim: &'a Element,
    },
}

/// A General Diagram of `nodes`, each listing in compartments the features
/// that aren't nodes themselves, with specialization, typing, composition
/// and `connections` edges between them.
fn general<'a>(
    model: &'a Model,
    nodes: Vec<&'a Element>,
    connections: Vec<&'a Element>,
    saved: &Layout,
) -> Scene {
    let mut nodes: Vec<&Element> = nodes
        .into_iter()
        .filter(|e| !saved.hidden.contains(&e.qualified_name))
        .collect();
    nodes.sort_by(|a, b| a.model_order().cmp(&b.model_order()));
    let index_of = |name: &QualifiedName| nodes.iter().position(|e| &e.qualified_name == name);

    let mut edges = Vec::new();
    for (i, element) in nodes.iter().enumerate() {
        for general in &element.specializes {
            if let Some(general) = index_of(general) {
                edges.push(Edge::Specialization {
                    specific: i,
                    general,
                });
            }
        }
        if let Some(definition) = element.typed_by.as_ref().and_then(index_of) {
            edges.push(Edge::Typing {
                usage: i,
                definition,
            });
        }
        if let Some(owner) = element.owner.as_ref().and_then(index_of) {
            edges.push(Edge::Ownership { owner, usage: i });
        }
        for usage in model
            .owned_members(&element.qualified_name)
            // A requirement's subject is what it constrains, not a part of it.
            .filter(|m| {
                m.kind == ElementKind::PartUsage
                    && element.subject.as_ref() != Some(&m.qualified_name)
            })
        {
            if let Some(part_type) = usage
                .typed_by
                .as_ref()
                .and_then(index_of)
                .filter(|&t| t != i)
            {
                edges.push(Edge::Composition {
                    owner: i,
                    part_type,
                    usage,
                });
            }
        }
    }
    for connection in connections {
        let joined: Vec<usize> = connection
            .ends
            .iter()
            .filter_map(|end| end.path.first().and_then(index_of))
            .collect();
        if let [a, b] = joined[..] {
            edges.push(Edge::Connection { a, b, connection });
        }
    }
    for (analysis, element) in nodes.iter().enumerate() {
        if let Some(part) = element.subject_target.as_ref().and_then(index_of) {
            edges.push(Edge::Subject { analysis, part });
        }
    }
    for claim in model.claims() {
        let satisfier = claim.satisfier.as_ref().and_then(index_of);
        let requirement = claim.satisfies.as_ref().and_then(index_of);
        if let (Some(satisfier), Some(requirement)) = (satisfier, requirement) {
            edges.push(Edge::Satisfy {
                satisfier,
                requirement,
                claim,
            });
        }
    }

    // Generals, definitions and owners go in layers above what refers to
    // them; connected usages go in neighbouring layers.
    let layering: Vec<(usize, usize)> = edges
        .iter()
        .map(|edge| match *edge {
            Edge::Specialization { specific, general } => (general, specific),
            Edge::Typing { usage, definition } => (definition, usage),
            Edge::Composition {
                owner, part_type, ..
            } => (owner, part_type),
            Edge::Ownership { owner, usage } => (owner, usage),
            Edge::Connection { a, b, .. } => (a, b),
            // Requirements sit above what satisfies them.
            Edge::Satisfy {
                satisfier,
                requirement,
                ..
            } => (requirement, satisfier),
            // The analysed part sits above its analysis.
            Edge::Subject { analysis, part } => (part, analysis),
        })
        .collect();

    let contents: Vec<notation::NodeContent> = nodes
        .iter()
        .map(|e| {
            let compartments = !saved.collapsed.contains(&e.qualified_name);
            notation::node_content(e, model, compartments, |member| index_of(member).is_none())
        })
        .collect();
    // Each parent's children in Model Order: an owned usage by where it is
    // declared, a composed part type by where its usage is.
    struct Child<'e> {
        declared_by: &'e Element,
        node: usize,
    }
    let mut children: BTreeMap<usize, Vec<Child>> = BTreeMap::new();
    for edge in &edges {
        let (owner, child) = match *edge {
            Edge::Ownership { owner, usage } => (
                owner,
                Child {
                    declared_by: nodes[usage],
                    node: usage,
                },
            ),
            Edge::Composition {
                owner,
                part_type,
                usage,
            } => (
                owner,
                Child {
                    declared_by: usage,
                    node: part_type,
                },
            ),
            _ => continue,
        };
        children.entry(owner).or_default().push(child);
    }
    let families: Vec<layout::Family> = children
        .into_iter()
        .map(|(parent, mut children)| {
            children.sort_by(|a, b| {
                a.declared_by
                    .model_order()
                    .cmp(&b.declared_by.model_order())
            });
            let mut order: Vec<usize> = Vec::new();
            for child in children {
                if !order.contains(&child.node) {
                    order.push(child.node);
                }
            }
            layout::Family {
                parent: Some(parent),
                children: order,
            }
        })
        .collect();
    let layout::Layered { rects, stacked } = place(&nodes, &contents, &layering, &families, saved);

    // Edges without a saved route that join the same two nodes are fanned
    // out, numbered from the lower node index so direction doesn't matter.
    let mut parallel: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    let mut fan_slot = Vec::with_capacity(edges.len());
    for edge in &edges {
        let (depicts, from, to) = describe(edge, &nodes);
        let routed = depicts
            .edge_key()
            .is_some_and(|key| saved.edges.contains_key(&key));
        let pair = (from.min(to), from.max(to));
        fan_slot.push((!routed && from != to).then(|| {
            let slot = parallel.entry(pair).or_default();
            *slot += 1;
            (pair, *slot - 1)
        }));
    }

    // Each edge's path: saved bends win; otherwise parallel edges fan out,
    // and an edge that would cross another node detours around it.
    let mut paths: Vec<Vec<Point>> = Vec::with_capacity(edges.len());
    let mut routed_by_user = Vec::with_capacity(edges.len());
    for (edge, slot) in edges.iter().zip(&fan_slot) {
        let (depicts, from, to) = describe(edge, &nodes);
        let saved_bends = depicts.edge_key().and_then(|key| saved.edges.get(&key));
        routed_by_user.push(saved_bends.is_some());
        let bends: Vec<Point> = match (saved_bends, slot) {
            (Some(points), _) => points.iter().map(|&[x, y]| Point::new(x, y)).collect(),
            (None, Some((pair, index))) if parallel[pair] > 1 => {
                notation::fan_bends(rects[pair.0], rects[pair.1], *index, parallel[pair])
            }
            (None, _) => {
                let obstacles: Vec<Rect> = (0..rects.len())
                    .filter(|&i| i != from && i != to)
                    .map(|i| rects[i])
                    .collect();
                notation::detour(rects[from], rects[to], &obstacles)
            }
        };
        paths.push(notation::route(rects[from], rects[to], &bends));
    }
    let CompositionTrees { trunks, branches } =
        composition_trees(&edges, &nodes, &rects, &stacked, &routed_by_user);
    let in_tree: Vec<bool> = branches.iter().map(Option::is_some).collect();
    for (path, branch) in paths.iter_mut().zip(&branches) {
        if let Some(branch) = branch {
            *path = branch.clone();
        }
    }

    // Ends crowding the same spot on a node's border are spread along it;
    // ends of routes the user set, or of a tree's branches, are left
    // exactly where they are.
    for (node, rect) in rects.iter().enumerate() {
        let mut ends: Vec<&mut Point> = Vec::new();
        for (((edge, path), user), tree) in edges
            .iter()
            .zip(paths.iter_mut())
            .zip(&routed_by_user)
            .zip(&in_tree)
        {
            let (_, from, to) = describe(edge, &nodes);
            if *user || *tree || from == to {
                continue;
            }
            let last = path.len() - 1;
            let (first, rest) = path.split_first_mut().expect("a route has two points");
            if from == node {
                ends.push(first);
            } else if to == node {
                ends.push(&mut rest[last - 1]);
            }
        }
        // A tree's diamond, and the ends of its branches, stay put.
        let fixed: Vec<Point> = trunks
            .iter()
            .filter(|(owner, _)| *owner == node)
            .map(|(_, trunk)| trunk.tip)
            .chain(edges.iter().zip(&branches).filter_map(|(edge, branch)| {
                let (_, _, child) = describe(edge, &nodes);
                (child == node).then(|| branch.as_ref()?.last().copied())?
            }))
            .collect();
        notation::spread_ends(*rect, &mut ends, &fixed);
    }

    let mut items: Vec<Item> = Vec::new();
    for (owner, trunk) in &trunks {
        let depicts = Depicts::CompositionTree {
            owner: nodes[*owner].qualified_name.clone(),
        };
        items.extend(notation::composition_trunk(depicts, trunk));
    }
    for ((edge, path), tree) in edges.iter().zip(paths).zip(in_tree) {
        let (depicts, _, child) = describe(edge, &nodes);
        // A stacked child's branch comes across into its left edge.
        let across = stacked[child];
        items.extend(match *edge {
            Edge::Specialization { .. } => notation::specialization(depicts, path),
            Edge::Typing { .. } => notation::defined_by(depicts, path),
            Edge::Composition { usage, .. } if tree => {
                notation::composition_branch(depicts, Some(usage), path, across)
            }
            Edge::Ownership { .. } if tree => {
                notation::composition_branch(depicts, None, path, across)
            }
            Edge::Composition { usage, .. } => notation::composition(depicts, Some(usage), path),
            Edge::Ownership { .. } => notation::composition(depicts, None, path),
            Edge::Connection { connection, .. } => notation::connection(depicts, connection, path),
            Edge::Satisfy { claim, .. } => notation::satisfy(
                depicts,
                crate::satisfaction::claim_status(claim).mark(),
                path,
            ),
            Edge::Subject { .. } => notation::subject(depicts, path),
        });
    }
    for ((element, content), rect) in nodes.iter().zip(&contents).zip(&rects) {
        items.extend(notation::node(element, content, *rect));
    }

    page(items, &rects)
}

/// A parent's composition edges to children below it, without routes the
/// user set, drawn as one tree as in the notation's parts trees: a
/// diamond at the middle of its bottom, a trunk down to a bar, and a
/// branch down to the top middle of each child. A child whose branch would
/// cross another node keeps an edge of its own, and a parent with one
/// child hangs it straight below where it can.
fn composition_trees(
    edges: &[Edge],
    nodes: &[&Element],
    rects: &[Rect],
    stacked: &[bool],
    routed_by_user: &[bool],
) -> CompositionTrees {
    let mut candidates: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
    for (e, edge) in edges.iter().enumerate() {
        let (_, owner, child) = describe(edge, nodes);
        let composes = matches!(edge, Edge::Composition { .. } | Edge::Ownership { .. });
        let below = rects[child].y >= rects[owner].bottom() + 2.0 * notation::DIAMOND_ROOM;
        if composes && !routed_by_user[e] && owner != child && below {
            candidates.entry(owner).or_default().push((e, child));
        }
    }
    let mut trunks: Vec<(usize, notation::Trunk)> = Vec::new();
    let mut branches: Vec<Option<Vec<Point>>> = vec![None; edges.len()];
    for (owner, members) in candidates {
        let parent = rects[owner];
        // Leaves the layout stacked in a column beside the trunk hang from
        // it by a branch into their left edge; the rest hang from the bar.
        let (column, row): (Vec<_>, Vec<_>) = members.into_iter().partition(|&(_, c)| stacked[c]);
        let top = row
            .iter()
            .chain(&column)
            .map(|&(_, c)| rects[c].y)
            .fold(f32::INFINITY, f32::min);
        let mut bar_y = (parent.bottom() + top) / 2.0;
        let crosses_none = |from: Point, to: Point, child: usize| {
            (0..rects.len())
                .filter(|&i| i != owner && i != child)
                .all(|i| !notation::crosses(from, to, rects[i]))
        };
        let row: Vec<(usize, usize)> = row
            .into_iter()
            .filter(|&(_, c)| {
                let x = rects[c].center_x();
                crosses_none(Point::new(x, bar_y), Point::new(x, rects[c].y), c)
            })
            .collect();
        let tip_x = match (&row[..], &column[..]) {
            ([], []) => continue,
            // One child hangs straight below, as far along the bottom as goes.
            ([(_, only)], []) => {
                let inset = notation::DIAMOND_ROOM;
                rects[*only]
                    .center_x()
                    .clamp(parent.x + inset, parent.right() - inset)
            }
            _ => parent.center_x(),
        };
        let bar = row
            .iter()
            .map(|&(_, c)| rects[c].center_x())
            .fold((tip_x, tip_x), |(low, high), x| (low.min(x), high.max(x)));
        // Bars of neighbouring trees at one height would read as one.
        while trunks.iter().any(|(_, other)| {
            (other.bar_y - bar_y).abs() < BAR_STAGGER / 2.0
                && other.bar.0 <= bar.1
                && bar.0 <= other.bar.1
        }) && bar_y + BAR_STAGGER < top - notation::DIAMOND_ROOM
        {
            bar_y += BAR_STAGGER;
        }
        let mut end = bar_y;
        for (e, c) in row {
            let x = rects[c].center_x();
            branches[e] = Some(vec![Point::new(x, bar_y), Point::new(x, rects[c].y)]);
        }
        for (e, c) in column {
            let y = rects[c].y + rects[c].height / 2.0;
            end = end.max(y);
            branches[e] = Some(vec![Point::new(tip_x, y), Point::new(rects[c].x, y)]);
        }
        trunks.push((
            owner,
            notation::Trunk {
                tip: Point::new(tip_x, parent.bottom()),
                end,
                bar_y,
                bar,
            },
        ));
    }
    CompositionTrees { trunks, branches }
}

/// A diagram's composition trees: each tree's trunk by its parent, and
/// each edge's branch if it is in a tree.
struct CompositionTrees {
    trunks: Vec<(usize, notation::Trunk)>,
    branches: Vec<Option<Vec<Point>>>,
}

/// How far apart the bars of two trees that would overlap are moved.
const BAR_STAGGER: f32 = 8.0;

/// A scene whose page holds every node rect with a margin, grown to
/// anything drawn beyond that (such as labels beside edges).
pub(crate) fn page(items: Vec<Item>, rects: &[Rect]) -> Scene {
    let mut left: f32 = 0.0;
    let mut top: f32 = 0.0;
    let mut right = rects.iter().map(Rect::right).fold(0.0, f32::max) + MARGIN;
    let mut bottom = rects.iter().map(Rect::bottom).fold(0.0, f32::max) + MARGIN;
    for item in &items {
        let e = notation::extent(&item.shape);
        left = left.min((e.x - EDGE_PADDING).floor());
        top = top.min((e.y - EDGE_PADDING).floor());
        right = right.max((e.right() + EDGE_PADDING).ceil());
        bottom = bottom.max((e.bottom() + EDGE_PADDING).ceil());
    }
    Scene {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
        items,
        origins: Default::default(),
    }
}

/// What `edge` depicts, and the indices of the nodes it runs from and to.
fn describe(edge: &Edge, nodes: &[&Element]) -> (Depicts, usize, usize) {
    let name = |i: usize| nodes[i].qualified_name.clone();
    match *edge {
        Edge::Subject { analysis, part } => (Depicts::Subject(name(analysis)), analysis, part),
        Edge::Specialization { specific, general } => (
            Depicts::Specialization {
                specific: name(specific),
                general: name(general),
            },
            specific,
            general,
        ),
        Edge::Typing { usage, definition } => (
            Depicts::Typing {
                usage: name(usage),
                definition: name(definition),
            },
            usage,
            definition,
        ),
        Edge::Composition {
            owner,
            part_type,
            usage,
        } => (
            Depicts::Composition {
                owner: name(owner),
                usage: usage.qualified_name.clone(),
            },
            owner,
            part_type,
        ),
        Edge::Ownership { owner, usage } => (
            Depicts::Composition {
                owner: name(owner),
                usage: name(usage),
            },
            owner,
            usage,
        ),
        Edge::Connection { a, b, connection } => {
            (Depicts::Connection(connection.qualified_name.clone()), a, b)
        }
        Edge::Satisfy {
            satisfier,
            requirement,
            claim,
        } => (
            Depicts::Satisfy(claim.qualified_name.clone()),
            satisfier,
            requirement,
        ),
    }
}

/// Node rects: saved placements as they are; every other node arranged
/// automatically (with the edges among them) and set below the saved ones,
/// so nothing saved moves and nothing new overlaps it. Also gives, for each
/// node, whether the layout stacked it beside its parent's trunk.
fn place(
    nodes: &[&Element],
    contents: &[notation::NodeContent],
    layering: &[(usize, usize)],
    families: &[layout::Family],
    saved: &Layout,
) -> layout::Layered {
    let saved_rect = |i: usize| {
        saved.nodes.get(&nodes[i].qualified_name).map(|p| {
            let [width, height] = p.size.unwrap_or_else(|| {
                let (w, h) = notation::node_size(&contents[i]);
                [w, h]
            });
            Rect {
                x: p.x,
                y: p.y,
                width,
                height,
            }
        })
    };
    let unplaced: Vec<usize> = (0..nodes.len())
        .filter(|&i| saved_rect(i).is_none())
        .collect();
    let local = |i: usize| unplaced.iter().position(|&u| u == i);
    let sizes: Vec<(f32, f32)> = unplaced
        .iter()
        .map(|&i| notation::node_size(&contents[i]))
        .collect();
    let edges: Vec<(usize, usize)> = layering
        .iter()
        .filter_map(|&(a, b)| Some((local(a)?, local(b)?)))
        .collect();
    let families: Vec<layout::Family> = families
        .iter()
        .map(|f| layout::Family {
            parent: f.parent.and_then(local),
            children: f.children.iter().filter_map(|&i| local(i)).collect(),
        })
        .collect();
    let auto = layout::layered(&sizes, &edges, &families);

    let saved_rects: Vec<Rect> = (0..nodes.len()).filter_map(saved_rect).collect();
    let (left, top) = if saved_rects.is_empty() {
        (MARGIN, MARGIN)
    } else {
        let left = saved_rects
            .iter()
            .map(|r| r.x)
            .fold(f32::INFINITY, f32::min)
            .max(0.0);
        let bottom = saved_rects.iter().map(Rect::bottom).fold(0.0, f32::max);
        (left, bottom + NEW_NODES_GAP)
    };
    let rects = (0..nodes.len())
        .map(|i| match saved_rect(i) {
            Some(rect) => rect,
            None => {
                let r = auto.rects[local(i).expect("an unplaced node has an automatic place")];
                Rect {
                    x: r.x + left,
                    y: r.y + top,
                    ..r
                }
            }
        })
        .collect();
    let stacked = (0..nodes.len())
        .map(|i| local(i).is_some_and(|l| auto.stacked[l]))
        .collect();
    layout::Layered { rects, stacked }
}
