use std::future::Future;

use axum::routing::get;
use axum::Router;

use axum_prometheus::{Handle, MakeDefaultHandle, PrometheusMetricLayerBuilder};

use crate::{Error, StartError};

pub fn prometheus_server(
    router: Router,
    addr: std::net::SocketAddr,
) -> (Router, impl Future<Output = Result<(), Error>>) {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix(env!("CARGO_PKG_NAME"))
        .with_metrics_from_fn(|| Handle::make_default_handle())
        .build_pair();

    let app = Router::new().route("/metrics", get(|| async move { metric_handle.render() }));

    let task = async move {
        if let Err(e) = axum::Server::try_bind(&addr)
            .map_err(|e| {
                Error::Start(StartError::BindError {
                    message: e.to_string(),
                    addr,
                })
            })?
            .serve(app.into_make_service())
            .await
        {
            return Err(<hyper::Error as Into<Error>>::into(e));
        }
        Ok(())
    };

    (router.layer(prometheus_layer), task)
}
