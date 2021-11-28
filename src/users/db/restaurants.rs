use actix_web_dev::error::{
    Result,
    ErrorType,
    ApiError,
};
use diesel::sql_types::Array;
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use super::*;
use reqwest;

use crate::schema::{
    restaurants,
};

#[derive(Serialize,Deserialize,Clone,Queryable,Identifiable)]
#[table_name = "restaurants"]
#[primary_key(id)]
pub struct Restaurants {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub phone: String,
    pub pass_hash: String,
    pub location_lat: f64,
    pub location_lng: f64,
    pub working_from: Vec<chrono::NaiveTime>,
    pub working_till: Vec<chrono::NaiveTime>,
    pub is_working: bool,
    pub is_deleted: bool,
    pub creation_datetime: chrono::NaiveDateTime,
    pub email: String,
}

#[derive(Serialize,Deserialize,Clone,AsChangeset,Queryable,Identifiable)]
#[table_name = "restaurants"]
#[primary_key(id)]
pub struct UpdateRestaurants {
    pub id: i64,
    pub address: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct NewRestaurant {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub password: String,
    pub working_from: Vec<chrono::NaiveTime>,
    pub working_till: Vec<chrono::NaiveTime>,
    pub email: String,
    pub lng: f64,
    pub lat: f64,
}

impl Restaurants {
    pub async fn check_mail(cour: &NewRestaurant) -> Result<()>  {
        
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
        <p>Здравствуйте, {}<p><br/>
        <p>Вы успешно зарегестрированы в сервисе topgo<p><br/>
    </body>
    </html>"#,cour.name);

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
        creds: &NewRestaurant, 
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        Self::check_mail(&creds).await?;
        diesel::insert_into(restaurants::table)
            .values(&(
                restaurants::id.eq(&id),
                restaurants::name.eq(&creds.name),
                restaurants::phone.eq(&creds.phone),
                restaurants::pass_hash.eq(make_hash(&creds.password)),
                restaurants::location_lng.eq(creds.lng),
                restaurants::location_lat.eq(creds.lat),
                restaurants::working_from.eq(&creds.working_from),
                restaurants::working_till.eq(&creds.working_till),
                restaurants::address.eq(&creds.address),
                restaurants::email.eq(&creds.email),
            ))
            .execute(conn)?;
        Ok(())
    }

    pub async fn from_id(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = restaurants::table
            .filter(restaurants::id.eq(id))
            .filter(restaurants::is_deleted.eq(false))
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn get(
        creds: &AuthData,
        conn: &PgConnection,
    ) -> Result<Self> {
        let pass_hash = make_hash(&creds.password);
        let r = restaurants::table
            .filter(restaurants::phone.eq(&creds.phone))
            .filter(restaurants::pass_hash.eq(pass_hash))
            .filter(restaurants::is_deleted.eq(false))
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn set(
        instance: &UpdateRestaurants,
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = diesel::update(restaurants::table
            .filter(restaurants::id.eq(instance.id)))
            .set(instance)
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn delete(
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::update(restaurants::table
            .filter(restaurants::id.eq(id)))
            .set((
                restaurants::is_deleted.eq(true),
                restaurants::phone.eq(&uuid::Uuid::new_v4().to_string())
            ))
            .execute(conn)?;
        Ok(())
    }

    pub async fn get_all(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = restaurants::table
            .filter(restaurants::is_deleted.eq(false))
            .get_results(conn)?;
        Ok(r)
    }
}

use diesel::sql_types::*;
use crate::enum_types::*;
#[derive(Serialize,Deserialize,Clone,QueryableByName)]
pub struct RestaurantsInfo {
    #[sql_type="Bigint"]
    pub order_id: i64,
    #[sql_type="Varchar"]
    pub client_phone: String,
    #[sql_type="Varchar"]
    pub client_comment: String,
    #[sql_type="Bigint"]
    pub order_price: i64,
    #[sql_type="Varchar"]
    pub details: String,
    #[sql_type="Time"]
    pub cooking_time: chrono::NaiveTime,
    #[sql_type="Double"]
    pub address_lat: f64,
    #[sql_type="Double"]
    pub address_lng: f64,
    #[sql_type="Varchar"]
    pub delivery_address: String,
    #[sql_type="Bool"]
    pub is_big_order: bool,
    #[sql_type="Orderstatus"]
    pub status: OrderStatus,
    #[sql_type="Paymethod"]
    pub method: PayMethod,
    #[sql_type="Nullable<Bigint>"]
    pub courier_id: Option<i64>,
    #[sql_type="Nullable<Varchar>"]
    pub courier_name: Option<String>,
    #[sql_type="Nullable<Varchar>"]
    pub courier_surname: Option<String>,
    #[sql_type="Nullable<Varchar>"]
    pub courier_patronymic: Option<String>,
    #[sql_type="Nullable<Varchar>"]
    pub courier_phone: Option<String>,
    #[sql_type="Nullable<Bigint>"]
    pub courier_rate_amount: Option<i64>,
    #[sql_type="Nullable<Bigint>"]
    pub courier_rate_count: Option<i64>,
    #[sql_type="Nullable<Varchar>"]
    pub courier_picture: Option<String>,
    #[sql_type="Bigint"]
    pub restaurant_id: i64,
    #[sql_type="Bigint"]
    pub courier_share: i64,
}

impl RestaurantsInfo {
    pub async fn get_history_by(
        restaurant_id: i64,
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = diesel::sql_query("SELECT * FROM restaurant_info WHERE restaurant_id=$1 and status = ANY('{Success,FailureByCourier,FailureByRestaurant}');") 
            .bind::<Bigint,_>(restaurant_id)
            .get_results(conn)?;
        Ok(r)
    }
    pub async fn get_by(
        restaurant_id: i64,
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = diesel::sql_query("SELECT * FROM restaurant_info WHERE restaurant_id=$1 and status = ANY('{CourierFinding,CourierConfirmation,Cooking,ReadyForDelivery,Delivering,Delivered}');") 
            .bind::<Bigint,_>(restaurant_id)
            .get_results(conn)?;
        Ok(r)
    }
}
