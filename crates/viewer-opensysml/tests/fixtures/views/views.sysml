package V {
    private import Views::*;
    private import StandardViewDefinitions::*;
    part def Engine;
    part def Wheel;
    part def Vehicle {
        part engine : Engine;
        part wheel : Wheel[4];
    }
    part vehicle : Vehicle {
        part body;
    }
    package Defs {
        part def Frame;
        attribute def Mass;
    }
    view def TreeView {
        render asTreeDiagram;
    }
    view 'vehicle tree' {
        expose vehicle::**;
        render asTreeDiagram;
    }
    view wiring {
        expose Vehicle;
        render asInterconnectionDiagram;
    }
    view inherited : TreeView {
        expose Vehicle::*;
    }
    view overview : GeneralView {
        expose Defs::*;
        expose Engine;
    }
    view table {
        expose Defs::*;
        render asElementTable;
    }
}
