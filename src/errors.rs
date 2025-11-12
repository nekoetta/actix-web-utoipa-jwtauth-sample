use actix_web::{error, HttpResponse};
use derive_more::{Display, Error};
use validator::ValidationErrors;
use serde_json::json;


#[derive(Debug, Display, Error, PartialEq)]
pub enum ServiceError {
    #[display("Internal Server Error")]
    InternalServerError,
    #[display("Validation Error")]
    ValidationError { value: ValidationErrors },
    #[display("Database Error: {message}")]
    DatabaseError { message: String },
    #[display("Authentication Error: {message}")]
    AuthenticationError { message: String },
}

impl error::ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        // Requirements: 11.2 - Hide detailed error information in production
        let is_production = crate::config::get_config()
            .map(|c| c.is_production())
            .unwrap_or(false);
        
        match self {
            ServiceError::ValidationError {value} => {
                // Log validation error with field details
                tracing::warn!(
                    error.type = "validation",
                    error.fields = ?value.field_errors(),
                    "Validation error occurred"
                );
                
                // Always return validation errors (they don't expose sensitive info)
                HttpResponse::BadRequest().json(value.field_errors())
            }
            ServiceError::InternalServerError => {
                // Log internal server error with full details
                tracing::error!(
                    error.type = "internal_server_error",
                    "Internal server error occurred"
                );
                
                // Return generic message (no details exposed)
                HttpResponse::InternalServerError().json(json!({
                    "error": "Internal Server Error",
                    "message": "An unexpected error occurred. Please try again later."
                }))
            }
            ServiceError::DatabaseError { message } => {
                // Log database error with full details
                tracing::error!(
                    error.type = "database_error",
                    error.message = %message,
                    "Database error occurred"
                );
                
                if is_production {
                    // Hide detailed database error in production
                    HttpResponse::InternalServerError().json(json!({
                        "error": "Database Error",
                        "message": "A database error occurred. Please try again later."
                    }))
                } else {
                    // Show details in development for debugging
                    HttpResponse::InternalServerError().json(json!({
                        "error": "Database Error",
                        "message": message,
                        "note": "Detailed errors are hidden in production"
                    }))
                }
            }
            ServiceError::AuthenticationError { message } => {
                // Log authentication error
                tracing::warn!(
                    error.type = "authentication_error",
                    error.message = %message,
                    "Authentication error occurred"
                );
                
                if is_production {
                    // Generic message in production to prevent user enumeration
                    HttpResponse::Unauthorized().json(json!({
                        "error": "Authentication Failed",
                        "message": "Invalid credentials"
                    }))
                } else {
                    // More detailed message in development
                    HttpResponse::Unauthorized().json(json!({
                        "error": "Authentication Failed",
                        "message": message,
                        "note": "Detailed errors are hidden in production"
                    }))
                }
            }
        }
    }
}
