use anyhow::{Context, Ok};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, encode, jws::decode};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    #[serde(rename = "sub")]
    sub: String,
    #[serde(rename = "session", skip_serializing_if = "Option::is_none")]
    session: Option<String>,
    #[serde(rename = "exp")]
    exp: i64,
}
pub fn create_access_token(
    user_id: &str,
    session_id: Option<&str>,
    expiry: i64,
    secret: &SecretString,
) -> anyhow::Result<String> {
    let utc = Utc::now() + Duration::seconds(expiry);
    let claims = Claims {
        sub: user_id.to_string(),
        session: session_id.map(|s| s.to_string()),
        exp: utc.timestamp() as usize,
    };

    let key = EncodingKey::from_secret(secret.expose_secret().as_bytes());
    let token =
        encode(&Header::default(), &claims, &key).context("Failed to encode access token")?;
    Ok(token)
}

pub fn validate_token(token: &str, secret: &SecretString) -> anyhow::Result<TokenData<Claims>> {
    let key = DecodingKey::from_secret(secret.expose_secret().as_bytes());
    let data =
        decode::<Claims>(token, &key, &Validation::default()).context("Failed to decode token")?;
    Ok(data)
}
