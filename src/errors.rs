use actix_web::{error, HttpResponse};
use derive_more::{Display, Error};
use validator::ValidationErrors;


#[derive(Debug, Display, Error, PartialEq)]
pub enum ServiceError {
    #[display("Internal Server Error")]
    InternalServerError,
    #[display("Validation Error")]
    ValidationError { value: ValidationErrors }
}

impl error::ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::ValidationError {value} => {
                // Log validation error with field details
                tracing::warn!(
                    error.type = "validation",
                    error.fields = ?value.field_errors(),
                    "Validation error occurred"
                );
                HttpResponse::BadRequest().json(value.field_errors())
            }
            ServiceError::InternalServerError => {
                // Log internal server error
                tracing::error!(
                    error.type = "internal_server_error",
                    "Internal server error occurred"
                );
                HttpResponse::InternalServerError().json("Internal Server Error, Please try later")
            }
        }
    }
}
