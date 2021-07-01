use actix_web::{
    web, http, dev, guard,
    App, HttpResponse, client::Client,
    HttpServer,
};
use actix_web_dev::auth::*;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use actix_web::middleware::Logger;
use diesel_migrations::run_pending_migrations;
extern crate env_logger;
use topgo::users::routes::users_routes;
use topgo::events::routes::event_routes;
use topgo::paintings::routes::painting_routes;
use topgo::payments::routes::payment_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    match run_pending_migrations(&pool.get().unwrap()) {
        Ok(_) => print!("migration success\n"),
        Err(e)=> print!("migration error: {}\n",&e),
    };

    actix_web_dev::init_auth(&pool.get().unwrap());
    let secret = Auth::gen_secret();

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    println!("starting server...");
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(secret.clone())
            .wrap(Logger::default())
            .service(web::scope("/api")
                .configure(users_routes)
                .configure(event_routes)
                .configure(painting_routes)
                .configure(payment_routes)
            )
    })
    .bind("0.0.0.0:8088")?
    .system_exit()
    .run()
    .await
}
