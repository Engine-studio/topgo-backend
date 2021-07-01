use actix_web_dev::error::{
    Result,
    ErrorType,
    ApiError,
};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::schema::paymentstatus;
use serde_json::Value;
use diesel::sql_types::{
    Array,
    Json,
    BigInt,
    Varchar,
    Nullable,
    Timestamp,
    BigSerial,
};

#[derive(Serialize,Deserialize,Clone,QueryableByName,Debug)]
pub struct Payment {
    #[sql_type="BigSerial"]
    pub payment_id: i64,
    #[sql_type="BigSerial"]
    pub total_summ: i64,
    #[sql_type="Varchar"]
    pub status: String,
    #[sql_type="Varchar"]
    pub comment: String,
    #[sql_type="Varchar"]
    pub ord_type: String,
    #[sql_type="Array<Json>"]
    pub paintings: Vec<Value>,
    #[sql_type="Timestamp"]
    pub start_date: chrono::NaiveDateTime,
    #[sql_type="Nullable<Timestamp>"]
    pub end_date: Option<chrono::NaiveDateTime>,
    #[sql_type="BigSerial"]
    pub customer_id: i64,
    #[sql_type="Varchar"]
    pub customer_email: String,
    #[sql_type="Varchar"]
    pub customer_phone: String,
    #[sql_type="Varchar"]
    pub customer_name: String,
    #[sql_type="Varchar"]
    pub customer_last_name: String,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct NewPayment {
    pub products: Vec<i64>,
    pub customer_id: i64,
    pub order_type: String,
    pub comment: String,
}

impl Payment {
    pub async fn new(
        instance: &NewPayment, 
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::sql_query("call AddPayment ($1,$2,$3,$4);")
            .bind::<Array<BigInt>,_>(&instance.products)
            .bind::<BigInt,_>(instance.customer_id)
            .bind::<Varchar,_>(&instance.order_type)
            .bind::<Varchar,_>(&instance.comment)
            .execute(conn)?;
        Ok(())
    }

    pub async fn list_orders(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = diesel::sql_query("select * from pays where status!='delivered';")
            .get_results(conn)?;
        println!("view: {:?}",r);
        Ok(r)
    }

    pub async fn list_orders_by_id(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        diesel::sql_query("select * from pays where customer_id=$1;")
            .bind::<BigInt,_>(id)
            .get_results(conn)
            .map_err(|e| e.into())
    }

    pub async fn to_delivered(
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::update(paymentstatus::table
            .filter(paymentstatus::id.eq(id)))
            .set(paymentstatus::order_status.eq("delivered"))
            .execute(conn)?;
        Ok(())
    }

    pub async fn rollback(
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::sql_query("call RmPayment($1);")
            .bind::<BigInt,_>(id)
            .execute(conn)?;
        Ok(())
    }
}
