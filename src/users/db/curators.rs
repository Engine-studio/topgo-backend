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
    curators,
};

use diesel::sql_types::*;

#[derive(Serialize,Deserialize,Clone,QueryableByName,Queryable,Identifiable)]
#[table_name = "curators"]
#[primary_key(id)]
pub struct Curators {
    #[sql_type="Bigint"]
    pub id: i64,
    #[sql_type="Varchar"]
    pub name: String,
    #[sql_type="Varchar"]
    pub surname: String,
    #[sql_type="Varchar"]
    pub patronymic: String,
    #[sql_type="Varchar"]
    pub phone: String,
    #[sql_type="Varchar"]
    pub pass_hash: String,
    #[sql_type="Bool"]
    pub is_deleted: bool,
    #[sql_type="Nullable<Varchar>"]
    pub picture: Option<String>,
    #[sql_type="Timestamp"]
    pub creation_datetime: chrono::NaiveDateTime,
}

#[derive(Serialize,Deserialize,Clone,AsChangeset,Queryable,Identifiable)]
#[table_name = "curators"]
#[primary_key(id)]
pub struct UpdateCurator {
    pub id: i64,
    pub picture: Option<Option<String>>,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct NewCurator {
    pub name: String,
    pub surname: String,
    pub patronymic: String,
    pub phone: String,
    pub password: String,
}

impl Curators {
    pub async fn new(
        creds: &NewCurator, 
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::insert_into(curators::table)
            .values(&(
                curators::id.eq(&id),
                curators::name.eq(&creds.name),
                curators::surname.eq(&creds.surname),
                curators::patronymic.eq(&creds.patronymic),
                curators::phone.eq(&creds.phone),
                curators::pass_hash.eq(make_hash(&creds.password)),
            ))
            .execute(conn)?;
        Ok(())
    }

    pub async fn get_random(
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = diesel::sql_query("SELECT * FROM get_random_curator;")
            .get_result(conn)?;
        Ok(r)
    }

    pub async fn from_id(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = curators::table
            .filter(curators::id.eq(id))
            .filter(curators::is_deleted.eq(false))
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn get(
        creds: &AuthData,
        conn: &PgConnection,
    ) -> Result<Self> {
        let pass_hash = make_hash(&creds.password);
        let r = curators::table
            .filter(curators::phone.eq(&creds.phone))
            .filter(curators::pass_hash.eq(pass_hash))
            .filter(curators::is_deleted.eq(false))
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn set(
        instance: &UpdateCurator,
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = diesel::update(curators::table
            .filter(curators::id.eq(instance.id)))
            .set(instance)
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn delete(
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::update(curators::table
            .filter(curators::id.eq(id)))
            .set(
                curators::is_deleted.eq(true)
            )
            .execute(conn)?;
        Ok(())
    }
    
    pub async fn get_all(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        let r = curators::table
            .filter(curators::is_deleted.eq(false))
            .get_results(conn)?;
        Ok(r)
    }

}
