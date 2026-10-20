package Car {
    private import ScalarValues::*;
    part def Engine { attribute mass : Real; }
    part def Vehicle {
        part engine : Engine;
        attribute totalMass : Real = engine.mass + 1000.0;
        assert constraint engineLight { engine.mass <= 175.0 }
    }
    requirement def MassLimit {
        subject v : Vehicle;
        require constraint { v.engine.mass <= 175.0 }
    }
    requirement massReq : MassLimit;
    part vehicle : Vehicle {
        part :>> engine { attribute :>> mass = 180.0; }
    }
    satisfy massReq by vehicle;
    analysis def MassRollup {
        subject v : Vehicle;
        in extra : Real = 0.0;
        return total : Real = v.totalMass + extra;
    }
    analysis rollup : MassRollup { subject v = vehicle; }
}
