use lazy_static::lazy_static;
use opentelemetry::{
    metrics::{Counter, Histogram, UpDownCounter, Meter},
    KeyValue,
};
use std::time::Instant;

lazy_static! {
    static ref METER: Meter = opentelemetry::global::meter("rust-api");
    
    // HTTP Metrics
    static ref HTTP_REQUESTS_TOTAL: Counter<u64> = METER
        .u64_counter("http_requests_total")
        .with_description("Total number of HTTP requests")
        .build();
    
    static ref HTTP_REQUEST_DURATION: Histogram<f64> = METER
        .f64_histogram("http_request_duration_seconds")
        .with_description("HTTP request duration in seconds")
        .build();
    
    static ref HTTP_REQUESTS_IN_FLIGHT: UpDownCounter<i64> = METER
        .i64_up_down_counter("http_requests_in_flight")
        .with_description("Number of HTTP requests currently being processed")
        .build();
    
    // Database Metrics
    static ref DB_QUERIES_TOTAL: Counter<u64> = METER
        .u64_counter("db_queries_total")
        .with_description("Total number of database queries")
        .build();
    
    static ref DB_QUERY_DURATION: Histogram<f64> = METER
        .f64_histogram("db_query_duration_seconds")
        .with_description("Database query duration in seconds")
        .build();
    
    // Authentication Metrics
    static ref AUTH_ATTEMPTS_TOTAL: Counter<u64> = METER
        .u64_counter("auth_attempts_total")
        .with_description("Total number of authentication attempts")
        .build();
    
    static ref JWT_VALIDATIONS_TOTAL: Counter<u64> = METER
        .u64_counter("jwt_validations_total")
        .with_description("Total number of JWT token validations")
        .build();
}

/// HTTP Metrics
pub struct HttpMetrics;

impl HttpMetrics {
    /// Record an HTTP request
    pub fn record_request(method: &str, path: &str, status: u16) {
        let labels = [
            KeyValue::new("method", method.to_string()),
            KeyValue::new("path", path.to_string()),
            KeyValue::new("status", status.to_string()),
        ];
        
        HTTP_REQUESTS_TOTAL.add(1, &labels);
    }
    
    /// Record HTTP request duration
    pub fn record_duration(method: &str, path: &str, duration_secs: f64) {
        let labels = [
            KeyValue::new("method", method.to_string()),
            KeyValue::new("path", path.to_string()),
        ];
        
        HTTP_REQUEST_DURATION.record(duration_secs, &labels);
    }
    
    /// Increment in-flight requests counter
    pub fn increment_in_flight() {
        HTTP_REQUESTS_IN_FLIGHT.add(1, &[]);
    }
    
    /// Decrement in-flight requests counter
    pub fn decrement_in_flight() {
        HTTP_REQUESTS_IN_FLIGHT.add(-1, &[]);
    }
}

/// Database Metrics
pub struct DbMetrics;

impl DbMetrics {
    /// Record a database query
    pub fn record_query(operation: &str) {
        let labels = [KeyValue::new("operation", operation.to_string())];
        DB_QUERIES_TOTAL.add(1, &labels);
    }
    
    /// Record database query duration
    pub fn record_duration(operation: &str, duration_secs: f64) {
        let labels = [KeyValue::new("operation", operation.to_string())];
        DB_QUERY_DURATION.record(duration_secs, &labels);
    }
    
    /// Record connection pool metrics
    /// This should be called periodically to track pool state
    pub fn record_pool_state(pool_size: u32, idle_connections: u32) {
        let pool_size_gauge = METER
            .u64_gauge("db_connection_pool_size")
            .with_description("Current size of the database connection pool")
            .build();
        
        let idle_gauge = METER
            .u64_gauge("db_connection_pool_idle")
            .with_description("Number of idle connections in the pool")
            .build();
        
        pool_size_gauge.record(pool_size as u64, &[]);
        idle_gauge.record(idle_connections as u64, &[]);
    }
}

/// Authentication Metrics
pub struct AuthMetrics;

impl AuthMetrics {
    /// Record an authentication attempt
    pub fn record_attempt(success: bool) {
        let labels = [KeyValue::new(
            "result",
            if success { "success" } else { "failure" }.to_string(),
        )];
        AUTH_ATTEMPTS_TOTAL.add(1, &labels);
    }
    
    /// Record a JWT validation
    pub fn record_jwt_validation(valid: bool) {
        let labels = [KeyValue::new(
            "result",
            if valid { "valid" } else { "invalid" }.to_string(),
        )];
        JWT_VALIDATIONS_TOTAL.add(1, &labels);
    }
}

/// Helper struct to measure duration automatically
pub struct DurationTimer {
    start: Instant,
}

impl DurationTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
    
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}
