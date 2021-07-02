#[derive(Debug,DbEnum)]
#[DieselType = "Paymethod"]
pub enum PayMethod {
    Cash,  // All variants must be fieldless
    Card,
    AlreadyPayed,
}

#[derive(Debug,DbEnum)]
#[DieselType = "Orderstatus"]
pub enum OrderStatus {
    CourierFinding,
    CourierConfirmation,
    Cooking,
    ReadyForDelivery,
    Delivering,
    FailureByCourier,
    FailureByInnocent,
    Success,
}

#[derive(Debug,DbEnum)]
#[DieselType = "Transporttype"]
pub enum TransportType {
    Car,  // All variants must be fieldless
    Feet,
    Bicycle,
}
