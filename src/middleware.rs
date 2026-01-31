use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;

use crate::metrics::Metrics;

pub async fn metrics_middleware(
    State(metrics): State<Arc<Metrics>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    // Extract endpoint path
    let endpoint = normalize_path(uri.path());

    // Track request size (approximate from headers)
    let request_size = request
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    // Increment active connections
    metrics.active_connections.inc();

    // Process request
    let response = next.run(request).await;

    // Calculate duration
    let duration = start.elapsed().as_secs_f64();

    // Get status code
    let status = response.status().as_u16();
    let status_str = status.to_string();

    // Determine error type
    let error_type = if status >= 500 {
        "server_error"
    } else if status >= 400 {
        "client_error"
    } else {
        "none"
    };

    let error_class = if status >= 500 {
        "5xx"
    } else if status >= 400 {
        "4xx"
    } else {
        "success"
    };

    // Track request size
    if request_size > 0.0 {
        metrics
            .http_request_size_bytes
            .with_label_values(&[method.as_str(), &endpoint])
            .observe(request_size);
    }

    // Track response size (approximate from headers)
    let response_size = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    if response_size > 0.0 {
        metrics
            .http_response_size_bytes
            .with_label_values(&[method.as_str(), &endpoint, &status_str])
            .observe(response_size);
    }

    // Record metrics
    metrics
        .http_requests_total
        .with_label_values(&[method.as_str(), &endpoint, &status_str])
        .inc();

    metrics
        .http_request_duration_seconds
        .with_label_values(&[method.as_str(), &endpoint])
        .observe(duration);

    // Track errors
    if error_type != "none" {
        metrics
            .http_errors_total
            .with_label_values(&[method.as_str(), &endpoint, &status_str, error_type])
            .inc();
    }

    // Track SLA violations (e.g., p95 > 500ms or p99 > 1s)
    if duration > 0.5 {
        metrics
            .sla_violations_total
            .with_label_values(&[&endpoint, "latency_p95"])
            .inc();
    }
    if duration > 1.0 {
        metrics
            .sla_violations_total
            .with_label_values(&[&endpoint, "latency_p99"])
            .inc();
    }

    // Decrement active connections
    metrics.active_connections.dec();

    response
}

fn normalize_path(path: &str) -> String {
    // Normalize paths to avoid high cardinality
    // For example: /api/v1/users/123 -> /api/v1/users/:id
    if path.starts_with("/api/") {
        // For API routes, try to normalize IDs
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() > 3 {
            // Check if last part looks like an ID (numeric or UUID-like)
            let last = parts.last().unwrap();
            if last.parse::<u64>().is_ok() || last.len() > 10 {
                // Replace with placeholder
                let mut normalized = parts[..parts.len() - 1].join("/");
                normalized.push_str("/:id");
                return normalized;
            }
        }
    }
    path.to_string()
}

