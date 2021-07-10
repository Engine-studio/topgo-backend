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
    Orders,
    NewOrder,
    Sessions,
    NewSession,
    OrderRequest,
    Notification,
    Finalization,
    CourierRating,
};

#[derive(Serialize,Deserialize)]
pub struct CourierToOrder {
    pub courier_id: i64,
    pub order_id: i64,
}

pub fn ordering_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/ordering")
        .route("/new", web::post().to(new_order))
        .route("/get_orders", web::post().to(get_orders))
        .route("/take_order", web::post().to(take_order))
        .route("/refuse_order", web::post().to(refuse_order))
        .route("/pick_order", web::post().to(pick_order))
        .route("/set_delivered_order", web::post().to(set_delivered_order))
        .route("/set_ready_for_delivery_order", web::post().to(ready_for_delivery_order))
        .route("/rate_courier", web::post().to(rate_courier))
        .route("/finalize_order", web::post().to(finalize_order))
        .route("/send_notification", web::post().to(send_notifiaction))
        .route("/get_notifications", web::post().to(get_notifications))
        .route("/get_by_restaurant", web::post().to(get_by_restaurant))
        .route("/create_session", web::post().to(create_session))
        .route("/cancel_session", web::post().to(cancel_session))
        .route("/get_session", web::post().to(get_session))
        .route("/get_orders_by_session_id", web::post().to(get_orders_by_session_id))
        .route("/get_orders_by_couriers_id", web::post().to(get_orders_by_couriers_id))
    );
}

pub async fn get_orders_by_couriers_id(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = Orders::get_orders_by_courier_id(auth.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_notifications(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = Notification::get(auth.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn send_notifiaction(
    auth: Auth,
    form: web::Json<Notification>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()) ||
        auth.roles.contains(&"curator".to_string()),"not permitted"); 
    let conn = conn.get()?;
    Notification::new(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(()))
}

pub async fn ready_for_delivery_order(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"restaurant".to_string()),"not permitted"); 
    let conn = conn.get()?;
    Orders::set_ready_for_delivery(form.id,&conn).await?;
    Ok(HttpResponse::Ok().json(()))
}

pub async fn set_delivered_order(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    let conn = conn.get()?;
    Orders::set_delivered(form.id,&conn).await?;
    Ok(HttpResponse::Ok().json(()))
}

pub async fn rate_courier(
    auth: Auth,
    form: web::Json<CourierRating>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"restaurant".to_string()),"not permitted"); 
    let conn = conn.get()?;
    CourierRating::new(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(()))
}

pub async fn finalize_order(
    auth: Auth,
    form: web::Json<Finalization>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"restaurant".to_string()),"not permitted"); 
    let conn = conn.get()?;
    Orders::finalize_order(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(()))
}

pub async fn new_order(
    auth: Auth,
    form: web::Json<NewOrder>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"restaurant".to_string()),"not permitted"); 
    require!(auth.id == form.restaurant_id, "not your id");
    let conn = conn.get()?;
    let form = form.into_inner();
    Orders::create_order(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(()))
}

pub async fn get_orders(
    auth: Auth,
    form: web::Json<OrderRequest>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    require!(auth.id == form.courier_id,"not you");
    let conn = conn.get()?;
    let form = form.into_inner();
    let r = Orders::get_suggested(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

#[derive(Serialize,Deserialize)]
pub struct Id {
    pub id: i64,
}

pub async fn get_by_restaurant(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"restaurant".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = Orders::get_orders_by_rest_id(auth.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn refuse_order(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let form = form.into_inner();
    let r = Orders::refuse_order(form.id, auth.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn take_order(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let form = form.into_inner();
    let r = Orders::take_order(form.id, auth.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_orders_by_session_id(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let form = form.into_inner();
    let r = Orders::get_orders_by_session_id(form.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn pick_order(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let form = form.into_inner();
    let r = Orders::pick_order(form.id, auth.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn create_session(
    auth: Auth,
    mut form: web::Json<NewSession>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    form.courier_id = auth.id;
    let conn = conn.get()?;
    let r = Sessions::new(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn cancel_session(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = Sessions::finish(auth.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_session(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let r = Sessions::get_by_courier(form.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}
