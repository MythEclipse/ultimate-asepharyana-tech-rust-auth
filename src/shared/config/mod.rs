use anyhow::Result;
use serde::Deserialize;
use std::sync::OnceLock;
use tracing::info;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub jwt: JwtSettings,
    pub observability: ObservabilitySettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppSettings {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub environment: String,
    pub debug: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub pool_size: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
    pub sslmode: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisSettings {
    pub url: String,
    pub pool_size: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub access_expiration: i64,   // seconds
    pub refresh_expiration: i64,  // seconds
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilitySettings {
    pub rust_log: String,
    pub prometheus_port: u16,
}

impl AppConfig {
    pub fn global() -> &'static AppConfig {
        CONFIG.get().expect("AppConfig not initialized")
    }

    pub fn from_env() -> Result<Self> {
        Ok(AppConfig {
            app: AppSettings {
                name: "rust-auth".to_string(),
                host: std::env::var("HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("HTTP_PORT")
                    .unwrap_or_else(|_| "8000".to_string())
                    .parse()?,
                environment: std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
                debug: std::env::var("RUST_LOG").as_deref() == Ok("debug"),
            },
            database: DatabaseSettings {
                url: std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set"),
                pool_size: std::env::var("POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                connect_timeout: std::env::var("CONNECT_TIMEOUT")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                idle_timeout: std::env::var("IDLE_TIMEOUT")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()?,
                max_lifetime: std::env::var("MAX_LIFETIME")
                    .unwrap_or_else(|_| "1800".to_string())
                    .parse()?,
                sslmode: std::env::var("SSLMODE").unwrap_or_else(|_| "disable".to_string()),
            },
            redis: RedisSettings {
                url: std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                pool_size: 20,
            },
            jwt: JwtSettings {
                secret: std::env::var("JWT_SECRET")
                    .expect("JWT_SECRET must be set"),
                access_expiration: 900,   // 15 minutes
                refresh_expiration: 604800, // 7 days
                issuer: "rust-auth".to_string(),
                audience: "asepharyana.my.id".to_string(),
            },
            observability: ObservabilitySettings {
                rust_log: std::env::var("RUST_LOG")
                    .unwrap_or_else(|_| "info".to_string()),
                prometheus_port: std::env::var("MONITORING_PROMETHEUS_PORT")
                    .unwrap_or_else(|_| "9490".to_string())
                    .parse()?,
            },
        })
    }

    pub fn init() -> Result<()> {
        dotenvy::dotenv().ok();
        let config = Self::from_env()?;
        let _ = CONFIG.set(config);
        info!("Configuration loaded successfully");
        Ok(())
    }
}
