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
    Event,
    NewEvent,
    UpdateEvent,
};

pub fn event_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/event")
        .route("", web::delete().to(delete_event))
        .route("", web::put().to(update_event))
        .route("", web::post().to(create_event))
        .route("", web::get().to(get_events))
        .route("/{id}", web::get().to(get_event_from_id))
    );
}

pub async fn create_event(
    auth: Auth,
    form: web::Json<NewEvent>,
    conn: web::Data<DbPool>,
    _req: HttpRequest
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not seller");
    let conn = conn.get()?;
    let form = form.into_inner();
    //let client = reqwest::Client::new();
    //let status = client
    //    .post("http://uploader:8088/upload/verify")
    //    .json(&vec![
    //        form.picture.clone(),
    //    ])
    //    .send()
    //    .await?
    //    .status()
    //    .as_u16();
    //require!(status == 200, "can't verify pictures");
    let r = Event::new(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn update_event(
    auth: Auth,
    form: web::Json<UpdateEvent>, 
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()), "not admin");
    let conn = conn.get()?;
    let form = form.into_inner();
    //let client = reqwest::Client::new();
    //let status = client
    //    .post("http://uploader:8088/upload/verify")
    //    .json(&vec![
    //        form.picture.clone(),
    //    ])
    //    .send()
    //    .await?
    //    .status()
    //    .as_u16();
    //require!(status == 200, "can't verify pictures");
    let r = Event::set(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

#[derive(Deserialize)]
pub struct Id {
    id: i64,
}

pub async fn get_event_from_id(
    path: web::Path<i64>,
    conn: web::Data<DbPool>,
    req: HttpRequest
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let id = path.0;
    let r = Event::from_id(id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_events(
    conn: web::Data<DbPool>,
    req: HttpRequest
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let r = Event::all(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn delete_event(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not admin");
    let conn = conn.get()?;
    let r = Event::delete(form.id, &conn).await?;
    //let client = reqwest::Client::new();
    //let status = client
    //    .post("http://uploader:8088/upload/delete")
    //    .json(&vec![
    //        r.picture,
    //    ])
    //    .send()
    //    .await?
    //    .status()
    //    .as_u16();
    //require!(status == 200, "can't verify pictures");
    Ok(HttpResponse::Ok().json(""))
}
