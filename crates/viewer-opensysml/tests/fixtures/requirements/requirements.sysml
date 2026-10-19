package Req {
    private import ScalarValues::*;
    part def Vehicle {
        attribute mass : Real;
        attribute range : Real;
        attribute cost : Real;
    }
    requirement def <'R1'> MassLimit {
        doc /* The vehicle shall weigh no more than the limit. */
        subject v : Vehicle;
        attribute limit : Real;
        require constraint { v.mass <= limit }
    }
    requirement def RangeTarget {
        subject v : Vehicle;
        require constraint { v.range >= 500.0 }
    }
    requirement def CostCap {
        subject v : Vehicle;
        require constraint { v.cost <= 10.0 }
    }
    requirement vehicleSpec {
        doc /* Top-level vehicle requirements. */
        subject v : Vehicle;
        requirement <'1'> massReq : MassLimit {
            attribute :>> limit = 2000.0;
        }
        requirement <'2'> rangeReq : RangeTarget;
        requirement <'3'> costReq : CostCap;
        requirement <'4'> spareReq : RangeTarget;
        requirement <'5'> carMassReq : MassLimit {
            attribute :>> limit = 1600.0;
        }
    }
    part vehicle : Vehicle {
        attribute :>> mass = 1800.0;
        attribute :>> range = 420.0;
    }
    part car : Vehicle {
        attribute :>> mass = 1500.0;
        satisfy vehicleSpec.carMassReq;
    }
    satisfy vehicleSpec.massReq by vehicle;
    satisfy vehicleSpec.rangeReq by vehicle;
    satisfy vehicleSpec.costReq by vehicle;
}
