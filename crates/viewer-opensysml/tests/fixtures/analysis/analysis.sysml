package A {
    private import ScalarValues::*;
    private import SequenceFunctions::*;
    private import NumericalFunctions::*;

    part def Component {
        attribute mass : Real;
    }
    part car : Component {
        part engine : Component { attribute :>> mass = 150.0; }
        part wheels : Component[4] { attribute :>> mass = 20.0; }
        // Rolls up from the engine and all four wheels.
        attribute :>> mass = engine.mass + sum(wheels.mass);
    }
}
