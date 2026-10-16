package F {
    private import ScalarValues::*;
    item def Fuel;
    port def InnerPort;
    port def FuelPort {
        out item fuel : Fuel;
        port inner : InnerPort;
    }
    part def Tank {
        out port fuelOut : FuelPort;
        attribute level : Real;
    }
    part def Engine {
        in port fuelIn : ~FuelPort;
        attribute fuelLevel : Real;
    }
    part def Driver;
    part def Vehicle {
        part tank : Tank;
        part engine : Engine;
        ref part driver : Driver;
        flow of Fuel from tank.fuelOut.fuel to engine.fuelIn.fuel;
        flow fuelFlow of Fuel from tank.fuelOut to engine.fuelIn;
        bind tank.level = engine.fuelLevel;
        bind tank.fuelOut = engine.fuelIn;
        connect tank.fuelOut.inner to engine.fuelIn.inner;
    }
}
