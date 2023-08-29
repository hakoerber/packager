#![allow(unused_imports)]
use axum::{
    extract::{Path, Query, State},
    headers,
    headers::Header,
    http::{
        header,
        header::{HeaderMap, HeaderName, HeaderValue},
        StatusCode,
    },
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};

use maud::html;

use std::str::FromStr;

use serde_variant::to_variant_name;

use sqlx::{
    error::DatabaseError,
    query,
    sqlite::{SqliteConnectOptions, SqliteError, SqlitePoolOptions, SqliteRow},
    Pool, Row, Sqlite,
};

use maud::Markup;

use serde::Deserialize;

use futures::TryFutureExt;
use futures::TryStreamExt;
use uuid::{uuid, Uuid};

use std::net::SocketAddr;

mod components;
mod models;

#[derive(Clone)]
pub struct AppState {
    database_pool: Pool<Sqlite>,
    client_state: ClientState,
}

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    database_url: String,
    #[arg(long, default_value_t = 3000)]
    port: u16,
}

#[derive(Clone)]
pub struct ClientState {
    pub active_category_id: Option<Uuid>,
    pub edit_item: Option<Uuid>,
    pub trip_edit_attribute: Option<models::TripAttribute>,
    pub trip_type_edit: Option<Uuid>,
}

impl ClientState {
    pub fn new() -> Self {
        ClientState {
            active_category_id: None,
            edit_item: None,
            trip_edit_attribute: None,
            trip_type_edit: None,
        }
    }
}

impl Default for ClientState {
    fn default() -> Self {
        Self::new()
    }
}

enum HtmxEvents {
    TripItemEdited,
}

impl From<HtmxEvents> for HeaderValue {
    fn from(val: HtmxEvents) -> Self {
        HeaderValue::from_static(val.to_str())
    }
}

impl HtmxEvents {
    fn to_str(&self) -> &'static str {
        match self {
            Self::TripItemEdited => "TripItemEdited",
        }
    }
}

enum HtmxResponseHeaders {
    Trigger,
    PushUrl,
}

impl From<HtmxResponseHeaders> for HeaderName {
    fn from(val: HtmxResponseHeaders) -> Self {
        match val {
            HtmxResponseHeaders::Trigger => HeaderName::from_static("hx-trigger"),
            HtmxResponseHeaders::PushUrl => HeaderName::from_static("hx-push-url"),
        }
    }
}

enum HtmxRequestHeaders {
    HtmxRequest,
}

impl From<HtmxRequestHeaders> for HeaderName {
    fn from(val: HtmxRequestHeaders) -> Self {
        match val {
            HtmxRequestHeaders::HtmxRequest => HeaderName::from_static("hx-request"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();

    let database_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            SqliteConnectOptions::from_str(&args.database_url)?.pragma("foreign_keys", "1"),
        )
        .await
        .unwrap();

    sqlx::migrate!().run(&database_pool).await?;

    let state = AppState {
        database_pool,
        client_state: ClientState::new(),
    };

    let icon_handler = || async {
        (
            [(header::CONTENT_TYPE, "image/svg+xml")],
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/luggage.svg")),
        )
    };

    // build our application with a route
    let app = Router::new()
        .route("/favicon.svg", get(icon_handler))
        .route("/assets/luggage.svg", get(icon_handler))
        .route("/", get(root))
        .nest(
            "/trips/",
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
                ),
        )
        .nest(
            "/inventory/",
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
        .fallback(|| async { (StatusCode::NOT_FOUND, "not found") })
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn root() -> (StatusCode, Markup) {
    (
        StatusCode::OK,
        components::Root::build(
            &components::home::Home::build(),
            &components::TopLevelPage::None,
        ),
    )
}

#[derive(Deserialize, Default)]
struct InventoryQuery {
    edit_item: Option<Uuid>,
}

async fn inventory_active(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(inventory_query): Query<InventoryQuery>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    state.client_state.edit_item = inventory_query.edit_item;
    state.client_state.active_category_id = Some(id);

    let inventory = models::Inventory::load(&state.database_pool)
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            )
        })?;

    let active_category: Option<&models::Category> = state
        .client_state
        .active_category_id
        .map(|id| {
            inventory
                .categories
                .iter()
                .find(|category| category.id == id)
                .ok_or((
                    StatusCode::NOT_FOUND,
                    components::ErrorPage::build(&format!(
                        "a category with id {id} does not exist"
                    )),
                ))
        })
        .transpose()?;

    Ok((
        StatusCode::OK,
        components::Root::build(
            &components::inventory::Inventory::build(
                active_category,
                &inventory.categories,
                state.client_state.edit_item,
            ),
            &components::TopLevelPage::Inventory,
        ),
    ))
}

async fn inventory_inactive(
    State(mut state): State<AppState>,
    Query(inventory_query): Query<InventoryQuery>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    state.client_state.edit_item = inventory_query.edit_item;
    state.client_state.active_category_id = None;

    let inventory = models::Inventory::load(&state.database_pool)
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            )
        })?;

    Ok((
        StatusCode::OK,
        components::Root::build(
            &components::inventory::Inventory::build(
                None,
                &inventory.categories,
                state.client_state.edit_item,
            ),
            &components::TopLevelPage::Inventory,
        ),
    ))
}

// async fn inventory(
//     mut state: AppState,
//     active_id: Option<Uuid>,
// ) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
//     state.client_state.active_category_id = active_id;

//     Ok((
//         StatusCode::OK,
//         components::Root::build(
//             &components::inventory::Inventory::build(state.client_state, categories).map_err(|e| match e {
//                 Error::NotFound { description } => {
//                     (StatusCode::NOT_FOUND, components::ErrorPage::build(&description))
//                 }
//                 _ => (
//                     StatusCode::INTERNAL_SERVER_ERROR,
//                     components::ErrorPage::build(&e.to_string()),
//                 ),
//             })?,
//             &TopLevelPage::Inventory,
//         ),
//     ))
// }

#[derive(Deserialize)]
struct NewItem {
    #[serde(rename = "new-item-name")]
    name: String,
    #[serde(rename = "new-item-weight")]
    weight: u32,
    // damn i just love how serde is integrated everywhere, just add a feature to the uuid in
    // cargo.toml and go
    #[serde(rename = "new-item-category-id")]
    category_id: Uuid,
}

#[derive(Deserialize)]
struct NewItemName {
    #[serde(rename = "new-item-name")]
    name: String,
}

async fn inventory_item_validate_name(
    State(state): State<AppState>,
    Form(new_item): Form<NewItemName>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    let exists = models::InventoryItem::name_exists(&state.database_pool, &new_item.name)
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            )
        })
        .await?;

    Ok((
        StatusCode::OK,
        components::inventory::InventoryNewItemFormName::build(Some(&new_item.name), exists),
    ))
}

async fn inventory_item_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(new_item): Form<NewItem>,
) -> Result<impl IntoResponse, (StatusCode, Markup)> {
    if new_item.name.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            components::ErrorPage::build("name cannot be empty"),
        ));
    }

    let new_id = models::InventoryItem::save(
        &state.database_pool,
        &new_item.name,
        new_item.category_id,
        new_item.weight,
    )
    .map_err(|error| match error {
        models::Error::Constraint { description } => (
            StatusCode::BAD_REQUEST,
            components::ErrorPage::build(&description),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            components::ErrorPage::build(&error.to_string()),
        ),
    })
    .await?;

    if is_htmx(&headers) {
        let inventory = models::Inventory::load(&state.database_pool)
            .await
            .map_err(|error| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    components::ErrorPage::build(&error.to_string()),
                )
            })?;

        // it's impossible to NOT find the item here, as we literally just added
        // it. but good error handling never hurts
        let active_category: Option<&models::Category> = Some(
            inventory
                .categories
                .iter()
                .find(|category| category.id == new_item.category_id)
                .ok_or((
                    StatusCode::NOT_FOUND,
                    components::ErrorPage::build(&format!(
                        "a category with id {new_id} was inserted but does not exist, this is a bug"
                    )),
                ))?,
        );

        Ok((
            StatusCode::OK,
            components::inventory::Inventory::build(
                active_category,
                &inventory.categories,
                state.client_state.edit_item,
            ),
        )
            .into_response())
    } else {
        Ok(Redirect::to(&format!(
            "/inventory/category/{id}/",
            id = new_item.category_id
        ))
        .into_response())
    }
}

async fn inventory_item_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Redirect, (StatusCode, Markup)> {
    let deleted = models::InventoryItem::delete(&state.database_pool, id)
        .map_err(|error| match error {
            models::Error::Constraint { ref description } => (
                StatusCode::NOT_IMPLEMENTED,
                components::ErrorPage::build(description),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            ),
        })
        .await?;

    if !deleted {
        Err((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!("item with id {id} not found")),
        ))
    } else {
        Ok(Redirect::to(
            headers
                .get("referer")
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    components::ErrorPage::build("no referer header found"),
                ))?
                .to_str()
                .map_err(|error| {
                    (
                        StatusCode::BAD_REQUEST,
                        components::ErrorPage::build(&format!(
                            "referer could not be converted: {}",
                            error
                        )),
                    )
                })?,
        ))
    }
}

#[derive(Deserialize)]
struct EditItem {
    #[serde(rename = "edit-item-name")]
    name: String,
    #[serde(rename = "edit-item-weight")]
    weight: u32,
}

async fn inventory_item_edit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(edit_item): Form<EditItem>,
) -> Result<Redirect, (StatusCode, Markup)> {
    if edit_item.name.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            components::ErrorPage::build("name cannot be empty"),
        ));
    }

    let id = models::Item::update(
        &state.database_pool,
        id,
        &edit_item.name,
        i64::try_from(edit_item.weight).map_err(|e| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                components::ErrorPage::build(&e.to_string()),
            )
        })?,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            components::ErrorPage::build(&e.to_string()),
        )
    })?
    .ok_or((
        StatusCode::NOT_FOUND,
        components::ErrorPage::build(&format!("item with id {id} not found", id = id)),
    ))?;

    Ok(Redirect::to(&format!("/inventory/category/{id}/", id = id)))
}

async fn inventory_item_cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Redirect, (StatusCode, String)> {
    let id = models::Item::find(&state.database_pool, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("item with id {id} not found", id = id),
        ))?;

    Ok(Redirect::to(&format!(
        "/inventory/category/{id}/",
        id = id.category_id
    )))
}

#[derive(Deserialize)]
struct NewTrip {
    #[serde(rename = "new-trip-name")]
    name: String,
    #[serde(rename = "new-trip-start-date")]
    date_start: time::Date,
    #[serde(rename = "new-trip-end-date")]
    date_end: time::Date,
}

async fn trip_create(
    State(state): State<AppState>,
    Form(new_trip): Form<NewTrip>,
) -> Result<Redirect, (StatusCode, Markup)> {
    if new_trip.name.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            components::ErrorPage::build("name cannot be empty"),
        ));
    }

    let new_id = models::Trip::save(
        &state.database_pool,
        &new_trip.name,
        new_trip.date_start,
        new_trip.date_end,
    )
    .map_err(|error| match error {
        models::Error::TimeParse { description } => (
            StatusCode::BAD_REQUEST,
            components::ErrorPage::build(&description),
        ),
        _ => (
            StatusCode::BAD_REQUEST,
            components::ErrorPage::build(&error.to_string()),
        ),
    })
    .await?;

    Ok(Redirect::to(&format!("/trips/{new_id}/")))
}

async fn trips(
    State(state): State<AppState>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    let trips = models::Trip::all(&state.database_pool)
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            )
        })
        .await?;

    Ok((
        StatusCode::OK,
        components::Root::build(
            &components::trip::TripManager::build(trips),
            &components::TopLevelPage::Trips,
        ),
    ))
}

#[derive(Debug, Deserialize)]
struct TripQuery {
    edit: Option<models::TripAttribute>,
    category: Option<Uuid>,
}

async fn trip(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(trip_query): Query<TripQuery>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    state.client_state.trip_edit_attribute = trip_query.edit;
    state.client_state.active_category_id = trip_query.category;

    let mut trip: models::Trip = models::Trip::find(&state.database_pool, id)
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            )
        })
        .await?
        .ok_or((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!("trip with id {} not found", id)),
        ))?;

    trip.load_trips_types(&state.database_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&e.to_string()),
            )
        })?;

    trip.sync_trip_items_with_inventory(&state.database_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&e.to_string()),
            )
        })?;

    trip.load_categories(&state.database_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&e.to_string()),
            )
        })?;

    let active_category: Option<&models::TripCategory> = state
        .client_state
        .active_category_id
        .map(|id| {
            trip.categories()
                .iter()
                .find(|category| category.category.id == id)
                .ok_or((
                    StatusCode::NOT_FOUND,
                    components::ErrorPage::build(&format!(
                        "an active category with id {id} does not exist"
                    )),
                ))
        })
        .transpose()?;

    Ok((
        StatusCode::OK,
        components::Root::build(
            &components::trip::Trip::build(
                &trip,
                state.client_state.trip_edit_attribute,
                active_category,
            ),
            &components::TopLevelPage::Trips,
        ),
    ))
}

async fn trip_type_remove(
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, (StatusCode, Markup)> {
    let found = models::Trip::trip_type_remove(&state.database_pool, trip_id, type_id)
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            )
        })
        .await?;

    if !found {
        Err((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!(
                "type {type_id} is not active for trip {trip_id}"
            )),
        ))
    } else {
        Ok(Redirect::to(&format!("/trips/{trip_id}/")))
    }
}

async fn trip_type_add(
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, (StatusCode, Markup)> {
    models::Trip::trip_type_add(&state.database_pool, trip_id, type_id)
        .map_err(|error| match error {
            models::Error::ReferenceNotFound { description } => (
                StatusCode::BAD_REQUEST,
                components::ErrorPage::build(&description),
            ),
            models::Error::Duplicate { description } => (
                StatusCode::BAD_REQUEST,
                components::ErrorPage::build(&description),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            ),
        })
        .await?;

    Ok(Redirect::to(&format!("/trips/{trip_id}/")))
}

#[derive(Deserialize)]
struct CommentUpdate {
    #[serde(rename = "new-comment")]
    new_comment: String,
}

async fn trip_comment_set(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
    Form(comment_update): Form<CommentUpdate>,
) -> Result<Redirect, (StatusCode, Markup)> {
    let found =
        models::Trip::set_comment(&state.database_pool, trip_id, &comment_update.new_comment)
            .map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    components::ErrorPage::build(&error.to_string()),
                )
            })
            .await?;

    if !found {
        Err((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!("trip with id {id} not found", id = trip_id)),
        ))
    } else {
        Ok(Redirect::to(&format!("/trips/{id}/", id = trip_id)))
    }
}

#[derive(Deserialize)]
struct TripUpdate {
    #[serde(rename = "new-value")]
    new_value: String,
}

async fn trip_edit_attribute(
    State(state): State<AppState>,
    Path((trip_id, attribute)): Path<(Uuid, models::TripAttribute)>,
    Form(trip_update): Form<TripUpdate>,
) -> Result<Redirect, (StatusCode, Markup)> {
    if attribute == models::TripAttribute::Name {
        if trip_update.new_value.is_empty() {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                components::ErrorPage::build("name cannot be empty"),
            ));
        }
    }
    models::Trip::set_attribute(
        &state.database_pool,
        trip_id,
        attribute,
        &trip_update.new_value,
    )
    .map_err(|error| match error {
        models::Error::NotFound { description } => (
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&description),
        ),
        _ => (
            StatusCode::BAD_REQUEST,
            components::ErrorPage::build(&error.to_string()),
        ),
    })
    .await?;

    Ok(Redirect::to(&format!("/trips/{trip_id}/")))
}

async fn trip_item_set_state(
    state: &AppState,
    trip_id: Uuid,
    item_id: Uuid,
    key: models::TripItemStateKey,
    value: bool,
) -> Result<(), (StatusCode, Markup)> {
    models::TripItem::set_state(&state.database_pool, trip_id, item_id, key, value)
        .map_err(|error| match error {
            models::Error::NotFound { description } => (
                StatusCode::NOT_FOUND,
                components::ErrorPage::build(&description),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            ),
        })
        .await?;

    Ok(())
}

async fn trip_row(
    state: &AppState,
    trip_id: Uuid,
    item_id: Uuid,
) -> Result<Markup, (StatusCode, Markup)> {
    let item = models::TripItem::find(&state.database_pool, trip_id, item_id)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                components::ErrorPage::build(&error.to_string()),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                components::ErrorPage::build(&format!(
                    "item with id {} not found for trip {}",
                    item_id, trip_id
                )),
            )
        })?;

    let item_row = components::trip::TripItemListRow::build(
        trip_id,
        &item,
        models::Item::get_category_max_weight(&state.database_pool, item.item.category_id)
            .await
            .map_err(|error| {
                (
                    StatusCode::BAD_REQUEST,
                    components::ErrorPage::build(&error.to_string()),
                )
            })?,
    );

    let category = models::TripCategory::find(&state.database_pool, trip_id, item.item.category_id)
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                components::ErrorPage::build(&error.to_string()),
            )
        })
        .await?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                components::ErrorPage::build(&format!(
                    "category with id {} not found",
                    item.item.category_id
                )),
            )
        })?;

    // TODO biggest_category_weight?
    let category_row =
        components::trip::TripCategoryListRow::build(trip_id, &category, true, 0, true);

    Ok(html!((item_row)(category_row)))
}

async fn trip_item_set_pick(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, Markup)> {
    Ok(trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::TripItemStateKey::Pick,
        true,
    )
    .await?)
    .map(|_| -> Result<Redirect, (StatusCode, Markup)> {
        Ok(Redirect::to(
            headers
                .get("referer")
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    components::ErrorPage::build("no referer header found"),
                ))?
                .to_str()
                .map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        components::ErrorPage::build(&format!(
                            "referer could not be converted: {}",
                            e
                        )),
                    )
                })?,
        ))
    })?
}

async fn trip_item_set_pick_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, HeaderMap, Markup), (StatusCode, Markup)> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::TripItemStateKey::Pick,
        true,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((
        StatusCode::OK,
        headers,
        trip_row(&state, trip_id, item_id).await?,
    ))
}

async fn trip_item_set_unpick(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, Markup)> {
    Ok(trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::TripItemStateKey::Pick,
        false,
    )
    .await?)
    .map(|_| -> Result<Redirect, (StatusCode, Markup)> {
        Ok(Redirect::to(
            headers
                .get("referer")
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    components::ErrorPage::build("no referer header found"),
                ))?
                .to_str()
                .map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        components::ErrorPage::build(&format!(
                            "referer could not be converted: {}",
                            e
                        )),
                    )
                })?,
        ))
    })?
}

async fn trip_item_set_unpick_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, HeaderMap, Markup), (StatusCode, Markup)> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::TripItemStateKey::Pick,
        false,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((
        StatusCode::OK,
        headers,
        trip_row(&state, trip_id, item_id).await?,
    ))
}

async fn trip_item_set_pack(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, Markup)> {
    Ok(trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::TripItemStateKey::Pack,
        true,
    )
    .await?)
    .map(|_| -> Result<Redirect, (StatusCode, Markup)> {
        Ok(Redirect::to(
            headers
                .get("referer")
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    components::ErrorPage::build("no referer header found"),
                ))?
                .to_str()
                .map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        components::ErrorPage::build(&format!(
                            "referer could not be converted: {}",
                            e
                        )),
                    )
                })?,
        ))
    })?
}

async fn trip_item_set_pack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, HeaderMap, Markup), (StatusCode, Markup)> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::TripItemStateKey::Pack,
        true,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((
        StatusCode::OK,
        headers,
        trip_row(&state, trip_id, item_id).await?,
    ))
}

async fn trip_item_set_unpack(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, Markup)> {
    Ok(trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::TripItemStateKey::Pack,
        false,
    )
    .await?)
    .map(|_| -> Result<Redirect, (StatusCode, Markup)> {
        Ok(Redirect::to(
            headers
                .get("referer")
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    components::ErrorPage::build("no referer header found"),
                ))?
                .to_str()
                .map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        components::ErrorPage::build(&format!(
                            "referer could not be converted: {}",
                            e
                        )),
                    )
                })?,
        ))
    })?
}

async fn trip_item_set_unpack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, HeaderMap, Markup), (StatusCode, Markup)> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::TripItemStateKey::Pack,
        false,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((
        StatusCode::OK,
        headers,
        trip_row(&state, trip_id, item_id).await?,
    ))
}

async fn trip_total_weight_htmx(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    let total_weight = models::Trip::find_total_picked_weight(&state.database_pool, trip_id)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                components::ErrorPage::build(&error.to_string()),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!("trip with id {trip_id} not found")),
        ))?;
    Ok((
        StatusCode::OK,
        components::trip::TripInfoTotalWeightRow::build(trip_id, total_weight),
    ))
}

#[derive(Deserialize)]
struct NewCategory {
    #[serde(rename = "new-category-name")]
    name: String,
}

async fn inventory_category_create(
    State(state): State<AppState>,
    Form(new_category): Form<NewCategory>,
) -> Result<Redirect, (StatusCode, Markup)> {
    if new_category.name.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            components::ErrorPage::build("name cannot be empty"),
        ));
    }

    let _new_id = models::Category::save(&state.database_pool, &new_category.name)
        .map_err(|error| match error {
            models::Error::Duplicate { description } => (
                StatusCode::BAD_REQUEST,
                components::ErrorPage::build(&description),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            ),
        })
        .await?;

    Ok(Redirect::to("/inventory/"))
}

async fn trip_state_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((trip_id, new_state)): Path<(Uuid, models::TripState)>,
) -> Result<impl IntoResponse, (StatusCode, Markup)> {
    let exists = models::Trip::set_state(&state.database_pool, trip_id, &new_state)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                components::ErrorPage::build(&e.to_string()),
            )
        })
        .await?;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!("trip with id {id} not found", id = trip_id)),
        ));
    }

    if is_htmx(&headers) {
        Ok((
            StatusCode::OK,
            components::trip::TripInfoStateRow::build(&new_state),
        )
            .into_response())
    } else {
        Ok(Redirect::to(&format!("/trips/{id}/", id = trip_id)).into_response())
    }
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get::<HeaderName>(HtmxRequestHeaders::HtmxRequest.into())
        .map(|value| value == "true")
        .unwrap_or(false)
}

#[derive(Debug, Deserialize)]
struct TripTypeQuery {
    edit: Option<Uuid>,
}

async fn trips_types(
    State(mut state): State<AppState>,
    Query(trip_type_query): Query<TripTypeQuery>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    state.client_state.trip_type_edit = trip_type_query.edit;

    let trip_types: Vec<models::TripsType> = models::TripsType::all(&state.database_pool)
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            )
        })
        .await?;

    Ok((
        StatusCode::OK,
        components::Root::build(
            &components::trip::types::TypeList::build(&state.client_state, trip_types),
            &components::TopLevelPage::Trips,
        ),
    ))
}

#[derive(Deserialize)]
struct NewTripType {
    #[serde(rename = "new-trip-type-name")]
    name: String,
}

async fn trip_type_create(
    State(state): State<AppState>,
    Form(new_trip_type): Form<NewTripType>,
) -> Result<Redirect, (StatusCode, Markup)> {
    if new_trip_type.name.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            components::ErrorPage::build("name cannot be empty"),
        ));
    }

    let _new_id = models::TripsType::save(&state.database_pool, &new_trip_type.name)
        .map_err(|error| match error {
            models::Error::Duplicate { description } => (
                StatusCode::BAD_REQUEST,
                components::ErrorPage::build(&description),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            ),
        })
        .await?;

    Ok(Redirect::to("/trips/types/"))
}

#[derive(Deserialize)]
struct TripTypeUpdate {
    #[serde(rename = "new-value")]
    new_value: String,
}

async fn trips_types_edit_name(
    State(state): State<AppState>,
    Path(trip_type_id): Path<Uuid>,
    Form(trip_update): Form<TripTypeUpdate>,
) -> Result<Redirect, (StatusCode, Markup)> {
    if trip_update.new_value.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            components::ErrorPage::build("name cannot be empty"),
        ));
    }

    let exists =
        models::TripsType::set_name(&state.database_pool, trip_type_id, &trip_update.new_value)
            .map_err(|error| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    components::ErrorPage::build(&error.to_string()),
                )
            })
            .await?;

    if !exists {
        Err((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!(
                "tript type with id {id} not found",
                id = trip_type_id
            )),
        ))
    } else {
        Ok(Redirect::to("/trips/types/"))
    }
}

async fn inventory_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    let item = models::InventoryItem::find(&state.database_pool, id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&e.to_string()),
            )
        })
        .await?
        .ok_or((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!("inventory item with id {id} not found")),
        ))?;

    Ok((
        StatusCode::OK,
        components::Root::build(
            &components::inventory::InventoryItem::build(&state.client_state, &item),
            &components::TopLevelPage::Inventory,
        ),
    ))
}

async fn trip_category_select(
    State(state): State<AppState>,
    Path((trip_id, category_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, HeaderMap, Markup), (StatusCode, Markup)> {
    let mut trip = models::Trip::find(&state.database_pool, trip_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&e.to_string()),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!("trip with id {trip_id} not found")),
        ))?;

    trip.load_categories(&state.database_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&e.to_string()),
            )
        })?;

    let active_category = trip
        .categories()
        .iter()
        .find(|c| c.category.id == category_id)
        .ok_or((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!("category with id {category_id} not found")),
        ))?;

    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::PushUrl.into(),
        format!("?={category_id}").parse().unwrap(),
    );

    Ok((
        StatusCode::OK,
        headers,
        components::trip::TripItems::build(Some(active_category), &trip),
    ))
}

async fn inventory_category_select(
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<(StatusCode, HeaderMap, Markup), (StatusCode, Markup)> {
    let inventory = models::Inventory::load(&state.database_pool)
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            )
        })?;

    let active_category: Option<&models::Category> = Some(
        inventory
            .categories
            .iter()
            .find(|category| category.id == category_id)
            .ok_or((
                StatusCode::NOT_FOUND,
                components::ErrorPage::build(&format!(
                    "a category with id {category_id} not found"
                )),
            ))?,
    );

    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::PushUrl.into(),
        format!("/inventory/category/{category_id}/")
            .parse()
            .unwrap(),
    );

    Ok((
        StatusCode::OK,
        headers,
        components::inventory::Inventory::build(
            active_category,
            &inventory.categories,
            state.client_state.edit_item,
        ),
    ))
}

async fn trip_packagelist(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    let mut trip = models::Trip::find(&state.database_pool, trip_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&e.to_string()),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!("trip with id {trip_id} not found")),
        ))?;

    trip.load_categories(&state.database_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&e.to_string()),
            )
        })?;

    Ok((
        StatusCode::OK,
        components::Root::build(
            &components::trip::packagelist::TripPackageList::build(&trip),
            &components::TopLevelPage::Trips,
        ),
    ))
}

async fn trip_item_packagelist_set_pack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::TripItemStateKey::Pack,
        true,
    )
    .await?;

    let item = models::TripItem::find(&state.database_pool, trip_id, item_id)
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!("an item with id {item_id} does not exist")),
        ))?;

    Ok((
        StatusCode::OK,
        components::trip::packagelist::TripPackageListRow::build(trip_id, &item),
    ))
}

async fn trip_item_packagelist_set_unpack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::TripItemStateKey::Pack,
        false,
    )
    .await?;

    // note that this cannot fail due to a missing item, as trip_item_set_state would already
    // return 404. but error handling cannot hurt ;)
    let item = models::TripItem::find(&state.database_pool, trip_id, item_id)
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                components::ErrorPage::build(&error.to_string()),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            components::ErrorPage::build(&format!("an item with id {item_id} does not exist")),
        ))?;

    Ok((
        StatusCode::OK,
        components::trip::packagelist::TripPackageListRow::build(trip_id, &item),
    ))
}
