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

    part drive {
        // Two links to FreeCAD: which object is the engine?
        part engine : Engine {
            @ToolLink { tool = "FreeCAD"; file = "cad/drive.FCStd"; object = "Engine"; }
            @ToolLink { tool = "FreeCAD"; file = "cad/old.FCStd"; object = "Engine"; }
            attribute :>> volume = 0.9e6 [mm^3] { @ToolProperty { name = "Volume"; } }
            attribute :>> density = 7850 [kg/m^3];
        }
        // A Tool Property, but no Tool Link to read it from.
        part spare : Engine {
            attribute :>> volume = 0.9e6 [mm^3] { @ToolProperty { name = "Volume"; } }
            attribute :>> density = 7850 [kg/m^3];
        }
    }
}
