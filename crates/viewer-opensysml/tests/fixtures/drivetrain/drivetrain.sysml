package D {
    port def FuelPort;
    port def TorquePort;
    port def StatusPort;
    port def PowerPort;

    interface def FuelInterface {
        end supply : FuelPort;
        end demand : ~FuelPort;
    }

    part def FuelTank {
        out port fuelOut : FuelPort;
    }
    part def Engine {
        in port fuelIn : ~FuelPort;
        in port power : ~PowerPort;
        port status : StatusPort;
        out port torqueOut : TorquePort;
    }
    part def Transmission {
        in port torqueIn : ~TorquePort;
        out port torqueOut : TorquePort;
    }
    part def Driveshaft {
        in port torqueIn : ~TorquePort;
        out port torqueOut : TorquePort;
    }
    part def Differential {
        in port torqueIn : ~TorquePort;
        out port left : TorquePort;
        out port right : TorquePort;
    }
    part def Wheel {
        in port hub : ~TorquePort;
    }
    part def AxleAssembly {
        in port torqueIn : ~TorquePort;
        part differential : Differential;
        part leftWheel : Wheel;
        part rightWheel : Wheel;
        connection drive connect torqueIn to differential.torqueIn;
        connection toLeft connect differential.left to leftWheel.hub;
        connection toRight connect differential.right to rightWheel.hub;
    }
    part def Controller {
        port status : ~StatusPort;
    }

    part def Vehicle {
        in port power : ~PowerPort;
        part fuelTank : FuelTank;
        part engine : Engine;
        part transmission : Transmission;
        part driveshaft : Driveshaft;
        part rearAxleAssembly : AxleAssembly;
        part controller : Controller;
        interface fuelLine : FuelInterface connect supply ::> fuelTank.fuelOut to demand ::> engine.fuelIn;
        connection torque connect engine.torqueOut to transmission.torqueIn;
        connection shaft connect transmission.torqueOut to driveshaft.torqueIn;
        connection axle connect driveshaft.torqueOut to rearAxleAssembly.torqueIn;
        connection leftDrive connect driveshaft.torqueOut to rearAxleAssembly.leftWheel.hub;
        connection engineStatus connect engine.status to controller.status;
        connection supply connect power to engine.power;
    }
}
