mod tests {
    use rust_api::middleware::{validator, UserClaims, ReqDataCreator, TracingMiddleware};
    use actix_web::{test, web, App, http::header::ContentType, Responder, HttpResponse, http::header};
    use actix_web_httpauth::middleware::HttpAuthentication;
    use jsonwebtoken::{encode, Header, EncodingKey};

    async fn dummy() -> impl Responder {
        HttpResponse::Ok().body("Hey there!")
    }

    fn create_valid_token() -> String {
        let claims = UserClaims{
            id: 1,
            username: "testuser".into(),
            exp: (chrono::Utc::now() + chrono::Duration::days(7)).timestamp()
        };

        let secret = rust_api::config::get_config().unwrap().jwt_secret;
        let secret = secret.split(" ").map(|hex_str| u8::from_str_radix(hex_str, 16).unwrap()).collect::<Vec<u8>>();
        encode(&Header::default(), &claims, &EncodingKey::from_secret(&secret)).expect("Error creating JWT token")
    }

    fn create_expired_token() -> String {
        let claims = UserClaims{
            id: 1,
            username: "testuser".into(),
            exp: (chrono::Utc::now() - chrono::Duration::days(1)).timestamp()
        };

        let secret = rust_api::config::get_config().unwrap().jwt_secret;
        let secret = secret.split(" ").map(|hex_str| u8::from_str_radix(hex_str, 16).unwrap()).collect::<Vec<u8>>();
        encode(&Header::default(), &claims, &EncodingKey::from_secret(&secret)).expect("Error creating JWT token")
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
        let token = create_valid_token();

        let req = test::TestRequest::default()
        .insert_header(ContentType::plaintext())
        .insert_header((header::AUTHORIZATION, format!("Bearer {}",token.to_string())))
        .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // Requirements: 10.1, 10.2 - JWT validation middleware tests
    #[actix_web::test]
    async fn test_jwt_auth_missing_header() {
        let auth = HttpAuthentication::bearer(validator);

        let app = test::init_service(
            App::new()
                .wrap(auth)
                .route("/protected", web::get().to(dummy)))
                .await;

        let req = test::TestRequest::get()
            .uri("/protected")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401);
    }

    #[actix_web::test]
    async fn test_jwt_auth_invalid_token() {
        let auth = HttpAuthentication::bearer(validator);

        let app = test::init_service(
            App::new()
                .wrap(auth)
                .route("/protected", web::get().to(dummy)))
                .await;

        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header((header::AUTHORIZATION, "Bearer invalid_token_here"))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401);
    }

    #[actix_web::test]
    async fn test_jwt_auth_expired_token() {
        let auth = HttpAuthentication::bearer(validator);

        let app = test::init_service(
            App::new()
                .wrap(auth)
                .route("/protected", web::get().to(dummy)))
                .await;

        let expired_token = create_expired_token();

        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", expired_token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401);
    }

    #[actix_web::test]
    async fn test_jwt_auth_malformed_header() {
        let auth = HttpAuthentication::bearer(validator);

        let app = test::init_service(
            App::new()
                .wrap(auth)
                .route("/protected", web::get().to(dummy)))
                .await;

        // Missing "Bearer " prefix
        let token = create_valid_token();
        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header((header::AUTHORIZATION, token))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401);
    }

    // Requirements: 10.1, 10.2 - TracingMiddleware tests
    #[actix_web::test]
    async fn test_tracing_middleware_success() {
        let app = test::init_service(
            App::new()
                .wrap(TracingMiddleware)
                .route("/test", web::get().to(dummy)))
                .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("user-agent", "test-agent"))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_tracing_middleware_with_different_methods() {
        let app = test::init_service(
            App::new()
                .wrap(TracingMiddleware)
                .route("/test", web::post().to(dummy))
                .route("/test", web::put().to(dummy))
                .route("/test", web::delete().to(dummy)))
                .await;

        // Test POST
        let req = test::TestRequest::post()
            .uri("/test")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Test PUT
        let req = test::TestRequest::put()
            .uri("/test")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Test DELETE
        let req = test::TestRequest::delete()
            .uri("/test")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_tracing_middleware_without_user_agent() {
        let app = test::init_service(
            App::new()
                .wrap(TracingMiddleware)
                .route("/test", web::get().to(dummy)))
                .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // Requirements: 10.1, 10.2 - ReqDataCreator middleware tests
    #[actix_web::test]
    async fn test_req_data_creator_without_token() {
        let pool = rust_api::create_test_connection_pool();
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .wrap(ReqDataCreator)
                .route("/test", web::get().to(dummy)))
                .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_req_data_creator_with_valid_token() {
        let pool = rust_api::create_test_connection_pool();
        
        // Insert a test user first
        {
            use rust_api::models::users::usecases::insert_new_user;
            let mut conn = pool.get().unwrap();
            let _ = insert_new_user(
                &mut conn,
                "testuser".to_string(),
                Some(12345),
                Some("Test".to_string()),
                Some("User".to_string()),
                Some("test@example.com".to_string()),
                None
            );
        }
        
        let token = create_valid_token();
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .wrap(ReqDataCreator)
                .route("/test", web::get().to(dummy)))
                .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_middleware_chain() {
        let pool = rust_api::create_test_connection_pool();
        let auth = HttpAuthentication::bearer(validator);
        
        // Insert a test user
        {
            use rust_api::models::users::usecases::insert_new_user;
            let mut conn = pool.get().unwrap();
            let _ = insert_new_user(
                &mut conn,
                "testuser".to_string(),
                Some(12345),
                Some("Test".to_string()),
                Some("User".to_string()),
                Some("test@example.com".to_string()),
                None
            );
        }
        
        let token = create_valid_token();
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .wrap(TracingMiddleware)
                .wrap(ReqDataCreator)
                .wrap(auth)
                .route("/protected", web::get().to(dummy)))
                .await;

        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
