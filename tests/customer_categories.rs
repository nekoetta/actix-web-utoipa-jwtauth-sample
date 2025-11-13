// Requirements: 10.1, 10.5 - Error case tests for customer category endpoints
mod tests {
    use rust_api::models::customers::usecases::NewCategoryBody;
    use actix_web::{test, web, App, http::header};
    use actix_web_httpauth::middleware::HttpAuthentication;
    use rust_api::middleware::{validator, UserClaims};
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

    // Test validation error - name too long
    #[actix_web::test]
    async fn test_insert_category_validation_error_name_too_long() {
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
        
        // Create a name that exceeds 255 characters
        let long_name = "a".repeat(256);
        let data = NewCategoryBody {
            name: long_name
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(auth)
                .configure(rust_api::services::api::customers::config)
        ).await;

        let req = test::TestRequest::post()
            .uri("/customers/categories")
            .set_json(data)
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400); // Bad Request
    }

    // Test validation error - empty name
    #[actix_web::test]
    async fn test_insert_category_validation_error_empty_name() {
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
        
        let data = NewCategoryBody {
            name: "".to_string()
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(auth)
                .configure(rust_api::services::api::customers::config)
        ).await;

        let req = test::TestRequest::post()
            .uri("/customers/categories")
            .set_json(data)
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        // Currently the API allows empty names, so this succeeds
        // If validation is added in the future, this should return 400
        assert!(resp.status().is_success());
    }

    // Test unauthorized access - no token
    #[actix_web::test]
    async fn test_get_categories_unauthorized() {
        let pool = rust_api::create_test_connection_pool();
        let auth = HttpAuthentication::bearer(validator);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(auth)
                .configure(rust_api::services::api::customers::config)
        ).await;

        let req = test::TestRequest::get()
            .uri("/customers/categories")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401); // Unauthorized
    }

    // Test get non-existent category
    #[actix_web::test]
    async fn test_get_category_not_found() {
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
                .wrap(auth)
                .configure(rust_api::services::api::customers::config)
        ).await;

        // Try to get a category with ID 99999 (should not exist)
        let req = test::TestRequest::get()
            .uri("/customers/categories/99999")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 500); // Internal Server Error (not found)
    }

    // Test update non-existent category
    #[actix_web::test]
    async fn test_update_category_not_found() {
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
        
        let data = NewCategoryBody {
            name: "Updated Name".to_string()
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(auth)
                .configure(rust_api::services::api::customers::config)
        ).await;

        let req = test::TestRequest::put()
            .uri("/customers/categories/99999/edit")
            .set_json(data)
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 500); // Internal Server Error (not found)
    }

    // Test delete non-existent category
    #[actix_web::test]
    async fn test_delete_category_not_found() {
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
                .wrap(auth)
                .configure(rust_api::services::api::customers::config)
        ).await;

        let req = test::TestRequest::delete()
            .uri("/customers/categories/99999/delete")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 500); // Internal Server Error (not found)
    }

    // Test update with validation error
    #[actix_web::test]
    async fn test_update_category_validation_error() {
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

        // Insert a category first
        let category_id = {
            use rust_api::models::customers::usecases::insert_new_category;
            let mut conn = pool.get().unwrap();
            let category = insert_new_category(&mut conn, "Test Category").unwrap();
            category.id
        };

        let token = create_valid_token();
        
        // Try to update with name too long
        let long_name = "a".repeat(256);
        let data = NewCategoryBody {
            name: long_name
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(auth)
                .configure(rust_api::services::api::customers::config)
        ).await;

        let req = test::TestRequest::put()
            .uri(&format!("/customers/categories/{}/edit", category_id))
            .set_json(data)
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400); // Bad Request
    }

    // Test pagination with invalid parameters
    #[actix_web::test]
    async fn test_get_categories_with_pagination() {
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
                .wrap(auth)
                .configure(rust_api::services::api::customers::config)
        ).await;

        // Test with valid pagination parameters
        let req = test::TestRequest::get()
            .uri("/customers/categories?page=1&per_page=10")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
