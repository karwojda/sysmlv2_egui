package Board {
    private import ScalarValues::*;
    private import ISQ::*;
    private import SI::*;
    private import EngineeringTools::*;

    part def Pcb {
        attribute power : PowerValue;
        attribute price : Real;
    }

    // The estimates from before the board was designed hold both budgets.
    part board : Pcb {
        @ToolLink { tool = "KiCad"; file = "ecad/board.kicad_pro"; object = "board"; }
        attribute :>> power = 0.4 [W] { @ToolProperty { name = "Power"; } }
        attribute :>> price = 9.5 { @ToolProperty { name = "Price"; } }
    }

    requirement def PowerBudget {
        subject b : Pcb;
        require constraint { b.power <= 0.5 [W] }
    }
    requirement powerBudget : PowerBudget;
    satisfy powerBudget by board;

    requirement def CostLimit {
        subject b : Pcb;
        require constraint { b.price <= 12.0 }
    }
    requirement costLimit : CostLimit;
    satisfy costLimit by board;
}
