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
        .route("/get_coords_by_address", web::post().to(get_coords_by_address))
        .route("/get_route_cost", web::post().to(get_distance_pay))
    );
}

#[derive(Deserialize)]
pub struct Dist {
    from_lat: f64,
    from_lng: f64,
    to_lat: f64,
    to_lng: f64,
}

pub async fn get_distance_pay(
    auth: Auth,
    data: web::Json<Dist>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
        use serde_json::Value;
        let url = format!(
            "http://router.project-osrm.org/route/v1/car/{},{};{},{}",data.from_lng,
            data.from_lat,data.to_lng,data.to_lat); 
        let resp: Value = reqwest::get(&url)
            .await?
            .json()
            .await?;
        let dist = resp["routes"][0]["distance"]
                .as_f64()
                .ok_or_else(||ApiError {
                    code: 500,
                    message: "error in getting coords from json".to_string(),
                    error_type: ErrorType::InternalError,
                })?;
        Ok(HttpResponse::Ok().json(json!({
            "cost": dist as i64
        })))
}

#[derive(Deserialize)]
pub struct Address {
    address: String,
}

pub async fn get_coords_by_address(
    auth: Auth,
    data: web::Json<Address>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
        use serde_json::Value;
        println!("data address {}",data.address);
        let url = "https://nominatim.openstreetmap.org/search/".to_string() +
            &data.address
            +"?format=json&addressdetails=1&limit=1&polygon_svg=1";

        let cl = reqwest::Client::new()
            .get(&url)
            .header("Accept-Encoding","gzip, deflate, br,utf-8")
            .header("User-Agent","PostmanRuntime/7.28.4")
            .header("Host","topgo.club");
        let resp: Value = cl.send() 
            .await?
            .json()
            .await?;
        println!("{:?}",resp);
        let lat = resp[0]["lat"].as_str().unwrap().parse::<f64>().unwrap();
        let lng = resp[0]["lon"].as_str().unwrap().parse::<f64>().unwrap();
        Ok(HttpResponse::Ok().json(json!({
            "lat": lat,
            "lng": lng,
        })))
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
    let mut form = form.into_inner();
    Orders::create_order(&mut form, &conn).await?;
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
    redis_conn: web::Data<RedisDbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"courier".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let mut conn_redis = redis_conn.get()?;
    let r = Sessions::finish(auth.id, &conn, &mut conn_redis).await?;
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
