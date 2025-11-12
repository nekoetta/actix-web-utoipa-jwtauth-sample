use actix_web::{get, web, HttpResponse, Responder, error};
use crate::{DbPool, middleware::ApiReqeustData, models::users::User, constants};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(constants::paths::USERS)
        .service(index)
    );
}

#[utoipa::path(
    get,
    tag = constants::tags::USERS,
    context_path = "/api/users",
    responses(
        (status = 200, description = "Register User", body = Vec<User>),
        (status = INTERNAL_SERVER_ERROR, description = "Register User Failed")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
#[get("/")]
#[tracing::instrument(skip(pool, req_data))]
pub async fn index(
    pool: web::Data<DbPool>,
    req_data: web::ReqData<ApiReqeustData>
) -> actix_web::Result<impl Responder> {
    use crate::models::users::usecases::*;

    dbg!(req_data); // current_user 使用例

    let users = web::block(move || -> Result<Vec<User>, diesel::result::Error> {
        let mut conn = pool.get()
            .map_err(|_| diesel::result::Error::BrokenTransactionManager)?;

        all_user(&mut conn)
    })
    .await?
    .map_err(|e| {
        tracing::error!(error = ?e, "Failed to fetch users");
        error::ErrorInternalServerError(e)
    })?;

    tracing::debug!(count = users.len(), "Users fetched successfully");
    Ok(HttpResponse::Ok().json(users))
}
