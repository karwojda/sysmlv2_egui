package Tonne {
    private import ISQ::*;
    private import SI::*;

    part def Box {
        attribute mass : MassValue;
        assert constraint underQuarterTonne { mass <= 0.25 [tonne] }   // expected: holds (200 kg)
        assert constraint underQuarterTonneByT { mass <= 0.25 [t] }    // expected: holds; `t` is unresolved
    }
    part box : Box { attribute :>> mass = 200 [kg]; }
}
