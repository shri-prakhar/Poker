use std::env;

use dotenvy::var;
use secrecy::SecretString;

#[derive(Debug , Clone)]
pub struct Setting {
    pub database_url: String,
    pub jwt_secret: SecretString,
    pub default_max_connections: String,
    pub access_token_exp: i64,
    pub refresh_token_exp: i64,
    pub bind_addr:String,
    pub redis_url: String
}

impl Setting{
        pub fn from_env() -> anyhow::Result<Self>{
            let database_url = env::var("DATABASE_URL")?;
            let jwt_secret = env::var("JWT_SECRET").map(SecretString::from)?;
            let default_max_connections = env::var("DEFAULT_MAX_CONNECTIONS")?;
            let access_token_exp = env::var("ACCESS_TOKEN_EXP")?;
            let refresh_token_exp = env::var("REFRESH_TOKEN_EXP")?;
            let bind_addr = env::var("BIND_ADDR")?;
            let redis_url = env::var("REDIS_URL")?;

            Ok(Self{
                database_url,
                jwt_secret,
                default_max_connections,
                access_token_exp,
                refresh_token_exp,
                bind_addr,
                redis_url
            })
        }
}