use serde::{Serialize,Deserialize};
#[derive(Debug,Clone,DbEnum,Serialize,Deserialize)]
#[DieselType = "Paymethod"]
#[DbValueStyle ="PascalCase"]
pub enum PayMethod {
    Cash,  // All variants must be fieldless
    Card,
    AlreadyPayed,
}

#[derive(Debug,Clone,DbEnum,Serialize,Deserialize)]
#[DieselType = "Orderstatus"]
#[DbValueStyle ="PascalCase"]
pub enum OrderStatus {
    CourierFinding,
    CourierConfirmation,
    Cooking,
    ReadyForDelivery,
    Delivering,
    Delivered,
    FailureByCourier,
    FailureByRestaurant,
    Success,
}

#[derive(Debug,Clone,DbEnum,Serialize,Deserialize)]
#[DieselType = "Transporttype"]
#[DbValueStyle ="PascalCase"]
pub enum TransportType {
    Car,  // All variants must be fieldless
    Feet,
    Bicycle,
}
