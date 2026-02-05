use serde::{Deserialize, Serialize};
use std::env;
use tracing::warn;

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
                    // Debug: Check which env vars are available
                    let has_db_url = env::var("DATABASE_URL").is_ok();
                    let has_postgres_url = env::var("POSTGRES_URL").is_ok();
                    let has_pg_url = env::var("PGDATABASE_URL").is_ok();
                    let has_db_public_url = env::var("DATABASE_PUBLIC_URL").is_ok();
                    
                    // Try multiple environment variable names (Railway provides DATABASE_URL for internal connections)
                    // Prefer DATABASE_URL (internal) over DATABASE_PUBLIC_URL (public proxy)
                    let db_url_result = env::var("DATABASE_URL")
                        .or_else(|_| env::var("POSTGRES_URL"))
                        .or_else(|_| env::var("PGDATABASE_URL"))
                        .or_else(|_| env::var("DATABASE_PUBLIC_URL")); // Fallback to public URL if internal not available
                    
                    let db_url = match db_url_result {
                        Ok(url) => {
                            // Log raw value length for debugging (without exposing sensitive data)
                            warn!("Found database URL in environment (length: {}). Source: DATABASE_URL={}, POSTGRES_URL={}, PGDATABASE_URL={}, DATABASE_PUBLIC_URL={}", 
                                url.len(), has_db_url, has_postgres_url, has_pg_url, has_db_public_url);
                            url
                        }
                        Err(_) => {
                            warn!("No database URL found in environment. Checked: DATABASE_URL={}, POSTGRES_URL={}, PGDATABASE_URL={}, DATABASE_PUBLIC_URL={}. Using default.", 
                                has_db_url, has_postgres_url, has_pg_url, has_db_public_url);
                            "postgresql://telemetrywatch:telemetrywatch@localhost:5432/telemetrywatch".to_string()
                        }
                    };
                    
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
                        warn!("Database URL is empty after trimming (original length: {}). This usually means Railway's DATABASE_URL is set but empty. Trying DATABASE_PUBLIC_URL as fallback.", db_url.len());
                        // Try DATABASE_PUBLIC_URL as last resort
                        if let Ok(public_url) = env::var("DATABASE_PUBLIC_URL") {
                            let public_trimmed = public_url.trim().trim_matches('"').trim_matches('\'').trim_matches('\n').trim_matches('\r').to_string();
                            if !public_trimmed.is_empty() {
                                warn!("Using DATABASE_PUBLIC_URL as fallback");
                                public_trimmed
                            } else {
                                warn!("DATABASE_PUBLIC_URL is also empty, using default");
                                "postgresql://telemetrywatch:telemetrywatch@localhost:5432/telemetrywatch".to_string()
                            }
                        } else {
                            "postgresql://telemetrywatch:telemetrywatch@localhost:5432/telemetrywatch".to_string()
                        }
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

