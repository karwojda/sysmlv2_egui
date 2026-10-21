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
        attribute :>> volume default = 0.2e6 [mm^3];   // Model Value only on the def
    }

    part def DriveTrain {
        attribute mass : MassValue;
    }

    part drive : DriveTrain {
        part engine : Engine {
            @ToolLink { tool = "FreeCAD"; file = "cad/engine.FCStd"; object = "Engine"; }
            attribute :>> volume = 0.9e6 [mm^3] { @ToolProperty { name = "Volume"; } }
            attribute :>> density = 7850 [kg/m^3];
        }
        part gearbox : Gearbox {
            @ToolLink { tool = "FreeCAD"; file = "cad/engine.FCStd"; object = "Assembly.Link."; }
            attribute :>> volume { @ToolProperty { name = "Volume"; } }   // Model Value only from Gearbox's default
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
