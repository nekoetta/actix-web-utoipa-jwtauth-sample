use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
pub mod usecases;

#[derive(Clone, Queryable, Deserialize, Serialize, ToSchema, Debug)]
pub struct User {
    pub id: i32,
    pub login_id: String,
    pub employee_number: Option<i32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub gecos: Option<String>
}
