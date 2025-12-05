use crate::config::Setting;
use anyhow::Ok;
use redis::Client as RedisClient;
use redis::aio::ConnectionManager;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub setting: Setting,
    pub redis: ConnectionManager, // stores the manager directly (Clone , Send , Sync)
                                  //it manages async connections automatically (re-connections, retries and async connection management through tokio)
}

impl AppState {
    pub async fn new(pool: PgPool, setting: Setting) -> Result<Self> {
        let client = RedisClient::open(setting.redis_url.as_str())?;

        let manager = client
            .get_tokio_connection_manager()
            .await
            .map_err(|e| anyhow::anyhow!("failed to create redis connection manager: {}", e))?;

        Ok(Self {
            pool,
            setting,
            redis: manager,
        })
    }
}
