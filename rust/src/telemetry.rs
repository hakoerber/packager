use std::fmt;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::time::Duration;

use axum::Router;
use http::Request;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{Span, Subscriber};

use tracing_log::LogTracer;
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    fmt::{
        format::{Format, Writer},
        FmtContext, FormatEvent, FormatFields, Layer,
    },
    layer::SubscriberExt,
    prelude::*,
    registry::{LookupSpan, Registry},
};
use uuid::Uuid;

use opentelemetry::{global, runtime::Tokio};
use tracing::Event;

pub enum OpenTelemetryConfig {
    Enabled,
    Disabled,
}

pub enum TokioConsoleConfig {
    Enabled,
    Disabled,
}

struct DebugFormat;

impl<S, N> FormatEvent<S, N> for DebugFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let metadata = event.metadata();

        write!(
            &mut writer,
            "{}\t{}:\t",
            metadata.level(),
            metadata.target()
        )?;

        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                write!(writer, "span: {}\t", span.metadata().name())?;
            }
        } else {
            write!(writer, "NO SPAN\t")?;
        };

        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

pub async fn init_tracing<Func, T>(
    opentelemetry_config: OpenTelemetryConfig,
    tokio_console_config: TokioConsoleConfig,
    args: crate::cmd::Args,
    f: Func,
) -> T
where
    Func: FnOnce(crate::cmd::Args) -> Pin<Box<dyn Future<Output = T>>>,
    T: std::process::Termination,
{
    let mut shutdown_functions: Vec<Box<dyn FnOnce() -> Result<(), Box<dyn std::error::Error>>>> =
        vec![];

    // default is the Full format, there is no way to specify this, but it can be
    // overridden via builder methods
    let stdout_format = Format::default()
        .pretty()
        .with_ansi(true)
        .with_target(true)
        .with_level(true)
        .with_file(false);

    let stdout_format = DebugFormat;

    let stdout_layer = Layer::default()
        .event_format(stdout_format)
        .with_writer(io::stdout);

    let stdout_filter = Targets::new()
        .with_default(LevelFilter::WARN)
        .with_targets(vec![
            (env!("CARGO_PKG_NAME"), LevelFilter::DEBUG),
            ("request", LevelFilter::DEBUG),
            ("runtime", LevelFilter::OFF),
            ("sqlx", LevelFilter::TRACE),
        ]);

    let stdout_layer = stdout_layer.with_filter(stdout_filter);

    let console_layer = match tokio_console_config {
        TokioConsoleConfig::Enabled => Some(console_subscriber::Builder::default().spawn()),
        TokioConsoleConfig::Disabled => None,
    };

    let opentelemetry_layer = match opentelemetry_config {
        OpenTelemetryConfig::Enabled => {
            global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
            // Sets up the machinery needed to export data to Jaeger
            // There are other OTel crates that provide pipelines for the vendors
            // mentioned earlier.
            let tracer = opentelemetry_jaeger::new_agent_pipeline()
                .with_service_name(env!("CARGO_PKG_NAME"))
                .with_max_packet_size(20_000)
                .with_auto_split_batch(true)
                // .install_batch(Tokio)
                .install_simple()
                .unwrap();

            let opentelemetry_filter = {
                Targets::new()
                    .with_default(LevelFilter::DEBUG)
                    .with_targets(vec![
                        (env!("CARGO_PKG_NAME"), LevelFilter::DEBUG),
                        ("request", LevelFilter::DEBUG),
                        ("runtime", LevelFilter::OFF),
                        ("sqlx", LevelFilter::DEBUG),
                    ])
            };

            let opentelemetry_layer = tracing_opentelemetry::layer()
                .with_tracer(tracer)
                .with_filter(opentelemetry_filter);

            shutdown_functions.push(Box::new(|| {
                println!("shutting down otel");
                global::shutdown_tracer_provider();
                Ok(())
            }));

            println!("set up otel");

            Some(opentelemetry_layer)
        }
        OpenTelemetryConfig::Disabled => None,
    };

    let registry = Registry::default()
        .with(console_layer)
        .with(opentelemetry_layer)
        // just an example, you can actuall pass Options here for layers that might be
        // set/unset at runtime
        .with(Some(stdout_layer))
        .with(None::<Layer<_>>);

    tracing::subscriber::set_global_default(registry).unwrap();

    tracing::debug!("tracing setup finished");

    tracing_log::log_tracer::Builder::new().init().unwrap();

    let result = f(args).await;

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

pub fn init_request_tracing(router: Router) -> Router {
    return router;
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
