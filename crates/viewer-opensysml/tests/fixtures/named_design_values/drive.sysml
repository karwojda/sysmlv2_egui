package Drive {
    private import ISQ::*;
    private import SI::*;
    private import EngineeringTools::*;

    part def Shaft;

    part drive {
        part shaft : Shaft {
            @ToolLink { tool = "FreeCAD"; file = "cad/drive.FCStd"; object = "Shaft"; }
            // A feature of its own, new on the usage: a body and no value.
            attribute bore : LengthValue { @ToolProperty { name = "Diameter"; } }
            assert constraint boreFits { bore <= 6 [mm] }
        }
        // The tool knows no object `Spare`.
        part spare : Shaft {
            @ToolLink { tool = "FreeCAD"; file = "cad/drive.FCStd"; object = "Spare"; }
            attribute bore : LengthValue { @ToolProperty { name = "Diameter"; } }
            assert constraint boreFits { bore <= 6 [mm] }
        }
    }
}
