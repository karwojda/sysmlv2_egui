package E {
    private import ScalarValues::*;
    private import SI::*;
    part def V {
        attribute a : Real;
        attribute b : Real;
        attribute ok : Boolean;
        attribute m :> ISQ::mass;
    }
    constraint def Cap {
        in x : Real;
        x <= 5.0
    }
    requirement def R {
        subject v : V;
        attribute limit : Real;
        require constraint { v.a <= limit }
        require constraint { (v.a + v.b) * 2 > 10 and not v.ok }
        require constraint { v.a == 3 or v.b != -1.5 implies v.ok }
        require constraint { v.m <= 1650 [kg] }
        require constraint { "x" == "y" xor true }
        require constraint { Cap(v.a) }
        require constraint { v.a - (v.b - 1) >= v.b ** 2 }
    }
}
