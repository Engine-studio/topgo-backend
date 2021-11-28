use std::sync::Arc;
///
use actix_web_dev::error::{
    Result,
    ErrorType,
    ApiError,
};
use bigdecimal::{ToPrimitive, Zero};
use chrono::NaiveTime;
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

use std::fs::File;
use std::io::Read;

fn read_a_file(name: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(name)?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    return Ok(data);
}

pub fn send_file(
    to: &str,
    fname: &str,
    title: &str,
) -> Result<()>  {
    
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};
    use lettre::message::{header, SinglePart};

    let part = SinglePart::builder()
     .content_type(
             "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
             .parse().unwrap())
     .body(read_a_file(fname).unwrap());

    let email = Message::builder()
    .from("noreply@topgo.club".parse().unwrap())
    .to(to.parse().unwrap())
    .subject(title)
    .singlepart(part)
    .map_err(|_e| {
        ApiError {
            code: 500,
            message: "err building msg".to_string(),
            error_type: ErrorType::InternalError,
        }
    })?;

    let creds = Credentials::new("topgo-noreply@yandex.ru".to_string(), 
        "2XkkLGRceLK".to_string());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp-pulse.com")
        .unwrap()
        .credentials(creds)
        .build();   

    // Send the email
    let _ = mailer.send(&email).map_err(|_e|{
        ApiError {
            code: 500,
            message: "err sending msg".to_string(),
            error_type: ErrorType::InternalError,
        }
    })?;
    println!("SEND SUCCESS");
    Ok(())
}


#[derive(Serialize,Deserialize,Clone,QueryableByName,Debug)]
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
    #[sql_type="Nullable<Timestamp>"]
    pub take_datetime: Option<chrono::NaiveDateTime>,
    #[sql_type="Orderstatus"]
    pub order_status: OrderStatus,
    #[sql_type="Varchar"]
    pub details: String,
    #[sql_type="Bool"]
    pub is_big_order: bool,
    #[sql_type="Time"]
    pub cooking_time: chrono::NaiveTime,
    #[sql_type="Nullable<Timestamp>"]
    pub delivery_datetime: Option<chrono::NaiveDateTime>,
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

#[derive(Serialize,Deserialize,Clone,QueryableByName,Debug)]
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
    #[sql_type="Nullable<Timestamp>"]
    pub take_datetime: Option<chrono::NaiveDateTime>,
    #[sql_type="Orderstatus"]
    pub order_status: OrderStatus,
    #[sql_type="Varchar"]
    pub details: String,
    #[sql_type="Bool"]
    pub is_big_order: bool,
    #[sql_type="Time"]
    pub cooking_time: chrono::NaiveTime,
    #[sql_type="Nullable<Timestamp>"]
    pub delivery_datetime: Option<chrono::NaiveDateTime>,
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

#[derive(Serialize,Deserialize,Clone,QueryableByName,Debug)]
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

#[derive(Serialize,Deserialize,Clone,QueryableByName,Debug)]
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
    println!("running init tasks");
    proc_couriers_for_them(p.clone());
    proc_couriers_for_curators(p.clone());
    proc_restaurants(p.clone()); 
    println!("starting");
    sched.add(Job::new("0 0 0 * * * *", move |_uuid, _l| {
        println!("every day task");
        proc_couriers_for_them(p.clone());
        proc_couriers_for_curators(p.clone());
    }).unwrap()).unwrap();

    let p = pool.clone();
    sched.add(Job::new("0 0 0 * * Mon *", move |_uuid, _l| {
        println!("every week task");
        proc_restaurants(p.clone()); 
    }).unwrap()).unwrap();

    let p = pool.clone();
    sched.add(Job::new("0 * * * * * *", move |_uuid, _l| {
        diesel::sql_query("select * from process_approvals();")
            .execute(&p.get().unwrap()).unwrap();
    }).unwrap()).unwrap();

    sched.start().await.unwrap();
}

fn proc_couriers_for_them(pool: Arc<r2d2::Pool<ConnectionManager<PgConnection>>>) {

    let couriers_ids: Vec<(i64,String)> = couriers::table
        .select((couriers::id,couriers::email))
        .get_results::<(i64,String)>(&pool.get().unwrap()).unwrap();
    println!("ids found {:?}",couriers_ids);
    for id in couriers_ids {
        let rows = diesel::sql_query("select * from courier_exel WHERE courier_id=$1")
            .bind::<Bigint,_>(id.0)
            .get_results::<CourierXLS>(&pool.get().unwrap()).unwrap();

            let fname = format!("summary/{}.xlsx",Uuid::new_v4());
            let mut workbook = Workbook::new(&fname);
            let mut sheet1 = workbook.add_worksheet(None).unwrap();
            println!("rows found {:?}",rows);
            for i_it in 0..rows.len() { 
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
                let i = ((i_it % 100) + 1) as u32;
                let ind = i_it as usize;
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
                sheet1.write_string(i, 9, &rows[ind].delivery_datetime
                    .map(|v|v.to_string()).unwrap_or(String::new()), None).unwrap();
                sheet1.write_number(i, 10, (rows[ind].order_price / 100) as f64, None).unwrap();
                sheet1.write_string(i, 11, &rows[ind].delivery_address, None).unwrap();
                sheet1.write_string(i, 12, &rows[ind].client_comment, None).unwrap();
                sheet1.write_string(i, 13, match rows[ind].method {
                    PayMethod::Cash =>"наличными",
                    PayMethod::Card =>"картой",
                    PayMethod::AlreadyPayed =>"оплачено заранее",
                }, None).unwrap();
                if i == 100 || ind as usize == rows.len()-1 {
                    workbook.close().expect("workbook can be closed");
                    send_file(
                        &id.1,
                        &fname, 
                        &("отчет за ".to_owned() + &(
                            chrono::Utc::today() - chrono::Duration::days(1)).to_string())
                    ).unwrap();
                    std::fs::remove_file(&fname).unwrap();
                    workbook = Workbook::new(&fname);
                    sheet1 = workbook.add_worksheet(None).unwrap();
                }
        }
    }
}

fn proc_couriers_for_curators(pool: Arc<r2d2::Pool<ConnectionManager<PgConnection>>>) {
        let rows = diesel::sql_query("select * from courier_for_curator_exel")
            .get_results::<CourierXLSTotal>(&pool.get().unwrap()).unwrap();
            println!("for curator {:?}",rows);
            let fname = format!("summary/{}.xlsx",Uuid::new_v4());
            let mut workbook = Workbook::new(&fname);
            let mut sheet1 = workbook.add_worksheet(None).unwrap();
            for i_it in 0..rows.len() { 
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
                let i = ((i_it % 100) + 1) as u32;
                let ind = i_it as usize;
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
                sheet1.write_string(i, 13, &rows[ind].delivery_datetime
                    .map(|v| v.to_string()).unwrap_or(String::new()), None).unwrap();
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
                if i == 100 || ind as usize == rows.len()-1 {
                    workbook.close().expect("workbook can be closed");
                    send_file(
                        "noreply@topgo.club", 
                        &fname, 
                        &("отчет по курьерам за ".to_owned() + &(
                            chrono::Utc::today() - chrono::Duration::days(1)).to_string())
                    ).unwrap();
                    std::fs::remove_file(&fname).unwrap();
                    workbook = Workbook::new(&fname);
                    sheet1 = workbook.add_worksheet(None).unwrap();
                }
        }

}

fn proc_restaurants(pool: Arc<r2d2::Pool<ConnectionManager<PgConnection>>>) {
    use topgo::schema::orders;
    let restaurant_ids: Vec<(i64,String)> = restaurants::table
        .select((restaurants::id,restaurants::email))
        .get_results::<(i64,String)>(&pool.get().unwrap()).unwrap();
    for id in restaurant_ids {
        let fname = format!("summary/{}.xlsx",Uuid::new_v4());
        let mut workbook = Workbook::new(&fname);
        let mut sheet1 = workbook.add_worksheet(None).unwrap();
        use diesel::dsl;
        use bigdecimal::BigDecimal;
        let orders_reject_pay = orders::table
                .filter(orders::status.eq(OrderStatus::FailureByRestaurant))
                .filter(orders::restaurant_id.eq(id.0))
                .filter(dsl::sql("status = ANY('{Success,FailureByCourier,FailureByRestaurant}') AND
         finalize_datetime::DATE < CURRENT_DATE AT TIME ZONE 'Europe/Moscow' AND
         finalize_datetime::DATE >= (CURRENT_DATE-7) AT TIME ZONE 'Europe/Moscow'"))
                .count()
                .execute(&pool.get().unwrap()).unwrap();
        let orders_reject_pay_sum = orders::table
                .filter(orders::status.eq(OrderStatus::FailureByRestaurant))
                .filter(dsl::sql("status = ANY('{Success,FailureByCourier,FailureByRestaurant}') AND
         finalize_datetime::DATE < CURRENT_DATE AT TIME ZONE 'Europe/Moscow' AND
         finalize_datetime::DATE >= (CURRENT_DATE-7) AT TIME ZONE 'Europe/Moscow'"))
                .filter(orders::restaurant_id.eq(id.0))
                .select(diesel::dsl::sum(orders::order_price))
                .get_result::<Option<BigDecimal>>(&pool.get().unwrap())
                .unwrap().unwrap_or(BigDecimal::zero());
        let orders_card_pay_sum = orders::table
                .filter(orders::method.eq(PayMethod::Card))
                .filter(dsl::sql("status = ANY('{Success,FailureByCourier,FailureByRestaurant}') AND
         finalize_datetime::DATE < CURRENT_DATE AT TIME ZONE 'Europe/Moscow' AND
         finalize_datetime::DATE >= (CURRENT_DATE-7) AT TIME ZONE 'Europe/Moscow'"))
                .filter(orders::restaurant_id.eq(id.0))
                .select(diesel::dsl::sum(orders::order_price))
                .get_result::<Option<BigDecimal>>(&pool.get().unwrap())
                .unwrap().unwrap_or(BigDecimal::zero());
        let orders_cash_pay_sum = orders::table
                .filter(orders::method.eq(PayMethod::Cash))
                .filter(dsl::sql("status = ANY('{Success,FailureByCourier,FailureByRestaurant}') AND
         finalize_datetime::DATE < CURRENT_DATE AT TIME ZONE 'Europe/Moscow' AND
         finalize_datetime::DATE >= (CURRENT_DATE-7) AT TIME ZONE 'Europe/Moscow'"))
                .filter(orders::restaurant_id.eq(id.0))
                .select(diesel::dsl::sum(orders::order_price))
                .get_result::<Option<BigDecimal>>(&pool.get().unwrap())
                .unwrap().unwrap_or(BigDecimal::zero());
        let orders_cash_pay = orders::table
                .filter(orders::method.eq(PayMethod::Cash))
                .filter(dsl::sql("status = ANY('{Success,FailureByCourier,FailureByRestaurant}') AND
         finalize_datetime::DATE < CURRENT_DATE AT TIME ZONE 'Europe/Moscow' AND
         finalize_datetime::DATE >= (CURRENT_DATE-7) AT TIME ZONE 'Europe/Moscow'"))
                .filter(orders::restaurant_id.eq(id.0))
                .count()
                .execute(&pool.get().unwrap()).unwrap();
        let orders_card_pay = orders::table
                .filter(orders::method.eq(PayMethod::Card))
                .filter(dsl::sql("status = ANY('{Success,FailureByCourier,FailureByRestaurant}') AND
         finalize_datetime::DATE < CURRENT_DATE AT TIME ZONE 'Europe/Moscow' AND
         finalize_datetime::DATE >= (CURRENT_DATE-7) AT TIME ZONE 'Europe/Moscow'"))
                .filter(orders::restaurant_id.eq(id.0))
                .count()
                .execute(&pool.get().unwrap()).unwrap();
        let orders_no_pay: usize = orders::table
                .filter(orders::method.eq(PayMethod::AlreadyPayed))
                .filter(dsl::sql("status = ANY('{Success,FailureByCourier,FailureByRestaurant}') AND
         finalize_datetime::DATE < CURRENT_DATE AT TIME ZONE 'Europe/Moscow' AND
         finalize_datetime::DATE >= (CURRENT_DATE-7) AT TIME ZONE 'Europe/Moscow'"))
                .filter(orders::restaurant_id.eq(id.0))
                .count()
                .execute(&pool.get().unwrap()).unwrap();
        let total_delivery: usize = orders::table
                .filter(orders::restaurant_id.eq(id.0))
                .filter(dsl::sql("status = ANY('{Success,FailureByCourier,FailureByRestaurant}') AND
         finalize_datetime::DATE < CURRENT_DATE AT TIME ZONE 'Europe/Moscow' AND
         finalize_datetime::DATE >= (CURRENT_DATE-7) AT TIME ZONE 'Europe/Moscow'"))
                .select(diesel::dsl::sum(orders::courier_share))
                .execute(&pool.get().unwrap()).unwrap();
        let to = chrono::Utc::today()-chrono::Duration::days(1);
        let from = chrono::Utc::today()-chrono::Duration::days(7);
        let r1 = format!(" - {} заказов, исполненных и оплаченных покупателями по терминалу на сумму {} руб ", orders_card_pay,orders_card_pay_sum.clone()/100);
        let r2 = format!(" - {} заказов, исполненных и оплаченных покупателями наличными на сумму {} руб ", orders_cash_pay,orders_cash_pay_sum.clone()/100);
        let r3 = format!(" - {} заказов, не исполненных заказчиком по вине ресторана на общую сумму {} руб", orders_reject_pay,orders_reject_pay_sum.clone()/100);
        let r4 = format!("2. Комиссия \"Top Go\" за прием оплаты заказа по терминалу за период с {} по {} составила: {} руб. (2% от общей суммы заказов принятых по терминалу за указанный период)", 
            from,to,orders_card_pay_sum.clone().to_u64().unwrap()*2/10000);
        let r5 = format!("2.1 Комиссия \"Top Go\" за {} выполненных заказов за период с {} по {} составила: {} руб. ",orders_cash_pay+orders_card_pay+orders_no_pay ,from,to,total_delivery/100);

        let r6 = format!("2.2 Сводная сумма комиссии \"Top Go\" за период с {} по {} составила: {} руб. ",from,to,(total_delivery+((orders_card_pay_sum.to_u64().unwrap() as usize)*98/100))/100);
        let r9 = format!("3. Сумма к перечислению за период с {} по {} г. составляет {} руб.",from,to,(total_delivery+((orders_card_pay_sum.to_u64().unwrap() as usize)*98/100))/100);
        let r7 = format!("по договору № _________ от 31.05.2021 за период с {} по {}",from,to);
        let r8 = format!("1. За период с {} по {} г. \"Top Go\"  были получены и переданы клиенту:",from,to);
                sheet1.write_string(0, 0, "Информационный отчет о движении денежных средств", None).unwrap();
                sheet1.write_string(1, 0, &r7, None).unwrap();
                sheet1.write_string(3, 0, &r8, None).unwrap();
                sheet1.write_string(5, 0, &r1, None).unwrap();
                sheet1.write_string(6, 0, &r2, None).unwrap();
                sheet1.write_string(7, 0, &r3, None).unwrap();
                sheet1.write_string(10, 0, &r4, None).unwrap();
                sheet1.write_string(11, 0, &r5, None).unwrap();
                sheet1.write_string(12, 0, &r6, None).unwrap();
                sheet1.write_string(14, 0, &r9, None).unwrap();
                sheet1.write_string(16, 0, "№", None).unwrap(); 
                sheet1.write_string(16, 1, "Общее количество заказов", None).unwrap(); 
                sheet1.write_string(16, 2, "Сумма заказов оплаченных по терминалу", None).unwrap(); 
                sheet1.write_string(16, 3, "Количество заказов оплаченных по терминалу", None).unwrap(); 
                sheet1.write_string(16, 4, "Сумма заказов оплаченных наличными", None).unwrap(); 
                sheet1.write_string(16, 5, "Количество заказов без оплаты", None).unwrap(); 
                sheet1.write_string(16, 6, "Итого сумма к перечислению", None).unwrap(); 

                sheet1.write_string(17, 0, "1", None).unwrap(); 
                sheet1.write_string(17, 1, &format!("{}",
                        orders_card_pay+orders_no_pay+orders_cash_pay), None).unwrap(); 
                sheet1.write_string(17, 2, &format!("{}",orders_card_pay_sum.clone()/100), None).unwrap(); 
                sheet1.write_string(17, 3, &format!("{}",orders_card_pay), None).unwrap(); 
                sheet1.write_string(17, 4, &format!("{}",orders_cash_pay_sum/100), None).unwrap(); 
                sheet1.write_string(17, 5, &format!("{}",orders_cash_pay), None).unwrap(); 
                sheet1.write_string(17, 6, 
                    &format!("{}",(total_delivery+
                            ((orders_card_pay_sum.to_u64().unwrap() as usize)*98/100))/100), None).unwrap(); 
                sheet1.write_string(18, 6, 
                    &format!("{}",(total_delivery+((orders_card_pay_sum.to_u64().unwrap() as usize)*98/100))/100), None).unwrap(); 
                sheet1.write_string(18, 0, "итого:", None).unwrap(); 

                    workbook.close().expect("workbook can be closed");
                    send_file(
                        &id.1,
                        &fname, 
                        &("отчет за ".to_owned() + &(
                            chrono::Utc::today() - chrono::Duration::days(7)).to_string())
                    ).unwrap();
                    send_file(
                        "noreply@topgo.club",
                        &fname, 
                        &("отчет за ".to_owned() + &(
                            chrono::Utc::today() - chrono::Duration::days(7)).to_string())
                    ).unwrap();
                    std::fs::remove_file(&fname).unwrap();
                    workbook = Workbook::new(&fname);
                    sheet1 = workbook.add_worksheet(None).unwrap();
    }
}

