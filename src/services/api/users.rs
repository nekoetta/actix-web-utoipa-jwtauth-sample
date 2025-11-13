use actix_web::{get, web, HttpResponse, Responder, error};
use serde::Deserialize;
use crate::{DbPool, middleware::ApiReqeustData, models::users::User, constants};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(constants::paths::USERS)
        .service(index)
    );
}

#[derive(Deserialize)]
pub struct PaginationParams {
    page: Option<i64>,
    per_page: Option<i64>,
}

#[utoipa::path(
    get,
    tag = constants::tags::USERS,
    context_path = "/api/users",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (default: 20)")
    ),
    responses(
        (status = 200, description = "Register User", body = Vec<User>),
        (status = INTERNAL_SERVER_ERROR, description = "Register User Failed")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
#[get("/")]
#[tracing::instrument(skip(pool, req_data, pagination))]
pub async fn index(
    pool: web::Data<DbPool>,
    req_data: web::ReqData<ApiReqeustData>,
    pagination: web::Query<PaginationParams>
) -> actix_web::Result<impl Responder> {
    use crate::models::users::usecases::*;

    dbg!(req_data); // current_user 使用例

    // Requirements: 11.1 - Pagination with query parameters
    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = pagination.per_page.unwrap_or(20).clamp(1, 100);

    let users = web::block(move || -> Result<Vec<User>, diesel::result::Error> {
        let mut conn = pool.get()
            .map_err(|_| diesel::result::Error::BrokenTransactionManager)?;

        if pagination.page.is_some() || pagination.per_page.is_some() {
            all_user_paginated(&mut conn, page, per_page)
        } else {
            all_user(&mut conn)
        }
    })
    .await?
    .map_err(|e| {
        tracing::error!(error = ?e, "Failed to fetch users");
        error::ErrorInternalServerError(e)
    })?;

    tracing::debug!(count = users.len(), page = page, per_page = per_page, "Users fetched successfully");
    Ok(HttpResponse::Ok().json(users))
}
