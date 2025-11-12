use actix_web::{web, App, HttpServer, http, middleware::Logger};
use actix_cors::Cors;
use rust_api::{create_connection_pool, DbPool, services, config::get_config, init_telemetry, middleware::TracingMiddleware};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration
    let config = get_config().map_err(|e| {
        eprintln!("Failed to load configuration: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;
    
    // Initialize telemetry (OpenTelemetry or env_logger)
    // Requirements: 13.1, 13.2 - Enable/disable OpenTelemetry via environment variables
    if config.is_otel_enabled() {
        // Initialize OpenTelemetry tracing
        if let Err(e) = init_telemetry(&config) {
            eprintln!("Failed to initialize OpenTelemetry: {}", e);
            eprintln!("Falling back to env_logger");
            env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
        } else {
            log::info!("Using OpenTelemetry for tracing");
        }
    } else {
        // Use traditional env_logger when OpenTelemetry is disabled
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
        log::info!("Using env_logger for logging (OpenTelemetry disabled)");
    }
    
    let pool: DbPool = create_connection_pool();
    let allow_origin = config.client_host.unwrap_or("http://localhost:3000".into());

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
            .wrap(TracingMiddleware)  // Requirements: 14.1 - Add HTTP tracing middleware
            .wrap(Logger::default())
            .configure(services::api::config)
            .configure(services::auth::config)
            .service(rust_api::swagger::ui())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
