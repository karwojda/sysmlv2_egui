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
    part def Gearbox :> Component;

    part def DriveTrain {
        attribute mass : MassValue;
    }

    part drive : DriveTrain {
        // Both links point into one file, so the tool is asked once.
        part engine : Engine {
            @ToolLink { tool = "FreeCAD"; file = "cad/drive.FCStd"; object = "Engine"; }
            attribute :>> volume = 0.9e6 [mm^3] { @ToolProperty { name = "Volume"; } }
            attribute :>> density = 7850 [kg/m^3];
        }
        part gearbox : Gearbox {
            @ToolLink { tool = "FreeCAD"; file = "cad/drive.FCStd"; object = "Assembly.Link."; }
            // In m³; the tool answers in mm³.
            attribute :>> volume = 0.0002 [m^3] { @ToolProperty { name = "Volume"; } }
            attribute :>> density = 2700 [kg/m^3];
        }
        // A sibling of the linked engine, of the same def, with no link.
        part spare : Engine {
            attribute :>> volume = 0.9e6 [mm^3];
            attribute :>> density = 7850 [kg/m^3];
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
