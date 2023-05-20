use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

use utoipa::Modify;
use utoipa::OpenApi;
use utoipa::openapi::{security, OpenApi as OA};
use utoipa_swagger_ui::SwaggerUi;
use crate::services::*;
use crate::models::*;

#[derive(OpenApi)]
#[openapi(modifiers(&SecurityAddonn))]
#[openapi(
    paths(
        api::users::index,
        api::customers::categories,
        api::customers::insert_category,
        api::customers::update_category,
        api::customers::get_category,
        api::customers::delete_category,
        auth::login
    ),
    components(schemas(
        users::usecases::NewUser,
        users::User,
        customers::usecases::NewCategoryBody,
        customers::CustomerCategory,
        auth::LoginInfo,
    ))
)]
struct ApiDoc;

struct SecurityAddonn;

impl Modify for SecurityAddonn {
    fn modify(&self, openapi: &mut OA) {
        let security_scheme = security::SecurityScheme::Http(
            security::HttpBuilder::new()
                .scheme(security::HttpAuthScheme::Bearer)
                .bearer_format("JWT")
                .build()
        );

        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme("BearerAuth", security_scheme);
    }
}

pub fn ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui/{_:.*}")
        .url("/api-doc/openapi.json", ApiDoc::openapi())
}

pub fn generate_openapi_schema() -> Result<(), Box<dyn Error>> {
    let schema = ApiDoc::openapi().to_pretty_json()?;
    let mut file = File::create("openapi_schema.json")?;
    file.write_all(&schema.into_bytes())?;
    Ok(())
}
