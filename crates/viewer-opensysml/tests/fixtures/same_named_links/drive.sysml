package Drive {
    private import ISQ::*;
    private import SI::*;
    private import EngineeringTools::*;

    part def Engine {
        attribute mass : MassValue;
        assert constraint engineMass { mass <= 7.5 [kg] }
    }

    // Two linked `engine.mass` under different parents: the front one's
    // Design Value fits, the rear one's is a volume.
    part drive {
        part front {
            part engine : Engine {
                @ToolLink { tool = "FreeCAD"; file = "cad/drive.FCStd"; object = "Front"; }
                attribute :>> mass = 7 [kg] { @ToolProperty { name = "Volume"; } }
            }
        }
        part rear {
            part engine : Engine {
                @ToolLink { tool = "FreeCAD"; file = "cad/drive.FCStd"; object = "Rear"; }
                attribute :>> mass = 7 [kg] { @ToolProperty { name = "Volume"; } }
            }
        }
    }
}
