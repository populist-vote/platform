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
}

// Initialize metrics (register with registry)
pub fn init_metrics() {
    // Register metrics with the global registry
    REGISTRY
        .register(Box::new(HTTP_REQUESTS_TOTAL.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(GRAPHQL_OPERATIONS.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(HTTP_REQUEST_DURATION.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(DB_CONNECTIONS.clone()))
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

// Middleware for tracking HTTP requests
pub async fn track_metrics(req: Request<axum::body::Body>, next: Next) -> axum::response::Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // Start timing
    let start = Instant::now();

    // Process the request
    let response = next.run(req).await;

    // Record timing
    let duration = start.elapsed().as_secs_f64();
    HTTP_REQUEST_DURATION
        .with_label_values(&[&method, &path])
        .observe(duration);

    // Record request
    let status = response.status().as_u16().to_string();
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&method, &path, &status])
        .inc();

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
        // Safely handle potentially null operation names
        let op_name = operation_name.unwrap_or("unknown");

        // Log the operation for debugging
        tracing::info!("GraphQL operation: {}", op_name);

        // Increment the counter for this operation
        GRAPHQL_OPERATIONS.with_label_values(&[op_name]).inc();

        // Execute the GraphQL operation
        next.run(ctx, operation_name).await
    }
}
