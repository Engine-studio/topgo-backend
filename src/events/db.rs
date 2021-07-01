use actix_web_dev::error::{
    Result,
    ErrorType,
    ApiError,
};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::schema::events;

#[derive(Serialize,Deserialize,Clone,Queryable,Identifiable)]
#[table_name = "events"]
#[primary_key(id)]
pub struct Event {
    pub id: i64,
    pub name: String,
    pub picture: String,
    pub event_type: String,
    pub place: String,
    pub description: String,
    pub start_date: chrono::NaiveDateTime,
    pub end_date: chrono::NaiveDateTime,
}

#[derive(Serialize,Deserialize,Clone,Insertable,Queryable)]
#[table_name = "events"]
pub struct NewEvent {
    pub name: String,
    pub picture: String,
    pub event_type: String,
    pub place: String,
    pub description: String,
    pub start_date: chrono::NaiveDateTime,
    pub end_date: chrono::NaiveDateTime,
}

#[derive(Serialize,Deserialize,Clone,AsChangeset,Queryable,Identifiable)]
#[table_name = "events"]
#[primary_key(id)]
pub struct UpdateEvent {
    pub id: i64,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub event_type: Option<String>,
    pub place: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<chrono::NaiveDateTime>,
    pub end_date: Option<chrono::NaiveDateTime>,
}

impl Event {
    pub async fn new(
        instance: &NewEvent, 
        conn: &PgConnection,
    ) -> Result<Self> {
        diesel::insert_into(events::table)
            .values(instance)
            .get_result(conn)
            .map_err(|e| e.into())
    }

    pub async fn from_id(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Self> {
        events::table
            .filter(events::id.eq(id))
            .get_result(conn)
            .map_err(|e| e.into())
    }

    pub async fn all(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        events::table
            .get_results(conn)
            .map_err(|e| e.into())
    }

    pub async fn set(
        instance: &UpdateEvent,
        conn: &PgConnection,
    ) -> Result<Self> {
        diesel::update(events::table
            .filter(events::id.eq(instance.id)))
            .set(instance)
            .get_result(conn)
            .map_err(|e| e.into())
    }

    pub async fn delete(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Self> {
        diesel::delete(events::table
            .filter(events::id.eq(id)))
            .get_result(conn)
            .map_err(|e| e.into())
    }
}
