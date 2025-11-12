use actix_web::{error, HttpMessage, web};
use actix_web::{dev::{ServiceRequest, forward_ready, Service, ServiceResponse, Transform}, Error};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use jsonwebtoken::{DecodingKey, Validation, Algorithm, decode};
use serde::{Serialize, Deserialize};
use std::future::{ready, Ready};
use futures_util::future::LocalBoxFuture;
use tracing::{info_span, Instrument};
use uuid::Uuid;

use crate::models::users::User;
use crate::models::users::usecases::search_user;
use crate::{config, DbPool, DbConnection};


#[derive(Serialize, Deserialize)]
pub struct UserClaims {
    pub id: i32,
    pub username: String,
    pub exp: i64,
}

pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);

    match validate_token(credentials.token().replace("Bearer ", "").as_str()) {
        Ok(res) => {
            if res == true {
                Ok(req)
            } else {
                tracing::warn!("Token validation failed: invalid token");
                Err((AuthenticationError::from(config).into(), req))
            }
        }
        Err(e) => {
            tracing::error!(error = ?e, "Token validation error");
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

#[tracing::instrument(skip(token), fields(auth.token_valid = tracing::field::Empty))]
fn validate_token(token: &str) -> Result<bool, Error> {
    use crate::metrics::AuthMetrics;
    
    let secret = config::get_config().unwrap().jwt_secret;
    let secret = secret.split(" ").map(|hex_str| u8::from_str_radix(hex_str, 16).unwrap()).collect::<Vec<u8>>();

    match decode::<UserClaims>(&token, &DecodingKey::from_secret(&secret), &Validation::new(Algorithm::HS256)) {
        Ok(_claims) => {
            tracing::Span::current().record("auth.token_valid", true);
            tracing::debug!("Token validation successful");
            
            // Requirements: 12.5 - Authentication metrics collection
            AuthMetrics::record_jwt_validation(true);
            
            Ok(true)
        }
        Err(err) => {
            tracing::Span::current().record("auth.token_valid", false);
            tracing::warn!(error = ?err, "Token validation failed");
            
            // Requirements: 12.5 - Authentication metrics collection
            AuthMetrics::record_jwt_validation(false);
            
            Err(err).map_err(error::ErrorInternalServerError)
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApiReqeustData {
    user: Option<User>
}

impl ApiReqeustData {
     fn set_current_user(&mut self, conn: &mut DbConnection, uid: String) -> Result<(), diesel::result::Error>{
        let users = search_user(conn, &uid).map_err(|e| {
            tracing::error!(error = ?e, uid = %uid, "Failed to search user in database");
            e
        })?;
        let user = users.first();
        self.user = match user {
            Some(user) => {
                tracing::debug!(user_id = %user.id, "Current user set successfully");
                Some(user.to_owned())
            }
            None => {
                tracing::debug!(uid = %uid, "User not found in database");
                None
            }
        };
        Ok(())
     }

     fn new() -> Self {
        ApiReqeustData { user: None }
     }
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct  ReqDataCreator;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for  ReqDataCreator
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform =  ReqDataCreatorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok( ReqDataCreatorMiddleware { service }))
    }
}

pub struct  ReqDataCreatorMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for  ReqDataCreatorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let pool = req.app_data::<web::Data<DbPool>>().unwrap();
        let mut req_data = ApiReqeustData::new();
        let mut conn = pool.get().unwrap();

        let bearer_token = if let Some(token) = req.headers().get("authorization") {
            token.to_str().unwrap_or_default().replace("Bearer ", "")
        } else {
            String::from("")
        };

        let secret = config::get_config().unwrap().jwt_secret;
        let secret = secret.split(" ").map(|hex_str| u8::from_str_radix(hex_str, 16).unwrap()).collect::<Vec<u8>>();
        let user_claims = decode::<UserClaims>(&bearer_token, &DecodingKey::from_secret(&secret), &Validation::new(Algorithm::HS256));

        let uid = if let Ok(data) = user_claims {
            data.claims.username
        } else {
            String::from("")
        };

        let result = req_data.set_current_user(&mut conn, uid);
        if let Ok(_) = result {
            req.extensions_mut().insert(req_data);
        } else if let Err(e) = result {
            tracing::warn!(error = ?e, "Failed to set current user in request data");
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await.map_err(|e| {
                tracing::error!(error = ?e, "Request processing error");
                e
            })?;
            Ok(res)
        })
    }
}

// HTTPトレーシングミドルウェア
// Requirements: 12.1, 14.1 - HTTP request tracing with minimal code changes

pub struct TracingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for TracingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TracingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TracingMiddlewareService { service }))
    }
}

pub struct TracingMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TracingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        use crate::metrics::{HttpMetrics, DurationTimer};
        
        // Generate unique request ID for tracing
        let request_id = Uuid::new_v4().to_string();
        
        // Extract request information for tracing and metrics
        let method = req.method().to_string();
        let path = req.path().to_string();
        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        
        // Requirements: 12.5 - HTTP metrics collection
        // Increment in-flight requests counter
        HttpMetrics::increment_in_flight();
        
        // Start duration timer
        let timer = DurationTimer::new();
        
        // Create span with HTTP attributes
        // Requirements: 12.1 - Record http.method, http.target, http.user_agent
        let span = info_span!(
            "http_request",
            http.method = %method,
            http.target = %path,
            http.user_agent = %user_agent,
            http.status_code = tracing::field::Empty,
            request_id = %request_id,
        );
        
        let fut = self.service.call(req);
        
        Box::pin(
            async move {
                let res = fut.await?;
                
                // Record status code after response is ready
                // Requirements: 12.1 - Record http.status_code
                let status_code = res.status().as_u16();
                tracing::Span::current().record("http.status_code", status_code);
                
                // Requirements: 12.5 - Record HTTP metrics
                // Record request count with labels
                HttpMetrics::record_request(&method, &path, status_code);
                
                // Record request duration
                let duration = timer.elapsed_secs();
                HttpMetrics::record_duration(&method, &path, duration);
                
                // Decrement in-flight requests counter
                HttpMetrics::decrement_in_flight();
                
                Ok(res)
            }
            .instrument(span)
        )
    }
}
