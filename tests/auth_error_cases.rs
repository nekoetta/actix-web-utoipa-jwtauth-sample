// Requirements: 10.1, 10.5 - Error case tests for authentication endpoints
mod tests {
    use actix_web::{test, web, App};
    use rust_api::services::auth::LoginInfo;
    use actix_limitation::Limiter;
    use std::time::Duration;

    // Test login with empty username
    #[actix_web::test]
    async fn test_login_empty_username() {
        let pool = rust_api::create_test_connection_pool();
        
        // Create rate limiter - try Redis first, fallback to memory
        let limiter = web::Data::new(
            Limiter::builder("redis://127.0.0.1:6379")
                .limit(100)
                .period(Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| {
                    Limiter::builder("memory://")
                        .limit(100)
                        .period(Duration::from_secs(60))
                        .build()
                        .expect("Failed to create limiter")
                })
        );

        let login_data = LoginInfo {
            username: "".to_string(),
            password: "password123".to_string(),
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(limiter)
                .configure(rust_api::services::auth::config)
        ).await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(login_data)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400); // Bad Request - validation error
    }

    // Test login with empty password
    #[actix_web::test]
    async fn test_login_empty_password() {
        let pool = rust_api::create_test_connection_pool();
        
        let limiter = web::Data::new(
            Limiter::builder("redis://127.0.0.1:6379")
                .limit(100)
                .period(Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| {
                    Limiter::builder("memory://")
                        .limit(100)
                        .period(Duration::from_secs(60))
                        .build()
                        .expect("Failed to create limiter")
                })
        );

        let login_data = LoginInfo {
            username: "testuser".to_string(),
            password: "".to_string(),
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(limiter)
                .configure(rust_api::services::auth::config)
        ).await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(login_data)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400); // Bad Request - validation error
    }

    // Test login with username too long
    #[actix_web::test]
    async fn test_login_username_too_long() {
        let pool = rust_api::create_test_connection_pool();
        
        let limiter = web::Data::new(
            Limiter::builder("redis://127.0.0.1:6379")
                .limit(100)
                .period(Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| {
                    Limiter::builder("memory://")
                        .limit(100)
                        .period(Duration::from_secs(60))
                        .build()
                        .expect("Failed to create limiter")
                })
        );

        let long_username = "a".repeat(256);
        let login_data = LoginInfo {
            username: long_username,
            password: "password123".to_string(),
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(limiter)
                .configure(rust_api::services::auth::config)
        ).await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(login_data)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400); // Bad Request - validation error
    }

    // Test login with invalid characters in username
    #[actix_web::test]
    async fn test_login_invalid_username_characters() {
        let pool = rust_api::create_test_connection_pool();
        
        let limiter = web::Data::new(
            Limiter::builder("redis://127.0.0.1:6379")
                .limit(100)
                .period(Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| {
                    Limiter::builder("memory://")
                        .limit(100)
                        .period(Duration::from_secs(60))
                        .build()
                        .expect("Failed to create limiter")
                })
        );

        let login_data = LoginInfo {
            username: "test@user!".to_string(), // Invalid characters
            password: "password123".to_string(),
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(limiter)
                .configure(rust_api::services::auth::config)
        ).await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(login_data)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400); // Bad Request - validation error
    }

    // Test login with valid username format (should pass validation but fail LDAP)
    #[actix_web::test]
    async fn test_login_valid_format_but_invalid_credentials() {
        let pool = rust_api::create_test_connection_pool();
        
        let limiter = web::Data::new(
            Limiter::builder("redis://127.0.0.1:6379")
                .limit(100)
                .period(Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| {
                    Limiter::builder("memory://")
                        .limit(100)
                        .period(Duration::from_secs(60))
                        .build()
                        .expect("Failed to create limiter")
                })
        );

        let login_data = LoginInfo {
            username: "validuser".to_string(),
            password: "wrongpassword".to_string(),
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(limiter)
                .configure(rust_api::services::auth::config)
        ).await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(login_data)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        // This will fail at LDAP connection or authentication stage
        // Expected: 500 (LDAP connection error) or 401 (authentication failed)
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    // Test login with malformed JSON
    #[actix_web::test]
    async fn test_login_malformed_json() {
        let pool = rust_api::create_test_connection_pool();
        
        let limiter = web::Data::new(
            Limiter::builder("redis://127.0.0.1:6379")
                .limit(100)
                .period(Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| {
                    Limiter::builder("memory://")
                        .limit(100)
                        .period(Duration::from_secs(60))
                        .build()
                        .expect("Failed to create limiter")
                })
        );

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(limiter)
                .configure(rust_api::services::auth::config)
        ).await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_payload("{invalid json")
            .insert_header(("content-type", "application/json"))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400); // Bad Request
    }

    // Test login with missing fields
    #[actix_web::test]
    async fn test_login_missing_fields() {
        let pool = rust_api::create_test_connection_pool();
        
        let limiter = web::Data::new(
            Limiter::builder("redis://127.0.0.1:6379")
                .limit(100)
                .period(Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| {
                    Limiter::builder("memory://")
                        .limit(100)
                        .period(Duration::from_secs(60))
                        .build()
                        .expect("Failed to create limiter")
                })
        );

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(limiter)
                .configure(rust_api::services::auth::config)
        ).await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_payload(r#"{"username": "testuser"}"#)
            .insert_header(("content-type", "application/json"))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400); // Bad Request
    }
}
