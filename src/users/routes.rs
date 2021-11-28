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
    Couriers,
    Curators,
    Admins,
    Restaurants,
    NewAdmin,
    NewCurator,
    NewCourier,
    NewRestaurant,
    UpdateCourier,
    UpdateCurator,
    UpdateAdmin,
    CouriersInfo,
    CouriersForAdmin,
    RestaurantsInfo,
    NullMoney,
    CouriersHistory,
};

#[derive(Deserialize)]
pub struct Id {
    id: i64,
}

pub fn users_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/users")
        .route("/login", web::post().to(login))
        .service(web::scope("/admin")
            .route("/new", web::post().to(create_admin))
            .route("/update", web::post().to(update_admin))
            .route("/delete", web::post().to(delete_admin))
        )
        .service(web::scope("/restaurants")
            .route("/new", web::post().to(create_restaurant))
            .route("/delete", web::post().to(delete_restaurant))
            .route("/get_all", web::post().to(get_all_restaurant))
            .route("/order_info", web::post().to(order_info_restaurant))
            .route("/order_history", web::post().to(order_history_restaurant))
        )
        .service(web::scope("/curators")
            .route("/new", web::post().to(create_curator))
            .route("/update", web::post().to(update_curator))
            .route("/delete", web::post().to(delete_curator))
            .route("/get_random", web::post().to(get_random_curator))
            .route("/get_all", web::post().to(get_all_curator))
        )
        .service(web::scope("/couriers")
            .route("/new", web::post().to(create_courier))
            .route("/update", web::post().to(update_courier))
            .route("/delete", web::post().to(delete_courier))
            .route("/toggle_ban", web::post().to(toggle_ban_courier))
            .route("/get_all", web::post().to(get_all_courier))
            .route("/couriers_for_administration", web::post().to(couriers_for_administration))
            .route("/get_current", web::post().to(get_self_current_courier))
            .route("/get", web::post().to(get_self_courier))
            .route("/get_history", web::post().to(get_self_history_courier))
            .route("/null_money", web::post().to(null_money_couriers))
        )
    );
}

pub async fn order_history_restaurant(
    auth: Auth,
    conn: web::Data<DbPool>,
    redis_conn: web::Data<RedisDbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"restaurant".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = RestaurantsInfo::get_history_by(auth.id, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn order_info_restaurant(
    auth: Auth,
    conn: web::Data<DbPool>,
    redis_conn: web::Data<RedisDbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"restaurant".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let mut redis_conn = redis_conn.get()?;
    let r = RestaurantsInfo::get_by(auth.id, &conn).await?;
    use crate::temp::db::get_coords;
    let c = get_coords(&mut redis_conn).await?;
    Ok(HttpResponse::Ok().json(json!({
            "orders": r,
            "coords": c,
    })))
}

pub async fn null_money_couriers(
    auth: Auth,
    form: web::Json<NullMoney>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"curator".to_string()) ||
        auth.roles.contains(&"admin".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = Couriers::null_money(&form, &conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_self_history_courier(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let r = CouriersHistory::get(auth.id,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_self_current_courier(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let r = CouriersHistory::get_curr(auth.id,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_self_courier(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let r = CouriersInfo::get_by(auth.id,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

use r2d2_redis::{RedisConnectionManager, r2d2, redis::{self, Commands}};
pub type RedisDbPool = r2d2::Pool<RedisConnectionManager>;
pub async fn couriers_for_administration(
    auth: Auth,
    conn: web::Data<DbPool>,
    redis_conn: web::Data<RedisDbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"curator".to_string()) ||
        auth.roles.contains(&"admin".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let mut redis_conn = redis_conn.get()?;
    let r = CouriersForAdmin::get(&conn).await?;
    use crate::temp::db::get_coords;
    let c = get_coords(&mut redis_conn).await?;
    Ok(HttpResponse::Ok().json(json!({
            "couriers": r,
            "coords": c,
    })))
}

pub async fn toggle_ban_courier(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"curator".to_string()) ||
        auth.roles.contains(&"admin".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = Couriers::toggle_ban(form.id,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_random_curator(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = conn.get()?;
    let r = Curators::get_random(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_all_courier(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"curator".to_string()) ||
        auth.roles.contains(&"admin".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = Couriers::get_all(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_all_curator(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()), "not admin");
    let conn = conn.get()?;
    let r = Curators::get_all(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_all_restaurant(
    auth: Auth,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"curator".to_string()) ||
        auth.roles.contains(&"admin".to_string()),"not permitted"); 
    let conn = conn.get()?;
    let r = Restaurants::get_all(&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn create_admin(
    auth: Auth,
    form: web::Json<NewAdmin>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not admin"); 
    let conn = conn.get()?;
    let form = form.into_inner();
    let roles = &vec![
        "admin".to_string(),
    ];
    let a = Auth::new(&form.phone, "plain", roles, &conn).await?;
    let s = Admins::new(&form,a.id, &conn).await?;
    Ok(HttpResponse::Ok().json(""))
}

pub async fn create_courier(
    auth: Auth,
    form: web::Json<NewCourier>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()) ||
        auth.roles.contains(&"curator".to_string()),"not admin or curator"); 
    let conn = conn.get()?;
    let form = form.into_inner();
    let roles = &vec![
        "courier".to_string(),
    ];
    let a = Auth::new(&form.phone, "plain", roles, &conn).await?;
    let s = Couriers::new(&form,a.id, &conn).await?;
    Ok(HttpResponse::Ok().json(""))
}

pub async fn create_curator(
    auth: Auth,
    form: web::Json<NewCurator>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    println!("{:?}",auth.roles);
    require!(auth.roles.contains(&"admin".to_string()),"not admin"); 
    let conn = conn.get()?;
    let form = form.into_inner();
    let roles = &vec![
        "curator".to_string(),
    ];
    let a = Auth::new(&form.phone, "plain", roles, &conn).await?;
    let s = Curators::new(&form,a.id, &conn).await?;
    Ok(HttpResponse::Ok().json(""))
}

pub async fn create_restaurant(
    auth: Auth,
    form: web::Json<NewRestaurant>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not admin"); 
    let conn = conn.get()?;
    let form = form.into_inner();
    let roles = &vec![
        "restaurant".to_string(),
    ];
    let a = Auth::new(&form.phone, "plain", roles, &conn).await?;
    let s = Restaurants::new(&form,a.id, &conn).await?;
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
    if let Ok(admin) = Admins::get(&form, &conn).await {
        let auth = Auth::get(&form.phone, "plain", &conn).await?;
        let jwt = auth.get_jwt(&secret).await?;
        return Ok(HttpResponse::Ok().json(json!({
            "jwt":jwt,
            "admin":admin,
        })));
    }
    if let Ok(restaurant) = Restaurants::get(&form, &conn).await {
        let auth = Auth::get(&form.phone, "plain", &conn).await?;
        let jwt = auth.get_jwt(&secret).await?;
        return Ok(HttpResponse::Ok().json(json!({
            "jwt":jwt,
            "restaurant":restaurant,
        })));
    }
    if let Ok(curator) = Curators::get(&form, &conn).await {
        let auth = Auth::get(&form.phone, "plain", &conn).await?;
        let jwt = auth.get_jwt(&secret).await?;
        return Ok(HttpResponse::Ok().json(json!({
            "jwt":jwt,
            "curator":curator,
        })));
    }
    if let Ok(courier) = Couriers::get(&form, &conn).await {
        let auth = Auth::get(&form.phone, "plain", &conn).await?;
        let jwt = auth.get_jwt(&secret).await?;
        let session = Couriers::get_session(auth.id, &conn)
            .await
            .ok();
        return Ok(HttpResponse::Ok().json(json!({
            "jwt": jwt,
            "courier": courier,
            "session": session,
        })));
    }
    Err(ApiError{
        code: 403,
        message: "invalid login or password or user not exists".to_string(),
        error_type: ErrorType::Auth,
    })
}

pub async fn update_admin(
    auth: Auth,
    form: web::Json<UpdateAdmin>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.id == form.id,"not your account");
    let conn = conn.get()?;
    let r = Admins::set(&form.into_inner(),&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn update_courier(
    auth: Auth,
    form: web::Json<UpdateCourier>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.id == form.id,"not your account");
    let conn = conn.get()?;
    let r = Couriers::set(&form.into_inner(),&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn update_curator(
    auth: Auth,
    form: web::Json<UpdateCurator>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.id == form.id,"not your account");
    let conn = conn.get()?;
    let r = Curators::set(&form.into_inner(),&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}



pub async fn delete_admin(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not admin"); 
    let conn = conn.get()?;
    let r = Admins::delete(form.id,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn delete_courier(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()) ||
        auth.roles.contains(&"curator".to_string()),"not admin or curator"); 
    let conn = conn.get()?;
    let r = Couriers::delete(form.id,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn delete_curator(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not admin"); 
    let conn = conn.get()?;
    let r = Curators::delete(form.id,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}

pub async fn delete_restaurant(
    auth: Auth,
    form: web::Json<Id>,
    conn: web::Data<DbPool>,
) -> Result<HttpResponse> {
    require!(auth.roles.contains(&"admin".to_string()),"not admin"); 
    let conn = conn.get()?;
    let r = Restaurants::delete(form.id,&conn).await?;
    Ok(HttpResponse::Ok().json(r))
}
