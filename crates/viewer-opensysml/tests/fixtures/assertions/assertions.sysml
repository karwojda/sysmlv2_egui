package C {
    private import ScalarValues::*;

    constraint def MaxRatio {
        in numerator : Real;
        in denominator : Real;
        in limit : Real;
        numerator / denominator <= limit
    }
    part def Tank {
        attribute fuelMass : Real;
        attribute fuelMassMax : Real;
        assert constraint fits { fuelMass <= fuelMassMax }
    }
    part def Vehicle {
        attribute mass : Real;
        attribute power : Real;
        part tank : Tank;
    }
    part vehicle : Vehicle {
        attribute :>> mass = 1400.0;
        attribute :>> power = 90.0;
        part :>> tank {
            attribute :>> fuelMass = 50.0;
            attribute :>> fuelMassMax = 45.0;
        }
        assert constraint light { mass <= 1500.0 }
        constraint heavy { mass > 2000.0 }
        assert constraint powerToWeight : MaxRatio {
            in numerator = mass;
            in denominator = power;
            in limit = 12.0;
        }
    }
}
