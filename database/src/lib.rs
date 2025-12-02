use anyhow::{Context, Ok};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::info;

pub mod models;

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
        .with_context(|| format!("Failed to connect to database {}", &database_url))?;
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    info!("Running database migrations ...");

    let migrator = sqlx::migrate!("./migrations");
    migrator.run(pool).await?;
    info!("Migrations applied");
    Ok(())
}
