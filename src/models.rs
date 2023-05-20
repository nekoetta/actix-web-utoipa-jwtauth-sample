use validator::Validate;

use crate::{traits::IntoValidator, errors::ServiceError};

pub mod users;
pub mod customers;

pub fn validate<T: Validate>(item: &impl IntoValidator<T>) -> Result<(), ServiceError>  {
    item.validator().validate().map_err(|err| ServiceError::ValidationError { value: err })
}
