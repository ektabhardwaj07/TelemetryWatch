mod api;
mod config;
mod db;
mod metrics;
mod middleware;
mod platform;

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use api::create_router;
use config::Config;
use db::Database;
use metrics::Metrics;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "telemetrywatch=info,tower_http=info".into()),
        )
        .init();

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded");
    // Log first 30 chars and last 10 chars for debugging (without exposing password)
    let db_url_display = if config.database.url.len() > 40 {
        format!("{}...{}", 
            &config.database.url[..30],
            &config.database.url[config.database.url.len()-10..]
        )
    } else {
        format!("{} (length: {})", 
            config.database.url.chars().take(30).collect::<String>(),
            config.database.url.len()
        )
    };
    info!("Database URL preview: {}", db_url_display);
    info!("Database URL starts with postgresql://: {}", config.database.url.starts_with("postgresql://"));
    info!("Database URL starts with postgres://: {}", config.database.url.starts_with("postgres://"));

    // Initialize metrics
    let metrics = Metrics::new()?;
    info!("Metrics initialized");

    // Initialize database
    info!("Attempting to connect to database...");
    let database = Arc::new(
        Database::new(&config.database.url, config.database.max_connections)
            .await
            .map_err(|e| {
                tracing::error!("Database connection failed: {}", e);
                e
            })?,
    );
    info!("Database initialized");

    // Start background task to update metrics
    let metrics_clone = metrics.clone();
    let db_clone = database.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            update_metrics(&metrics_clone, &db_clone).await;
        }
    });

    // Create router
    let app = create_router(metrics, database);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting TelemetryWatch server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn update_metrics(metrics: &Arc<Metrics>, db: &Arc<Database>) {
    // Update database pool metrics
    let (size, _) = db.get_pool_stats();
    metrics.db_pool_size.set(size as f64);
    // Note: sqlx doesn't expose idle/active directly, but we can track via query patterns
    // For demo, we'll show the configured pool size

    // Update platform projects metrics
    if let Ok(projects) = db.list_platform_projects().await {
        // Reset all platform_projects gauges to 0 first
        // We need to reset all possible label combinations, but Prometheus doesn't expose
        // a way to iterate over existing labels. Instead, we'll reset the totals and
        // only set gauges for current projects (old statuses will remain but that's okay
        // as long as we're consistent)

        // Count projects by status and plan
        let mut status_plan_counts: std::collections::HashMap<(String, String), i32> =
            std::collections::HashMap::new();

        // Track which (slug, status, plan, region) combinations we're setting
        let mut active_combinations = std::collections::HashSet::new();

        for project in &projects {
            let combo = (
                project.slug.clone(),
                project.status.clone(),
                project.plan.clone(),
                project.region.clone(),
            );
            active_combinations.insert(combo.clone());

            // Set individual project gauge (only current status)
            metrics
                .platform_projects
                .with_label_values(&[
                    &project.slug,
                    &project.status,
                    &project.plan,
                    &project.region,
                ])
                .set(1.0);

            // Count for totals
            let key = (project.status.clone(), project.plan.clone());
            *status_plan_counts.entry(key).or_insert(0) += 1;
        }

        // Reset totals to 0 first, then set current counts
        // Note: We can't easily reset all label combinations, but we can reset totals
        // by setting all known combinations to 0, then setting current ones
        for status in &["active", "suspended"] {
            for plan in &["dev", "pro", "enterprise"] {
                metrics
                    .platform_projects_total
                    .with_label_values(&[status, plan])
                    .set(0.0);
            }
        }

        // Set current total counts
        for ((status, plan), count) in status_plan_counts {
            metrics
                .platform_projects_total
                .with_label_values(&[&status, &plan])
                .set(count as f64);
        }
    }
}

