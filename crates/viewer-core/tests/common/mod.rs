//! Fixtures shared by the core's integration tests.

#![allow(dead_code)]

use viewer_core::Model;
use viewer_core::model::{Direction, ExposeScope, ModelBuilder};

/// A drivetrain after slide 40 of the notation reference: fuel tank,
/// engine, transmission, driveshaft, a rear axle assembly with its own
/// parts, and a controller on an undirected status port. The fake-backend
/// twin of `viewer-opensysml/tests/fixtures/drivetrain`.
pub fn drivetrain() -> Model {
    let q = |s: &str| format!("D::{s}");
    let port = |b: ModelBuilder,
                owner: &str,
                name: &str,
                ty: &str,
                direction: Option<Direction>,
                conjugated: bool| {
        let mut b = b.port(&q(&format!("{owner}::{name}"))).typed_by(&q(ty));
        if conjugated {
            b = b.conjugated();
        }
        if let Some(d) = direction {
            b = b.direction(d);
        }
        b
    };
    let mut b = Model::builder().package("D");
    for definition in ["FuelPort", "TorquePort", "StatusPort", "PowerPort"] {
        b = b.port_def(&q(definition));
    }
    b = b.interface_def(&q("FuelInterface"));
    b = port(b, "FuelInterface", "demand", "FuelPort", None, true);
    b = port(b, "FuelInterface", "supply", "FuelPort", None, false);
    b = b.part_def(&q("AxleAssembly"));
    b = port(
        b,
        "AxleAssembly",
        "torqueIn",
        "TorquePort",
        Some(Direction::In),
        true,
    );
    for (name, ty) in [
        ("differential", "Differential"),
        ("leftWheel", "Wheel"),
        ("rightWheel", "Wheel"),
    ] {
        b = b
            .part(&q(&format!("AxleAssembly::{name}")))
            .typed_by(&q(ty));
    }
    let a = |s: &str| q(&format!("AxleAssembly::{s}"));
    b = b
        .connection(&a("drive"))
        .end(&[&a("torqueIn")])
        .end(&[&a("differential"), &q("Differential::torqueIn")]);
    b = b
        .connection(&a("toLeft"))
        .end(&[&a("differential"), &q("Differential::left")])
        .end(&[&a("leftWheel"), &q("Wheel::hub")]);
    b = b
        .connection(&a("toRight"))
        .end(&[&a("differential"), &q("Differential::right")])
        .end(&[&a("rightWheel"), &q("Wheel::hub")]);
    b = b.part_def(&q("Controller"));
    b = port(b, "Controller", "status", "StatusPort", None, true);
    b = b.part_def(&q("Differential"));
    b = port(
        b,
        "Differential",
        "left",
        "TorquePort",
        Some(Direction::Out),
        false,
    );
    b = port(
        b,
        "Differential",
        "right",
        "TorquePort",
        Some(Direction::Out),
        false,
    );
    b = port(
        b,
        "Differential",
        "torqueIn",
        "TorquePort",
        Some(Direction::In),
        true,
    );
    b = b.part_def(&q("Driveshaft"));
    b = port(
        b,
        "Driveshaft",
        "torqueIn",
        "TorquePort",
        Some(Direction::In),
        true,
    );
    b = port(
        b,
        "Driveshaft",
        "torqueOut",
        "TorquePort",
        Some(Direction::Out),
        false,
    );
    b = b.part_def(&q("Engine"));
    b = port(b, "Engine", "fuelIn", "FuelPort", Some(Direction::In), true);
    b = port(b, "Engine", "power", "PowerPort", Some(Direction::In), true);
    b = port(b, "Engine", "status", "StatusPort", None, false);
    b = port(
        b,
        "Engine",
        "torqueOut",
        "TorquePort",
        Some(Direction::Out),
        false,
    );
    b = b.part_def(&q("FuelTank"));
    b = port(
        b,
        "FuelTank",
        "fuelOut",
        "FuelPort",
        Some(Direction::Out),
        false,
    );
    b = b.part_def(&q("Transmission"));
    b = port(
        b,
        "Transmission",
        "torqueIn",
        "TorquePort",
        Some(Direction::In),
        true,
    );
    b = port(
        b,
        "Transmission",
        "torqueOut",
        "TorquePort",
        Some(Direction::Out),
        false,
    );
    b = b.part_def(&q("Vehicle"));
    b = port(
        b,
        "Vehicle",
        "power",
        "PowerPort",
        Some(Direction::In),
        true,
    );
    for (name, ty) in [
        ("controller", "Controller"),
        ("driveshaft", "Driveshaft"),
        ("engine", "Engine"),
        ("fuelTank", "FuelTank"),
        ("rearAxleAssembly", "AxleAssembly"),
        ("transmission", "Transmission"),
    ] {
        b = b.part(&q(&format!("Vehicle::{name}"))).typed_by(&q(ty));
    }
    let v = |s: &str| q(&format!("Vehicle::{s}"));
    b = b
        .connection(&v("axle"))
        .end(&[&v("driveshaft"), &q("Driveshaft::torqueOut")])
        .end(&[&v("rearAxleAssembly"), &q("AxleAssembly::torqueIn")]);
    b = b
        .connection(&v("engineStatus"))
        .end(&[&v("engine"), &q("Engine::status")])
        .end(&[&v("controller"), &q("Controller::status")]);
    b = b
        .interface(&v("fuelLine"))
        .typed_by(&q("FuelInterface"))
        .end(&[&v("fuelTank"), &q("FuelTank::fuelOut")])
        .end(&[&v("engine"), &q("Engine::fuelIn")]);
    b = b
        .connection(&v("leftDrive"))
        .end(&[&v("driveshaft"), &q("Driveshaft::torqueOut")])
        .end(&[
            &v("rearAxleAssembly"),
            &q("AxleAssembly::leftWheel"),
            &q("Wheel::hub"),
        ]);
    b = b
        .connection(&v("shaft"))
        .end(&[&v("transmission"), &q("Transmission::torqueOut")])
        .end(&[&v("driveshaft"), &q("Driveshaft::torqueIn")]);
    b = b
        .connection(&v("supply"))
        .end(&[&v("power")])
        .end(&[&v("engine"), &q("Engine::power")]);
    b = b.part_def(&q("Wheel"));
    b = port(b, "Wheel", "hub", "TorquePort", Some(Direction::In), true);
    b = b
        .connection(&v("torque"))
        .end(&[&v("engine"), &q("Engine::torqueOut")])
        .end(&[&v("transmission"), &q("Transmission::torqueIn")]);
    b.build()
}

/// A vehicle with every kind of wire: an anonymous item flow, a named flow,
/// bindings between attributes and between ports, an unnamed connection
/// between nested ports, and a reference part. The fake-backend twin of
/// `viewer-opensysml/tests/fixtures/flows`, generated names included.
pub fn flows() -> Model {
    use viewer_core::model::ElementKind;
    Model::builder()
        .package("F")
        .part_def("F::Driver")
        .part_def("F::Engine")
        .port("F::Engine::fuelIn")
        .typed_by("F::FuelPort")
        .conjugated()
        .direction(Direction::In)
        .attribute("F::Engine::fuelLevel")
        .typed_by("ScalarValues::Real")
        .unsupported("F::Fuel", "item def")
        .port_def("F::FuelPort")
        .unsupported("F::FuelPort::fuel", "item")
        .typed_by("F::Fuel")
        .direction(Direction::Out)
        .port("F::FuelPort::inner")
        .typed_by("F::InnerPort")
        .port_def("F::InnerPort")
        .part_def("F::Tank")
        .port("F::Tank::fuelOut")
        .typed_by("F::FuelPort")
        .direction(Direction::Out)
        .attribute("F::Tank::level")
        .typed_by("ScalarValues::Real")
        .part_def("F::Vehicle")
        .unnamed("F::Vehicle::@3", ElementKind::FlowUsage)
        .payload("F::Fuel")
        .end(&["F::Vehicle::tank", "F::Tank::fuelOut", "F::FuelPort::fuel"])
        .end(&[
            "F::Vehicle::engine",
            "F::Engine::fuelIn",
            "F::FuelPort::fuel",
        ])
        .unnamed("F::Vehicle::@5", ElementKind::BindingConnection)
        .end(&["F::Vehicle::tank", "F::Tank::level"])
        .end(&["F::Vehicle::engine", "F::Engine::fuelLevel"])
        .unnamed("F::Vehicle::@6", ElementKind::BindingConnection)
        .end(&["F::Vehicle::tank", "F::Tank::fuelOut"])
        .end(&["F::Vehicle::engine", "F::Engine::fuelIn"])
        .unnamed("F::Vehicle::@7", ElementKind::ConnectionUsage)
        .end(&["F::Vehicle::tank", "F::Tank::fuelOut", "F::FuelPort::inner"])
        .end(&[
            "F::Vehicle::engine",
            "F::Engine::fuelIn",
            "F::FuelPort::inner",
        ])
        .part("F::Vehicle::driver")
        .typed_by("F::Driver")
        .reference()
        .part("F::Vehicle::engine")
        .typed_by("F::Engine")
        .flow("F::Vehicle::fuelFlow")
        .payload("F::Fuel")
        .end(&["F::Vehicle::tank", "F::Tank::fuelOut"])
        .end(&["F::Vehicle::engine", "F::Engine::fuelIn"])
        .part("F::Vehicle::tank")
        .typed_by("F::Tank")
        .build()
}

/// Views of a small vehicle: a parts tree, an interconnection view, a view
/// inheriting its rendering from a view definition, a `GeneralView` without
/// a rendering, and a table. The fake-backend twin of
/// `viewer-opensysml/tests/fixtures/views`.
pub fn views() -> Model {
    Model::builder()
        .package("V")
        .package("V::Defs")
        .part_def("V::Defs::Frame")
        .attribute_def("V::Defs::Mass")
        .part_def("V::Engine")
        .view_def("V::TreeView")
        .renders("Views::asTreeDiagram")
        .part_def("V::Vehicle")
        .part("V::Vehicle::engine")
        .typed_by("V::Engine")
        .part("V::Vehicle::wheel")
        .typed_by("V::Wheel")
        .multiplicity("4", "4")
        .part_def("V::Wheel")
        .view("V::inherited")
        .typed_by("V::TreeView")
        .exposes("V::Vehicle", ExposeScope::Members)
        .view("V::overview")
        .typed_by("StandardViewDefinitions::GeneralView")
        .exposes("V::Defs", ExposeScope::Members)
        .exposes("V::Engine", ExposeScope::Element)
        .view("V::table")
        .exposes("V::Defs", ExposeScope::Members)
        .renders("Views::asElementTable")
        .part("V::vehicle")
        .typed_by("V::Vehicle")
        .part("V::vehicle::body")
        .view("V::vehicle tree")
        .exposes("V::vehicle", ExposeScope::ElementAndDescendants)
        .renders("Views::asTreeDiagram")
        .view("V::wiring")
        .exposes("V::Vehicle", ExposeScope::Element)
        .renders("Views::asInterconnectionDiagram")
        .build()
}

/// Parts annotated with metadata in each of its forms (`@X`, `#X`,
/// `metadata X`), including a usage with two annotations. The fake-backend
/// twin of `viewer-opensysml/tests/fixtures/metadata`.
pub fn metadata() -> Model {
    Model::builder()
        .package("M")
        .part_def("M::Airbag")
        .annotated("M::Safety")
        .part_def("M::Brake")
        .annotated("M::Safety")
        .part_def("M::Radio")
        .unsupported("M::Safety", "metadata def")
        .part_def("M::Seat")
        .annotated("M::Supplier")
        .unsupported("M::Supplier", "metadata def")
        .part_def("M::Vehicle")
        .part("M::Vehicle::airbag")
        .typed_by("M::Airbag")
        .annotated("M::Safety")
        .annotated("M::Supplier")
        .part("M::Vehicle::radio")
        .typed_by("M::Radio")
        .build()
}

/// Requirements on a vehicle: definitions with short ids, subjects and
/// docs, a specification with nested requirements, and two vehicles that
/// satisfy some of them. The fake-backend twin of
/// `viewer-opensysml/tests/fixtures/requirements`.
pub fn requirements() -> Model {
    let real = "ScalarValues::Real";
    Model::builder()
        .package("Req")
        .requirement_def("Req::CostCap")
        .subject("Req::CostCap::v")
        .requires("v.cost <= 10.0")
        .part("Req::CostCap::v")
        .typed_by("Req::Vehicle")
        .requirement_def("Req::MassLimit")
        .short_name("R1")
        .subject("Req::MassLimit::v")
        .doc("The vehicle shall weigh no more than the limit.")
        .requires("v.mass <= limit")
        .attribute("Req::MassLimit::limit")
        .typed_by(real)
        .part("Req::MassLimit::v")
        .typed_by("Req::Vehicle")
        .requirement_def("Req::RangeTarget")
        .subject("Req::RangeTarget::v")
        .requires("v.range >= 500.0")
        .part("Req::RangeTarget::v")
        .typed_by("Req::Vehicle")
        .part_def("Req::Vehicle")
        .attribute("Req::Vehicle::cost")
        .typed_by(real)
        .attribute("Req::Vehicle::mass")
        .typed_by(real)
        .attribute("Req::Vehicle::range")
        .typed_by(real)
        .part("Req::car")
        .typed_by("Req::Vehicle")
        .attribute("Req::car::mass")
        .part("Req::vehicle")
        .typed_by("Req::Vehicle")
        .attribute("Req::vehicle::mass")
        .attribute("Req::vehicle::range")
        .requirement("Req::vehicleSpec")
        .subject("Req::vehicleSpec::v")
        .doc("Top-level vehicle requirements.")
        .requirement("Req::vehicleSpec::costReq")
        .short_name("3")
        .typed_by("Req::CostCap")
        .requirement("Req::vehicleSpec::massReq")
        .short_name("1")
        .typed_by("Req::MassLimit")
        .attribute("Req::vehicleSpec::massReq::limit")
        .requirement("Req::vehicleSpec::rangeReq")
        .short_name("2")
        .typed_by("Req::RangeTarget")
        .requirement("Req::vehicleSpec::spareReq")
        .short_name("4")
        .typed_by("Req::RangeTarget")
        .part("Req::vehicleSpec::v")
        .typed_by("Req::Vehicle")
        .satisfy("Req::@8", "Req::vehicleSpec::massReq", "Req::vehicle")
        .satisfy("Req::@9", "Req::vehicleSpec::rangeReq", "Req::vehicle")
        .satisfy("Req::@10", "Req::vehicleSpec::costReq", "Req::vehicle")
        .satisfy("Req::car::@1", "Req::vehicleSpec::massReq", "Req::car")
        .build()
}
