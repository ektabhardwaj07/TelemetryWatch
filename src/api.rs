use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tower_http::services::ServeDir;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::db::Database;
use crate::metrics::Metrics;
use crate::middleware::metrics_middleware;
use crate::platform::{CreatePlatformProject, PlatformProject};

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        ready,
        get_metrics,
        status,
        list_platform_projects,
        create_platform_project,
        suspend_platform_project,
        resume_platform_project,
    ),
    components(schemas(
        PlatformProject,
        CreatePlatformProject,
    )),
    tags(
        (name = "Health", description = "Health and readiness endpoints"),
        (name = "Metrics", description = "Prometheus metrics endpoint"),
        (name = "Platform", description = "Platform control plane API for managing Supabase projects"),
    ),
    info(
        title = "TelemetryWatch Platform Control Plane API",
        description = "REST API for managing multiple Supabase OSS projects as a platform provider",
        version = "1.0.0",
        contact(
            name = "TelemetryWatch",
            url = "https://github.com/ektabhardwaj07/TelemetryWatch",
        ),
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development"),
        (url = "https://telemetrywatch-production-22dc.up.railway.app", description = "Railway production"),
    ),
)]
struct ApiDoc;

pub fn create_router(metrics: Arc<Metrics>, db: Arc<Database>) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
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
        .route("/", get(serve_index))
        .nest_service("/static", ServeDir::new("static"))
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

/// Health check endpoint
/// 
/// Returns 200 OK if the service is running.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = String, example = "OK")
    )
)]
async fn health() -> Response {
    (StatusCode::OK, "OK").into_response()
}

/// Readiness check endpoint
/// 
/// Returns 200 OK if the service and database are ready to accept traffic.
#[utoipa::path(
    get,
    path = "/ready",
    tag = "Health",
    responses(
        (status = 200, description = "Service is ready", body = String, example = "Ready"),
        (status = 503, description = "Service is not ready", body = String, example = "Not Ready")
    )
)]
async fn ready(State(state): State<AppState>) -> Response {
    match state.db.health_check().await {
        Ok(_) => (StatusCode::OK, "Ready").into_response(),
        Err(e) => {
            tracing::error!("Readiness check failed: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, "Not Ready").into_response()
        }
    }
}

/// Prometheus metrics endpoint
/// 
/// Returns metrics in Prometheus format for scraping.
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "Metrics",
    responses(
        (status = 200, description = "Prometheus metrics", content_type = "text/plain")
    )
)]
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

/// Application status endpoint
/// 
/// Returns detailed status including database health and version.
#[utoipa::path(
    get,
    path = "/api/v1/status",
    tag = "Health",
    responses(
        (status = 200, description = "Application status", 
         content_type = "application/json",
         body = Object)
    )
)]
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

/// List all registered Supabase projects
/// 
/// Returns a list of all platform projects with their metadata.
#[utoipa::path(
    get,
    path = "/api/v1/platform/projects",
    tag = "Platform",
    responses(
        (status = 200, description = "List of platform projects", body = [PlatformProject]),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Register a new Supabase project
/// 
/// Creates a new platform project entry with the provided metadata.
#[utoipa::path(
    post,
    path = "/api/v1/platform/projects",
    tag = "Platform",
    request_body = CreatePlatformProject,
    responses(
        (status = 201, description = "Project created successfully", body = PlatformProject),
        (status = 500, description = "Failed to create project")
    )
)]
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

/// Suspend a platform project
/// 
/// Changes the project status to 'suspended'. In production, this would also trigger
/// actions in the actual Supabase instance.
#[utoipa::path(
    post,
    path = "/api/v1/platform/projects/{id}/suspend",
    tag = "Platform",
    params(
        ("id" = i64, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Project suspended successfully", body = PlatformProject),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Failed to suspend project")
    )
)]
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

/// Resume a suspended platform project
/// 
/// Changes the project status from 'suspended' to 'active'. In production, this would
/// also trigger actions in the actual Supabase instance.
#[utoipa::path(
    post,
    path = "/api/v1/platform/projects/{id}/resume",
    tag = "Platform",
    params(
        ("id" = i64, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Project resumed successfully", body = PlatformProject),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Failed to resume project")
    )
)]
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

async fn serve_index() -> impl IntoResponse {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(html) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html")],
            html,
        )
            .into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "Dashboard not available",
        )
            .into_response(),
    }
}


