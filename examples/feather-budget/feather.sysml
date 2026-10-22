package FeatherBudget {
    doc /* The RoyalBlue54L Feather, a board designed in KiCad (ecad/), and
         * the budgets it must meet: an envelope, a power budget, a cost
         * limit and a part-count limit. Its dimensions come from the KiCad
         * layout, its part count and the sums of its parts' Power and Price
         * fields from the schematic, as Design Values; the values written
         * here are the Model's pre-design estimates, used only where KiCad
         * can't be read. */

    private import ISQ::*;
    private import SI::*;
    private import ScalarValues::*;
    private import EngineeringTools::*;

    part def Board {
        attribute width : LengthValue;
        attribute height : LengthValue;
        attribute thickness : LengthValue;
        attribute area : AreaValue;
        attribute power : PowerValue;
        /* In US dollars, which SysML's quantity library can't express. */
        attribute price : Real;
        attribute componentCount : Natural;
    }

    part feather : Board {
        @ToolLink { tool = "KiCad"; file = "ecad/RoyalBlue54L-Feather.kicad_pro"; object = "board"; }
        // A Feather is 2.0 x 0.9 inches on 1.6 mm FR-4.
        attribute :>> width = 50.8 [mm] { @ToolProperty { name = "Width"; } }
        attribute :>> height = 22.86 [mm] { @ToolProperty { name = "Height"; } }
        attribute :>> thickness = 1.6 [mm] { @ToolProperty { name = "Thickness"; } }
        attribute :>> area = 1161.3 [mm^2] { @ToolProperty { name = "Area"; } }
        // Estimates made before the schematic was drawn.
        attribute :>> power = 0.3 [W] { @ToolProperty { name = "Power"; } }
        attribute :>> price = 10.0 { @ToolProperty { name = "Price"; } }
        attribute :>> componentCount = 40 { @ToolProperty { name = "ComponentCount"; } }
    }

    // ---- Requirements ----------------------------------------------------

    requirement def <'WL'> WidthLimit {
        doc /* The board is no wider than the limit. */
        subject board : Board;
        attribute limit : LengthValue;
        require constraint { board.width <= limit }
    }
    requirement def <'TL'> ThicknessLimit {
        doc /* The board is no thicker than the limit. */
        subject board : Board;
        attribute limit : LengthValue;
        require constraint { board.thickness <= limit }
    }
    requirement def <'PB'> PowerBudget {
        doc /* The board draws no more power than the budget. */
        subject board : Board;
        attribute budget : PowerValue;
        require constraint { board.power <= budget }
    }
    requirement def <'CL'> CostLimit {
        doc /* The board's parts cost no more than the limit. */
        subject board : Board;
        attribute limit : Real;
        require constraint { board.price <= limit }
    }
    requirement def <'NL'> PartCountLimit {
        doc /* The board has no more placed parts than the limit. */
        subject board : Board;
        attribute limit : Natural;
        require constraint { board.componentCount <= limit }
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

    requirement budgets {
        doc /* The board stays within its power, cost and assembly budgets. */
        subject board : Board;
        requirement <'3'> boardPower : PowerBudget {
            doc /* At most 500 mW. */
            attribute :>> budget = 0.5 [W];
        }
        requirement <'4'> boardCost : CostLimit {
            doc /* At most 15 US dollars of parts. */
            attribute :>> limit = 15.0;
        }
        requirement <'5'> boardPartCount : PartCountLimit {
            doc /* At most 60 placed parts. */
            attribute :>> limit = 60;
        }
    }

    satisfy envelope.boardWidth by feather;
    satisfy envelope.boardThickness by feather;
    satisfy budgets.boardPower by feather;
    satisfy budgets.boardCost by feather;
    satisfy budgets.boardPartCount by feather;
}
