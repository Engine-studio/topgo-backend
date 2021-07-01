use actix_web::{
    web, http, dev, guard,
    App, HttpResponse, client::Client,
    HttpServer, HttpRequest, Responder,
};
use serde::Deserialize;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use reqwest;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

use actix_web_dev::error::{
    Result,
    ApiError,
    ErrorType,
};
use actix_web_dev::auth::{
    Auth,
};
use super::db::{
    Payment,
    NewPayment
};

pub fn payment_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/payments")
        .route("/ftf", web::post().to(create_ftf_payment))
        .route("", web::get().to(list_payments))
        .route("", web::put().to(set_processed))
        .route("", web::delete().to(delete_payment))
        .route("/{id}", web::get().to(list_payments_by_id))
    );
}

pub async fn create_ftf_payment(
    auth: Auth,
    form: web::Json<NewPayment>,
    conn: web::Data<DbPool>,
    _req: HttpRequest
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"customer".to_string()),"not a customer");
    require!(auth.id == form.customer_id, "not your id");
    require!(form.order_type == "ftf","not a ftf payment");
    let conn = conn.get()?;
    let form = form.into_inner();
    let r = Payment::new(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn list_payments(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()), "not admin");
    let conn = conn.get()?;
    let r = Payment::list_orders(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn list_payments_by_id(
    auth: Auth,
    path: web::Path<i64>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"customer".to_string()), "not a customer");
    require!(auth.id == path.0, "not your id");
    let conn = conn.get()?;
    let r = Payment::list_orders_by_id(path.0,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

#[derive(Deserialize)]
pub struct Id {
    id: i64,
}

pub async fn set_processed(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
    req: HttpRequest
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not admin");
    let conn = conn.get()?;
    let r = Payment::to_delivered(form.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn delete_payment(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not admin");
    let conn = conn.get()?;
    let r = Payment::rollback(form.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}
