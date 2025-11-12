use actix_web::{web, App, HttpServer, http, middleware::Logger, cookie::{Key, SameSite}};
use actix_cors::Cors;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_limitation::Limiter;
use std::time::Duration;
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
    let allow_origin = config.client_host.clone().unwrap_or("http://localhost:3000".into());
    
    // Requirements: 11.2 - CSRF protection with SameSite cookie attributes
    let session_secret = Key::from(&config.get_session_secret());
    let cookie_secure = config.is_cookie_secure();
    
    // Requirements: 11.2 - Rate limiting to prevent brute force attacks
    let rate_limiter = web::Data::new(
        Limiter::builder("redis://127.0.0.1:6379")
            .limit(config.get_rate_limit_requests())
            .period(Duration::from_secs(config.get_rate_limit_period_secs()))
            .build()
            .unwrap_or_else(|e| {
                log::warn!("Failed to connect to Redis for rate limiting: {}. Using in-memory limiter.", e);
                // Fallback to in-memory rate limiter if Redis is not available
                Limiter::builder("memory://")
                    .limit(config.get_rate_limit_requests())
                    .period(Duration::from_secs(config.get_rate_limit_period_secs()))
                    .build()
                    .expect("Failed to create in-memory rate limiter")
            })
    );

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&allow_origin)
            .allowed_origin("http://localhost:8080")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
            .expose_headers(vec![http::header::AUTHORIZATION]);

        // Configure session middleware with SameSite protection
        let session_middleware = SessionMiddleware::builder(
            CookieSessionStore::default(),
            session_secret.clone()
        )
        .cookie_name("rust-api-session".to_string())
        .cookie_secure(cookie_secure)
        .cookie_same_site(SameSite::Strict)  // CSRF protection via SameSite
        .cookie_http_only(true)  // Prevent XSS attacks
        .build();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(rate_limiter.clone())  // Requirements: 11.2 - Rate limiter for login endpoint
            .wrap(cors)
            .wrap(session_middleware)  // Requirements: 11.2 - Session with CSRF protection
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
