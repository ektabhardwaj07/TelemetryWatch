use prometheus::{
    Encoder, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, IntCounter, IntCounterVec,
    Opts, Registry, TextEncoder,
};
use std::sync::Arc;

pub struct Metrics {
    pub registry: Registry,
    pub http_requests_total: IntCounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub active_connections: Gauge,
    pub database_queries_total: IntCounter,
    pub database_query_duration_seconds: Histogram,
    // Error tracking
    pub http_errors_total: IntCounterVec,
    pub http_error_rate: GaugeVec,
    // Database connection pool metrics
    pub db_pool_size: Gauge,
    pub db_pool_idle: Gauge,
    pub db_pool_active: Gauge,
    pub db_pool_wait_time_seconds: Histogram,
    // Request/Response payload metrics
    pub http_request_size_bytes: HistogramVec,
    pub http_response_size_bytes: HistogramVec,
    // Endpoint-specific error rates
    pub endpoint_error_rate: GaugeVec,
    // SLA compliance tracking
    pub sla_violations_total: IntCounterVec,
}

impl Metrics {
    pub fn new() -> anyhow::Result<Arc<Self>> {
        let registry = Registry::new();

        let http_requests_total = IntCounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests"),
            &["method", "endpoint", "status"],
        )?;
        registry.register(Box::new(http_requests_total.clone()))?;

        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
            &["method", "endpoint"],
        )?;
        registry.register(Box::new(http_request_duration_seconds.clone()))?;

        let active_connections = Gauge::with_opts(Opts::new(
            "active_connections",
            "Number of active connections",
        ))?;
        registry.register(Box::new(active_connections.clone()))?;

        let database_queries_total = IntCounter::with_opts(Opts::new(
            "database_queries_total",
            "Total number of database queries",
        ))?;
        registry.register(Box::new(database_queries_total.clone()))?;

        let database_query_duration_seconds = Histogram::with_opts(
            HistogramOpts::new(
                "database_query_duration_seconds",
                "Database query duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
        )?;
        registry.register(Box::new(database_query_duration_seconds.clone()))?;

        // Error tracking metrics
        let http_errors_total = IntCounterVec::new(
            Opts::new("http_errors_total", "Total number of HTTP errors"),
            &["method", "endpoint", "status", "error_type"],
        )?;
        registry.register(Box::new(http_errors_total.clone()))?;

        let http_error_rate = GaugeVec::new(
            Opts::new("http_error_rate", "HTTP error rate (errors per second)"),
            &["error_class"],
        )?;
        registry.register(Box::new(http_error_rate.clone()))?;

        // Database connection pool metrics
        let db_pool_size = Gauge::with_opts(Opts::new(
            "db_pool_size",
            "Total size of the database connection pool",
        ))?;
        registry.register(Box::new(db_pool_size.clone()))?;

        let db_pool_idle = Gauge::with_opts(Opts::new(
            "db_pool_idle",
            "Number of idle connections in the pool",
        ))?;
        registry.register(Box::new(db_pool_idle.clone()))?;

        let db_pool_active = Gauge::with_opts(Opts::new(
            "db_pool_active",
            "Number of active connections in the pool",
        ))?;
        registry.register(Box::new(db_pool_active.clone()))?;

        let db_pool_wait_time_seconds = Histogram::with_opts(
            HistogramOpts::new(
                "db_pool_wait_time_seconds",
                "Time spent waiting for a database connection",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
        )?;
        registry.register(Box::new(db_pool_wait_time_seconds.clone()))?;

        // Request/Response payload metrics
        let http_request_size_bytes = HistogramVec::new(
            HistogramOpts::new(
                "http_request_size_bytes",
                "HTTP request payload size in bytes",
            )
            .buckets(vec![
                100.0, 500.0, 1000.0, 5000.0, 10000.0, 50000.0, 100000.0, 500000.0, 1000000.0,
            ]),
            &["method", "endpoint"],
        )?;
        registry.register(Box::new(http_request_size_bytes.clone()))?;

        let http_response_size_bytes = HistogramVec::new(
            HistogramOpts::new(
                "http_response_size_bytes",
                "HTTP response payload size in bytes",
            )
            .buckets(vec![
                100.0, 500.0, 1000.0, 5000.0, 10000.0, 50000.0, 100000.0, 500000.0, 1000000.0,
            ]),
            &["method", "endpoint", "status"],
        )?;
        registry.register(Box::new(http_response_size_bytes.clone()))?;

        // Endpoint-specific error rates
        let endpoint_error_rate = GaugeVec::new(
            Opts::new(
                "endpoint_error_rate",
                "Error rate per endpoint (errors per second)",
            ),
            &["endpoint", "error_class"],
        )?;
        registry.register(Box::new(endpoint_error_rate.clone()))?;

        // SLA compliance tracking
        let sla_violations_total = IntCounterVec::new(
            Opts::new("sla_violations_total", "Total number of SLA violations"),
            &["endpoint", "sla_type"],
        )?;
        registry.register(Box::new(sla_violations_total.clone()))?;

        Ok(Arc::new(Self {
            registry,
            http_requests_total,
            http_request_duration_seconds,
            active_connections,
            database_queries_total,
            database_query_duration_seconds,
            http_errors_total,
            http_error_rate,
            db_pool_size,
            db_pool_idle,
            db_pool_active,
            db_pool_wait_time_seconds,
            http_request_size_bytes,
            http_response_size_bytes,
            endpoint_error_rate,
            sla_violations_total,
        }))
    }

    pub fn gather(&self) -> anyhow::Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

