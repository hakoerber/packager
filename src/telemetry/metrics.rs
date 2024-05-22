use std::future::Future;

use axum::routing::get;
use axum::Router;

use axum_prometheus::{Handle, MakeDefaultHandle, PrometheusMetricLayerBuilder};
use tokio::net::TcpListener;

use crate::{Error, StartError};

/// Serves metrics on the specified `addr`.
///
/// You will get two outputs back: Another router, and a task that you have
/// to run to actually spawn the metrics server endpoint
pub(crate) fn prometheus_server(
    router: Router,
    addr: std::net::SocketAddr,
) -> (Router, impl Future<Output = Result<(), Error>>) {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix(env!("CARGO_PKG_NAME"))
        .with_metrics_from_fn(Handle::make_default_handle)
        .build_pair();

    let app = Router::new().route("/metrics", get(|| async move { metric_handle.render() }));

    let task = async move {
        axum::serve(
            TcpListener::bind(addr).await.map_err(|e| {
                Error::Start(StartError::BindError {
                    addr,
                    message: e.to_string(),
                })
            })?,
            app,
        )
        .await
        // Error = Infallible
        .unwrap();
        unreachable!()
    };

    (router.layer(prometheus_layer), task)
}
