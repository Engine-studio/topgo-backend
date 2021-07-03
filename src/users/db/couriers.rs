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
    couriers_to_curators,
};

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
    pub current_rate: Option<i16>,
    pub picture: Option<String>,
    pub cash: i64,
    pub term: i64,
    pub salary: i64,
    pub creation_datetime: chrono::NaiveDateTime,
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
    pub curator_id: i64,
}

impl Couriers {
    pub async fn new(
        creds: &NewCourier, 
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::insert_into(couriers::table)
            .values(&(
                couriers::id.eq(&id),
                couriers::name.eq(&creds.name),
                couriers::surname.eq(&creds.surname),
                couriers::patronymic.eq(&creds.patronymic),
                couriers::phone.eq(&creds.phone),
                couriers::pass_hash.eq(make_hash(&creds.password)),
            ))
            .execute(conn)?;
        diesel::insert_into(couriers_to_curators::table)
            .values(&(
                    couriers_to_curators::curator_id.eq(creds.curator_id),
                    couriers_to_curators::courier_id.eq(id),
            ))
            .execute(conn)?;
        Ok(())
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
