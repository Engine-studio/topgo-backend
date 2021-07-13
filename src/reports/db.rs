use actix_web_dev::error::{
    Result,
    ErrorType,
    ApiError,
};
use serde::{Serialize, Deserialize};
use diesel::{prelude::*, query_dsl::InternalJoinDsl};
use diesel::pg::PgConnection;
use super::*;

use crate::schema::{
    restaurants_xls_reports,
    restaurants_for_curators_xls_reports,
    couriers_for_curators_xls_reports,
    couriers_xls_reports,
};
use diesel::sql_types::{
    Bigint,
    Varchar,
    Integer,
    Time,
    Double,
    Bool,
};
use crate::enum_types::*;

#[derive(Serialize,Deserialize,Clone,Queryable)]
pub struct CourierReport {
    pub id: i64,
    pub courier_id: i64,
    pub filename: String,
    pub creation_date: chrono::NaiveDate,
}

#[derive(Serialize,Deserialize,Clone,Queryable)]
pub struct CourierCuratorReport {
    pub id: i64,
    pub filename: String,
    pub creation_date: chrono::NaiveDate,
}

#[derive(Serialize,Deserialize,Clone,Queryable)]
pub struct RestaurantReport {
    pub id: i64,
    pub restaurant_id: i64,
    pub filename: String,
    pub creation_date: chrono::NaiveDate,
}

#[derive(Serialize,Deserialize,Clone,Queryable)]
pub struct RestaurantCuratorReport {
    pub id: i64,
    pub filename: String,
    pub creation_date: chrono::NaiveDate,
}

impl RestaurantReport {
    pub async fn get (
        id: i64,
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = restaurants_xls_reports::table
            .filter(restaurants_xls_reports::restaurant_id.eq(id))
            .get_results(conn)?;
        Ok(r)
    }
}

impl CourierReport {
    pub async fn get (
        id: i64,
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = couriers_xls_reports::table
            .filter(couriers_xls_reports::courier_id.eq(id))
            .get_results(conn)?;
        Ok(r)
    }
}

impl CourierCuratorReport {
    pub async fn get (
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = couriers_for_curators_xls_reports::table
            .get_results(conn)?;
        Ok(r)
    }
}

impl RestaurantCuratorReport {
    pub async fn get (
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = restaurants_for_curators_xls_reports::table
            .get_results(conn)?;
        Ok(r)
    }
}
