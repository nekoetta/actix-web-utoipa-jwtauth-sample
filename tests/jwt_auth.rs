mod tests {
    use rust_api::middleware::{validator, UserClaims};
    use actix_web::{test, web, App, http::header::ContentType, Responder, HttpResponse, http::header};
    use actix_web_httpauth::middleware::HttpAuthentication;
    use jsonwebtoken::{encode, Header, EncodingKey};

    async fn dummy() -> impl Responder {
        HttpResponse::Ok().body("Hey there!")
    }

    #[actix_web::test]
    async fn test_jwt_auth_wrapper() {
        let auth = HttpAuthentication::bearer(validator);

        let app = test::init_service(
            App::new()
                .wrap(auth)
                .route("/", web::get().to(dummy)))
                .await;

        // no authorization header
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());

        // with authorization header
        let claims = UserClaims{
                id: 1,
                username: "dummy".into(),
                exp: (chrono::Utc::now() + chrono::Duration::days(7)).timestamp()
            };

        let secret = rust_api::config::get_config().unwrap().jwt_secret;
        let secret = secret.split(" ").map(|hex_str| u8::from_str_radix(hex_str, 16).unwrap()).collect::<Vec<u8>>();
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(&secret)).expect("Error creating JWT token");

        let req = test::TestRequest::default()
        .insert_header(ContentType::plaintext())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}",token.to_string())))
        .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

    }
}
