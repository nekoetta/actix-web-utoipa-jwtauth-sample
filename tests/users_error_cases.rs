// Requirements: 10.1, 10.5 - Error case tests for users endpoints
mod tests {
    use actix_web::{test, web, App, http::header};
    use actix_web_httpauth::middleware::HttpAuthentication;
    use rust_api::middleware::{validator, UserClaims, ReqDataCreator};
    use jsonwebtoken::{encode, Header, EncodingKey};

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

    // Test unauthorized access to users endpoint
    #[actix_web::test]
    async fn test_get_users_unauthorized() {
        let pool = rust_api::create_test_connection_pool();
        let auth = HttpAuthentication::bearer(validator);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(ReqDataCreator)
                .wrap(auth)
                .configure(rust_api::services::api::users::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/users/")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401); // Unauthorized
    }

    // Test users endpoint with invalid token
    #[actix_web::test]
    async fn test_get_users_invalid_token() {
        let pool = rust_api::create_test_connection_pool();
        let auth = HttpAuthentication::bearer(validator);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(ReqDataCreator)
                .wrap(auth)
                .configure(rust_api::services::api::users::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/users/")
            .insert_header((header::AUTHORIZATION, "Bearer invalid_token"))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401); // Unauthorized
    }

    // Test users endpoint with valid token
    #[actix_web::test]
    async fn test_get_users_success() {
        let pool = rust_api::create_test_connection_pool();
        let auth = HttpAuthentication::bearer(validator);
        
        // Insert test user
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
                .app_data(web::Data::new(pool.clone()))
                .wrap(ReqDataCreator)
                .wrap(auth)
                .configure(rust_api::services::api::users::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/users/")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // Test users endpoint with pagination
    #[actix_web::test]
    async fn test_get_users_with_pagination() {
        let pool = rust_api::create_test_connection_pool();
        let auth = HttpAuthentication::bearer(validator);
        
        // Insert test user
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
                .app_data(web::Data::new(pool.clone()))
                .wrap(ReqDataCreator)
                .wrap(auth)
                .configure(rust_api::services::api::users::config)
        ).await;

        // Test with valid pagination parameters
        let req = test::TestRequest::get()
            .uri("/users/?page=1&per_page=10")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // Test users endpoint with negative page number (should be clamped to 1)
    #[actix_web::test]
    async fn test_get_users_with_negative_page() {
        let pool = rust_api::create_test_connection_pool();
        let auth = HttpAuthentication::bearer(validator);
        
        // Insert test user
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
                .app_data(web::Data::new(pool.clone()))
                .wrap(ReqDataCreator)
                .wrap(auth)
                .configure(rust_api::services::api::users::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/users/?page=-1&per_page=10")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // Test users endpoint with excessive per_page (should be clamped to 100)
    #[actix_web::test]
    async fn test_get_users_with_excessive_per_page() {
        let pool = rust_api::create_test_connection_pool();
        let auth = HttpAuthentication::bearer(validator);
        
        // Insert test user
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
                .app_data(web::Data::new(pool.clone()))
                .wrap(ReqDataCreator)
                .wrap(auth)
                .configure(rust_api::services::api::users::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/users/?page=1&per_page=1000")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
