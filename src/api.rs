use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::db::Database;
use crate::metrics::Metrics;
use crate::middleware::metrics_middleware;
use crate::platform::{CreatePlatformProject, PlatformProject};

pub fn create_router(metrics: Arc<Metrics>, db: Arc<Database>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(get_metrics))
        .route("/api/v1/status", get(status))
        .route(
            "/api/v1/platform/projects",
            get(list_platform_projects).post(create_platform_project),
        )
        .route(
            "/api/v1/platform/projects/:id/suspend",
            post(suspend_platform_project),
        )
        .route(
            "/api/v1/platform/projects/:id/resume",
            post(resume_platform_project),
        )
        .layer(middleware::from_fn_with_state(
            metrics.clone(),
            metrics_middleware,
        ))
        .with_state(AppState { metrics, db })
}

#[derive(Clone)]
pub struct AppState {
    pub metrics: Arc<Metrics>,
    pub db: Arc<Database>,
}

async fn health() -> Response {
    (StatusCode::OK, "OK").into_response()
}

async fn ready(State(state): State<AppState>) -> Response {
    match state.db.health_check().await {
        Ok(_) => (StatusCode::OK, "Ready").into_response(),
        Err(e) => {
            tracing::error!("Readiness check failed: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, "Not Ready").into_response()
        }
    }
}

async fn get_metrics(State(state): State<AppState>) -> Response {
    match state.metrics.gather() {
        Ok(metrics) => (
            StatusCode::OK,
            [("Content-Type", "text/plain; version=0.0.4")],
            metrics,
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to gather metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to gather metrics").into_response()
        }
    }
}

async fn status(State(state): State<AppState>) -> Response {
    let db_status = match state.db.health_check().await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };
    let response = serde_json::json!({
        "status": "operational",
        "database": db_status,
        "version": env!("CARGO_PKG_VERSION")
    });

    (
        StatusCode::OK,
        [("Content-Type", "application/json")],
        serde_json::to_string(&response).unwrap(),
    )
        .into_response()
}

async fn list_platform_projects(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.list_platform_projects().await {
        Ok(projects) => (StatusCode::OK, Json(projects)).into_response(),
        Err(e) => {
            tracing::error!("Failed to list platform projects: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list platform projects",
            )
                .into_response()
        }
    }
}

async fn create_platform_project(
    State(state): State<AppState>,
    Json(payload): Json<CreatePlatformProject>,
) -> impl IntoResponse {
    match state.db.create_platform_project(payload).await {
        Ok(project) => (StatusCode::CREATED, Json(project)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create platform project: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create platform project",
            )
                .into_response()
        }
    }
}

async fn suspend_platform_project(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state
        .db
        .update_platform_project_status(id, "suspended")
        .await
    {
        Ok(Some(project)) => (StatusCode::OK, Json(project)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Project not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to suspend platform project {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to suspend platform project",
            )
                .into_response()
        }
    }
}

async fn resume_platform_project(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.db.update_platform_project_status(id, "active").await {
        Ok(Some(project)) => (StatusCode::OK, Json(project)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Project not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to resume platform project {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to resume platform project",
            )
                .into_response()
        }
    }
}


