use actix_web_dev::error::{
    Result,
    ErrorType,
    ApiError,
};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use xlsxwriter::*;
use diesel::pg::PgConnection;

use diesel::r2d2::ConnectionManager;
use topgo::schema::couriers;
use diesel::sql_types::{
    Bigint,
    Varchar,
    Integer,
    Time,
    Double,
    Bool,
    Timestamp,
    Date,
    Nullable,
};
use topgo::enum_types::*;

#[macro_use]
extern crate diesel;

#[derive(Serialize,Deserialize,Clone,QueryableByName)]
pub struct CourierXLS {
    #[sql_type="Bigint"]
    pub courier_id: i64,
    #[sql_type="Date"]
    pub session_day: chrono::NaiveDate,
    #[sql_type="Time"]
    pub start_time: chrono::NaiveTime,
    #[sql_type="Nullable<Time>"]
    pub end_real_time: Option<chrono::NaiveTime>,
    #[sql_type="Bigint"]
    pub order_id: i64,
    #[sql_type="Timestamp"]
    pub take_datetime: chrono::NaiveDateTime,
    #[sql_type="Orderstatus"]
    pub order_status: OrderStatus,
    #[sql_type="Varchar"]
    pub details: String,
    #[sql_type="Bool"]
    pub is_big_order: bool,
    #[sql_type="Time"]
    pub cooking_time: chrono::NaiveTime,
    #[sql_type="Timestamp"]
    pub delivery_datetime: chrono::NaiveDateTime,
    #[sql_type="Bigint"]
    pub courier_salary: i64,
    #[sql_type="Bigint"]
    pub order_price: i64,
    #[sql_type="Varchar"]
    pub delivery_address: String,
    #[sql_type="Varchar"]
    pub client_comment: String,
    #[sql_type="Paymethod"]
    pub method: PayMethod,
}

#[derive(Serialize,Deserialize,Clone,QueryableByName)]
pub struct CourierXLSTotal {
    #[sql_type="Bigint"]
    pub courier_id: i64,
    #[sql_type="Varchar"]
    pub phone: String,
    #[sql_type="Varchar"]
    pub name: String,
    #[sql_type="Varchar"]
    pub surname: String,
    #[sql_type="Varchar"]
    pub patronymic: String,
    #[sql_type="Date"]
    pub session_day: chrono::NaiveDate,
    #[sql_type="Time"]
    pub start_time: chrono::NaiveTime,
    #[sql_type="Nullable<Time>"]
    pub end_real_time: Option<chrono::NaiveTime>,
    #[sql_type="Bigint"]
    pub order_id: i64,
    #[sql_type="Timestamp"]
    pub take_datetime: chrono::NaiveDateTime,
    #[sql_type="Orderstatus"]
    pub order_status: OrderStatus,
    #[sql_type="Varchar"]
    pub details: String,
    #[sql_type="Bool"]
    pub is_big_order: bool,
    #[sql_type="Time"]
    pub cooking_time: chrono::NaiveTime,
    #[sql_type="Timestamp"]
    pub delivery_datetime: chrono::NaiveDateTime,
    #[sql_type="Bigint"]
    pub courier_salary: i64,
    #[sql_type="Bigint"]
    pub delivery_cost: i64,
    #[sql_type="Bigint"]
    pub order_price: i64,
    #[sql_type="Varchar"]
    pub delivery_address: String,
    #[sql_type="Varchar"]
    pub client_comment: String,
    #[sql_type="Varchar"]
    pub client_phone: String,
    #[sql_type="Paymethod"]
    pub method: PayMethod,
}

#[derive(Serialize,Deserialize,Clone,QueryableByName)]
pub struct RestaurantXLS {
    #[sql_type="Bigint"]
    pub restaurant_id: i64,
    #[sql_type="Bigint"]
    pub order_id: i64,
    #[sql_type="Timestamp"]
    pub take_datetime: chrono::NaiveDateTime,
    #[sql_type="Orderstatus"]
    pub order_status: OrderStatus,
    #[sql_type="Varchar"]
    pub details: String,
    #[sql_type="Bool"]
    pub is_big_order: bool,
    #[sql_type="Time"]
    pub cooking_time: chrono::NaiveTime,
    #[sql_type="Timestamp"]
    pub delivery_datetime: chrono::NaiveDateTime,
    #[sql_type="Bigint"]
    pub order_price: i64,
    #[sql_type="Varchar"]
    pub delivery_address: String,
    #[sql_type="Varchar"]
    pub client_comment: String,
    #[sql_type="Varchar"]
    pub client_phone: String,
    #[sql_type="Paymethod"]
    pub method: PayMethod,
}

#[derive(Serialize,Deserialize,Clone,QueryableByName)]
pub struct RestaurantXLSTotal {
    #[sql_type="Bigint"]
    pub restaurant_id: i64,
    #[sql_type="Varchar"]
    pub name: String,
    #[sql_type="Varchar"]
    pub phone: String,
    #[sql_type="Varchar"]
    pub address: String,
    #[sql_type="Bigint"]
    pub order_id: i64,
    #[sql_type="Timestamp"]
    pub take_datetime: chrono::NaiveDateTime,
    #[sql_type="Orderstatus"]
    pub order_status: OrderStatus,
    #[sql_type="Varchar"]
    pub details: String,
    #[sql_type="Bool"]
    pub is_big_order: bool,
    #[sql_type="Time"]
    pub cooking_time: chrono::NaiveTime,
    #[sql_type="Timestamp"]
    pub delivery_datetime: chrono::NaiveDateTime,
    #[sql_type="Bigint"]
    pub order_price: i64,
    #[sql_type="Varchar"]
    pub delivery_address: String,
    #[sql_type="Varchar"]
    pub client_comment: String,
    #[sql_type="Varchar"]
    pub client_phone: String,
    #[sql_type="Paymethod"]
    pub method: PayMethod,
}

fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let couriers_ids: Vec<i64> = couriers::table
        .filter(couriers::is_deleted.eq(false))
        .select(couriers::id)
        .get_results::<i64>(&pool.get().unwrap()).unwrap();
    for id in couriers_ids {
        let rows = diesel::sql_query("select * from courier_exel WHERE courier_id=$1")
            .bind::<Bigint,_>(id)
            .get_results::<CourierXLS>(&pool.get().unwrap()).unwrap();
        let workbook = Workbook::new("summary/test.xlsx");
        let mut sheet1 = workbook.add_worksheet(None).unwrap();
            sheet1.write_string(0, 0, "день сессии", None).unwrap();
            sheet1.write_string(0, 1, "время начала сессии", None).unwrap();
            sheet1.write_string(0, 2, "время конца сессии", None).unwrap();
            sheet1.write_string(0, 3, "номер заказа", None).unwrap();
            sheet1.write_string(0, 4, "забрано", None).unwrap();
            sheet1.write_string(0, 5, "статус заказа", None).unwrap();
            sheet1.write_string(0, 6, "детали заказа", None).unwrap();
            sheet1.write_string(0, 7, "большой заказ", None).unwrap();
            sheet1.write_string(0, 8, "время готовки", None).unwrap();
            sheet1.write_string(0, 9, "доставлено", None).unwrap();
            sheet1.write_string(0, 10, "стоимость заказа", None).unwrap();
            sheet1.write_string(0, 11, "адрес доставки", None).unwrap();
            sheet1.write_string(0, 12, "комментарий клиента", None).unwrap();
            sheet1.write_string(0, 13, "способ оплаты", None).unwrap(); 
            for i in 1..=rows.len() { 
                let i = i as u32;
                let ind = i as usize;
                sheet1.write_string(i, 0, &rows[ind].session_day.to_string(), None).unwrap();
                sheet1.write_string(i, 1, &rows[ind].start_time.to_string(), None).unwrap();
                sheet1.write_string(i, 2, &rows[ind].end_real_time.map(|v|{
                    v.to_string()
                }).unwrap_or("на момент создания отчета сессия не была закончена".to_string()),
                None).unwrap();
                sheet1.write_number(i, 3, rows[ind].order_id as f64, None).unwrap();
                sheet1.write_number(i, 4, (rows[ind].courier_salary / 100) as f64, None).unwrap();
                sheet1.write_string(i, 5, match rows[ind].order_status {
                    OrderStatus::Success => "успешно доставлено",
                    OrderStatus::FailureByRestaurant => "отменено по вине ресторана",
                    OrderStatus::FailureByCourier => "отменено по вине курьера",
                    _ => panic!(),
                }, None).unwrap();
                sheet1.write_string(i, 6, &rows[ind].details, None).unwrap();
                sheet1.write_boolean(i, 7, rows[ind].is_big_order, None).unwrap();
                sheet1.write_string(i, 8, &rows[ind].cooking_time.to_string(), None).unwrap();
                sheet1.write_string(i, 9, &rows[ind].delivery_datetime.to_string(), None).unwrap();
                sheet1.write_number(i, 10, (rows[ind].order_price / 100) as f64, None).unwrap();
                sheet1.write_string(i, 11, &rows[ind].delivery_address, None).unwrap();
                sheet1.write_string(i, 12, &rows[ind].client_comment, None).unwrap();
                sheet1.write_string(i, 13, match rows[ind].method {
                    PayMethod::Cash =>"наличными",
                    PayMethod::Card =>"картой",
                    PayMethod::AlreadyPayed =>"оплачено заранее",
                }, None).unwrap();
        }
        workbook.close().expect("workbook can be closed");

    }
}
