use actix_web_dev::error::{
    Result,
    ErrorType,
    ApiError,
};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use super::*;

use crate::schema::{
    couriers,
    curators,
    admins,
    restaurants,
    sessions,
};
use crate::ordering::db::Sessions;
#[derive(Serialize,Deserialize,Clone,Queryable,Identifiable)]
#[table_name = "couriers"]
#[primary_key(id)]
pub struct Couriers {
    pub id: i64,
    pub name: String,
    pub surname: String,
    pub patronymic: String,
    pub phone: String,
    pub pass_hash: String,
    pub is_blocked: bool,
    pub is_warned: bool,
    pub is_deleted: bool,
    pub is_in_order: bool,
    pub current_rate_ammount: i64,
    pub current_rate_count: i64,
    pub picture: Option<String>,
    pub cash: i64,
    pub term: i64,
    pub salary: i64,
    pub creation_datetime: chrono::NaiveDateTime,
    pub email: String,
}

#[derive(Serialize,Deserialize,Clone,AsChangeset,Queryable,Identifiable)]
#[table_name = "couriers"]
#[primary_key(id)]
pub struct UpdateCourier {
    pub id: i64,
    pub picture: Option<Option<String>>,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct NewCourier {
    pub name: String,
    pub surname: String,
    pub patronymic: String,
    pub phone: String,
    pub password: String,
    pub email: String,
}

impl Couriers {
    pub async fn check_mail(cour: &NewCourier) -> Result<()>  {
        
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{Message, SmtpTransport, Transport};
        use lettre::message::{header, MultiPart, SinglePart};

        let html = format!(r#"<!DOCTYPE html>
    <html lang="ru">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Hello from Lettre!</title>
    </head>
    <body>
        <p>Здравствуйте, {} {} {}<p><br/>
        <p>Вы успешно зарегестрированы в сервисе topgo<p><br/>
    </body>
    </html>"#,cour.surname,cour.name,cour.patronymic);

        println!("msg {}",html);
        let email = Message::builder()
        .from("noreply@topgo.club".parse().unwrap())
        .to(cour.email.parse().unwrap())
        .subject("New Request")
        .multipart(
                MultiPart::alternative() // This is composed of two parts.
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::parse("text/plain; charset=utf8")
                                .unwrap())
                            .body("вы успешно зарегестрированы в сервисе topgo".to_string())
                            // Every message should have a plain text fallback.
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::parse(
                                "text/html; charset=utf8").unwrap())
                            .body(html.to_string()),
                    ),
        ).map_err(|e| {
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
        let _ = mailer.send(&email).map_err(|e|{
            ApiError {
                code: 500,
                message: "err sending msg".to_string(),
                error_type: ErrorType::InternalError,
            }
        })?;
        Ok(())
    }
    pub async fn new(
        creds: &NewCourier, 
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        Self::check_mail(&creds).await?;
        diesel::insert_into(couriers::table)
            .values(&(
                couriers::id.eq(&id),
                couriers::name.eq(&creds.name),
                couriers::surname.eq(&creds.surname),
                couriers::patronymic.eq(&creds.patronymic),
                couriers::phone.eq(&creds.phone),
                couriers::email.eq(&creds.email),
                couriers::pass_hash.eq(make_hash(&creds.password)),
            ))
            .execute(conn)?;
        Ok(())
    }

    pub async fn get_session(
        courier_id: i64,
        conn: &PgConnection,
    ) -> Result<Sessions> {
        let r = sessions::table
            .filter(sessions::courier_id.eq(courier_id))
            .get_result::<Sessions>(conn)?;
        Ok(r)
    }

    pub async fn from_id(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = couriers::table
            .filter(couriers::id.eq(id))
            .filter(couriers::is_deleted.eq(false))
            .get_result::<Self>(conn)?;
        Ok(r)
    }
    
    pub async fn null_money(
        data: &NullMoney,
        conn: &PgConnection,
    ) -> Result<()> {
        if data.all {
        diesel::update(couriers::table)
            .filter(couriers::id.eq(data.courier_id))
            .filter(couriers::is_deleted.eq(false))
            .set((
                couriers::salary.eq(0),
                couriers::cash.eq(0),
                couriers::term.eq(0),
                ))
            .execute(conn)?;
        } else if data.salary {
        diesel::update(couriers::table)
            .filter(couriers::id.eq(data.courier_id))
            .filter(couriers::is_deleted.eq(false))
            .set((
                couriers::salary.eq(0),
                ))
            .execute(conn)?;
        } else if data.card {
        diesel::update(couriers::table)
            .filter(couriers::id.eq(data.courier_id))
            .filter(couriers::is_deleted.eq(false))
            .set((
                couriers::term.eq(0),
                ))
            .execute(conn)?;
        } else if data.cash {
        diesel::update(couriers::table)
            .filter(couriers::id.eq(data.courier_id))
            .filter(couriers::is_deleted.eq(false))
            .set((
                couriers::cash.eq(0),
                ))
            .execute(conn)?;
        }
        Ok(())
    }


    pub async fn get(
        creds: &AuthData,
        conn: &PgConnection,
    ) -> Result<Self> {
        let pass_hash = make_hash(&creds.password);
        let r = couriers::table
            .filter(couriers::phone.eq(&creds.phone))
            .filter(couriers::pass_hash.eq(pass_hash))
            .filter(couriers::is_deleted.eq(false))
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn set(
        instance: &UpdateCourier,
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = diesel::update(couriers::table
            .filter(couriers::id.eq(instance.id)))
            .set(instance)
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn delete(
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::update(couriers::table
            .filter(couriers::id.eq(id)))
            .set(
                couriers::is_deleted.eq(true)
            )
            .execute(conn)?;
        Ok(())
    }

    pub async fn toggle_ban(
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::update(couriers::table
            .filter(couriers::id.eq(id)))
            .set(
                couriers::is_blocked.eq(diesel::dsl::not(couriers::is_blocked))
            )
            .execute(conn)?;
        Ok(())
    }

    pub async fn get_all(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = couriers::table
            .filter(couriers::is_deleted.eq(false))
            .get_results(conn)?;
        Ok(r)
    }

}

use diesel::sql_types::*;
use crate::enum_types::*;
#[derive(Serialize,Deserialize,Clone,QueryableByName)]
pub struct CouriersInfo {
    #[sql_type="Bigint"]
    pub courier_id: i64,
    #[sql_type="Varchar"]
    pub courier_name: String,
    #[sql_type="Varchar"]
    pub courier_surname: String,
    #[sql_type="Varchar"]
    pub courier_patronymic: String,
    #[sql_type="Varchar"]
    pub courier_picture: String,
    #[sql_type="Varchar"]
    pub courier_phone: String,
    #[sql_type="Bigint"]
    pub courier_current_rate_count: i64,
    #[sql_type="Bigint"]
    pub courier_current_rate_ammount: i64,
    #[sql_type="Bigint"]
    pub courier_card_balance: i64,
    #[sql_type="Bigint"]
    pub courier_salary: i64,
    #[sql_type="Bigint"]
    pub courier_cash_balance: i64,
    #[sql_type="Bool"]
    pub courier_is_in_order: bool,
    #[sql_type="Bool"]
    pub courier_is_warned: bool,
    #[sql_type="Bool"]
    pub courier_is_blocked: bool,
    #[sql_type="Bigint"]
    pub order_id: i64,
    #[sql_type="Orderstatus"]
    pub order_status: OrderStatus,
    #[sql_type="Bool"]
    pub is_big_order: bool,
    #[sql_type="Varchar"]
    pub delivery_address: String,
    #[sql_type="Double"]
    pub address_lng: f64,
    #[sql_type="Double"]
    pub address_lat: f64,
    #[sql_type="Time"]
    pub cooking_time: chrono::NaiveTime,
    #[sql_type="Varchar"]
    pub order_details: String,
    #[sql_type="Bigint"]
    pub order_price: i64,
    #[sql_type="Varchar"]
    pub client_comment: String,
    #[sql_type="Varchar"]
    pub client_phone: String,
    #[sql_type="Transporttype"]
    pub transport: TransportType,
}

impl CouriersInfo {
    pub async fn get_by(
        courier_id: i64,
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = diesel::sql_query("select * from courier_info WHERE courier_id=$1;") 
            .bind::<Bigint,_>(courier_id)
            .get_result(conn)?;
        Ok(r)
    }
}

#[derive(Serialize,Deserialize,Clone,QueryableByName)]
pub struct CouriersForAdmin {
    #[sql_type="Bigint"]
    pub id: i64,
    #[sql_type="Varchar"]
    pub name: String,
    #[sql_type="Varchar"]
    pub surname: String,
    #[sql_type="Varchar"]
    pub patronymic: String,
    #[sql_type="Nullable<Transporttype>"]
    pub transport: Option<TransportType>,
    #[sql_type="Bigint"]
    pub current_rate_count: i64,
    #[sql_type="Bigint"]
    pub current_rate_amount: i64,
    #[sql_type="Varchar"]
    pub phone: String,
    #[sql_type="Nullable<Varchar>"]
    pub picture: Option<String>,
    #[sql_type="Bool"]
    pub is_in_order: bool,
    #[sql_type="Nullable<Bigint>"]
    pub order_id: Option<i64>,
    #[sql_type="Nullable<Orderstatus>"]
    pub order_status: Option<OrderStatus>,
    #[sql_type="Bool"]
    pub is_blocked: bool,
    #[sql_type="Bigint"]
    pub salary: i64,
    #[sql_type="Bigint"]
    pub term: i64,
    #[sql_type="Bigint"]
    pub cash: i64,
    #[sql_type="Bool"]
    pub is_deleted: bool,
    #[sql_type="Bool"]
    pub is_in_session: bool,
}

impl CouriersForAdmin {
    pub async fn get(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = diesel::sql_query("select * from courier_for_admin;") 
            .get_results(conn)?;
        Ok(r)
    }
}

#[derive(Serialize,Deserialize)]
pub struct NullMoney {
    pub courier_id: i64,
    pub salary: bool,
    pub all: bool,
    pub cash: bool,
    pub card: bool,
}

#[derive(Serialize,Deserialize,Clone,QueryableByName)]
pub struct CouriersHistory {

    #[sql_type="Varchar"]
    pub restaurant_address: String,
    #[sql_type="Varchar"]
    pub delivery_address: String,
    #[sql_type="Time"]
    pub cooking_time: chrono::NaiveTime,
    #[sql_type="Paymethod"]
    pub method: PayMethod,
    #[sql_type="Bigint"]
    pub order_price: i64,
    #[sql_type="Timestamp"]
    pub take_datetime: chrono::NaiveDateTime,
    #[sql_type="Timestamp"]
    pub delivery_datetime: chrono::NaiveDateTime,
    #[sql_type="Nullable<Bigint>"]
    pub politeness_rate: Option<i64>,
    #[sql_type="Nullable<Bigint>"]
    pub look_rate: Option<i64>,
    #[sql_type="Bigint"]
    pub courier_id: i64,
    #[sql_type="Nullable<Orderstatus>"]
    pub order_status: Option<OrderStatus>,
}

impl CouriersHistory {
    pub async fn get(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = diesel::sql_query("select * from courier_history where courier_id=$1;") 
            .bind::<Bigint,_>(id)
            .get_results(conn)?;
        Ok(r)
    }
}
