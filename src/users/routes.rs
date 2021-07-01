use actix_web::{
    web, http, dev, guard,
    App, HttpResponse, client::Client,
    HttpServer, HttpRequest, Responder,
};
use serde::Deserialize;
use futures_util::TryFutureExt;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

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
    AuthData,
    Seller,
    Customer,
    UpdateCustomer,
    UpdateSeller
};

pub fn users_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/users")
        .route("/seller", web::post().to(create_seller))
        .route("/customer", web::post().to(create_customer))
        .route("/login", web::post().to(login))
        .route("/customer", web::put().to(update_customer))
        .route("/seller", web::put().to(update_seller))
        .route("/seller/password", web::put().to(update_seller_pass))
        .route("/customer/password", web::put().to(update_customer_pass))
        .route("/seller", web::delete().to(delete_seller))
        .route("/seller/unverifyed", web::get().to(list_unverifyed_sellers))
        .route("/seller/verifyed", web::get().to(list_verifyed_sellers))
        .route("/seller/verify/{id}", web::put().to(verify_seller_by_id))
        .route("/seller/{id}", web::get().to(get_seller_from_id))
        .route("/customer/{id}", web::get().to(get_customer_from_id))
    );
}

#[derive(Deserialize)]
pub struct CreateSeller {
    pub auth: AuthData,
    pub seller: UpdateSeller,
}

pub async fn create_seller(
    form: web::Json<CreateSeller>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let form = form.into_inner();
    let roles = &vec![
        "seller".to_string(),
        "customer".to_string()
    ];
    let a = Auth::new(&form.auth.mail, "plain", roles, &conn).await?;
    let s = Seller::new(&form.auth,Some(a.id), &conn).await?;
    let mut upd = form.seller;
    upd.id = s.id;
    Seller::set(&upd, &conn).await?;
    Ok(HttpResponse::Ok().json(""))
}

pub async fn create_customer(
    form: web::Json<AuthData>,
    conn: web::Data<DbPool>,
    _req: HttpRequest
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let form = form.into_inner();
    let a = Auth::new(&form.mail, "plain", &vec!["customer".to_string()], &conn).await?;
    Customer::new(&form,Some(a.id), &conn).await?;
    Ok(HttpResponse::Ok().json(""))
}

pub async fn login(
    form: web::Json<AuthData>,
    conn: web::Data<DbPool>,
    secret: web::Data<AuthSecret>,
    req: HttpRequest
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let form = form.into_inner();
    if let Ok(customer) = Customer::get(&form, &conn).await {
        let auth = Auth::get(&form.mail, "plain", &conn).await?;
        let jwt = auth.get_jwt(&secret).await?;
        return Ok(HttpResponse::Ok().json(json!({
            "jwt":jwt,
            "customer":customer,
            "is_admin":auth.roles.contains(&"admin".to_string()),
        })));
    }
    if let Ok(seller) = Seller::get(&form, &conn).await {
        if seller.verifyed == false {
            return Ok(HttpResponse::Ok().json(json!({
                "is_admin": false,
                "jwt": (),
                "seller": {
                    "verifyed": false,
                }
            })));
        }
        let auth = Auth::get(&form.mail, "plain", &conn).await?;
        let jwt = auth.get_jwt(&secret).await?;
        return Ok(HttpResponse::Ok().json(json!({
            "jwt":jwt,
            "seller":seller,
            "is_admin":auth.roles.contains(&"admin".to_string()),
        })));
    }
    Err(ApiError{
        code: 403,
        message: "invalid login or password or user not exists".to_string(),
        error_type: ErrorType::Auth,
    })
}

pub async fn update_customer(
    auth: Auth,
    form: web::Json<UpdateCustomer>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!((auth.roles.contains(&"customer".to_string()) &&
        auth.id == form.id) || 
        auth.roles.contains(&"admin".to_string()),"not your account");
    let conn = conn.get()?;
    let r = Customer::set(&form.into_inner(),&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

#[derive(Deserialize)]
pub struct Pass {
    new_password: String,
}

pub async fn update_customer_pass(
    auth: Auth,
    form: web::Json<Pass>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not admin");
    let conn = conn.get()?;
    Customer::update_pass(auth.id,&form.new_password,&conn).await?;
    Ok(HttpResponse::Ok().json(""))
}

pub async fn list_unverifyed_sellers(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not admin");
    let conn = conn.get()?;
    let r = Seller::list_unverifyed(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn list_verifyed_sellers(
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let r = Seller::list_verifyed(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn update_seller_pass(
    auth: Auth,
    form: web::Json<Pass>,
    conn: web::Data<DbPool>,
    req: HttpRequest
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    Seller::update_pass(auth.id,&form.new_password,&conn).await?;
    Ok(HttpResponse::Ok().json(""))
}
#[derive(Deserialize)]
pub struct Id {
    id: i64,
}

pub async fn verify_seller_by_id(
    path: web::Path<i64>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let id = path.0;
    Seller::verify(id, &conn).await?;
    Ok(HttpResponse::Ok().json(""))
}

pub async fn get_customer_from_id(
    path: web::Path<i64>,
    conn: web::Data<DbPool>,
    req: HttpRequest
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let id = path.0;
    let r = Customer::from_id(id,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_seller_from_id(
    path: web::Path<i64>,
    conn: web::Data<DbPool>,
    req: HttpRequest
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let id = path.0;
    let r = Seller::from_id(id,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn update_seller(
    auth: Auth,
    form: web::Json<UpdateSeller>,
    conn: web::Data<DbPool>,
    req: HttpRequest
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()) ||
        (auth.roles.contains(&"seller".to_string()) &&
            auth.id == form.id) ,"not permited");
    let conn = conn.get()?;
    let r = Seller::set(&form,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn delete_seller(
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
    req: HttpRequest
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let r = Seller::delete(form.id,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}
