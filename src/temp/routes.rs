use actix_web::{
    web, http, dev, guard,
    App, HttpResponse, client::Client,
    HttpServer, HttpRequest, Responder,
};
use serde::Deserialize;
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
    self as db,
    CoordsWithStamp,
    CourierLocation,
    rm_coords,
    get_coords,
    set_coords,
};

pub fn location_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/location")
        .route("/add", web::post().to(add_location))
        .route("/remove", web::post().to(rm_location))
        .route("/get", web::post().to(get_location))
    );
}

pub async fn add_location(
    auth: Auth,
    form: web::Json<CourierLocation>,
    redis_conn: web::Data<RedisDbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not courier"); 
    require!(auth.id == form.courier_id,"not your id"); 
    let mut conn = redis_conn.get()?;
    let form = form.into_inner();
    set_coords(form, &mut conn).await?;
    Ok(HttpResponse::Ok().json(""))
}

pub async fn get_location(
    auth: Auth,
    redis_conn: web::Data<RedisDbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()) || 
        auth.roles.contains(&"restaurant".to_string()) || 
        auth.roles.contains(&"curator".to_string()),"not admin"); 
    let mut conn = redis_conn.get()?;
    let r = get_coords(&mut conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn rm_location(
    auth: Auth,
    redis_conn: web::Data<RedisDbPool>,
) -> Result<HttpResponse> {
    let mut conn = redis_conn.get()?;
    rm_coords(auth.id, &mut conn).await?;
    Ok(HttpResponse::Ok().json(""))
}
