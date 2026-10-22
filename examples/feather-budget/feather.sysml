package FeatherBudget {
    doc /* The RoyalBlue54L Feather, a board designed in KiCad (ecad/), and
         * the envelope it must fit. Its dimensions come from the KiCad
         * layout as Design Values; the values written here are the Model's
         * estimates, used only where KiCad can't be read. */

    private import ISQ::*;
    private import SI::*;
    private import EngineeringTools::*;

    part def Board {
        attribute width : LengthValue;
        attribute height : LengthValue;
        attribute thickness : LengthValue;
        attribute area : AreaValue;
    }

    part feather : Board {
        @ToolLink { tool = "KiCad"; file = "ecad/RoyalBlue54L-Feather.kicad_pro"; object = "board"; }
        // A Feather is 2.0 x 0.9 inches on 1.6 mm FR-4.
        attribute :>> width = 50.8 [mm] { @ToolProperty { name = "Width"; } }
        attribute :>> height = 22.86 [mm] { @ToolProperty { name = "Height"; } }
        attribute :>> thickness = 1.6 [mm] { @ToolProperty { name = "Thickness"; } }
        attribute :>> area = 1161.3 [mm^2] { @ToolProperty { name = "Area"; } }
    }

    // ---- Requirements ----------------------------------------------------

    requirement def <'W'> WidthLimit {
        doc /* The board is no wider than the limit. */
        subject board : Board;
        attribute limit : LengthValue;
        require constraint { board.width <= limit }
    }
    requirement def <'T'> ThicknessLimit {
        doc /* The board is no thicker than the limit. */
        subject board : Board;
        attribute limit : LengthValue;
        require constraint { board.thickness <= limit }
    }

    requirement envelope {
        doc /* The board fits its enclosure. */
        subject board : Board;
        requirement <'1'> boardWidth : WidthLimit {
            doc /* At most 60 mm wide. */
            attribute :>> limit = 60 [mm];
        }
        requirement <'2'> boardThickness : ThicknessLimit {
            doc /* At most 1.6 mm thick. */
            attribute :>> limit = 1.6 [mm];
        }
    }

    satisfy envelope.boardWidth by feather;
    satisfy envelope.boardThickness by feather;
}
