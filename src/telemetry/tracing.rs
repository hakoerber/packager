use std::fmt;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::time::Duration;

use axum::Router;

use http::Request;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Span;

use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    fmt::{format::Format, Layer},
    layer::SubscriberExt,
    prelude::*,
    registry::Registry,
};

use tracing::Instrument;

use uuid::Uuid;

pub(crate) enum OpenTelemetryConfig {
    Enabled,
    Disabled,
}

pub(crate) enum TokioConsoleConfig {
    Enabled,
    Disabled,
}

fn get_stdout_layer<
    T: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
>() -> impl tracing_subscriber::Layer<T> {
    // default is the Full format, there is no way to specify this, but it can be
    // overridden via builder methods
    let stdout_format = Format::default()
        .pretty()
        .with_ansi(true)
        .with_target(true)
        .with_level(true)
        .with_file(false);

    let stdout_filter = Targets::new()
        .with_default(LevelFilter::WARN)
        .with_targets(vec![
            (env!("CARGO_PKG_NAME"), LevelFilter::DEBUG),
            ("sqlx", LevelFilter::DEBUG),
            ("runtime", LevelFilter::OFF),
            ("tokio", LevelFilter::OFF),
        ]);

    let stdout_layer = Layer::default()
        .event_format(stdout_format)
        .with_writer(io::stdout)
        .with_filter(stdout_filter);

    stdout_layer.boxed()
}

#[cfg(feature = "otel")]
fn get_opentelemetry_layer<
    T: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
>(
    config: &OpenTelemetryConfig,
    shutdown_functions: &mut Vec<ShutdownFunction>,
) -> Option<impl tracing_subscriber::Layer<T>> {
    match config {
        OpenTelemetryConfig::Enabled => {
            use opentelemetry::{global, KeyValue};
            use opentelemetry_otlp::WithExportConfig as _;
            use opentelemetry_sdk::{
                trace::{RandomIdGenerator, Sampler},
                Resource,
            };

            use tonic::metadata::MetadataMap;

            let mut metadata = MetadataMap::with_capacity(3);
            metadata.insert("x-host", "localhost".parse().unwrap());

            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint("http://localhost:4317")
                        .with_timeout(Duration::from_secs(3))
                        .with_metadata(metadata),
                )
                .with_trace_config(
                    opentelemetry_sdk::trace::config()
                        .with_sampler(Sampler::AlwaysOn)
                        .with_id_generator(RandomIdGenerator::default())
                        .with_max_events_per_span(64)
                        .with_max_attributes_per_span(16)
                        .with_max_events_per_span(16)
                        .with_resource(Resource::new(vec![KeyValue::new(
                            "service.name",
                            "packager",
                        )])),
                )
                .install_batch(opentelemetry_sdk::runtime::Tokio)
                .unwrap();

            // let tracer = provider.tracer(env!("CARGO_PKG_NAME"));

            let opentelemetry_filter = {
                Targets::new()
                    .with_default(LevelFilter::DEBUG)
                    .with_targets(vec![
                        (env!("CARGO_PKG_NAME"), LevelFilter::DEBUG),
                        ("sqlx", LevelFilter::DEBUG),
                        ("runtime", LevelFilter::OFF),
                        ("tokio", LevelFilter::OFF),
                    ])
            };

            let opentelemetry_layer = tracing_opentelemetry::layer()
                .with_tracer(tracer)
                .with_filter(opentelemetry_filter);

            shutdown_functions.push(Box::new(|| {
                global::shutdown_tracer_provider();
                Ok(())
            }));

            Some(opentelemetry_layer)
        }
        OpenTelemetryConfig::Disabled => None,
    }
}

type ShutdownFunction = Box<dyn FnOnce() -> Result<(), Box<dyn std::error::Error>>>;

pub(crate) async fn init<Func, T>(
    #[cfg(feature = "otel")] opentelemetry_config: OpenTelemetryConfig,
    #[cfg(feature = "tokio-console")] tokio_console_config: TokioConsoleConfig,
    args: crate::cli::Args,
    f: Func,
) -> T
where
    Func: FnOnce(crate::cli::Args) -> Pin<Box<dyn Future<Output = T>>>,
    T: std::process::Termination,
{
    // mut is dependent on features (it's only required when opentelemetry is set), so
    // let's just disable the lint
    #[allow(unused_mut)]
    let mut shutdown_functions: Vec<ShutdownFunction> = vec![];

    #[cfg(feature = "tokio-console")]
    let console_layer = match tokio_console_config {
        TokioConsoleConfig::Enabled => Some(console_subscriber::Builder::default().spawn()),
        TokioConsoleConfig::Disabled => None,
    };

    let stdout_layer = get_stdout_layer();

    #[cfg(feature = "otel")]
    let opentelemetry_layer =
        get_opentelemetry_layer(&opentelemetry_config, &mut shutdown_functions);

    let registry = Registry::default();

    #[cfg(feature = "tokio-console")]
    let registry = registry.with(console_layer);

    #[cfg(feature = "otel")]
    let registry = registry.with(opentelemetry_layer);
    // just an example, you can actuall pass Options here for layers that might be
    // set/unset at runtime

    let registry = registry.with(stdout_layer).with(None::<Layer<_>>);

    tracing::subscriber::set_global_default(registry).unwrap();

    tracing_log::log_tracer::Builder::new().init().unwrap();

    let result = f(args)
        .instrument(tracing::debug_span!(target: env!("CARGO_PKG_NAME"), env!("CARGO_PKG_NAME")))
        .await;

    for shutdown_func in shutdown_functions {
        shutdown_func().unwrap();
    }
    result
}

struct Latency(Duration);

impl fmt::Display for Latency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_micros())
    }
}

pub(crate) fn init_request_tracing(router: Router) -> Router {
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(|_request: &Request<_>| {
                let request_id = Uuid::new_v4();
                tracing::debug_span!(
                    target: "packager::request",
                    "request",
                    %request_id,
                )
            })
            .on_request(|request: &Request<_>, _span: &Span| {
                let request_headers = request.headers();
                let http_version = request.version();
                tracing::debug!(
                    target: "packager::request",
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
                        target: "packager::request",
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
                                target: "packager::request",
                                %latency,
                                "request failed with error response {}",
                                code,
                            );
                        }
                        ServerErrorsFailureClass::Error(message) => {
                            tracing::error!(
                                target: "packager::request",
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
