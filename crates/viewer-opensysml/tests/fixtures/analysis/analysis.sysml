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

    analysis def MassCheck {
        subject c : Component;
        in limit : Real = 300.0;
        objective {
            doc /* The mass stays within the limit. */
            require constraint { result <= limit }
        }
        return result : Real = c.mass;
    }
    analysis carMass : MassCheck {
        subject c = car;
    }
}
