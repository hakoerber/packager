use axum::{
    error_handling::HandleErrorLayer,
    http::header::HeaderMap,
    http::StatusCode,
    middleware,
    routing::{get, post},
    BoxError, Router,
};
use serde::de;
use uuid::Uuid;

use std::{fmt, time::Duration};
use tower::{timeout::TimeoutLayer, ServiceBuilder};

use crate::{components::route, AppState, Error, RequestError, TopLevelPage};

use super::auth;

mod html;
mod routes;
use routes::*;

#[tracing::instrument]
fn get_referer(headers: &HeaderMap) -> Result<&str, Error> {
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

fn uuid_or_empty<'de, D>(input: D) -> Result<Option<Uuid>, D::Error>
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
            // thse are routes that require authentication
            Router::new()
                .route("/", get(root))
                .nest(
                    (&TopLevelPage::Trips.path()).into(),
                    Router::new()
                        .route("/", get(trips).post(trip_create))
                        .route("/types/", get(trips_types).post(trip_type_create))
                        .route("/types/:id/edit/name/submit", post(trips_types_edit_name))
                        .route("/:id/", get(trip))
                        .route("/:id/comment/submit", post(trip_comment_set))
                        .route("/:id/categories/:id/select", post(trip_category_select))
                        .route("/:id/packagelist/", get(trip_packagelist))
                        .route(
                            "/:id/packagelist/item/:id/pack",
                            post(trip_item_packagelist_set_pack_htmx),
                        )
                        .route(
                            "/:id/packagelist/item/:id/unpack",
                            post(trip_item_packagelist_set_unpack_htmx),
                        )
                        .route(
                            "/:id/packagelist/item/:id/ready",
                            post(trip_item_packagelist_set_ready_htmx),
                        )
                        .route(
                            "/:id/packagelist/item/:id/unready",
                            post(trip_item_packagelist_set_unready_htmx),
                        )
                        .route("/:id/state/:id", post(trip_state_set))
                        .route("/:id/total_weight", get(trip_total_weight_htmx))
                        .route("/:id/type/:id/add", get(trip_type_add))
                        .route("/:id/type/:id/remove", get(trip_type_remove))
                        .route("/:id/edit/:attribute/submit", post(trip_edit_attribute))
                        .route(
                            "/:id/items/:id/pick",
                            get(trip_item_set_pick).post(trip_item_set_pick_htmx),
                        )
                        .route(
                            "/:id/items/:id/unpick",
                            get(trip_item_set_unpick).post(trip_item_set_unpick_htmx),
                        )
                        .route(
                            "/:id/items/:id/pack",
                            get(trip_item_set_pack).post(trip_item_set_pack_htmx),
                        )
                        .route(
                            "/:id/items/:id/unpack",
                            get(trip_item_set_unpack).post(trip_item_set_unpack_htmx),
                        )
                        .route(
                            "/:id/items/:id/ready",
                            get(trip_item_set_ready).post(trip_item_set_ready_htmx),
                        )
                        .route(
                            "/:id/items/:id/unready",
                            get(trip_item_set_unready).post(trip_item_set_unready_htmx),
                        )
                        .route(
                            "/:id/todo/:id/done",
                            get(trip_todo_done).post(trip_todo_done_htmx),
                        )
                        .route(
                            "/:id/todo/:id/undone",
                            get(trip_todo_undone).post(trip_todo_undone_htmx),
                        )
                        .route("/:id/todo/:id/edit", post(trip_todo_edit))
                        .route("/:id/todo/:id/edit/save", post(trip_todo_edit_save))
                        .route("/:id/todo/:id/edit/cancel", post(trip_todo_edit_cancel))
                        .route(
                            &<crate::models::trips::todos::Todo as route::Create>::with_prefix(
                                "/:id/todo",
                            ),
                            post(<crate::models::trips::todos::Todo as route::Create>::create),
                        )
                        .route(
                            &<crate::models::trips::todos::Todo as route::Delete>::with_prefix(
                                "/:id/todo",
                            ),
                            post(<crate::models::trips::todos::Todo as route::Delete>::delete),
                        ),
                )
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
