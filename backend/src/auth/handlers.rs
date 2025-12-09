use actix_web::cookie::{Cookie, SameSite, time::Duration as CookieDuration};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use base64::Engine;
use base64::engine::general_purpose;
use chrono::{Duration, Utc};
use database::models::{
    create_user, create_user_sessions, find_by_email_user, find_by_hash_tokens, find_by_id_user,
    insert_tokens, revoke,
};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use validator::Validate;

use crate::auth::jwt::{Claims, create_access_token};
use crate::auth::password::{hash_password, verify_password};
use crate::config::Setting;
use crate::errors::ServiceError;
use crate::state::AppState;

const REFRESH_COOKIE_NAME: &str = "refresh_token";
#[derive(Debug, Deserialize, Validate)]
pub struct SignUpDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub display_name: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub device_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct AccessTokenResponse {
    access_token: String,
    expires_in: i64,
}

fn generate_refresh_token() -> anyhow::Result<String> {
    let mut buffer = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut buffer)
        .map_err(|e| anyhow::anyhow!("failed to get randomness: {}", e))?;
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(&buffer))
}

fn hash_refresh(token: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(token.as_bytes());
    hex::encode(hash.finalize())
}

fn cookie_max_age(setting: &Setting) -> i64 {
    setting.refresh_token_exp
}

fn build_refresh_cookie(setting: &Setting, token: &str) -> Cookie<'static> {
    let mut cookie = Cookie::build(REFRESH_COOKIE_NAME, token.to_string())
        .path("/")
        .same_site(SameSite::Strict)
        .http_only(true);

    let secs = cookie_max_age(setting);
    if secs > 0 {
        cookie = cookie.max_age(CookieDuration::seconds(secs));
    }
    let secure =
        !(setting.bind_addr.starts_with("127.") || setting.bind_addr.starts_with("localhost"));
    cookie.secure(secure).finish()
}

fn build_clear_cookie(setting: &Setting) -> Cookie<'static> {
    let cookie = Cookie::build(REFRESH_COOKIE_NAME, "")
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(CookieDuration::seconds(0));

    let secure =
        !(setting.bind_addr.starts_with("127.") || setting.bind_addr.starts_with("localhost"));
    cookie.secure(secure).finish()
}

pub async fn signup(
    app: web::Data<AppState>,
    payload: web::Json<SignUpDto>,
) -> Result<HttpResponse, ServiceError> {
    payload
        .validate()
        .map_err(|e| ServiceError::ValidationError(format!("{:?}", e)))?;
    let email = payload.email.trim().to_lowercase();
    if find_by_email_user(&app.pool, &email).await?.is_some() {
        return Err(ServiceError::Conflict("User Already Exists".into()));
    }
    let hashed = hash_password(&payload.password)
        .await
        .map_err(|e| ServiceError::ExternalError(e))?;
    let user_id = create_user(&app.pool, &email, &hashed, payload.display_name.as_deref())
        .await //as_ref -> Option<&String> as_deref -> Option<&str>
        .map_err(|e| ServiceError::DataBaseError(e.to_string()))?;

    let session_id = create_user_sessions(&app.pool, user_id, payload.device_name.as_deref())
        .await
        .map_err(|e| ServiceError::DataBaseError(e.to_string()))?;

    let access_token = create_access_token(
        &user_id.to_string(),
        Some(&session_id.to_string()),
        app.setting.access_token_exp,
        &app.setting.jwt_secret,
    )
    .map_err(|e| ServiceError::ExternalError(e))?;

    let refresh_plain = generate_refresh_token()?;
    let refresh_hash = hash_refresh(&refresh_plain);
    let expiry_at = (Utc::now() + Duration::seconds(app.setting.refresh_token_exp)).into();
    insert_tokens(&app.pool, user_id, &refresh_hash, expiry_at)
        .await
        .map_err(|e| ServiceError::DataBaseError(e.to_string()))?;

    let cookie = build_refresh_cookie(&app.setting, &refresh_plain);
    let body = AccessTokenResponse {
        access_token,
        expires_in: app.setting.access_token_exp,
    };

    Ok(HttpResponse::Ok().cookie(cookie).json(body)).map_err(|e| ServiceError::ExternalError(e))
}

pub async fn login(
    app: web::Data<AppState>,
    payload: web::Json<LoginDto>,
) -> Result<HttpResponse, ServiceError> {
    payload
        .validate()
        .map_err(|e| ServiceError::ValidationError(format!("{:?}", e)))?;
    let email = payload.email.trim().to_lowercase();
    let user = find_by_email_user(&app.pool, &email)
        .await
        .map_err(|e| ServiceError::DataBaseError(e.to_string()))?
        .ok_or("user name not found")
        .map_err(|e| ServiceError::DataBaseError(e.to_string()))?;

    if !verify_password(&user.hashed_password, &payload.password).await {
        return Err(ServiceError::Unauthorized("Invalid Credentials".into()));
    }

    let session_id = create_user_sessions(&app.pool, user.id, payload.device_name.as_deref())
        .await
        .map_err(|e| ServiceError::ExternalError(e))?;

    let access_token = create_access_token(
        &user.id.to_string(),
        Some(&session_id.to_string()),
        app.setting.access_token_exp,
        &app.setting.jwt_secret,
    )
    .map_err(|e| ServiceError::ExternalError(e))?;

    let refresh_plain = generate_refresh_token()?;
    let refresh_hash = hash_refresh(&refresh_plain);
    let expires_at = (Utc::now() + Duration::seconds(app.setting.refresh_token_exp)).into();
    insert_tokens(&app.pool, user.id, &refresh_hash, expires_at)
        .await
        .map_err(|e| ServiceError::DataBaseError(e.to_string()))?;

    let cookie = build_refresh_cookie(&app.setting, &refresh_plain);
    let body = AccessTokenResponse {
        access_token,
        expires_in: app.setting.access_token_exp,
    };

    Ok(HttpResponse::Ok().cookie(cookie).json(body))
}

#[derive(Debug, Deserialize)]
pub struct RefreshDto {
    pub refresh_token: Option<String>,
}

pub async fn refresh_token(
    app: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<RefreshDto>,
) -> Result<HttpResponse, ServiceError> {
    let plain_token = req
        .cookie(REFRESH_COOKIE_NAME)
        .map(|v| v.value().to_string())
        .or_else(|| payload.refresh_token.clone());
    let hashed = plain_token.ok_or(ServiceError::BadRequest("Missing refresh token".into()))?;
    let rec = find_by_hash_tokens(&app.pool, &hashed)
        .await
        .map_err(|e| ServiceError::Unauthorized(e.to_string()))?
        .ok_or(ServiceError::Unauthorized("Invalid Refresh Token".into()))?;

    if rec.revoked || rec.expires_at < Utc::now() {
        return Err(ServiceError::Unauthorized(
            "refresh token invalid/expired".into(),
        ));
    }
    revoke(&app.pool, rec.id)
        .await
        .map_err(|e| ServiceError::ExternalError(e))?;

    let user_id = rec.user_id;

    let new_plain_token = generate_refresh_token()?;
    let new_hashed = hash_refresh(&new_plain_token);
    let expires_at_new = (Utc::now() + Duration::seconds(app.setting.refresh_token_exp)).into();
    insert_tokens(&app.pool, user_id, &new_hashed, expires_at_new)
        .await
        .map_err(|e| ServiceError::ExternalError(e))?;

    let access_token = create_access_token(
        &user_id.to_string(),
        None,
        app.setting.access_token_exp,
        &app.setting.jwt_secret,
    )
    .map_err(|e| ServiceError::ExternalError(e))?;

    let cookie = build_refresh_cookie(&app.setting, &new_plain_token);
    let body = AccessTokenResponse {
        access_token,
        expires_in: app.setting.access_token_exp,
    };

    Ok(HttpResponse::Ok().cookie(cookie).json(body))
}

#[derive(Debug, Deserialize)]
pub struct LogoutDto {
    refresh_tokens: Option<String>,
}

pub async fn logout(
    app: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<LogoutDto>,
) -> Result<HttpResponse, ServiceError> {
    let plain_token = req
        .cookie(REFRESH_COOKIE_NAME)
        .map(|v| v.value().to_string())
        .or_else(|| payload.refresh_tokens.clone());
    if let Some(plain) = plain_token {
        let hashed = hash_refresh(&plain);
        if let Some(r) = find_by_hash_tokens(&app.pool, &hashed)
            .await
            .map_err(|e| ServiceError::ExternalError(e))?
        {
            revoke(&app.pool, r.id)
                .await
                .map_err(|e| ServiceError::ExternalError(e))?;
        }
    }
    let clear = build_clear_cookie(&app.setting);
    Ok(HttpResponse::Ok()
        .cookie(clear)
        .json(serde_json::json!({"ok" : true})))
}

pub async fn me(app: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse, ServiceError> {
    let claims_opt = req.extensions().get::<Claims>().cloned();
    let claims = claims_opt.ok_or(ServiceError::Unauthorized("no claims".into()))?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ServiceError::BadRequest("Invalid Sub Claims".into()))?;

    let user = find_by_id_user(&app.pool, user_id)
        .await
        .map_err(|e| ServiceError::DataBaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!(
        {
            "id" : user.id,
            "email" : user.email,
            "display_name" : user.display_name,
            "created_at" : user.created_at
        }
    )))
}
