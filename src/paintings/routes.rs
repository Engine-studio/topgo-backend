use actix_web::{
    web,
    HttpResponse,  
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
use actix_web_dev::auth::Auth;
use super::db::{
    Painting,
    NewPainting,
    UpdatePainting,
};

pub fn painting_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/paintings")
        .route("", web::post().to(create_painting))
        .route("", web::put().to(update_painting))
        .route("/verify", web::put().to(verify))
        .route("/get/unverifyed", web::get().to(get_unverifyed))
        .route("/get/verifyed", web::get().to(get_verifyed))
        .route("", web::delete().to(delete_painting))
        .route("/{id}", web::get().to(get_painting_from_id))
    );
}

pub async fn create_painting(
    auth: Auth,
    form: web::Json<NewPainting>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"seller".to_string()),"not seller");
    require!(form.seller_id == auth.id,"not your seller id");
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
    Painting::new(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(""))
}

pub async fn update_painting(
    auth: Auth,
    form: web::Json<UpdatePainting>, 
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
    let r = Painting::set(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

#[derive(Deserialize)]
pub struct Id {
    id: i64,
}

pub async fn verify(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()), "not admin");
    let conn = conn.get()?;
    Painting::verify(form.id, &conn).await?;
    Ok(HttpResponse::Ok().json(""))
}

pub async fn get_unverifyed(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()), "not admin");
    let conn = conn.get()?;
    let r = Painting::get_unverifyed(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}


pub async fn get_verifyed(
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let r = Painting::get_verifyed(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_painting_from_id(
    path: web::Path<i64>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let id = path.0;
    let r = Painting::from_id(id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn delete_painting(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not admin");
    let conn = conn.get()?;
    let r = Painting::delete(form.id, &conn).await?;
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

