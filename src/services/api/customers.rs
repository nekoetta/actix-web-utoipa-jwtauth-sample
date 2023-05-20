use actix_web::{get, put, delete, web, HttpResponse, Responder, post};
use crate::{DbPool, models::customers::usecases::NewCategoryBody};

const API_PREFIX: &str = "/customers";
const _API_TAG: &str = "customers"; // TODO

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(API_PREFIX)
        .service(categories)
        .service(insert_category)
        .service(update_category)
        .service(get_category)
        .service(delete_category)
    );
}

#[utoipa::path(
    post,
    tag = "customers", // TODO
    context_path = "/api/customers", // TODO
    request_body = NewCategoryBody,
    responses(
        (status = 200, description = "customer category insert successfully"),
        (status = INTERNAL_SERVER_ERROR, description = "failed to insert customer category"),
        (status = BAD_REQUEST, description = "validation error"),
        (status = UNAUTHORIZED, description = "invalid authorization token")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
#[post("/categories")]
pub async fn insert_category(
    pool: web::Data<DbPool>,
    form: web::Json<NewCategoryBody>
) -> actix_web::Result<impl Responder> {
    use crate::models::customers::usecases::*;
    let category = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        insert_new_category(&mut conn, &form.name)
    })
    .await??;

    Ok(HttpResponse::Ok().json(category))
}

#[utoipa::path(
    put,
    tag = "customers", // TODO
    context_path = "/api/customers", // TODO
    request_body = NewCategoryBody,
    responses(
        (status = 200, description = "customer category update successfully"),
        (status = INTERNAL_SERVER_ERROR, description = "failed to update customer category"),
        (status = BAD_REQUEST, description = "validation error"),
        (status = UNAUTHORIZED, description = "invalid authorization token")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
#[put("/categories/{id}/edit")]
pub async fn update_category(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    form: web::Json<NewCategoryBody>
) -> actix_web::Result<impl Responder> {
    use crate::models::customers::usecases::*;

    let category_id = path.into_inner();

    let category = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        update_category(&mut conn, category_id, &form.name)
    })
    .await??;

    Ok(HttpResponse::Ok().json(category))
}

#[utoipa::path(
    get,
    tag = "customers", // TODO
    context_path = "/api/customers", // TODO
    responses(
        (status = 200, description = "customer category list", body = Vec<CustomerCategory>),
        (status = INTERNAL_SERVER_ERROR, description = "failed to get customer categories"),
        (status = UNAUTHORIZED, description = "invalid authorization token")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
#[get("/categories")]
pub async fn categories(
    pool: web::Data<DbPool>,
) -> actix_web::Result<impl Responder> {
    use crate::models::customers::usecases::*;

    let categories = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        all_categories(&mut conn)
    })
    .await??;

    Ok(HttpResponse::Ok().json(categories))
}

#[utoipa::path(
    get,
    tag = "customers", // TODO
    context_path = "/api/customers", // TODO
    responses(
        (status = 200, description = "customer category detail", body = CustomerCategory),
        (status = INTERNAL_SERVER_ERROR, description = "failed to get category detail"),
        (status = UNAUTHORIZED, description = "invalid authorization token")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
#[get("/categories/{id}")]
pub async fn get_category(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    use crate::models::customers::usecases::*;
    let category_id = path.into_inner();

    let category = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        get_category(&mut conn, category_id)
    })
    .await??;

    Ok(HttpResponse::Ok().json(category))
}

#[utoipa::path(
    delete,
    tag = "customers", // TODO
    context_path = "/api/customers", // TODO
    responses(
        (status = 200, description = "delete customer category", body = CustomerCategory),
        (status = INTERNAL_SERVER_ERROR, description = "failed to delete customer category"),
        (status = UNAUTHORIZED, description = "invalid authorization token")
    ),
    security(
        ("BearerAuth" = [])
    )
)]
#[delete("/categories/{id}/delete")]
pub async fn delete_category(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    use crate::models::customers::usecases::*;
    let category_id = path.into_inner();

    let category = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        destroy_category(&mut conn, category_id)
    })
    .await??;

    Ok(HttpResponse::Ok().json(category))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_test_connection_pool;
    use actix_web::{
        http::{header::ContentType},
        test, App, web
    };

    #[actix_web::test]
    async fn test_insert_category() {
        let pool = create_test_connection_pool();

        let data = NewCategoryBody {
            name: "test".into()
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(insert_category)
            )
            .await;

        let req = test::TestRequest::post()
            .uri("/categories")
            .set_json(data)
            .insert_header(ContentType::json())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
