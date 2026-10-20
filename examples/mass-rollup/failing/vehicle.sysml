package MassRollup {
    doc /* A vehicle whose mass rolls up from its parts, with requirements on
         * the totals. Masses are in kilograms.
         *
         * This variant fails: the powertrain is over its budget
         * (180 + 85 = 265 kg > 250) and nothing claims the tank requirement
         * yet. Change a part's mass and save: the viewer re-checks the
         * requirements and recolours them. See ../passing for one that meets
         * the whole budget. */

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
    part def FuelTank :> Component {
        attribute fuelMass : Real;
        attribute fuelMassMax : Real;
        // Every fuel tank: the fuel it carries must fit.
        assert constraint fuelFits { fuelMass <= fuelMassMax }
    }
    part def Wheel :> Component;
    part def Powertrain :> Component;
    part def Vehicle :> Component {
        attribute power : Real;    // kW
    }

    // Mass per unit of power, kg/kW, at most a limit.
    constraint def MassPerPower {
        in totalMass : Real;
        in power : Real;
        in limit : Real;
        totalMass / power <= limit
    }

    part vehicle : Vehicle {
        part powertrain : Powertrain {
            part engine : Engine { attribute :>> mass = 180.0; }
            part transmission : Transmission { attribute :>> mass = 85.0; }
            // The powertrain's mass rolls up from its parts.
            attribute :>> mass = engine.mass + transmission.mass;
        }
        part body : Body { attribute :>> mass = 950.0; }
        part fuelTank : FuelTank {
            attribute :>> mass = 60.0;
            attribute :>> fuelMass = 50.0;
            attribute :>> fuelMassMax = 45.0;
        }
        part wheels : Wheel[4] { attribute :>> mass = 22.0; }
        // The vehicle's mass rolls up from its subsystems, all four wheels included.
        attribute :>> mass = powertrain.mass + body.mass + fuelTank.mass + sum(wheels.mass);
        attribute :>> power = 90.0;
        // The vehicle carries at most 16 kg per kW.
        assert constraint powerToWeight : MassPerPower {
            in totalMass = mass;
            in power = power;
            in limit = 16.0;
        }
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

    // ---- Analysis --------------------------------------------------------

    // Works out a vehicle's total mass and checks it against a limit.
    analysis def MassAnalysis {
        subject v : Vehicle;
        in limit : Real = 1400.0;
        objective {
            doc /* The total mass stays within the limit. */
            require constraint { totalMass <= limit }
        }
        return totalMass : Real = v.mass;
    }

    analysis massAnalysis : MassAnalysis {
        subject v = vehicle;
    }

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
        expose massAnalysis;
        render asTreeDiagram;
    }
}
