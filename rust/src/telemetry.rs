use std::fmt;
use std::io;
use std::time::Duration;

use axum::Router;
use http::Request;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{Level, Span};

use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    fmt::{format::Format, Layer},
    layer::SubscriberExt,
    prelude::*,
    registry::Registry,
};
use uuid::Uuid;

use opentelemetry::global;

pub fn otel_init(f: impl FnOnce() -> ()) {
    f()
}

pub fn init_tracing() {
    // default is the Full format, there is no way to specify this, but it can be
    // overridden via builder methods
    let stdout_format = Format::default()
        .pretty()
        .with_ansi(true)
        .with_target(true)
        .with_level(true)
        .with_file(false);

    let stdout_layer = Layer::default()
        .event_format(stdout_format)
        .with_writer(io::stdout);

    let stdout_filter = Targets::new()
        .with_default(LevelFilter::OFF)
        .with_targets(vec![
            (env!("CARGO_PKG_NAME"), Level::DEBUG),
            // this is for axum requests
            ("request", Level::DEBUG),
            // required for tokio-console as by the docs
            // ("tokio", Level::TRACE),
            // ("runtime", Level::TRACE),
        ]);

    let stdout_layer = stdout_layer.with_filter(stdout_filter);

    let console_layer = console_subscriber::Builder::default().spawn();

    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    // Sets up the machinery needed to export data to Jaeger
    // There are other OTel crates that provide pipelines for the vendors
    // mentioned earlier.
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(env!("CARGO_PKG_NAME"))
        .install_simple()
        .unwrap();

    let opentelemetry_filter = Targets::new()
        .with_default(LevelFilter::OFF)
        .with_targets(vec![
            (env!("CARGO_PKG_NAME"), Level::DEBUG),
            // this is for axum requests
            ("request", Level::DEBUG),
            // required for tokio-console as by the docs
            // ("tokio", Level::TRACE),
            // ("runtime", Level::TRACE),
        ]);

    let opentelemetry = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(opentelemetry_filter);

    let registry = Registry::default()
        .with(console_layer)
        .with(opentelemetry)
        // just an example, you can actuall pass Options here for layers that might be
        // set/unset at runtime
        .with(Some(stdout_layer))
        .with(None::<Layer<_>>);

    tracing::subscriber::set_global_default(registry).unwrap();
}

struct Latency(Duration);

impl fmt::Display for Latency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_micros())
    }
}

pub fn init_request_tracing(router: Router) -> Router {
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(|_request: &Request<_>| {
                let request_id = Uuid::new_v4();
                tracing::debug_span!(
                    target: "request",
                    "request",
                    %request_id,
                )
            })
            .on_request(|request: &Request<_>, _span: &Span| {
                let request_headers = request.headers();
                let http_version = request.version();
                tracing::debug!(
                    target: "request",
                    method = request.method().as_str(),
                    path = request.uri().path(),
                    ?http_version,
                    ?request_headers,
                    "request received",
                );
            })
            .on_response(
                |response: &axum::response::Response, latency: Duration, _span: &Span| {
                    let response_headers = response.headers();
                    let latency = Latency(latency);
                    tracing::debug!(
                        target: "request",
                        %latency,
                        status = response.status().as_str(),
                        ?response_headers,
                        "finished processing request",
                    );
                },
            )
            .on_failure(
                |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                    let latency = Latency(latency);
                    match error {
                        ServerErrorsFailureClass::StatusCode(code) => {
                            tracing::error!(
                                target: "request",
                                %latency,
                                "request failed with error response {}",
                                code,
                            );
                        }
                        ServerErrorsFailureClass::Error(message) => {
                            tracing::error!(
                                target: "request",
                                %latency,
                                "request failed: {}",
                                message,
                            );
                        }
                    }
                },
            ),
    )
}
