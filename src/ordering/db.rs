use actix_web_dev::error::{
    Result,
    ErrorType,
    ApiError,
};
use serde::{Serialize, Deserialize};
use diesel::{prelude::*, query_dsl::InternalJoinDsl};
use diesel::pg::PgConnection;
use super::*;

use crate::schema::{
    orders,
    couriers_approvals,
    courier_rating,
    notifications,
    sessions,
};
use diesel::sql_types::{
    Bigint,
    Varchar,
    Integer,
    Time,
    Double,
    Bool,
};
use crate::enum_types::*;

#[derive(Serialize,Deserialize,Clone,Insertable)]
#[table_name="courier_rating"]
pub struct CourierRating {
    pub courier_id: i64,
    pub order_id: i64,
    pub look: i16,
    pub politeness: i16,
}

impl CourierRating {
    pub async fn new (
        data: &Self, 
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::insert_into(courier_rating::table)
            .values(data)
            .execute(conn)?;
        Ok(())
    }
}
#[derive(Serialize,Deserialize,Clone,QueryableByName,Insertable)]
#[table_name="notifications"]
pub struct Notification {
    #[sql_type="Varchar"]
    pub title: String,
    #[sql_type="Varchar"]
    pub message: String,
}

impl Notification {

    pub async fn new (
        data: &Notification, 
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::insert_into(notifications::table)
            .values(data)
            .execute(conn)?;
        Ok(())
    }

    pub async fn get (
        courier_id: i64, 
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = diesel::sql_query("select * from get_notification($1);")
            .bind::<Bigint,_>(courier_id)
            .get_results::<Self>(conn)?;
        Ok(r)
    }
}

#[derive(Serialize,Deserialize,Clone,QueryableByName)]
pub struct SuggestedOrders {
    #[sql_type="Bigint"]
    pub order_id: i64,
    #[sql_type="Varchar"]
    pub restaurant_name: String,
    #[sql_type="Varchar"]
    pub restaurant_address: String,
    #[sql_type="Double"]
    pub restaurant_lat: f64,
    #[sql_type="Double"]
    pub restaurant_lng: f64,
    #[sql_type="Double"]
    pub destination_lat: f64,
    #[sql_type="Double"]
    pub destination_lng: f64,
    #[sql_type="Varchar"]
    pub destination_address: String,
    #[sql_type="Time"]
    pub cooking_time: chrono::NaiveTime,
    #[sql_type="Paymethod"]
    pub payment_method: PayMethod,
    #[sql_type="Bigint"]
    pub pay_ammount: i64,
    #[sql_type="Double"]
    pub distance: f64,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct OrderRequest {
    pub courier_id: i64,
    pub lat: f64,
    pub lng: f64,
}

#[derive(Serialize,Deserialize,Clone,Queryable)]
pub struct Orders {
    pub id: i64,
    pub restaurant_id: Option<i64>,
    pub session_id: Option<i64>,
    pub details: String,
    pub is_big_order: bool,
    pub delivery_address: String,
    pub address_lat: f64,
    pub address_lng: f64,
    pub method: PayMethod,
    pub courier_share: i64,
    pub order_price: i64,
    pub cooking_time: chrono::NaiveTime, 
    pub client_phone: String,
    pub client_comment: String,
    pub status: OrderStatus,
    pub finalize_comment: Option<String>,
    pub finalize_datetime: Option<chrono::NaiveDateTime>,
    pub take_datetime: Option<chrono::NaiveDateTime>,
    pub delivery_datetime: Option<chrono::NaiveDateTime>,
    pub creation_datetime: chrono::NaiveDateTime,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct Finalization {
    pub order_id: i64,
    pub is_success: bool,
    pub courier_fault: bool,
    pub comment: String,
}

#[derive(Serialize,Deserialize,Clone,Insertable)]
#[table_name="orders"]
pub struct NewOrder {
    pub restaurant_id: i64,
    pub details: String,
    pub is_big_order: bool,
    pub delivery_address: String,
    pub address_lat: f64,
    pub address_lng: f64,
    pub method: PayMethod,
    pub courier_share: i64,
    pub order_price: i64,
    pub cooking_time: chrono::NaiveTime, 
    pub client_phone: String,
    pub client_comment: String,
}

impl Orders {
    pub async fn get_suggested(
        data: &OrderRequest, 
        conn: &PgConnection,
    ) -> Result<Vec<SuggestedOrders>> {
        let r = diesel::sql_query("select * from find_suitable_orders($1,$2,$3);")
            .bind::<Double,_>(data.lat)
            .bind::<Double,_>(data.lng)
            .bind::<Bigint,_>(data.courier_id)
            .get_results::<SuggestedOrders>(conn)?;
        Ok(r)
    }
    pub async fn create_order (
        data: &NewOrder, 
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::insert_into(orders::table)
            .values(data)
            .execute(conn)?;
        Ok(())
    }

    pub async fn get_orders_by_session_id (
        session_id: i64,
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = orders::table
            .filter(orders::session_id.eq(session_id))
            .get_results::<Self>(conn)?;
        Ok(r)
    }

    pub async fn get_orders_by_courier_id (
        id: i64,
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = orders::table
            .inner_join(sessions::table.on(orders::session_id.eq(sessions::id.nullable())))
            .filter(sessions::courier_id.eq(id))
            .select(orders::id)
            .get_results::<i64>(conn)?;
        let r = orders::table
            .filter(orders::id.eq_any(r))
            .get_results::<Self>(conn)?;
        Ok(r)
    }
    
    pub async fn get_orders_by_rest_id (
        id: i64,
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = orders::table
            .filter(orders::restaurant_id.eq(id))
            .get_results::<Self>(conn)?;
        Ok(r)
    }
    
    pub async fn take_order (
        order_id: i64,
        courier_id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::sql_query("select * from take_order($1,$2);")
            .bind::<Bigint,_>(order_id)
            .bind::<Bigint,_>(courier_id)
            .execute(conn)?;
        Ok(())
    }

    pub async fn refuse_order (
        order_id: i64,
        courier_id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::sql_query("select * from refuse_order($1,$2);")
            .bind::<Bigint,_>(order_id)
            .bind::<Bigint,_>(courier_id)
            .execute(conn)?;
        Ok(())
    }

    pub async fn pick_order (
        order_id: i64,
        courier_id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::sql_query("select * from pick_order($1,$2);")
            .bind::<Bigint,_>(order_id)
            .bind::<Bigint,_>(courier_id)
            .execute(conn)?;
        Ok(())
    }

    pub async fn set_ready_for_delivery (
        order_id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::sql_query("select * from set_ready_for_delivery($1);")
            .bind::<Bigint,_>(order_id)
            .execute(conn)?;
        Ok(())
    }

    pub async fn set_delivered (
        order_id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::sql_query("select * from set_delived($1);")
            .bind::<Bigint,_>(order_id)
            .execute(conn)?;
        Ok(())
    }

    pub async fn finalize_order (
        data: &Finalization,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::sql_query("select * from finalize_order($1,$2,$3,$4);")
            .bind::<Bigint,_>(data.order_id)
            .bind::<Bool,_>(data.is_success)
            .bind::<Bool,_>(data.courier_fault)
            .bind::<Varchar,_>(&data.comment)
            .execute(conn)?;
        Ok(())
    }
}

#[derive(Serialize,Deserialize,Clone,Insertable)]
#[table_name="sessions"]
pub struct NewSession {
   pub courier_id: i64,
   pub start_time: chrono::NaiveTime,
   pub end_time: chrono::NaiveTime,
   pub has_terminal: bool,
   pub transport: TransportType,
}

#[derive(Serialize,Deserialize,Clone,Queryable)]
pub struct Sessions {
   pub id: i64,
   pub courier_id: i64,
   pub start_time: chrono::NaiveTime,
   pub end_time: chrono::NaiveTime,
   pub session_day: chrono::NaiveDate,
   pub end_real_time: Option<chrono::NaiveTime>,
   pub has_terminal: bool,
   pub transport: TransportType,
}

impl Sessions {
    pub async fn new(
        data: &NewSession,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::insert_into(sessions::table)
            .values(data)
            .execute(conn)?;
        Ok(())
    }

    pub async fn finish(
        courier_id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::sql_query("select * from end_session($1);")
            .bind::<Bigint,_>(courier_id)
            .execute(conn)?;
        Ok(())
    }

    pub async fn get_by_courier(
        courier_id: i64,
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = sessions::table
            .filter(sessions::courier_id.eq(courier_id))
            .get_results::<Self>(conn)?;
        Ok(r)
    }
}
