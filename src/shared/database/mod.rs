use crate::shared::config::AppConfig;
use anyhow::Result;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use tracing::info;

pub async fn init_pool() -> Result<PgPool> {
    let config = AppConfig::global();

    let connect_opts: PgConnectOptions = config.database.url.parse()?;

    let pool = PgPoolOptions::new()
        .max_connections(config.database.pool_size)
        .acquire_timeout(std::time::Duration::from_secs(config.database.connect_timeout))
        .idle_timeout(std::time::Duration::from_secs(config.database.idle_timeout))
        .max_lifetime(std::time::Duration::from_secs(config.database.max_lifetime))
        .connect_with(connect_opts)
        .await?;

    // Verify connection
    sqlx::query("SELECT 1").execute(&pool).await?;
    info!("Database connection pool established");

    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    info!("Database migrations applied successfully");
    Ok(())
}
