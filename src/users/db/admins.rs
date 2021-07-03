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
    admins,
};

#[derive(Serialize,Deserialize,Clone,Queryable,Identifiable)]
#[table_name = "admins"]
#[primary_key(id)]
pub struct Admins {
    pub id: i64,
    pub name: String,
    pub surname: String,
    pub patronymic: String,
    pub phone: String,
    pub pass_hash: String,
    pub is_deleted: bool,
    pub picture: Option<String>,
    pub creation_datetime: chrono::NaiveDateTime,
}

#[derive(Serialize,Deserialize,Clone,AsChangeset,Queryable,Identifiable)]
#[table_name = "admins"]
#[primary_key(id)]
pub struct UpdateAdmin {
    pub id: i64,
    pub picture: Option<Option<String>>,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct NewAdmin {
    pub name: String,
    pub surname: String,
    pub patronymic: String,
    pub phone: String,
    pub password: String,
}

impl Admins {
    pub async fn new(
        creds: &NewAdmin, 
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::insert_into(admins::table)
            .values(&(
                admins::id.eq(&id),
                admins::name.eq(&creds.name),
                admins::surname.eq(&creds.surname),
                admins::patronymic.eq(&creds.patronymic),
                admins::phone.eq(&creds.phone),
                admins::pass_hash.eq(make_hash(&creds.password)),
            ))
            .execute(conn)?;
        Ok(())
    }

    pub async fn from_id(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = admins::table
            .filter(admins::id.eq(id))
            .filter(admins::is_deleted.eq(false))
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn get(
        creds: &AuthData,
        conn: &PgConnection,
    ) -> Result<Self> {
        let pass_hash = make_hash(&creds.password);
        println!("hash:: {}",pass_hash);
        let r = admins::table
            .filter(admins::phone.eq(&creds.phone))
            .filter(admins::pass_hash.eq(pass_hash))
            .filter(admins::is_deleted.eq(false))
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn set(
        instance: &UpdateAdmin,
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = diesel::update(admins::table
            .filter(admins::id.eq(instance.id)))
            .set(instance)
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn delete(
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::update(admins::table
            .filter(admins::id.eq(id)))
            .set(
                admins::is_deleted.eq(true)
            )
            .execute(conn)?;
        Ok(())
    }

}
