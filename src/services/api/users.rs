use actix_web::{get, web, HttpResponse, Responder, error};
use crate::{DbPool, middleware::ApiReqeustData};

const API_PREFIX: &str = "/users";
const _API_TAG: &str = "users"; // TODO

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(API_PREFIX)
        .service(index)
    );
}

#[utoipa::path(
    get,
    tag = "users", // TODO
    context_path = "/api/users", // TODO
    responses(
        (status = 200, description = "Register User", body = Vec<User>),
        (status = INTERNAL_SERVER_ERROR, description = "Register User Failed")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
#[get("/")]
pub async fn index(
    pool: web::Data<DbPool>,
    req_data: web::ReqData<ApiReqeustData>
) -> actix_web::Result<impl Responder> {
    use crate::models::users::usecases::*;

    dbg!(req_data); // current_user 使用例

    let users = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        all_user(&mut conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(users))
}
