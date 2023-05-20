use actix_web::{error, HttpMessage, web};
use actix_web::{dev::{ServiceRequest, forward_ready, Service, ServiceResponse, Transform}, Error};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use jsonwebtoken::{DecodingKey, Validation, Algorithm, decode};
use serde::{Serialize, Deserialize};
use std::future::{ready, Ready};
use futures_util::future::LocalBoxFuture;

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
                Err((AuthenticationError::from(config).into(), req))
            }
        }
        Err(_) => Err((AuthenticationError::from(config).into(), req)),
    }
}

fn validate_token(token: &str) -> Result<bool, Error> {
    let secret = config::get_config().unwrap().jwt_secret;
    let secret = secret.split(" ").map(|hex_str| u8::from_str_radix(hex_str, 16).unwrap()).collect::<Vec<u8>>();

    match decode::<UserClaims>(&token, &DecodingKey::from_secret(&secret), &Validation::new(Algorithm::HS256)) {
        Ok(_claims) => {
            Ok(true)
        }
        Err(err) => Err(err).map_err(error::ErrorInternalServerError)
    }
}

#[derive(Clone, Debug)]
pub struct ApiReqeustData {
    user: Option<User>
}

impl ApiReqeustData {
     fn set_current_user(&mut self, conn: &mut DbConnection, uid: String) -> Result<(), diesel::result::Error>{
        let users = search_user(conn, &uid)?;
        let user = users.first();
        self.user = match user {
            Some(user) => Some(user.to_owned()),
            None => None
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
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
