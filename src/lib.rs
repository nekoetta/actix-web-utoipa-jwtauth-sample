use diesel::pg::PgConnection;
use diesel::r2d2;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub type DbConnection = PgConnection;
pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<DbConnection>>;

pub fn create_connection_pool() -> DbPool {
    let config = config::get_config().expect("Failed to load configuration");

    let manager = r2d2::ConnectionManager::<DbConnection>::new(&config.database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool - check DATABASE_URL")
}

// Test helper function - always available but only used in tests
// This is exported publicly so integration tests can use it
pub fn create_test_connection_pool() -> DbPool {
    let config = config::get_config().expect("Failed to load test configuration");

    let manager = r2d2::ConnectionManager::<DbConnection>::new(&config.test_database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create test database connection pool - check TEST_DATABASE_URL");
    let mut conn = pool.get().expect("Failed to get connection from test pool");
    run_test_migrations(&mut conn).expect("Failed to run test migrations");
    pool
}

fn run_test_migrations(connection: &mut impl diesel_migrations::MigrationHarness<diesel::pg::Pg>) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
    
    // Check if migrations have already been applied
    let applied_migrations = connection.applied_migrations()?;
    
    if applied_migrations.is_empty() {
        // No migrations applied yet, run them
        connection.run_pending_migrations(MIGRATIONS)?;
    }
    // If migrations are already applied, do nothing
    
    Ok(())
}

pub mod config;
pub mod models;
pub mod schema;
pub mod services;
pub mod swagger;
pub mod middleware;
pub mod errors;
pub mod traits;
pub mod metrics;
pub mod constants;

/// Initialize OpenTelemetry tracing and metrics with OTLP exporter
/// 
/// This function sets up the OpenTelemetry tracing and metrics pipeline with:
/// - OTLP exporter configured to send traces and metrics to the specified endpoint
/// - Resource attributes (service name and version)
/// - Integration with tracing-subscriber for unified logging
/// - Metrics collection for HTTP, database, and authentication operations
/// 
/// # Arguments
/// * `config` - Configuration containing OpenTelemetry settings
/// 
/// # Returns
/// * `Ok(())` if initialization succeeds or OpenTelemetry is disabled
/// * `Err(Box<dyn Error>)` if initialization fails
/// 
/// # Requirements
/// - 12.4: OpenTelemetry integration for distributed tracing
/// - 12.5: Metrics collection for observability
/// - 14.2: Minimal invasive implementation
pub fn init_telemetry(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::global;
    use opentelemetry_otlp::WithExportConfig;
    
    // Skip initialization if OpenTelemetry is disabled
    if !config.is_otel_enabled() {
        log::info!("OpenTelemetry is disabled");
        return Ok(());
    }

    log::info!(
        "Initializing OpenTelemetry with endpoint: {}, service: {}, version: {}",
        config.get_otel_endpoint(),
        config.get_otel_service_name(),
        config.get_otel_service_version()
    );

    // Initialize OTLP span exporter with tonic (gRPC)
    let trace_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(config.get_otel_endpoint())
        .build()?;

    // Create tracer provider with batch exporter and resource attributes
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(trace_exporter)
        .build();

    // Set global tracer provider
    global::set_tracer_provider(tracer_provider.clone());

    // Get tracer from global provider
    let tracer = global::tracer(config.get_otel_service_name());

    // Initialize OTLP metrics exporter with tonic (gRPC)
    // Requirements: 12.5 - Metrics collection
    let metrics_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(config.get_otel_endpoint())
        .build()?;

    // Create metrics reader with periodic export
    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(metrics_exporter)
        .with_interval(std::time::Duration::from_secs(30))
        .build();

    // Create meter provider with reader
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .build();

    // Set global meter provider
    global::set_meter_provider(meter_provider);

    // Create OpenTelemetry tracing layer
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create environment filter for log levels
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));

    // Initialize tracing subscriber with OpenTelemetry layer
    tracing_subscriber::registry()
        .with(telemetry_layer)
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    log::info!("OpenTelemetry tracing and metrics initialized successfully");
    Ok(())
}
