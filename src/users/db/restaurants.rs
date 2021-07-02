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
    pub location_lat: f64,
    pub location_lng: f64,
    pub working_from: Vec<chrono::NaiveTime>,
    pub working_till: Vec<chrono::NaiveTime>,
}

impl Restaurants {
    pub async fn new(
        creds: &NewRestaurant, 
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::insert_into(restaurants::table)
            .values(&(
                restaurants::id.eq(&id),
                restaurants::name.eq(&creds.name),
                restaurants::phone.eq(&creds.phone),
                restaurants::pass_hash.eq(make_hash(&creds.password)),
                restaurants::location_lng.eq(&creds.location_lng),
                restaurants::location_lat.eq(&creds.location_lat),
                restaurants::working_from.eq(&creds.working_from),
                restaurants::working_till.eq(&creds.working_till),
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
            .set(
                restaurants::is_deleted.eq(true)
            )
            .execute(conn)?;
        Ok(())
    }

}
