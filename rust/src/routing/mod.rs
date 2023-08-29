use axum::{
    error_handling::HandleErrorLayer,
    extract::State,
    http::header::HeaderMap,
    http::StatusCode,
    middleware,
    routing::{get, post},
    BoxError, Router,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions,
};

use std::{str::FromStr, time::Duration};
use tower::{timeout::TimeoutLayer, ServiceBuilder};

use crate::{AppState, Error, RequestError, TopLevelPage};

use super::auth;

mod html;
mod routes;
use routes::*;

//#[tracing::instrument]
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

//#[tracing::instrument]
async fn simple_handler(State(state): State<AppState>) -> &'static str {
    use tracing::Instrument;
    let pool = async {
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(
                SqliteConnectOptions::from_str("/tmp/tmp.SmE1WKBVMf")
                    .unwrap()
                    .log_statements(log::LevelFilter::Warn)
                    .log_slow_statements(
                        log::LevelFilter::Warn,
                        std::time::Duration::from_millis(100),
                    )
                    .pragma("foreign_keys", "1"),
            )
            .await
            .unwrap()
    }
    .instrument(tracing::warn_span!("init_pool"))
    .await;

    tracing::warn!("test event");

    async {
        sqlx::query("SELECT * FROM users")
            .execute(&pool)
            .await
            .unwrap()
    }
    .instrument(tracing::warn_span!("test_span"))
    .await;
    "ok"
}

//#[tracing::instrument]
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/favicon.svg", get(icon))
        .route("/assets/luggage.svg", get(icon))
        .route("/q", get(simple_handler))
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
