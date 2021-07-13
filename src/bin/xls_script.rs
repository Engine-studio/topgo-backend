use std::sync::Arc;

use actix_web_dev::error::{
    Result,
    ErrorType,
    ApiError,
};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use xlsxwriter::*;
use diesel::pg::PgConnection;
use uuid::Uuid;

use diesel::r2d2::ConnectionManager;
use topgo::schema::{
    couriers,
    restaurants,
};
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

use topgo::schema::{
    couriers_xls_reports,
    couriers_for_curators_xls_reports,
    restaurants_xls_reports,
    restaurants_for_curators_xls_reports,
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

use tokio;
use tokio_cron_scheduler::{JobScheduler, JobToRun, Job};

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let mut sched = JobScheduler::new();
    let pool = Arc::new(pool);

    let p = pool.clone();
    sched.add(Job::new("0 1 * * *", move |uuid, l| {
        println!("every day task");
        proc_couriers_for_them(p.clone());
        proc_couriers_for_curators(p.clone());
    }).unwrap());

    let p = pool.clone();
    sched.add(Job::new("0 1 * * 1", move |uuid, l| {
        println!("every week task");
        proc_restaurants_for_curators(p.clone()); 
        proc_restaurants(p.clone()); 
    }).unwrap());

    let p = pool.clone();
    sched.add(Job::new("1/60 * * * * *", move |uuid, l| {
        println!("every min task");
        diesel::sql_query("select * from process_approvals();")
            .execute(&p.get().unwrap()).unwrap();
    }).unwrap());

    sched.start().await;
}

fn proc_couriers_for_them(pool: Arc<r2d2::Pool<ConnectionManager<PgConnection>>>) {

    let couriers_ids: Vec<i64> = couriers::table
        .select(couriers::id)
        .get_results::<i64>(&pool.get().unwrap()).unwrap();
    for id in couriers_ids {
        let rows = diesel::sql_query("select * from courier_exel WHERE courier_id=$1")
            .bind::<Bigint,_>(id)
            .get_results::<CourierXLS>(&pool.get().unwrap()).unwrap();
        let fname = format!("summary/{}.xlsx",Uuid::new_v4());
        let workbook = Workbook::new(&fname);
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
        diesel::insert_into(couriers_xls_reports::table)
            .values(&(
                couriers_xls_reports::courier_id.eq(id),
                couriers_xls_reports::filename.eq(&fname))
            )
            .execute(&pool.get().unwrap()).unwrap();
    }
}

fn proc_couriers_for_curators(pool: Arc<r2d2::Pool<ConnectionManager<PgConnection>>>) {

        let rows = diesel::sql_query("select * from courier_for_curator_exel")
            .get_results::<CourierXLSTotal>(&pool.get().unwrap()).unwrap();
        let fname = format!("summary/{}.xlsx",Uuid::new_v4());
        let workbook = Workbook::new(&fname);
        let mut sheet1 = workbook.add_worksheet(None).unwrap();
            sheet1.write_string(0, 0, "телефон курьера", None).unwrap();
            sheet1.write_string(0, 1, "имя курьера", None).unwrap();
            sheet1.write_string(0, 2, "фамилия курьера", None).unwrap();
            sheet1.write_string(0, 3, "отчество курьера", None).unwrap();
            sheet1.write_string(0, 4, "день сессии", None).unwrap();
            sheet1.write_string(0, 5, "время начала сессии", None).unwrap();
            sheet1.write_string(0, 6, "время конца сессии", None).unwrap();
            sheet1.write_string(0, 7, "номер заказа", None).unwrap();
            sheet1.write_string(0, 8, "забрано курьером", None).unwrap();
            sheet1.write_string(0, 9, "статус заказа", None).unwrap();
            sheet1.write_string(0, 10, "детали заказа", None).unwrap();
            sheet1.write_string(0, 11, "большой заказ", None).unwrap();
            sheet1.write_string(0, 12, "время готовки", None).unwrap();
            sheet1.write_string(0, 13, "доставлено", None).unwrap();
            sheet1.write_string(0, 14, "стоимость заказа", None).unwrap();
            sheet1.write_string(0, 15, "стоимость доставки", None).unwrap();
            sheet1.write_string(0, 16, "адрес доставки", None).unwrap();
            sheet1.write_string(0, 17, "телефон клиента", None).unwrap();
            sheet1.write_string(0, 18, "комментарий клиента", None).unwrap();
            sheet1.write_string(0, 19, "способ оплаты", None).unwrap(); 
            for i in 1..=rows.len() { 
                let i = i as u32;
                let ind = i as usize;
                sheet1.write_string(i, 0, &rows[ind].phone.to_string(), None).unwrap();
                sheet1.write_string(i, 1, &rows[ind].name.to_string(), None).unwrap();
                sheet1.write_string(i, 2, &rows[ind].surname.to_string(), None).unwrap();
                sheet1.write_string(i, 3, &rows[ind].patronymic.to_string(), None).unwrap();
                sheet1.write_string(i, 4, &rows[ind].session_day.to_string(), None).unwrap();
                sheet1.write_string(i, 5, &rows[ind].start_time.to_string(), None).unwrap();
                sheet1.write_string(i, 6, &rows[ind].end_real_time.map(|v|{
                    v.to_string()
                }).unwrap_or("на момент создания отчета сессия не была закончена".to_string()),
                None).unwrap();
                sheet1.write_number(i, 7, rows[ind].order_id as f64, None).unwrap();
                sheet1.write_number(i, 8, (rows[ind].courier_salary / 100) as f64, None).unwrap();
                sheet1.write_string(i, 9, match rows[ind].order_status {
                    OrderStatus::Success => "успешно доставлено",
                    OrderStatus::FailureByRestaurant => "отменено по вине ресторана",
                    OrderStatus::FailureByCourier => "отменено по вине курьера",
                    _ => panic!(),
                }, None).unwrap();
                sheet1.write_string(i, 10, &rows[ind].details, None).unwrap();
                sheet1.write_boolean(i, 11, rows[ind].is_big_order, None).unwrap();
                sheet1.write_string(i, 12, &rows[ind].cooking_time.to_string(), None).unwrap();
                sheet1.write_string(i, 13, &rows[ind].delivery_datetime.to_string(), None).unwrap();
                sheet1.write_number(i, 14, (rows[ind].order_price / 100) as f64, None).unwrap();
                sheet1.write_number(i, 15, (rows[ind].delivery_cost / 100) as f64, None).unwrap();
                sheet1.write_string(i, 16, &rows[ind].delivery_address, None).unwrap();
                sheet1.write_string(i, 17, &rows[ind].client_phone, None).unwrap();
                sheet1.write_string(i, 18, &rows[ind].client_comment, None).unwrap();
                sheet1.write_string(i, 19, match rows[ind].method {
                    PayMethod::Cash =>"наличными",
                    PayMethod::Card =>"картой",
                    PayMethod::AlreadyPayed =>"оплачено заранее",
                }, None).unwrap();
        }
        workbook.close().expect("workbook can be closed");
        diesel::insert_into(couriers_for_curators_xls_reports::table)
            .values(&(
                couriers_for_curators_xls_reports::filename.eq(&fname))
            )
            .execute(&pool.get().unwrap()).unwrap();
}

fn proc_restaurants(pool: Arc<r2d2::Pool<ConnectionManager<PgConnection>>>) {

    let restaurant_ids: Vec<i64> = restaurants::table
        .select(restaurants::id)
        .get_results::<i64>(&pool.get().unwrap()).unwrap();
    for id in restaurant_ids {
        let rows = diesel::sql_query("select * from restaurant_exel WHERE restaurant_id=$1")
            .bind::<Bigint,_>(id)
            .get_results::<RestaurantXLS>(&pool.get().unwrap()).unwrap();
        let fname = format!("summary/{}.xlsx",Uuid::new_v4());
        let workbook = Workbook::new(&fname);
        let mut sheet1 = workbook.add_worksheet(None).unwrap();
            sheet1.write_string(0, 0, "номер заказа", None).unwrap();
            sheet1.write_string(0, 1, "забрано", None).unwrap();
            sheet1.write_string(0, 2, "статус заказа", None).unwrap();
            sheet1.write_string(0, 3, "детали заказа", None).unwrap();
            sheet1.write_string(0, 4, "большой заказ", None).unwrap();
            sheet1.write_string(0, 5, "время готовки", None).unwrap();
            sheet1.write_string(0, 6, "доставлено", None).unwrap();
            sheet1.write_string(0, 7, "стоимость заказа", None).unwrap();
            sheet1.write_string(0, 8, "адрес доставки", None).unwrap();
            sheet1.write_string(0, 9, "комментарий клиента", None).unwrap();
            sheet1.write_string(0, 10, "телефон клиента", None).unwrap();
            sheet1.write_string(0, 11, "способ оплаты", None).unwrap(); 
            for i in 1..=rows.len() { 
                let i = i as u32;
                let ind = i as usize;
                sheet1.write_number(i, 0, rows[ind].order_id as f64, None).unwrap();
                sheet1.write_string(i, 1, &rows[ind].take_datetime.to_string(), None).unwrap();
                sheet1.write_string(i, 2, match rows[ind].order_status {
                    OrderStatus::Success => "успешно доставлено",
                    OrderStatus::FailureByRestaurant => "отменено по вине ресторана",
                    OrderStatus::FailureByCourier => "отменено по вине курьера",
                    _ => panic!(),
                }, None).unwrap();
                sheet1.write_string(i, 3, &rows[ind].details, None).unwrap();
                sheet1.write_boolean(i,4, rows[ind].is_big_order, None).unwrap();
                sheet1.write_string(i, 5, &rows[ind].cooking_time.to_string(), None).unwrap();
                sheet1.write_string(i, 6, &rows[ind].delivery_datetime.to_string(), None).unwrap();
                sheet1.write_number(i, 7, (rows[ind].order_price / 100) as f64, None).unwrap();
                sheet1.write_string(i, 8, &rows[ind].delivery_address, None).unwrap();
                sheet1.write_string(i, 9, &rows[ind].client_comment, None).unwrap();
                sheet1.write_string(i, 10, &rows[ind].client_phone, None).unwrap();
                sheet1.write_string(i, 11, match rows[ind].method {
                    PayMethod::Cash =>"наличными",
                    PayMethod::Card =>"картой",
                    PayMethod::AlreadyPayed =>"оплачено заранее",
                }, None).unwrap();
        }
        workbook.close().expect("workbook can be closed");
        diesel::insert_into(restaurants_xls_reports::table)
            .values(&(
                restaurants_xls_reports::restaurant_id.eq(id),
                restaurants_xls_reports::filename.eq(&fname))
            )
            .execute(&pool.get().unwrap()).unwrap();
    }
}

fn proc_restaurants_for_curators(pool: Arc<r2d2::Pool<ConnectionManager<PgConnection>>>) {
        let rows = diesel::sql_query("select * from restaurant_for_curator_exel")
            .get_results::<RestaurantXLSTotal>(&pool.get().unwrap()).unwrap();
        let fname = format!("summary/{}.xlsx",Uuid::new_v4());
        let workbook = Workbook::new(&fname);
        let mut sheet1 = workbook.add_worksheet(None).unwrap();
            sheet1.write_string(0, 0, "название ресторана", None).unwrap();
            sheet1.write_string(0, 1, "телефон ресторана", None).unwrap();
            sheet1.write_string(0, 2, "aдрес ресторана", None).unwrap();
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
            sheet1.write_string(0, 13, "телефон клиента", None).unwrap();
            sheet1.write_string(0, 14, "способ оплаты", None).unwrap(); 
            for i in 1..=rows.len() { 
                let i = i as u32;
                let ind = i as usize;
                sheet1.write_string(i, 0, &rows[ind].name.to_string(), None).unwrap();
                sheet1.write_string(i, 1, &rows[ind].phone.to_string(), None).unwrap();
                sheet1.write_string(i, 2, &rows[ind].address.to_string(), None).unwrap();
                sheet1.write_number(i, 3, rows[ind].order_id as f64, None).unwrap();
                sheet1.write_string(i, 4, &rows[ind].take_datetime.to_string(), None).unwrap();
                sheet1.write_string(i, 5, match rows[ind].order_status {
                    OrderStatus::Success => "успешно доставлено",
                    OrderStatus::FailureByRestaurant => "отменено по вине ресторана",
                    OrderStatus::FailureByCourier => "отменено по вине курьера",
                    _ => panic!(),
                }, None).unwrap();
                sheet1.write_string(i, 6, &rows[ind].details, None).unwrap();
                sheet1.write_boolean(i,7, rows[ind].is_big_order, None).unwrap();
                sheet1.write_string(i, 8, &rows[ind].cooking_time.to_string(), None).unwrap();
                sheet1.write_string(i, 9, &rows[ind].delivery_datetime.to_string(), None).unwrap();
                sheet1.write_number(i, 10, (rows[ind].order_price / 100) as f64, None).unwrap();
                sheet1.write_string(i, 11, &rows[ind].delivery_address, None).unwrap();
                sheet1.write_string(i, 12, &rows[ind].client_comment, None).unwrap();
                sheet1.write_string(i, 13, &rows[ind].client_phone, None).unwrap();
                sheet1.write_string(i, 14, match rows[ind].method {
                    PayMethod::Cash =>"наличными",
                    PayMethod::Card =>"картой",
                    PayMethod::AlreadyPayed =>"оплачено заранее",
                }, None).unwrap();
        }
        workbook.close().expect("workbook can be closed");
        diesel::insert_into(restaurants_for_curators_xls_reports::table)
            .values(&(
                restaurants_for_curators_xls_reports::filename.eq(&fname))
            )
            .execute(&pool.get().unwrap()).unwrap();
}
