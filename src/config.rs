use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8080),
            },
            database: DatabaseConfig {
                url: {
                    // Try multiple environment variable names (Railway provides DATABASE_URL for internal connections)
                    // Prefer DATABASE_URL (internal) over DATABASE_PUBLIC_URL (public proxy)
                    let db_url = env::var("DATABASE_URL")
                        .or_else(|_| env::var("POSTGRES_URL"))
                        .or_else(|_| env::var("PGDATABASE_URL"))
                        .or_else(|_| env::var("DATABASE_PUBLIC_URL")) // Fallback to public URL if internal not available
                        .unwrap_or_else(|_| "postgresql://telemetrywatch:telemetrywatch@localhost:5432/telemetrywatch".to_string());
                    // Trim whitespace, newlines, and quotes that might be accidentally added
                    let trimmed = db_url
                        .trim()
                        .trim_matches('\n')
                        .trim_matches('\r')
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                    // Ensure it's not empty
                    if trimmed.is_empty() {
                        "postgresql://telemetrywatch:telemetrywatch@localhost:5432/telemetrywatch".to_string()
                    } else {
                        trimmed
                    }
                },
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|c| c.parse().ok())
                    .unwrap_or(10),
            },
            metrics: MetricsConfig {
                enabled: env::var("METRICS_ENABLED")
                    .ok()
                    .and_then(|e| e.parse().ok())
                    .unwrap_or(true),
            },
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();
        Ok(Self::default())
    }
}

