use crate::modules::auth::route as auth_route;
use crate::modules::user::route as user_route;
use crate::shared::config::AppConfig;
use crate::shared::database;
use crate::shared::middlewares::auth;
use crate::shared::middlewares::cors::cors_layer;
use crate::shared::state::AppState;
use anyhow::Result;
use axum::{middleware, Router};
use deadpool_redis::Config as RedisConfig;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

pub struct Application {
    state: Arc<AppState>,
}

impl Application {
    pub async fn build() -> Result<Self> {
        // Initialize configuration
        AppConfig::init()?;
        let config = AppConfig::global();

        // Initialize tracing
        tracing_subscriber::fmt()
            .with_env_filter(&config.observability.rust_log)
            .with_target(true)
            .with_thread_ids(true)
            .init();

        // Initialize OpenTelemetry metrics
        crate::shared::middlewares::metrics::init_otel_metrics();

        info!("{} v{} starting up", config.app.name, "0.1.0");
        info!("Environment: {}", config.app.environment);

        // Initialize database pool
        let db_pool = database::init_pool().await?;

        // Initialize Redis pool
        let redis_cfg = RedisConfig::from_url(&config.redis.url);
        let redis_pool = redis_cfg
            .create_pool(None::<deadpool_redis::Runtime>)
            .map_err(|e| anyhow::anyhow!("Failed to create Redis pool: {}", e))?;

        // Verify Redis connection
        match redis_pool.get().await {
            Ok(mut conn) => {
                let _: Result<String, _> = redis::cmd("PING").query_async(&mut *conn).await;
                info!("Redis connection established");
            }
            Err(e) => {
                tracing::warn!(?e, "Redis connection failed (non-fatal)");
            }
        }

        let state = AppState::arc(db_pool, redis_pool);

        Ok(Self { state })
    }

    pub fn router(&self) -> Router {
        Router::new()
            // Auth routes (public - no auth middleware)
            .merge(auth_route::routes())
            // User routes with auth middleware
            .merge(
                user_route::routes()
                    .route_layer(middleware::from_fn_with_state(
                        self.state.clone(),
                        auth::auth_middleware,
                    )),
            )
            // Global middleware stack
            .layer(TraceLayer::new_for_http())
            .layer(axum::middleware::from_fn(
                crate::shared::middlewares::metrics::otel_metrics_middleware,
            ))
            .layer(cors_layer())
            .with_state(self.state.clone())
    }

    pub async fn run(&self) -> Result<()> {
        let config = AppConfig::global();
        let addr = format!("{}:{}", config.app.host, config.app.port);
        let listener = TcpListener::bind(&addr).await?;

        info!("Server listening on http://{}", addr);

        axum::serve(
            listener,
            self.router().into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await?;

        Ok(())
    }
}
