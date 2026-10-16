package M {
    metadata def Safety;
    metadata def Supplier;
    part def Brake {
        @Safety;
    }
    part def Seat {
        metadata Supplier;
    }
    #Safety part def Airbag;
    part def Radio;
    part def Vehicle {
        #Safety #Supplier part airbag : Airbag;
        part radio : Radio;
    }
}
