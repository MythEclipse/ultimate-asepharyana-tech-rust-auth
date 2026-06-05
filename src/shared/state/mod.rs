use crate::shared::config::AppConfig;
use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;
use std::sync::Arc;

/// Application state shared across all handlers via Axum's State extractor.
#[derive(Clone)]
pub struct AppState {
    pub config: &'static AppConfig,
    pub db: PgPool,
    pub redis: RedisPool,
}

impl AppState {
    pub fn new(db: PgPool, redis: RedisPool) -> Self {
        Self {
            config: AppConfig::global(),
            db,
            redis,
        }
    }

    pub fn arc(db: PgPool, redis: RedisPool) -> Arc<Self> {
        Arc::new(Self::new(db, redis))
    }
}
