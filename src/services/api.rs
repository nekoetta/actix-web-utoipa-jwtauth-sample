use actix_web::{web};
use actix_web_httpauth::middleware::HttpAuthentication;
use crate::middleware::{validator, ReqDataCreator};

const API_PREFIX: &str = "/api";

pub mod users;
pub mod customers;

pub fn config(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(validator);

    cfg.service(
        web::scope(API_PREFIX)
        .wrap(ReqDataCreator)
        .wrap(auth)
        .configure(users::config)
        .configure(customers::config)
    );
}
