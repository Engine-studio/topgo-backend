use actix_web_dev::error::{
    Result,
    ErrorType,
    ApiError,
};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::schema::paintings;

#[derive(Serialize,Deserialize,Clone,Queryable,Identifiable)]
#[table_name = "paintings"]
#[primary_key(id)]
pub struct Painting {
    pub id: i64,
    pub seller_id: i64,
    pub name: String,
    pub paint_size: String,
    pub picture: String,
    pub price: i64,
    pub charact: String,
    pub author_name: String,
    pub description: String,
    pub status: String,
    pub create_datetime: chrono::NaiveDateTime,
}

#[derive(Serialize,Deserialize,Clone,Insertable,Queryable)]
#[table_name = "paintings"]
pub struct NewPainting {
    pub seller_id: i64,
    pub name: String,
    pub paint_size: String,
    pub price: i64,
    pub picture: String,
    pub charact: String,
    pub author_name: String,
    pub description: String,
}

#[derive(Serialize,Deserialize,Clone,AsChangeset,Queryable,Identifiable)]
#[table_name = "paintings"]
#[primary_key(id)]
pub struct UpdatePainting {
    pub id: i64,
    pub name: Option<String>,
    pub paint_size: Option<String>,
    pub price: Option<i64>,
    pub picture: Option<String>,
    pub charact: Option<String>,
    pub author_name: Option<String>,
    pub description: Option<String>,
}

impl Painting {
    pub async fn new(
        instance: &NewPainting, 
        conn: &PgConnection,
    ) -> Result<Self> {
        diesel::insert_into(paintings::table)
            .values(instance)
            .get_result(conn)
            .map_err(|e| e.into())
    }

    pub async fn from_id(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Self> {
        paintings::table
            .filter(paintings::id.eq(id))
            .get_result(conn)
            .map_err(|e| e.into())
    }

    pub async fn verify(
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::update(paintings::table
            .filter(paintings::id.eq(id)))
            .set(paintings::status.eq("verifyed"))
            .execute(conn)?;
        Ok(())
    }

    pub async fn set(
        instance: &UpdatePainting,
        conn: &PgConnection,
    ) -> Result<Self> {
        diesel::update(paintings::table
            .filter(paintings::id.eq(instance.id)))
            .set(instance)
            .get_result(conn)
            .map_err(|e| e.into())
    }

    pub async fn get_unverifyed(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        paintings::table
            .filter(paintings::status.eq("unverifyed"))
            .load(conn)
            .map_err(|e| e.into())
    }

    pub async fn delete(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Self> {
        diesel::delete(paintings::table
            .filter(paintings::id.eq(id)))
            .get_result(conn)
            .map_err(|e| e.into())
    }

    pub async fn get_verifyed(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        paintings::table
            .filter(paintings::status.eq("verifyed"))
            .load(conn)
            .map_err(|e| e.into())
    }

}

