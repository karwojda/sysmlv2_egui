package Structure {
    attribute def Fuel;
    attribute def Mass;
    port def FuelPort {
        out attribute fuel : Fuel;
    }
    interface def FuelInterface {
        end supply : FuelPort;
        end demand : ~FuelPort;
    }
    connection def FuelLine {
        end a : FuelPort;
        end b : FuelPort;
    }
    part def Engine {
        doc /* Turns fuel into motion. */
        attribute mass : Mass;
        in port fuelIn : ~FuelPort;
    }
    part def Tank {
        out port fuelOut : FuelPort;
    }
    part def Vehicle {
        part engine : Engine [1];
        part tanks : Tank [1..2];
        attribute mass : Mass;
        connection line : FuelLine connect tanks.fuelOut to engine.fuelIn;
        interface iface : FuelInterface connect supply ::> tanks.fuelOut to demand ::> engine.fuelIn;
    }
    part vehicle : Vehicle;
    part spare : Engine [0..*];
    connection link connect vehicle to spare;
}
