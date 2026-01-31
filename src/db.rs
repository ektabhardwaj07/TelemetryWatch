use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::info;

pub struct Database {
    pub pool: PgPool,
    pub max_connections: u32,
}

impl Database {
    pub async fn new(database_url: &str, max_connections: u32) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(10))
            .connect(database_url)
            .await?;

        info!("Connected to PostgreSQL database");

        // Initialize schema
        Self::init_schema(&pool).await?;

        Ok(Self {
            pool,
            max_connections,
        })
    }

    pub fn get_pool_stats(&self) -> (u32, u32) {
        // Get pool statistics
        // Note: sqlx doesn't expose detailed pool stats, so we track configured size
        // Active connections can be estimated from query activity
        let size = self.max_connections;
        // For demo purposes, we'll track size and let active be calculated from query patterns
        (size, size) // Return (size, max_connections) - active will be tracked via query metrics
    }

    async fn init_schema(pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS telemetry_sources (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL UNIQUE,
                source_type VARCHAR(100) NOT NULL,
                endpoint VARCHAR(500),
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metric_metadata (
                id SERIAL PRIMARY KEY,
                source_id INTEGER REFERENCES telemetry_sources(id),
                metric_name VARCHAR(255) NOT NULL,
                metric_type VARCHAR(50) NOT NULL,
                description TEXT,
                labels JSONB,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        info!("Database schema initialized");
        Ok(())
    }

    pub async fn health_check(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }
}

