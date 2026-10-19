package NestedSatisfy {
    private import ScalarValues::*;

    part def Vehicle {
        attribute mass : Real;
    }
    requirement def MassLimit {
        subject v : Vehicle;
        require constraint { v.mass <= 1600.0 }
    }
    requirement byForm : MassLimit;
    requirement nestedForm : MassLimit;

    // The same part, with the same values, claims both requirements.
    part car : Vehicle {
        attribute :>> mass = 1500.0;
        satisfy nestedForm;          // satisfier implied: car (SysML v2 Part 1, 7.21.4)
    }
    satisfy byForm by car;           // satisfier named
}
