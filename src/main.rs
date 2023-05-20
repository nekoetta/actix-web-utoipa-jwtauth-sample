use actix_web::{web, App, HttpServer, http, middleware::Logger};
use actix_cors::Cors;
use rust_api::{create_connection_pool, DbPool, services, config::get_config};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool: DbPool = create_connection_pool();
    let config = get_config();
    let allow_origin = config.unwrap().client_host.unwrap_or("http://localhost:3000".into());

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&allow_origin)
            .allowed_origin("http://localhost:8080")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
            .expose_headers(vec![http::header::AUTHORIZATION]);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .configure(services::api::config)
            .configure(services::auth::config)
            .service(rust_api::swagger::ui())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
