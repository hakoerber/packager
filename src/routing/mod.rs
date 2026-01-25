use axum::{
    error_handling::HandleErrorLayer,
    http::{header::HeaderMap, StatusCode},
    middleware,
    routing::get,
    BoxError, Router,
};

use serde::de;
use uuid::Uuid;

use std::{fmt, time::Duration};
use tower::{timeout::TimeoutLayer, ServiceBuilder};

use crate::{AppState, RunError, RequestError};

use super::auth;

pub mod html;

mod routes;
use routes::{debug, icon, root};

#[tracing::instrument]
pub fn get_referer(headers: &HeaderMap) -> Result<&str, RunError> {
    headers
        .get("referer")
        .ok_or(RunError::Request(RequestError::RefererNotFound))?
        .to_str()
        .map_err(|error| {
            RunError::Request(RequestError::RefererInvalid {
                message: error.to_string(),
            })
        })
}

pub fn uuid_or_empty<'de, D>(input: D) -> Result<Option<Uuid>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct NoneVisitor;

    impl de::Visitor<'_> for NoneVisitor {
        type Value = Option<Uuid>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "invalid input")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(Uuid::try_from(value).map_err(|e| {
                    E::custom(format!("UUID parsing failed: {e}"))
                })?))
            }
        }
    }

    input.deserialize_str(NoneVisitor)
}

#[tracing::instrument]
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/favicon.svg", get(icon))
        .route("/assets/luggage.svg", get(icon))
        .route(
            "/notfound",
            get(|| async {
                RunError::Request(RequestError::NotFound {
                    message: "hi".to_string(),
                })
            }),
        )
        .route(
            "/slow",
            get(|| async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                "Ok"
            }),
        )
        .route("/debug", get(debug))
        .merge(
            // these are routes that require authentication
            Router::new()
                .route("/", get(root))
                .merge(crate::domains::trips::router())
                .merge(crate::domains::inventory::router())
                .merge(crate::domains::products::router())
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth::authorize,
                )),
        )
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_: BoxError| async {
                    tracing::warn!("request timeout");
                    StatusCode::REQUEST_TIMEOUT
                }))
                .layer(TimeoutLayer::new(Duration::from_millis(500))),
        )
        // .propagate_x_request_id()
        .fallback(|| async {
            RunError::Request(RequestError::NotFound {
                message: "no route found".to_string(),
            })
        })
        .with_state(state)
}
