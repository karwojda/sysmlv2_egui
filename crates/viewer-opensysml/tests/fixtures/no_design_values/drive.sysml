package Drive {
    private import ISQ::*;
    private import SI::*;
    private import EngineeringTools::*;

    part def Component {
        attribute volume : VolumeValue;
        attribute density : MassDensityValue;
        attribute mass : MassValue = volume * density;
    }
    part def Engine :> Component {
        assert constraint engineMass { mass <= 7.5 [kg] }
    }
    part def Gearbox :> Component {
        // The estimate, on the def only.
        attribute :>> volume default = 0.2e6 [mm^3];
        assert constraint gearboxMass { mass <= 0.6 [kg] }
    }
    part def Pump {
        attribute mass : MassValue;
        assert constraint pumpMass { mass <= 3 [kg] }
    }

    part def DriveTrain {
        attribute mass : MassValue;
    }

    part drive : DriveTrain {
        part engine : Engine {
            @ToolLink { tool = "FreeCAD"; file = "cad/drive.FCStd"; object = "Engine"; }
            attribute :>> volume = 0.9e6 [mm^3] { @ToolProperty { name = "Volume"; } }
            attribute :>> density = 7850 [kg/m^3];
        }
        part gearbox : Gearbox {
            @ToolLink { tool = "FreeCAD"; file = "cad/drive.FCStd"; object = "Assembly.Link."; }
            // Without a Design Value, Gearbox's default must not show through.
            attribute :>> volume { @ToolProperty { name = "Volume"; } }
            attribute :>> density = 2700 [kg/m^3];
        }
        // Unlinked: judged on their Model Values whatever the tool says.
        part spareEngine : Engine {
            attribute :>> volume = 0.9e6 [mm^3];
            attribute :>> density = 7850 [kg/m^3];
        }
        part spareGearbox : Gearbox {
            attribute :>> density = 2700 [kg/m^3];
        }
        // Linked in another file, to a volume, on a feature that is a mass.
        part pump : Pump {
            @ToolLink { tool = "FreeCAD"; file = "cad/pump.FCStd"; object = "Pump"; }
            attribute :>> mass = 2 [kg] { @ToolProperty { name = "Volume"; } }
        }
        attribute :>> mass = engine.mass + gearbox.mass;
    }

    requirement def DriveMass {
        subject d : DriveTrain;
        require constraint { d.mass <= 8 [kg] }
    }
    requirement driveMass : DriveMass;
    satisfy driveMass by drive;
}
