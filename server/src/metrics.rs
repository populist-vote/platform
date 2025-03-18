use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute};
use axum::http::Request;
use axum::middleware::Next;
use lazy_static::lazy_static;
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGaugeVec, Registry, TextEncoder,
};
use std::{env, sync::Arc, time::Instant};

// Create a global registry
lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // Request metrics
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = IntCounterVec::new(
        prometheus::opts!("http_requests_total", "Total number of HTTP requests"),
        &["method", "path", "status"],
    )
    .expect("metric can be created");

    // GraphQL operation metrics
    pub static ref GRAPHQL_OPERATIONS: IntCounterVec = IntCounterVec::new(
        prometheus::opts!("graphql_operations_total", "Total number of GraphQL operations"),
        &["operation_name"],
    )
    .expect("metric can be created");

    pub static ref GRAPHQL_ERRORS: IntCounterVec = IntCounterVec::new(
        prometheus::opts!("graphql_errors_total", "Total number of GraphQL errors"),
        &["operation_name", "error_type"]
    )
    .expect("metric can be created");

    // Request duration
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        ),
        &["method", "path"],
    )
    .expect("metric can be created");

    // Database connections
    pub static ref DB_CONNECTIONS: IntGaugeVec = IntGaugeVec::new(
        prometheus::opts!("db_connections", "Number of active database connections"),
        &["pool_name"],
    )
    .expect("metric can be created");

    // NEW METRICS FOR WEB SERVER PERFORMANCE

    // Active requests (saturation)
    pub static ref HTTP_REQUESTS_IN_FLIGHT: IntGaugeVec = IntGaugeVec::new(
        prometheus::opts!("http_requests_in_flight", "Number of requests currently being processed"),
        &["method", "path"],
    )
    .expect("metric can be created");

    // Response size (throughput)
    pub static ref HTTP_RESPONSE_SIZE_BYTES: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "http_response_size_bytes",
            "Size of HTTP response in bytes"
        )
        .buckets(vec![64.0, 256.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0, 4194304.0]),
        &["method", "path", "status"],
    )
    .expect("metric can be created");

    // Request size
    pub static ref HTTP_REQUEST_SIZE_BYTES: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "http_request_size_bytes",
            "Size of HTTP request in bytes"
        )
        .buckets(vec![64.0, 256.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0]),
        &["method", "path"],
    )
    .expect("metric can be created");

    // Dedicated error counter
    pub static ref HTTP_ERRORS_TOTAL: IntCounterVec = IntCounterVec::new(
        prometheus::opts!("http_errors_total", "Total number of HTTP errors"),
        &["method", "path", "status", "error_type"],
    )
    .expect("metric can be created");

    // Histogram with proper buckets for percentile calculations
    // This ensures accurate p50, p90, p99 percentiles in Grafana
    pub static ref HTTP_REQUEST_DURATION_HISTOGRAM: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "http_request_duration_histogram_seconds",
            "HTTP request duration histogram optimized for percentiles"
        )
        .buckets(vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0
        ]),
        &["method", "path"],
    )
    .expect("metric can be created");
}

// Initialize metrics (register with registry)
pub fn init_metrics() {
    // Register existing metrics
    REGISTRY
        .register(Box::new(HTTP_REQUESTS_TOTAL.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(GRAPHQL_OPERATIONS.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(GRAPHQL_ERRORS.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(HTTP_REQUEST_DURATION.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(DB_CONNECTIONS.clone()))
        .expect("collector can be registered");

    // Register new metrics
    REGISTRY
        .register(Box::new(HTTP_REQUESTS_IN_FLIGHT.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(HTTP_RESPONSE_SIZE_BYTES.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(HTTP_REQUEST_SIZE_BYTES.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(HTTP_ERRORS_TOTAL.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(HTTP_REQUEST_DURATION_HISTOGRAM.clone()))
        .expect("collector can be registered");
}

// Update database connection metrics
pub fn update_db_connections(pool_name: &str, connections: i64) {
    DB_CONNECTIONS
        .with_label_values(&[pool_name])
        .set(connections);
}

// Function to expose metrics
pub async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

// Enhanced middleware for tracking HTTP requests with additional metrics
pub async fn track_metrics(req: Request<axum::body::Body>, next: Next) -> axum::response::Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // Estimate request size (headers + body if available)
    let request_size = req.headers().iter().fold(0, |acc, (name, value)| {
        acc + name.as_str().len() + value.len()
    }) as f64;

    // Record request size
    HTTP_REQUEST_SIZE_BYTES
        .with_label_values(&[&method, &path])
        .observe(request_size);

    // Increment in-flight requests counter
    HTTP_REQUESTS_IN_FLIGHT
        .with_label_values(&[&method, &path])
        .inc();

    // Start timing
    let start = Instant::now();

    // Process the request
    let response = next.run(req).await;

    // Calculate duration
    let duration = start.elapsed().as_secs_f64();

    // Record timing to both histograms
    HTTP_REQUEST_DURATION
        .with_label_values(&[&method, &path])
        .observe(duration);

    HTTP_REQUEST_DURATION_HISTOGRAM
        .with_label_values(&[&method, &path])
        .observe(duration);

    // Get response status
    let status = response.status().as_u16().to_string();

    // Record request count
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&method, &path, &status])
        .inc();

    // Track errors specifically
    if status.starts_with('4') || status.starts_with('5') {
        let error_type = if status.starts_with('4') {
            "client_error"
        } else {
            "server_error"
        };
        HTTP_ERRORS_TOTAL
            .with_label_values(&[&method, &path, &status, error_type])
            .inc();
    }

    // Estimate response size based on content-length if available
    if let Some(content_length) = response.headers().get("content-length") {
        if let Ok(length) = content_length.to_str().unwrap_or("0").parse::<f64>() {
            HTTP_RESPONSE_SIZE_BYTES
                .with_label_values(&[&method, &path, &status])
                .observe(length);
        }
    }

    // Decrement in-flight requests counter
    HTTP_REQUESTS_IN_FLIGHT
        .with_label_values(&[&method, &path])
        .dec();

    response
}

pub async fn metrics_auth(req: Request<axum::body::Body>, next: Next) -> axum::response::Response {
    // Get token from environment
    let expected_token = match env::var("METRICS_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            tracing::error!("METRICS_TOKEN environment variable not set");
            return axum::response::Response::builder()
                .status(401)
                .body(axum::body::Body::empty())
                .unwrap();
        }
    };

    // Check Authorization header
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..]; // Skip "Bearer " prefix

            if token == expected_token {
                return next.run(req).await;
            }
        }
        _ => {}
    }

    // Return 401 Unauthorized for any authorization failure
    axum::response::Response::builder()
        .status(401) // Using 401 instead of 502 for authentication failures
        .body(axum::body::Body::empty())
        .unwrap()
}

pub struct PrometheusMetricsExtension;

impl ExtensionFactory for PrometheusMetricsExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(PrometheusMetricsExtensionImpl)
    }
}

struct PrometheusMetricsExtensionImpl;

#[async_trait::async_trait]
impl Extension for PrometheusMetricsExtensionImpl {
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> async_graphql::Response {
        let op_name = operation_name.unwrap_or("unknown");

        // Log and count the operation
        tracing::info!("GraphQL operation: {}", op_name);
        GRAPHQL_OPERATIONS.with_label_values(&[op_name]).inc();

        // Execute the GraphQL operation
        let response = next.run(ctx, operation_name).await;

        // Track errors if they exist
        if !response.errors.is_empty() {
            for error in &response.errors {
                // Extract error type from the extension data
                if let Some(extensions) = &error.extensions {
                    if let Some(error_type) = extensions.get("type") {
                        let error_type_str = error_type.to_string();
                        // Increment counter with the enum variant name
                        GRAPHQL_ERRORS
                            .with_label_values(&[op_name, &error_type_str])
                            .inc();

                        tracing::error!(
                            "GraphQL error in operation {}: {} (type: {})",
                            op_name,
                            error.message,
                            error_type_str
                        );
                    }
                } else {
                    // Fallback for errors without extension data
                    GRAPHQL_ERRORS
                        .with_label_values(&[op_name, "unknown"])
                        .inc();

                    tracing::error!(
                        "GraphQL error in operation {} without type: {}",
                        op_name,
                        error.message
                    );
                }
            }
        }

        response
    }
}
