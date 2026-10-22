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

    part def DriveTrain {
        attribute mass : MassValue;
    }

    part drive : DriveTrain {
        part engine : Engine {
            @ToolLink { tool = "FreeCAD"; file = "cad/drive.FCStd"; object = "Engine"; }
            // A body and no value: the Model writes no Model Value for it.
            attribute :>> volume { @ToolProperty { name = "Volume"; } }
            attribute :>> density = 7850 [kg/m^3];
        }
        part gearbox : Gearbox {
            @ToolLink { tool = "FreeCAD"; file = "cad/drive.FCStd"; object = "Assembly.Link."; }
            // Redefines Gearbox's own unnamed `:>> volume default`.
            attribute :>> volume { @ToolProperty { name = "Volume"; } }
            attribute :>> density = 2700 [kg/m^3];
        }
        // Another Gearbox, unlinked: it keeps the def's default.
        part spare : Gearbox {
            attribute :>> density = 2700 [kg/m^3];
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
