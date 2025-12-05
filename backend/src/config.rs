use dotenvy::var;
use secrecy::SecretString;
use std::env;

#[derive(Debug, Clone)]
pub struct Setting {
    pub database_url: String,
    pub jwt_secret: SecretString,
    pub default_max_connections: i16,
    pub access_token_exp: i64,
    pub refresh_token_exp: i64,
    pub bind_addr: String,
    pub redis_url: String,
    pub worker_threads: usize,
}

impl Setting {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = env::var("DATABASE_URL")?;
        let jwt_secret = env::var("JWT_SECRET").map(SecretString::from)?; //this doesn't allow unintentional pr accidental access of secrets
        let default_max_connections = env::var("DEFAULT_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".into())
            .parse::<i16>()?;
        let access_token_exp = env::var("ACCESS_TOKEN_EXP")
            .unwrap_or_else(|_| "900".into())
            .parse::<i64>()?;
        let refresh_token_exp = env::var("REFRESH_TOKEN_EXP")
            .unwrap_or_else(|_| "604800".into())
            .parse::<i64>()?;
        let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".into());
        let worker_threads = env::var("WORKER_THREADS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or_else(|| num_cpus::get());

        Ok(Self {
            database_url,
            jwt_secret,
            default_max_connections,
            access_token_exp,
            refresh_token_exp,
            bind_addr,
            redis_url,
            worker_threads,
        })
    }
}
