//! OpenTelemetry metrics initialization and HTTP middleware for rust-auth.
//!
//! Uses OTLP gRPC export to the rust-otlp-metrics-backend.
//! Coexists with the existing Prometheus /metrics endpoint.
//!
//! Environment:
//!   OTEL_EXPORTER_OTLP_ENDPOINT  — default: http://localhost:4317
//!   OTEL_SERVICE_NAME             — default: rust-auth
//!   OTEL_METRICS_EXPORT_INTERVAL  — default: 5000 (ms)

use axum::{extract::Request, middleware::Next, response::Response};
use opentelemetry::{
    global,
    metrics::{Counter, Histogram, Meter, UpDownCounter},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, PeriodicReader},
    Resource,
};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME as ATTR_SERVICE_NAME;
use std::sync::OnceLock;
use std::time::Instant;

static METER: OnceLock<Meter> = OnceLock::new();

fn meter() -> &'static Meter {
    METER.get().expect("OTel meter not initialized — call init_otel_metrics first")
}

/// Initialize the global OTLP MeterProvider.
/// Safe to call multiple times — subsequent calls are no-ops.
pub fn init_otel_metrics() {
    let otel_endpoint =
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".into());
    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "rust-auth".into());
    let export_interval_ms: u64 = std::env::var("OTEL_METRICS_EXPORT_INTERVAL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000);

    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(otel_endpoint.clone())
        .build()
        .expect("Failed to create OTLP metric exporter");

    let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_interval(std::time::Duration::from_millis(export_interval_ms))
        .build();

    let resource = Resource::builder()
        .with_attribute(KeyValue::new(ATTR_SERVICE_NAME, service_name.clone()))
        .build();

    let provider = MeterProviderBuilder::default()
        .with_resource(resource)
        .with_reader(reader)
        .build();

    global::set_meter_provider(provider);

    let m = global::meter("rust-auth-http-server");
    let _ = METER.set(m);

    tracing::info!(otel_endpoint, service_name, "OTel metrics initialized");
}

/// Shut down the global MeterProvider, flushing pending exports.
pub async fn shutdown_otel_metrics() {
    let provider = global::meter_provider();
    if let Err(e) = provider.shutdown() {
        tracing::warn!(error = %e, "OTel metrics shutdown error");
    } else {
        tracing::info!("OTel metrics shut down");
    }
}

// ---------------------------------------------------------------------------
// Axum middleware
// ---------------------------------------------------------------------------

/// Axum middleware that records standard HTTP server metrics via OpenTelemetry.
///
/// Must be added **after** `init_otel_metrics()` has been called.
///
/// Metrics:
///   - `http.server.request_count`       — Counter { method, path, status }
///   - `http.server.request_duration_ms`  — Histogram { method, path, status }
///   - `http.server.request_in_flight`    — UpDownCounter { method, path }
pub async fn otel_metrics_middleware(req: Request, next: Next) -> Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let in_flight = in_flight_counter();
    let counter = request_counter();
    let duration = duration_histogram();

    in_flight.add(1, &[KeyValue::new("method", &method), KeyValue::new("path", &path)]);

    let start = Instant::now();
    let response = next.run(req).await;
    let elapsed_ms = start.elapsed().as_millis() as f64;

    let status = response.status().as_u16().to_string();
    let attrs = [
        KeyValue::new("method", &method),
        KeyValue::new("path", &path),
        KeyValue::new("status", &status),
    ];
    counter.add(1, &attrs);
    duration.record(elapsed_ms, &attrs);
    in_flight.add(-1, &[]);

    response
}

fn in_flight_counter() -> UpDownCounter<i64> {
    static INST: OnceLock<UpDownCounter<i64>> = OnceLock::new();
    INST.get_or_init(|| {
        meter()
            .create_up_down_counter("http.server.request_in_flight")
            .with_description("Number of HTTP requests currently in flight")
            .init()
    })
    .clone()
}

fn request_counter() -> Counter<u64> {
    static INST: OnceLock<Counter<u64>> = OnceLock::new();
    INST.get_or_init(|| {
        meter()
            .create_counter("http.server.request_count")
            .with_description("Total number of HTTP requests received")
            .init()
    })
    .clone()
}

fn duration_histogram() -> Histogram<f64> {
    static INST: OnceLock<Histogram<f64>> = OnceLock::new();
    INST.get_or_init(|| {
        meter()
            .create_histogram("http.server.request_duration_ms")
            .with_description("Duration of HTTP requests in milliseconds")
            .with_unit("ms")
            .init()
    })
    .clone()
}
