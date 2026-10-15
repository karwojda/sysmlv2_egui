package Vehicles {
    part def Vehicle;
    part def Car :> Vehicle;
    part def Truck :> Vehicle;
    part def ElectricCar :> Car;
    part def Engine;
    requirement def TopSpeed;
    package Parts {
        part def Wheel;
    }
}
