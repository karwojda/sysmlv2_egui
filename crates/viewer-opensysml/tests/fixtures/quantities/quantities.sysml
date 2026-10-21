package Q {
    private import ISQ::*;
    private import SI::*;

    part engine {
        attribute volume : VolumeValue = 1.0e6 [mm^3];
        attribute density : MassDensityValue = 7850 [kg/m^3];
        // 1.0e6 mm³ × 7850 kg/m³ is exactly 157/20 kg.
        attribute mass : MassValue = volume * density;
        attribute doubled : VolumeValue = volume * 2;
        attribute perVolume : MassDensityValue = mass / volume;
        attribute sum : VolumeValue = volume + volume;
        attribute huge = 10000000000000000000000 * 3;
    }
}
