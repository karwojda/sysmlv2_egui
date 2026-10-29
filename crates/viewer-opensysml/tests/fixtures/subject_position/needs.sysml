package Needs {
    part def Rider;
    part def RearUnit;
    requirement def StakeholderNeed {
        subject unit : RearUnit;
    }
    requirement riderNeeds : StakeholderNeed {
        stakeholder rider : Rider;
    }
}
