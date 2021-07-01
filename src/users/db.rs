use actix_web_dev::error::{
    Result,
    ErrorType,
    ApiError,
};
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::schema::{customer,seller};
use ring::digest::{Context, Digest, SHA256};
use data_encoding::BASE64;

#[derive(Serialize,Deserialize,Clone,Queryable,Identifiable)]
#[table_name = "customer"]
#[primary_key(id)]
pub struct Customer {
    pub id: i64,
    pub mail: String,
    pub name: String,
    pub last_name: String,
    pub pass_hash: String,
    pub verifyed: bool,
    pub phone_number: String,
    pub photo: Option<String>,
    pub register_data: chrono::NaiveDateTime,
}

#[derive(Serialize,Deserialize,Clone,AsChangeset,Queryable,Identifiable)]
#[table_name = "customer"]
#[primary_key(id)]
pub struct UpdateCustomer {
    pub id: i64,
    pub name: Option<String>,
    pub last_name: Option<String>,
    pub phone_number: Option<String>,
    pub photo: Option<Option<String>>,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct AuthData {
    pub mail: String,
    pub password: String,
}

fn make_hash(password: &str) -> String {
    let mut context = Context::new(&SHA256); 
    context.update(password.as_bytes());
    let pass_hash = context.finish();
    BASE64.encode(pass_hash.as_ref())
}

impl Customer {
    pub async fn new(
        creds: &AuthData, 
        id: Option<i64>,
        conn: &PgConnection,
    ) -> Result<()> {
        let values = if let Some(id) = id{
        diesel::insert_into(customer::table)
            .values(&(
                customer::id.eq(&id),
                customer::pass_hash.eq(make_hash(&creds.password)),
                customer::mail.eq(&creds.mail),
            ))
            .execute(conn)?;
        } else {
        diesel::insert_into(customer::table)
            .values(&(
                customer::pass_hash.eq(make_hash(&creds.password)),
                customer::mail.eq(&creds.mail),
            ))
            .execute(conn)?;
        };
        Ok(())
    }

    pub async fn from_id(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = customer::table
            .filter(customer::id.eq(id))
            .filter(customer::verifyed.eq(true))
            .get_results::<Self>(conn)?;
        if let Some(u) = r.get(0) {
            Ok(u.clone())
        } else {
            Err(ApiError{
                code: 404,
                message: "seller not found".to_string(),
                error_type: ErrorType::Auth,
            })
        }
    }

    pub async fn get(
        creds: &AuthData,
        conn: &PgConnection,
    ) -> Result<Self> {
        let pass_hash = make_hash(&creds.password);
        let r = customer::table
            .filter(customer::mail.eq(&creds.mail))
            .filter(customer::pass_hash.eq(pass_hash))
            .filter(customer::verifyed.eq(true))
            .get_results::<Self>(conn)?;
        if let Some(u) = r.get(0) {
            Ok(u.clone())
        } else {
            Err(ApiError{
                code: 404,
                message: "seller not found".to_string(),
                error_type: ErrorType::Auth,
            })
        }
    }

    pub async fn update_pass(
        id: i64,
        new_pass: &str,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::update(customer::table
            .filter(customer::id.eq(id)))
            .set(customer::pass_hash.eq(make_hash(new_pass)))
            .execute(conn)?;
        Ok(())
    }

    pub async fn set(
        instance: &UpdateCustomer,
        conn: &PgConnection,
    ) -> Result<Self> {
        let r = diesel::update(customer::table
            .filter(customer::id.eq(instance.id)))
            .set(instance)
            .get_result::<Self>(conn)?;
        Ok(r)
    }

    pub async fn verify(
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
       diesel::update(customer::table.filter(customer::id.eq(id)))
           .set(customer::verifyed.eq(true))
           .execute(conn)?;
        Ok(())
    }
}

#[derive(Serialize,Deserialize,Clone,Queryable,AsChangeset,Identifiable)]
#[table_name = "seller"]
pub struct Seller {
    pub id: i64,
    pub mail: String,
    pub name: String,
    pub last_name: String,
    pub pass_hash: String,
    pub verifyed: bool,
    pub phone_number: String,
    pub bio: String,
    pub photo: Option<String>,
    pub portfolio: Option<Vec<String>>,
    pub register_data: chrono::NaiveDateTime,
}

#[derive(Serialize,Deserialize,Clone,Queryable,AsChangeset)]
#[table_name = "seller"]
pub struct UpdateSeller {
    pub id: i64,
    pub mail: Option<String>,
    pub name: Option<String>,
    pub last_name: Option<String>,
    pub pass_hash: Option<String>,
    pub phone_number: Option<String>,
    pub bio: Option<String>,
    pub photo: Option<Option<String>>,
    pub portfolio: Option<Option<Vec<String>>>,
}

impl Seller {
    pub async fn new(
        creds: &AuthData, 
        id: Option<i64>,
        conn: &PgConnection,
    ) -> Result<Seller> {
        if let Some(id) = id {
            diesel::insert_into(seller::table)
                .values(&(
                    seller::id.eq(id),
                    seller::pass_hash.eq(make_hash(&creds.password)),
                    seller::mail.eq(&creds.mail),
                ))
                .get_result(conn)
                .map_err(|e| e.into())
        } else {
            diesel::insert_into(seller::table)
                .values(&(
                    seller::pass_hash.eq(make_hash(&creds.password)),
                    seller::mail.eq(&creds.mail),
                ))
                .get_result(conn)
                .map_err(|e| e.into())
        }
    }

    pub async fn from_id(
        id: i64,
        conn: &PgConnection,
    ) -> Result<Self> {
        seller::table
            .filter(seller::id.eq(id))
            .get_result::<Self>(conn)
            .map_err(|e| e.into())
    }

    pub async fn get(
        creds: &AuthData,
        conn: &PgConnection,
    ) -> Result<Self> {
        let pass_hash = make_hash(&creds.password);
        seller::table
            .filter(seller::mail.eq(&creds.mail))
            .filter(seller::pass_hash.eq(pass_hash))
            .get_result::<Self>(conn)
            .map_err(|e| e.into())
    }

    pub async fn list_unverifyed(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        seller::table
            .filter(seller::verifyed.eq(false))
            .get_results(conn)
            .map_err(|e| e.into())
    }

    pub async fn list_verifyed(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        seller::table
            .filter(seller::verifyed.eq(true))
            .get_results(conn)
            .map_err(|e| e.into())
    }

    pub async fn delete(
        id: i64,
        conn: &PgConnection,
    ) -> Result<usize> {
        diesel::delete(seller::table
            .filter(seller::id.eq(id)))
            .execute(conn)
            .map_err(|e| e.into())
    }

    pub async fn update_pass(
        id: i64,
        new_pass: &str,
        conn: &PgConnection,
    ) -> Result<()> {
        diesel::update(seller::table
            .filter(seller::id.eq(id)))
            .set(seller::pass_hash.eq(make_hash(&new_pass)))
            .execute(conn)?;
        Ok(())
    }

    pub async fn set(
        instance: &UpdateSeller,
        conn: &PgConnection,
    ) -> Result<Self> {
        diesel::update(seller::table
            .filter(seller::id.eq(instance.id)))
            .set(instance)
            .get_result::<Self>(conn)
            .map_err(|e| e.into())
    }

    pub async fn verify(
        id: i64,
        conn: &PgConnection,
    ) -> Result<()> {
       diesel::update(seller::table.filter(seller::id.eq(id)))
           .set(seller::verifyed.eq(true))
           .execute(conn)?;
        Ok(())
    }
}
