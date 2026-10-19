package MassRollup {
    doc /* A vehicle whose mass rolls up from its parts, with requirements on
         * the totals. Masses are in kilograms. Change a part's mass and save:
         * the viewer re-checks the requirements and recolours them. */

    private import ScalarValues::*;
    private import SequenceFunctions::*;
    private import NumericalFunctions::*;
    private import Views::*;

    // ---- Structure -------------------------------------------------------

    part def Component {
        attribute mass : Real;
    }
    part def Engine :> Component;
    part def Transmission :> Component;
    part def Body :> Component;
    part def FuelTank :> Component;
    part def Wheel :> Component;
    part def Powertrain :> Component;
    part def Vehicle :> Component;

    part vehicle : Vehicle {
        part powertrain : Powertrain {
            part engine : Engine { attribute :>> mass = 180.0; }
            part transmission : Transmission { attribute :>> mass = 85.0; }
            // The powertrain's mass rolls up from its parts.
            attribute :>> mass = engine.mass + transmission.mass;
        }
        part body : Body { attribute :>> mass = 950.0; }
        part fuelTank : FuelTank { attribute :>> mass = 60.0; }
        part wheels : Wheel[4] { attribute :>> mass = 22.0; }
        // The vehicle's mass rolls up from its subsystems, all four wheels included.
        attribute :>> mass = powertrain.mass + body.mass + fuelTank.mass + sum(wheels.mass);
    }

    // ---- Requirements ----------------------------------------------------

    requirement def <'MR'> MassLimit {
        doc /* The subject's total mass shall not exceed the limit. */
        subject s : Component;
        attribute limit : Real;
        require constraint { s.mass <= limit }
    }

    requirement massSpec {
        doc /* Mass budget for the vehicle. */
        subject v : Vehicle;
        requirement <'1'> vehicleMass : MassLimit {
            doc /* Total vehicle mass at most 1400 kg. */
            attribute :>> limit = 1400.0;
        }
        requirement <'2'> powertrainMass : MassLimit {
            doc /* Powertrain mass at most 250 kg. */
            attribute :>> limit = 250.0;
        }
        requirement <'3'> tankMass : MassLimit {
            doc /* Fuel tank mass at most 70 kg; not yet allocated. */
            attribute :>> limit = 70.0;
        }
    }

    satisfy massSpec.vehicleMass by vehicle;
    satisfy massSpec.powertrainMass by vehicle.powertrain;

    // ---- Views -----------------------------------------------------------

    view massOverview {
        doc /* The parts tree with the mass budget it must meet. Parts and
             * requirements are exposed one by one so that their mass
             * attributes stay in compartments. */
        expose vehicle;
        expose vehicle::powertrain;
        expose vehicle::powertrain::engine;
        expose vehicle::powertrain::transmission;
        expose vehicle::body;
        expose vehicle::fuelTank;
        expose vehicle::wheels;
        expose massSpec;
        expose massSpec::vehicleMass;
        expose massSpec::powertrainMass;
        expose massSpec::tankMass;
        render asTreeDiagram;
    }
}
