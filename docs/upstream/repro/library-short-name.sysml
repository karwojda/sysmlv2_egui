package LibraryShortName {
    private import ISQ::*;
    private import SI::*;

    part def Vehicle {
        attribute <m> mass :> ISQ::mass;
    }
    requirement def MassLimit {
        subject v : Vehicle;
        require constraint { v.m <= 1650 [kg] }
    }
}
