use actix_web::{
    web, http, dev, guard,
    App, HttpResponse, client::Client,
    HttpServer, HttpRequest, Responder,
};
use serde::{Serialize,Deserialize};
use futures_util::TryFutureExt;
use r2d2_redis::{RedisConnectionManager, r2d2, redis::{self, Commands}};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type RedisDbPool = r2d2::Pool<RedisConnectionManager>;

use actix_web_dev::error::{
    Result,
    ApiError,
    ErrorType,
};
use actix_web_dev::auth::{
    Auth,
    AuthSecret,
};
use super::db::{
    CourierReport,
    CourierCuratorReport,
    RestaurantReport,
    RestaurantCuratorReport,
};

#[derive(Serialize,Deserialize)]
pub struct CourierToOrder {
    pub courier_id: i64,
    pub order_id: i64,
}

pub fn reports_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/reports")
        .route("/restaurant", web::post().to(restaurant_get))
        .route("/restaurant_curators", web::post().to(restaurant_curators_get))
        .route("/courier", web::post().to(courier_get))
        .route("/courier_curators", web::post().to(courier_curators_get))
    );
}

pub async fn restaurant_get(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"restaurant".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = RestaurantReport::get(auth.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn courier_get(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = CourierReport::get(auth.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn courier_curators_get(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"curator".to_string()) ||
        auth.roles.contains(&"admin".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = CourierCuratorReport::get(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn restaurant_curators_get(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"curator".to_string()) ||
        auth.roles.contains(&"admin".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = RestaurantCuratorReport::get(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

