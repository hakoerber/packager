use axum::{
    error_handling::HandleErrorLayer,
    http::{header::HeaderMap, StatusCode},
    middleware,
    routing::{get, post},
    BoxError, Router,
};
use serde::de;
use uuid::Uuid;

use std::{fmt, time::Duration};
use tower::{timeout::TimeoutLayer, ServiceBuilder};

use crate::{AppState, Error, RequestError, TopLevelPage};

use super::auth;

pub mod html;

mod routes;
use routes::*;

#[tracing::instrument]
pub fn get_referer(headers: &HeaderMap) -> Result<&str, Error> {
    headers
        .get("referer")
        .ok_or(Error::Request(RequestError::RefererNotFound))?
        .to_str()
        .map_err(|error| {
            Error::Request(RequestError::RefererInvalid {
                message: error.to_string(),
            })
        })
}

pub fn uuid_or_empty<'de, D>(input: D) -> Result<Option<Uuid>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct NoneVisitor;

    impl<'vi> de::Visitor<'vi> for NoneVisitor {
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
                Error::Request(RequestError::NotFound {
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
                .merge(crate::components::trips::routes::router())
                .nest(
                    (&TopLevelPage::Inventory.path()).into(),
                    Router::new()
                        .route("/", get(inventory_inactive))
                        .route("/categories/:id/select", post(inventory_category_select))
                        .route("/category/", post(inventory_category_create))
                        .route("/category/:id/", get(inventory_active))
                        .route("/item/", post(inventory_item_create))
                        .route("/item/:id/", get(inventory_item))
                        .route("/item/:id/cancel", get(inventory_item_cancel))
                        .route("/item/:id/delete", get(inventory_item_delete))
                        .route("/item/:id/edit", post(inventory_item_edit))
                        .route("/item/name/validate", post(inventory_item_validate_name)),
                )
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
            Error::Request(RequestError::NotFound {
                message: "no route found".to_string(),
            })
        })
        .with_state(state)
}
