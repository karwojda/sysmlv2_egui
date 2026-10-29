//! Seam 1: Layout edits, checked through the scene they produce and the
//! Diagram File they would save.

use viewer_core::editing::{Edit, MIN_NODE_SIZE, apply};
use viewer_core::scene::{Depicts, Rect, Shape};
use viewer_core::{
    DiagramFile, DiagramKind, Layout, Model, NodePlacement, QualifiedName, Scene,
    build_scene_with_layout,
};

fn model() -> Model {
    Model::builder()
        .package("P")
        .part_def("P::Vehicle")
        .attribute("P::Vehicle::mass")
        .part_def("P::Car")
        .specializes("P::Vehicle")
        .part_def("P::Truck")
        .specializes("P::Vehicle")
        .build()
}

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

/// An open Diagram: its Layout, redrawn after every edit.
struct Editing {
    model: Model,
    layout: Layout,
    scene: Scene,
}

impl Editing {
    fn new() -> Self {
        let model = model();
        let layout = Layout::default();
        let scene =
            build_scene_with_layout(&model, DiagramKind::General, &qn("P"), &layout).unwrap();
        Self {
            model,
            layout,
            scene,
        }
    }

    fn edit(&mut self, edit: Edit) -> &mut Self {
        apply(&mut self.layout, &self.model, &self.scene, &edit);
        self.scene =
            build_scene_with_layout(&self.model, DiagramKind::General, &qn("P"), &self.layout)
                .unwrap();
        self
    }

    fn bounds(&self, name: &str) -> Rect {
        self.scene.node_bounds(&qn(name)).unwrap()
    }

    fn saved(&self) -> DiagramFile {
        let mut file = DiagramFile::new("p", DiagramKind::General, qn("P"));
        file.layout = self.layout.clone();
        DiagramFile::parse(&file.to_text()).unwrap()
    }
}

#[test]
fn moving_a_node_moves_only_that_node() {
    let mut diagram = Editing::new();
    let (car, truck, vehicle) = (
        diagram.bounds("P::Car"),
        diagram.bounds("P::Truck"),
        diagram.bounds("P::Vehicle"),
    );

    diagram.edit(Edit::Move {
        elements: vec![qn("P::Car")],
        dx: 15.0,
        dy: 120.0,
    });

    assert_eq!(
        diagram.bounds("P::Car"),
        Rect {
            x: car.x + 15.0,
            y: car.y + 120.0,
            ..car
        }
    );
    assert_eq!(diagram.bounds("P::Truck"), truck);
    assert_eq!(diagram.bounds("P::Vehicle"), vehicle);
}

#[test]
fn a_move_is_saved_in_the_diagram_file() {
    let mut diagram = Editing::new();
    let car = diagram.bounds("P::Car");

    diagram.edit(Edit::Move {
        elements: vec![qn("P::Car")],
        dx: 15.0,
        dy: 120.0,
    });

    assert_eq!(
        diagram.saved().layout.nodes[&qn("P::Car")],
        NodePlacement {
            x: car.x + 15.0,
            y: car.y + 120.0,
            size: None
        }
    );
}

#[test]
fn nodes_stop_at_the_top_and_left_of_the_page() {
    let mut diagram = Editing::new();

    diagram.edit(Edit::Move {
        elements: vec![qn("P::Car")],
        dx: -5000.0,
        dy: -5000.0,
    });

    let car = diagram.bounds("P::Car");
    assert_eq!((car.x, car.y), (0.0, 0.0));
}

#[test]
fn moving_several_nodes_moves_them_together() {
    let mut diagram = Editing::new();
    let (car, truck, vehicle) = (
        diagram.bounds("P::Car"),
        diagram.bounds("P::Truck"),
        diagram.bounds("P::Vehicle"),
    );

    diagram.edit(Edit::Move {
        elements: vec![qn("P::Car"), qn("P::Truck")],
        dx: 30.0,
        dy: 50.0,
    });

    for (name, before) in [("P::Car", car), ("P::Truck", truck)] {
        let after = diagram.bounds(name);
        assert_eq!(
            (after.x, after.y),
            (before.x + 30.0, before.y + 50.0),
            "{name}"
        );
    }
    assert_eq!(diagram.bounds("P::Vehicle"), vehicle);
}

#[test]
fn nodes_moved_together_all_stop_when_one_reaches_the_edge_of_the_page() {
    let mut diagram = Editing::new();
    let (car, truck) = (diagram.bounds("P::Car"), diagram.bounds("P::Truck"));
    let (left, top) = (car.x.min(truck.x), car.y.min(truck.y));

    diagram.edit(Edit::Move {
        elements: vec![qn("P::Car"), qn("P::Truck")],
        dx: -5000.0,
        dy: -5000.0,
    });

    let (moved_car, moved_truck) = (diagram.bounds("P::Car"), diagram.bounds("P::Truck"));
    assert_eq!((moved_car.x, moved_car.y), (car.x - left, car.y - top));
    assert_eq!(
        (moved_truck.x, moved_truck.y),
        (truck.x - left, truck.y - top)
    );
}

#[test]
fn bends_move_with_nodes_moved_together_only_when_both_ends_move() {
    let key = Depicts::Specialization {
        specific: qn("P::Car"),
        general: qn("P::Vehicle"),
    }
    .edge_key()
    .unwrap();
    let routed = || {
        let mut diagram = Editing::new();
        let car = diagram.bounds("P::Car");
        diagram.edit(Edit::Route {
            edge: key.clone(),
            bends: vec![[car.x - 40.0, car.y - 30.0]],
        });
        let bend = diagram.layout.edges[&key][0];
        (diagram, bend)
    };

    let (mut both, bend) = routed();
    both.edit(Edit::Move {
        elements: vec![qn("P::Car"), qn("P::Vehicle")],
        dx: 25.0,
        dy: 15.0,
    });
    assert_eq!(
        both.layout.edges[&key],
        vec![[bend[0] + 25.0, bend[1] + 15.0]]
    );

    let (mut one, bend) = routed();
    one.edit(Edit::Move {
        elements: vec![qn("P::Car")],
        dx: 25.0,
        dy: 15.0,
    });
    assert_eq!(one.layout.edges[&key], vec![bend]);
}

#[test]
fn resizing_sets_a_size_no_smaller_than_the_minimum() {
    let mut diagram = Editing::new();

    diagram.edit(Edit::Resize {
        element: qn("P::Vehicle"),
        width: 300.0,
        height: 160.0,
    });
    assert_eq!(
        (
            diagram.bounds("P::Vehicle").width,
            diagram.bounds("P::Vehicle").height
        ),
        (300.0, 160.0)
    );

    diagram.edit(Edit::Resize {
        element: qn("P::Vehicle"),
        width: 1.0,
        height: 1.0,
    });
    let vehicle = diagram.bounds("P::Vehicle");
    assert_eq!([vehicle.width, vehicle.height], MIN_NODE_SIZE);
}

#[test]
fn hiding_and_showing_again() {
    let mut diagram = Editing::new();
    let vehicle = diagram.bounds("P::Vehicle");

    diagram.edit(Edit::Hide(qn("P::Vehicle")));
    assert!(diagram.scene.node_bounds(&qn("P::Vehicle")).is_none());
    assert!(diagram.saved().layout.hidden.contains(&qn("P::Vehicle")));

    diagram.edit(Edit::Show(qn("P::Vehicle")));
    assert_eq!(
        diagram.bounds("P::Vehicle"),
        vehicle,
        "it comes back where it was"
    );
}

#[test]
fn show_all_brings_back_every_hidden_element() {
    let mut diagram = Editing::new();

    diagram
        .edit(Edit::Hide(qn("P::Car")))
        .edit(Edit::Hide(qn("P::Truck")))
        .edit(Edit::ShowAll);

    assert!(diagram.scene.node_bounds(&qn("P::Car")).is_some());
    assert!(diagram.scene.node_bounds(&qn("P::Truck")).is_some());
}

#[test]
fn collapsing_drops_compartments_and_shrinks_to_fit_and_expanding_restores_them() {
    let mut diagram = Editing::new();
    let expanded = diagram.bounds("P::Vehicle");
    let texts = |diagram: &Editing| -> Vec<String> {
        diagram
            .scene
            .items_depicting(&Depicts::Element(qn("P::Vehicle")))
            .filter_map(|s| match s {
                Shape::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect()
    };

    diagram.edit(Edit::Resize {
        element: qn("P::Vehicle"),
        width: 400.0,
        height: 200.0,
    });
    diagram.edit(Edit::Collapse(qn("P::Vehicle")));
    assert_eq!(texts(&diagram), ["«part def»", "Vehicle"]);
    assert!(diagram.bounds("P::Vehicle").height < expanded.height);
    assert_eq!(
        (
            diagram.bounds("P::Vehicle").x,
            diagram.bounds("P::Vehicle").y
        ),
        (expanded.x, expanded.y)
    );

    diagram.edit(Edit::Expand(qn("P::Vehicle")));
    assert_eq!(texts(&diagram).len(), 4);
    assert_eq!(diagram.bounds("P::Vehicle"), expanded);
}

#[test]
fn reset_returns_to_the_automatic_arrangement() {
    let mut diagram = Editing::new();
    let automatic = diagram.scene.clone();

    diagram
        .edit(Edit::Move {
            elements: vec![qn("P::Car")],
            dx: 200.0,
            dy: 200.0,
        })
        .edit(Edit::Hide(qn("P::Truck")))
        .edit(Edit::Collapse(qn("P::Vehicle")))
        .edit(Edit::Reset);

    assert_eq!(diagram.scene, automatic);
}

#[test]
fn an_edit_that_changes_nothing_leaves_the_layout_untouched() {
    let mut diagram = Editing::new();

    diagram
        .edit(Edit::ShowAll)
        .edit(Edit::Expand(qn("P::Vehicle")))
        .edit(Edit::Show(qn("P::Car")));

    assert_eq!(diagram.layout, Layout::default());
}

#[test]
fn reset_keeps_orphaned_layout_entries() {
    let mut diagram = Editing::new();
    diagram.layout.nodes.insert(
        qn("P::Renamed"),
        NodePlacement {
            x: 1.0,
            y: 2.0,
            size: None,
        },
    );
    diagram.layout.hidden.insert(qn("P::Gone"));

    diagram
        .edit(Edit::Move {
            elements: vec![qn("P::Car")],
            dx: 10.0,
            dy: 0.0,
        })
        .edit(Edit::Reset);

    assert_eq!(
        diagram.layout.orphaned(&diagram.model),
        [&qn("P::Gone"), &qn("P::Renamed")]
    );
    assert!(!diagram.layout.nodes.contains_key(&qn("P::Car")));
}

#[test]
fn routing_an_edge_bends_it_and_clearing_the_route_straightens_it_again() {
    let mut diagram = Editing::new();
    let edge = Depicts::Specialization {
        specific: qn("P::Car"),
        general: qn("P::Vehicle"),
    };
    let key = edge.edge_key().unwrap();
    let path = |diagram: &Editing| -> Vec<viewer_core::scene::Point> {
        diagram
            .scene
            .edges()
            .into_iter()
            .find(|(d, _)| **d == edge)
            .unwrap()
            .1
            .to_vec()
    };
    let straight = path(&diagram);

    diagram.edit(Edit::Route {
        edge: key.clone(),
        bends: vec![[400.0, 300.0]],
    });
    assert_eq!(
        path(&diagram)[1],
        viewer_core::scene::Point::new(400.0, 300.0)
    );
    assert_eq!(diagram.saved().layout.edges[&key], vec![[400.0, 300.0]]);

    diagram.edit(Edit::Route {
        edge: key.clone(),
        bends: Vec::new(),
    });
    assert_eq!(path(&diagram), straight);
    assert!(diagram.saved().layout.edges.is_empty());
}

#[test]
fn routing_an_edge_keeps_every_node_where_it_was() {
    let mut diagram = Editing::new();
    let before: Vec<Rect> = ["P::Vehicle", "P::Car", "P::Truck"]
        .iter()
        .map(|n| diagram.bounds(n))
        .collect();
    let key = Depicts::Specialization {
        specific: qn("P::Car"),
        general: qn("P::Vehicle"),
    }
    .edge_key()
    .unwrap();

    diagram.edit(Edit::Route {
        edge: key,
        bends: vec![[900.0, 900.0]],
    });

    let after: Vec<Rect> = ["P::Vehicle", "P::Car", "P::Truck"]
        .iter()
        .map(|n| diagram.bounds(n))
        .collect();
    assert_eq!(before, after);
}
