use actix_web::{get, put, delete, web, HttpResponse, Responder, post};
use crate::{DbPool, models::customers::usecases::NewCategoryBody, models::customers::CustomerCategory, constants};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(constants::paths::CUSTOMERS)
        .service(categories)
        .service(insert_category)
        .service(update_category)
        .service(get_category)
        .service(delete_category)
    );
}

#[utoipa::path(
    post,
    tag = constants::tags::CUSTOMERS,
    context_path = "/api/customers",
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
#[tracing::instrument(skip(pool, form), fields(category.name = %form.name))]
pub async fn insert_category(
    pool: web::Data<DbPool>,
    form: web::Json<NewCategoryBody>
) -> actix_web::Result<impl Responder> {
    use crate::models::customers::usecases::*;
    let category = web::block(move || -> Result<CustomerCategory, crate::errors::ServiceError> {
        let mut conn = pool.get()
            .map_err(|e| {
                tracing::error!(error = ?e, "Failed to get database connection");
                crate::errors::ServiceError::InternalServerError
            })?;

        insert_new_category(&mut conn, &form.name)
    })
    .await?
    .map_err(|e| {
        tracing::error!(error = ?e, "Failed to insert category");
        e
    })?;

    tracing::info!("Category inserted successfully");
    Ok(HttpResponse::Ok().json(category))
}

#[utoipa::path(
    put,
    tag = constants::tags::CUSTOMERS,
    context_path = "/api/customers",
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
#[tracing::instrument(skip(pool, form), fields(category.id = %path, category.name = %form.name))]
pub async fn update_category(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    form: web::Json<NewCategoryBody>
) -> actix_web::Result<impl Responder> {
    use crate::models::customers::usecases::*;

    let category_id = path.into_inner();

    let category = web::block(move || -> Result<CustomerCategory, crate::errors::ServiceError> {
        let mut conn = pool.get()
            .map_err(|e| {
                tracing::error!(error = ?e, "Failed to get database connection");
                crate::errors::ServiceError::InternalServerError
            })?;

        update_category(&mut conn, category_id, &form.name)
    })
    .await?
    .map_err(|e| {
        tracing::error!(error = ?e, category_id = %category_id, "Failed to update category");
        e
    })?;

    tracing::info!(category_id = %category_id, "Category updated successfully");
    Ok(HttpResponse::Ok().json(category))
}

#[utoipa::path(
    get,
    tag = constants::tags::CUSTOMERS,
    context_path = "/api/customers",
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
#[tracing::instrument(skip(pool))]
pub async fn categories(
    pool: web::Data<DbPool>,
) -> actix_web::Result<impl Responder> {
    use crate::models::customers::usecases::*;

    let categories = web::block(move || -> Result<Vec<CustomerCategory>, crate::errors::ServiceError> {
        let mut conn = pool.get()
            .map_err(|e| {
                tracing::error!(error = ?e, "Failed to get database connection");
                crate::errors::ServiceError::InternalServerError
            })?;

        all_categories(&mut conn)
    })
    .await?
    .map_err(|e| {
        tracing::error!(error = ?e, "Failed to fetch categories");
        e
    })?;

    tracing::debug!(count = categories.len(), "Categories fetched successfully");
    Ok(HttpResponse::Ok().json(categories))
}

#[utoipa::path(
    get,
    tag = constants::tags::CUSTOMERS,
    context_path = "/api/customers",
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
#[tracing::instrument(skip(pool), fields(category.id = %path))]
pub async fn get_category(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    use crate::models::customers::usecases::*;
    let category_id = path.into_inner();

    let category = web::block(move || -> Result<CustomerCategory, crate::errors::ServiceError> {
        let mut conn = pool.get()
            .map_err(|e| {
                tracing::error!(error = ?e, "Failed to get database connection");
                crate::errors::ServiceError::InternalServerError
            })?;

        get_category(&mut conn, category_id)
    })
    .await?
    .map_err(|e| {
        tracing::error!(error = ?e, category_id = %category_id, "Failed to fetch category");
        e
    })?;

    tracing::debug!(category_id = %category_id, "Category fetched successfully");
    Ok(HttpResponse::Ok().json(category))
}

#[utoipa::path(
    delete,
    tag = constants::tags::CUSTOMERS,
    context_path = "/api/customers",
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
#[tracing::instrument(skip(pool), fields(category.id = %path))]
pub async fn delete_category(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    use crate::models::customers::usecases::*;
    let category_id = path.into_inner();

    let category = web::block(move || -> Result<CustomerCategory, crate::errors::ServiceError> {
        let mut conn = pool.get()
            .map_err(|e| {
                tracing::error!(error = ?e, "Failed to get database connection");
                crate::errors::ServiceError::InternalServerError
            })?;

        destroy_category(&mut conn, category_id)
    })
    .await?
    .map_err(|e| {
        tracing::error!(error = ?e, category_id = %category_id, "Failed to delete category");
        e
    })?;

    tracing::info!(category_id = %category_id, "Category deleted successfully");
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
