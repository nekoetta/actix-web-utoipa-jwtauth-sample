use std::borrow::Cow;
use actix_web::{post, web, HttpResponse, Responder, error, http::header};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::Deserialize;
use tracing::Instrument;
use utoipa::{IntoParams, ToSchema};
use crate::{DbPool, models::users::User, middleware::UserClaims, config};

const _API_TAG: &str = "auth"; // TODO

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login);
}

#[derive(Clone, Deserialize, IntoParams, ToSchema, Debug)]
pub struct LoginInfo {
    username: String,
    password: String
}

#[utoipa::path(
    post,
    tag = "auth", // TODO
    responses(
        (status = 200, description = "Login User", headers(
            ("authorization" = String, description = "Authorization Header")
        )),
        (status = UNAUTHORIZED),
        (status = INTERNAL_SERVER_ERROR, description = "Login User Failed")
    ),
    request_body = LoginInfo
)]
#[post("/login")]
#[tracing::instrument(skip(pool, info), fields(auth.username = %info.username, auth.ldap_bind = tracing::field::Empty, auth.user_search = tracing::field::Empty))]
pub async fn login(
    pool: web::Data<DbPool>,
    info: web::Json<LoginInfo>
) -> actix_web::Result<impl Responder> {
    use ldap3::LdapConnAsync;
    println!("{:?}", &info);

    let config = config::get_config().map_err(|e| {
        tracing::error!(error = ?e, "Failed to get configuration");
        error::ErrorInternalServerError(e)
    })?;

    let (conn, mut ldap) = LdapConnAsync::new(&config.ldap_uri)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, ldap_uri = %config.ldap_uri, "Failed to connect to LDAP server");
        error::ErrorInternalServerError(e)
    })?;

    ldap3::drive!(conn);

    let dn = format!("{}={}, {}", &config.ldap_uid_column, &info.username, &config.ldap_user_dn);
    
    // LDAP bind operation with tracing
    let bind_span = tracing::info_span!("ldap_bind", auth.ldap_bind = tracing::field::Empty);
    let result = async {
        let result = ldap.simple_bind(&dn, &info.password)
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, dn = %dn, "LDAP bind failed");
                error::ErrorInternalServerError(e)
            })?;
        Ok::<_, actix_web::Error>(result)
    }
    .instrument(bind_span.clone())
    .await?;

    if let Ok(_status) = result.success() {
        use crate::metrics::AuthMetrics;
        
        bind_span.record("auth.ldap_bind", "success");
        tracing::Span::current().record("auth.ldap_bind", "success");
        
        // Requirements: 12.5 - Authentication metrics collection
        AuthMetrics::record_attempt(true);
        // partner should not be able to login
        let guard_filter = "(&(cn=Partner)(objectCategory=CN=Group*))";
        let guard_result = ldap.search(&config.ldap_user_dn, ldap3::Scope::OneLevel, &guard_filter, vec!["member"])
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, filter = %guard_filter, "LDAP group search failed");
                error::ErrorInternalServerError(e)
            })?
            .0.first().unwrap().to_owned();

        let guard_entry = ldap3::SearchEntry::construct(guard_result);

        if let Some(members) = guard_entry.attrs.get("members") {
            if members.iter().any(|member| member.contains(&info.username)) {
                tracing::warn!(username = %info.username, "Login denied: user is in Partner group");
                return Ok(HttpResponse::Forbidden().finish())
            }
        };

        use crate::models::users::usecases::*;

        // LDAP user search operation with tracing
        let search_span = tracing::info_span!("ldap_user_search", auth.user_search = tracing::field::Empty);
        let search_filter = format!("(&({}={}){})",&config.ldap_uid_column, &info.username, &config.ldap_filter);
        let search_entry = async {
            let result = ldap.search(&config.ldap_user_dn, ldap3::Scope::OneLevel, &search_filter, vec!["employeeNumber", "sn", "givenName", "mail", "gecos"])
                .await
                .map_err(|e| {
                    tracing::error!(error = ?e, filter = %search_filter, "LDAP user search failed");
                    error::ErrorInternalServerError(e)
                })?
                .0.first().unwrap().to_owned();
            Ok::<_, actix_web::Error>(result)
        }
        .instrument(search_span.clone())
        .await?;
        
        search_span.record("auth.user_search", "success");
        tracing::Span::current().record("auth.user_search", "success");

        let user_info = ldap3::SearchEntry::construct(search_entry);

        let cloned_pool = pool.clone();
        let cloned_info = info.clone();
        let users = web::block(move || {
            let mut conn = cloned_pool.get().expect("couldn't get db connection from pool");

            search_user(&mut conn, &cloned_info.username)
        })
        .await?
        .map_err(|e| {
            tracing::error!(error = ?e, username = %info.username, "Failed to search user in database");
            error::ErrorInternalServerError(e)
        })?;

        let employee_number = if let Some(v) = user_info.attrs.get("employeeNumber") {
            if let Some(first) = v.first() {
                let parsed = first.parse::<i32>();
                if let Ok(num) = parsed {
                    Some(num)
                } else {
                    None
                }
            }  else {
                None
            }
        } else {
            None
        };

        let first_name = if let Some(v) = user_info.attrs.get("givenName") {
            v.first().cloned()
        } else {
            None
        };

        let last_name = if let Some(v) = user_info.attrs.get("sn") {
            v.first().cloned()
        } else {
            None
        };

        let email = if let Some(v) = user_info.attrs.get("mail") {
            v.first().cloned()
        } else {
            None
        };

        let gecos = if let Some(v) = user_info.attrs.get("gecos") {
            v.first().cloned()
        } else {
            None
        };

        let user: Cow<'_, User> = match users.first() {
            Some(user) => {
                tracing::debug!(user_id = %user.id, "Existing user found");
                Cow::Borrowed(user)
            }
            None => {
                let username_for_log = info.username.clone();
                tracing::info!(username = %username_for_log, "Creating new user");
                let user = web::block(move || {
                    let mut conn = pool.get().expect("couldn't get db connection from pool");
                    insert_new_user(
                        &mut conn,
                        info.username.to_string(),
                        employee_number,
                        first_name,
                        last_name,
                        email,
                        gecos
                    )
                })
                .await?
                .map_err(|e| {
                    tracing::error!(error = ?e, username = %username_for_log, "Failed to insert new user");
                    error::ErrorInternalServerError(e)
                })?;
                tracing::info!(user_id = %user.id, "New user created successfully");
                Cow::Owned(user)
            }
        };

        let user_id = user.id;
        let username = user.login_id.clone();
        
        let claims = UserClaims{
                id: user.id,
                username: user.into_owned().login_id,
                exp: (chrono::Utc::now() + chrono::Duration::days(7)).timestamp()
            };

        let secret = crate::config::get_config().unwrap().jwt_secret;
        let secret = secret.split(" ").map(|hex_str| u8::from_str_radix(hex_str, 16).unwrap()).collect::<Vec<u8>>();
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(&secret)).expect("Error creating JWT token");

        tracing::info!(user_id = %user_id, username = %username, "Login successful");
        let mut response = HttpResponse::Ok();
        response.insert_header((header::AUTHORIZATION, format!("Bearer {}",token.to_string())));

        Ok(response.finish())
    } else {
        use crate::metrics::AuthMetrics;
        
        tracing::Span::current().record("auth.ldap_bind", "failed");
        tracing::warn!(username = %info.username, "Login failed: invalid credentials");
        
        // Requirements: 12.5 - Authentication metrics collection
        AuthMetrics::record_attempt(false);
        
        Ok(HttpResponse::Unauthorized().finish())
    }
}
