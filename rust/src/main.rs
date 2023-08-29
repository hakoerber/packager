#![allow(unused_imports)]
use axum::{
    extract::{Path, Query, State},
    headers,
    headers::Header,
    http::{header, header::HeaderMap, StatusCode},
    response::{Html, Redirect},
    routing::{get, post},
    Form, Router,
};

use std::str::FromStr;

use serde_variant::to_variant_name;

use sqlx::{
    error::DatabaseError,
    query, query_as,
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

use crate::components::*;
use crate::models::*;

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
    port: Option<u16>,
}

#[derive(Clone)]
pub struct ClientState {
    pub active_category_id: Option<Uuid>,
    pub edit_item: Option<Uuid>,
    pub trip_edit_attribute: Option<TripAttribute>,
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

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let database_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            SqliteConnectOptions::from_str(
                &std::env::var("DATABASE_URL").expect("env DATABASE_URL not found"),
            )?
            .pragma("foreign_keys", "1"),
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
        .route("/trips/", get(trips))
        .route("/trips/types/", get(trips_types).post(trip_type_create))
        .route(
            "/trips/types/:id/edit/name/submit",
            post(trips_types_edit_name),
        )
        .route("/trip/", post(trip_create))
        .route("/trip/:id/", get(trip))
        .route("/trip/:id/comment/submit", post(trip_comment_set))
        .route("/trip/:id/state/:id", post(trip_state_set))
        .route("/trip/:id/type/:id/add", get(trip_type_add))
        .route("/trip/:id/type/:id/remove", get(trip_type_remove))
        .route(
            "/trip/:id/edit/:attribute/submit",
            post(trip_edit_attribute),
        )
        .route("/trip/:id/items/:id/pick", get(trip_item_set_pick))
        .route("/trip/:id/items/:id/unpick", get(trip_item_set_unpick))
        .route("/trip/:id/items/:id/pack", get(trip_item_set_pack))
        .route("/trip/:id/items/:id/unpack", get(trip_item_set_unpack))
        .route("/inventory/", get(inventory_inactive))
        .route("/inventory/category/", post(inventory_category_create))
        .route("/inventory/item/", post(inventory_item_create))
        .route(
            "/inventory/item/name/validate",
            post(inventory_item_validate_name),
        )
        .route("/inventory/category/:id/", get(inventory_active))
        .route("/inventory/item/:id/delete", get(inventory_item_delete))
        .route("/inventory/item/:id/edit", post(inventory_item_edit))
        .route("/inventory/item/:id/cancel", get(inventory_item_cancel))
        // .route(
        //     "/inventory/category/:id/items",
        //     post(htmx_inventory_category_items),
        // );
        .with_state(state);

    let args = Args::parse();

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port.unwrap_or(3000)));
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
        Root::build(&Home::build(), &TopLevelPage::None),
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
    inventory(state, Some(id)).await
}

async fn inventory_inactive(
    State(mut state): State<AppState>,
    Query(inventory_query): Query<InventoryQuery>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    state.client_state.edit_item = inventory_query.edit_item;
    inventory(state, None).await
}

async fn inventory(
    mut state: AppState,
    active_id: Option<Uuid>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    state.client_state.active_category_id = active_id;

    let mut categories = query_as!(
        DbCategoryRow,
        "SELECT id,name,description FROM inventory_items_categories"
    )
    .fetch(&state.database_pool)
    .map_ok(|row: DbCategoryRow| row.try_into())
    .try_collect::<Vec<Result<Category, models::Error>>>()
    .await
    // we have two error handling lines here. these are distinct errors
    // this one is the SQL error that may arise during the query
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&e.to_string()),
        )
    })?
    .into_iter()
    .collect::<Result<Vec<Category>, models::Error>>()
    // and this one is the model mapping error that may arise e.g. during
    // reading of the rows
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&e.to_string()),
        )
    })?;

    for category in &mut categories {
        category
            .populate_items(&state.database_pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorPage::build(&e.to_string()),
                )
            })?;
    }

    Ok((
        StatusCode::OK,
        Root::build(
            &Inventory::build(state.client_state, categories).map_err(|e| match e {
                Error::NotFound { description } => {
                    (StatusCode::NOT_FOUND, ErrorPage::build(&description))
                }
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorPage::build(&e.to_string()),
                ),
            })?,
            &TopLevelPage::Inventory,
        ),
    ))
}

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
    let results = query!(
        "SELECT id
        FROM inventory_items
        WHERE name = ?",
        new_item.name,
    )
    .fetch(&state.database_pool)
    .map_ok(|_| Ok(()))
    .try_collect::<Vec<Result<(), models::Error>>>()
    .await
    // we have two error handling lines here. these are distinct errors
    // this one is the SQL error that may arise during the query
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&e.to_string()),
        )
    })?
    .into_iter()
    .collect::<Result<Vec<()>, models::Error>>()
    // and this one is the model mapping error that may arise e.g. during
    // reading of the rows
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&e.to_string()),
        )
    })?;

    Ok((
        StatusCode::OK,
        InventoryNewItemFormName::build(Some(&new_item.name), !results.is_empty()),
    ))
}

async fn inventory_item_create(
    State(state): State<AppState>,
    Form(new_item): Form<NewItem>,
) -> Result<Redirect, (StatusCode, String)> {
    if new_item.name.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "name cannot be empty".to_string(),
        ));
    }

    let id = Uuid::new_v4();
    let id_param = id.to_string();
    let name = &new_item.name;
    let category_id = new_item.category_id.to_string();
    query!(
        "INSERT INTO inventory_items
            (id, name, description, weight, category_id)
        VALUES
            (?, ?, ?, ?, ?)",
        id_param,
        name,
        "",
        new_item.weight,
        category_id,
    )
    .execute(&state.database_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref error) => {
            let sqlite_error = error.downcast_ref::<SqliteError>();
            if let Some(code) = sqlite_error.code() {
                match &*code {
                    "787" => {
                        // SQLITE_CONSTRAINT_FOREIGNKEY
                        (
                            StatusCode::BAD_REQUEST,
                            format!("category {id} not found", id = new_item.category_id),
                        )
                    }
                    "2067" => {
                        // SQLITE_CONSTRAINT_UNIQUE
                        (
                            StatusCode::BAD_REQUEST,
                            format!(
                                "item with name \"{name}\" already exists in category {id}",
                                name = new_item.name,
                                id = new_item.category_id
                            ),
                        )
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("got error with unknown code: {}", sqlite_error.to_string()),
                    ),
                }
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("got error without code: {}", sqlite_error.to_string()),
                )
            }
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("got unknown error: {}", e.to_string()),
        ),
    })?;

    Ok(Redirect::to(&format!(
        "/inventory/category/{id}/",
        id = new_item.category_id
    )))
}

async fn inventory_item_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Redirect, (StatusCode, String)> {
    let id_param = id.to_string();
    let results = query!(
        "DELETE FROM inventory_items
        WHERE id = ?",
        id_param,
    )
    .execute(&state.database_pool)
    .await
    .map_err(|error| match error {
        sqlx::Error::Database(ref error) => {
            let sqlite_error = error.downcast_ref::<SqliteError>();
            if let Some(code) = sqlite_error.code() {
                match &*code {
                    "787" => {
                        // SQLITE_CONSTRAINT_FOREIGNKEY
                        (
                            StatusCode::BAD_REQUEST,
                            // TODO: this is not perfect, as both foreign keys
                            // may be responsible for the error. how can we tell
                            // which one?
                            format!("item {} cannot be deleted because it's on use in trips. instead, archive it", code.to_string()),
                        )
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("got error with unknown code: {}", sqlite_error.to_string()),
                    ),
                }
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("got error without code: {}", sqlite_error.to_string()),
                )
            }
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("got unknown error: {}", error.to_string()),
        ),
    })?;

    if results.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            format!("item with id {id} not found", id = id),
        ))
    } else {
        Ok(Redirect::to(
            headers
                .get("referer")
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    "no referer header found".to_string(),
                ))?
                .to_str()
                .map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("referer could not be converted: {}", e),
                    )
                })?,
        ))
    }
}

// async fn htmx_inventory_category_items(
//     Path(id): Path<String>,
// ) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
//     let pool = SqlitePoolOptions::new()
//         .max_connections(5)
//         .connect("sqlite:///home/hannes-private/sync/items/items.sqlite")
//         .await
//         .unwrap();

//     let items = query!(&format!(
//     //TODO bind this stuff!!!!!!! no sql injection pls
//         "SELECT
//             i.id, i.name, i.description, i.weight, i.category_id
//         FROM inventory_items_categories AS c
//         INNER JOIN inventoryitems AS i
//         ON i.category_id = c.id WHERE c.id = '{id}';",
//         id = id,
//     ))
//     .fetch(&pool)
//     .map_ok(|row| row.try_into())
//     .try_collect::<Vec<Result<Item, models::Error>>>()
//     .await
//     // we have two error handling lines here. these are distinct errors
//     // this one is the SQL error that may arise during the query
//     .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Html::from(e.to_string())))?
//     .into_iter()
//     .collect::<Result<Vec<Item>, models::Error>>()
//     // and this one is the model mapping error that may arise e.g. during
//     // reading of the rows
//     .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Html::from(e.to_string())))?;

//     Ok((
//         StatusCode::OK,
//         Html::from(
//             InventoryItemList::build(&items)
//                 .await
//                 .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Html::from(e.to_string())))?
//                 .to_string(),
//         ),
//     ))
// }
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
            ErrorPage::build("name cannot be empty"),
        ));
    }

    let id = Item::update(
        &state.database_pool,
        id,
        &edit_item.name,
        i64::try_from(edit_item.weight).map_err(|e| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorPage::build(&e.to_string()),
            )
        })?,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&e.to_string()),
        )
    })?
    .ok_or((
        StatusCode::NOT_FOUND,
        ErrorPage::build(&format!("item with id {id} not found", id = id)),
    ))?;

    Ok(Redirect::to(&format!("/inventory/category/{id}/", id = id)))
}

async fn inventory_item_cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Redirect, (StatusCode, String)> {
    let id = Item::find(&state.database_pool, id)
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
) -> Result<Redirect, (StatusCode, String)> {
    if new_trip.name.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "name cannot be empty".to_string(),
        ));
    }

    let id = Uuid::new_v4();
    let id_param = id.to_string();
    let date_start = new_trip
        .date_start
        .format(DATE_FORMAT)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let date_end = new_trip
        .date_end
        .format(DATE_FORMAT)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let trip_state = TripState::new();
    query!(
        "INSERT INTO trips
            (id, name, date_start, date_end, state)
        VALUES
            (?, ?, ?, ?, ?)",
        id_param,
        new_trip.name,
        date_start,
        date_end,
        trip_state
    )
    .execute(&state.database_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref error) => {
            let sqlite_error = error.downcast_ref::<SqliteError>();
            if let Some(code) = sqlite_error.code() {
                match &*code {
                    "2067" => {
                        // SQLITE_CONSTRAINT_UNIQUE
                        (
                            StatusCode::BAD_REQUEST,
                            format!(
                                "trip with name \"{name}\" already exists",
                                name = new_trip.name,
                            ),
                        )
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("got error with unknown code: {}", sqlite_error.to_string()),
                    ),
                }
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("got error without code: {}", sqlite_error.to_string()),
                )
            }
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("got unknown error: {}", e.to_string()),
        ),
    })?;

    Ok(Redirect::to(&format!("/trip/{id}/", id = id)))
}

async fn trips(
    State(state): State<AppState>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    let trips: Vec<models::Trip> = query_as!(
        DbTripRow,
        "SELECT
            id,
            name,
            CAST (date_start AS TEXT) date_start,
            CAST (date_end AS TEXT) date_end,
            state,
            location,
            temp_min,
            temp_max,
            comment
        FROM trips",
    )
    .fetch(&state.database_pool)
    .map_ok(|row| row.try_into())
    .try_collect::<Vec<Result<models::Trip, models::Error>>>()
    .await
    // we have two error handling lines here. these are distinct errors
    // this one is the SQL error that may arise during the query
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&e.to_string()),
        )
    })?
    .into_iter()
    .collect::<Result<Vec<models::Trip>, models::Error>>()
    // and this one is the model mapping error that may arise e.g. during
    // reading of the rows
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&e.to_string()),
        )
    })?;

    Ok((
        StatusCode::OK,
        Root::build(&TripManager::build(trips), &TopLevelPage::Trips),
    ))
}

#[derive(Debug, Deserialize)]
struct TripQuery {
    edit: Option<TripAttribute>,
    category: Option<Uuid>,
}

async fn trip(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(trip_query): Query<TripQuery>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    state.client_state.trip_edit_attribute = trip_query.edit;
    state.client_state.active_category_id = trip_query.category;

    let id_param = id.to_string();
    let mut trip: models::Trip = query_as!(
        DbTripRow,
        "SELECT
                id,
                name,
                CAST (date_start AS TEXT) AS date_start,
                CAST (date_end AS TEXT) AS date_end,
                state,
                location,
                temp_min,
                temp_max,
                comment
            FROM trips
            WHERE id = ?",
        id_param,
    )
    .fetch_one(&state.database_pool)
    .map_ok(|row| row.try_into())
    .await
    .map_err(|e: sqlx::Error| match e {
        sqlx::Error::RowNotFound => (
            StatusCode::NOT_FOUND,
            ErrorPage::build(&format!("trip with id {} not found", id)),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&e.to_string()),
        ),
    })?
    .map_err(|e: Error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&e.to_string()),
        )
    })?;

    trip.load_trips_types(&state.database_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorPage::build(&e.to_string()),
            )
        })?;

    trip.sync_trip_items_with_inventory(&state.database_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorPage::build(&e.to_string()),
            )
        })?;

    trip.load_categories(&state.database_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorPage::build(&e.to_string()),
            )
        })?;

    Ok((
        StatusCode::OK,
        Root::build(
            &components::Trip::build(&state.client_state, &trip).map_err(|e| match e {
                Error::NotFound { description } => {
                    (StatusCode::NOT_FOUND, ErrorPage::build(&description))
                }
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorPage::build(&e.to_string()),
                ),
            })?,
            &TopLevelPage::Trips,
        ),
    ))
}

async fn trip_type_remove(
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, (StatusCode, Markup)> {
    let trip_id = trip_id.to_string();
    let type_id = type_id.to_string();
    let results = query!(
        "DELETE FROM trips_to_trips_types AS ttt
        WHERE ttt.trip_id = ?
            AND ttt.trip_type_id = ?
        ",
        trip_id,
        type_id
    )
    .execute(&state.database_pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, ErrorPage::build(&e.to_string())))?;

    if results.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            ErrorPage::build(&format!("type {type_id} is not active for trip {trip_id}")),
        ))
    } else {
        Ok(Redirect::to(&format!("/trip/{trip_id}/")))
    }
}

async fn trip_type_add(
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, (StatusCode, Markup)> {
    let trip_id = trip_id.to_string();
    let type_id = type_id.to_string();
    query!(
        "INSERT INTO trips_to_trips_types
        (trip_id, trip_type_id) VALUES (?, ?)",
        trip_id,
        type_id
    )
    .execute(&state.database_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref error) => {
            let sqlite_error = error.downcast_ref::<SqliteError>();
            if let Some(code) = sqlite_error.code() {
                match &*code {
                    "787" => {
                        // SQLITE_CONSTRAINT_FOREIGNKEY
                        (
                            StatusCode::BAD_REQUEST,
                            // TODO: this is not perfect, as both foreign keys
                            // may be responsible for the error. how can we tell
                            // which one?
                            ErrorPage::build(&format!("invalid id: {}", code.to_string())),
                        )
                    }
                    "2067" => {
                        // SQLITE_CONSTRAINT_UNIQUE
                        (
                            StatusCode::BAD_REQUEST,
                            ErrorPage::build(&format!(
                                "type {type_id} is already active for trip {trip_id}"
                            )),
                        )
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorPage::build(&format!(
                            "got error with unknown code: {}",
                            sqlite_error.to_string()
                        )),
                    ),
                }
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorPage::build(&format!(
                        "got error without code: {}",
                        sqlite_error.to_string()
                    )),
                )
            }
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&format!("got unknown error: {}", e.to_string())),
        ),
    })?;

    Ok(Redirect::to(&format!("/trip/{trip_id}/")))
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
    let trip_id = trip_id.to_string();
    let result = query!(
        "UPDATE trips
        SET comment = ?
        WHERE id = ?",
        comment_update.new_comment,
        trip_id,
    )
    .execute(&state.database_pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, ErrorPage::build(&e.to_string())))?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            ErrorPage::build(&format!("trip with id {id} not found", id = trip_id)),
        ))
    } else {
        Ok(Redirect::to(&format!("/trip/{id}/", id = trip_id)))
    }
}

#[derive(Deserialize)]
struct TripUpdate {
    #[serde(rename = "new-value")]
    new_value: String,
}

async fn trip_edit_attribute(
    State(state): State<AppState>,
    Path((trip_id, attribute)): Path<(Uuid, TripAttribute)>,
    Form(trip_update): Form<TripUpdate>,
) -> Result<Redirect, (StatusCode, Markup)> {
    if let TripAttribute::Name = attribute {
        if trip_update.new_value.is_empty() {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorPage::build("name cannot be empty"),
            ));
        }
    }
    let result = query(&format!(
        "UPDATE trips
        SET {attribute} = ?
        WHERE id = ?",
        attribute = to_variant_name(&attribute).unwrap()
    ))
    .bind(trip_update.new_value)
    .bind(trip_id.to_string())
    .execute(&state.database_pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, ErrorPage::build(&e.to_string())))?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            ErrorPage::build(&format!("trip with id {id} not found", id = trip_id)),
        ))
    } else {
        Ok(Redirect::to(&format!("/trip/{trip_id}/")))
    }
}

async fn trip_item_set_state(
    state: &AppState,
    trip_id: Uuid,
    item_id: Uuid,
    key: TripItemStateKey,
    value: bool,
) -> Result<(), (StatusCode, Markup)> {
    let result = query(&format!(
        "UPDATE trips_items
        SET {key} = ?
        WHERE trip_id = ?
        AND item_id = ?",
        key = to_variant_name(&key).unwrap()
    ))
    .bind(value)
    .bind(trip_id.to_string())
    .bind(item_id.to_string())
    .execute(&state.database_pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, ErrorPage::build(&e.to_string())))?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            ErrorPage::build(&format!(
                "trip with id {trip_id} or item with id {item_id} not found"
            )),
        ))
    } else {
        Ok(())
    }
}

async fn trip_item_set_pick(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, Markup)> {
    Ok(trip_item_set_state(&state, trip_id, item_id, TripItemStateKey::Pick, true).await?).map(
        |_| -> Result<Redirect, (StatusCode, Markup)> {
            Ok(Redirect::to(
                headers
                    .get("referer")
                    .ok_or((
                        StatusCode::BAD_REQUEST,
                        ErrorPage::build("no referer header found"),
                    ))?
                    .to_str()
                    .map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            ErrorPage::build(&format!("referer could not be converted: {}", e)),
                        )
                    })?,
            ))
        },
    )?
}

async fn trip_item_set_unpick(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, Markup)> {
    Ok(trip_item_set_state(&state, trip_id, item_id, TripItemStateKey::Pick, false).await?).map(
        |_| -> Result<Redirect, (StatusCode, Markup)> {
            Ok(Redirect::to(
                headers
                    .get("referer")
                    .ok_or((
                        StatusCode::BAD_REQUEST,
                        ErrorPage::build("no referer header found"),
                    ))?
                    .to_str()
                    .map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            ErrorPage::build(&format!("referer could not be converted: {}", e)),
                        )
                    })?,
            ))
        },
    )?
}

async fn trip_item_set_pack(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, Markup)> {
    Ok(trip_item_set_state(&state, trip_id, item_id, TripItemStateKey::Pack, true).await?).map(
        |_| -> Result<Redirect, (StatusCode, Markup)> {
            Ok(Redirect::to(
                headers
                    .get("referer")
                    .ok_or((
                        StatusCode::BAD_REQUEST,
                        ErrorPage::build("no referer header found"),
                    ))?
                    .to_str()
                    .map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            ErrorPage::build(&format!("referer could not be converted: {}", e)),
                        )
                    })?,
            ))
        },
    )?
}

async fn trip_item_set_unpack(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, Markup)> {
    Ok(trip_item_set_state(&state, trip_id, item_id, TripItemStateKey::Pack, false).await?).map(
        |_| -> Result<Redirect, (StatusCode, Markup)> {
            Ok(Redirect::to(
                headers
                    .get("referer")
                    .ok_or((
                        StatusCode::BAD_REQUEST,
                        ErrorPage::build("no referer header found"),
                    ))?
                    .to_str()
                    .map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            ErrorPage::build(&format!("referer could not be converted: {}", e)),
                        )
                    })?,
            ))
        },
    )?
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
            ErrorPage::build("name cannot be empty"),
        ));
    }

    let id = Uuid::new_v4();
    let id_param = id.to_string();
    query!(
        "INSERT INTO inventory_items_categories
            (id, name)
        VALUES
            (?, ?)",
        id_param,
        new_category.name
    )
    .execute(&state.database_pool)
    .map_err(|e| match e {
        sqlx::Error::Database(ref error) => {
            let sqlite_error = error.downcast_ref::<SqliteError>();
            if let Some(code) = sqlite_error.code() {
                match &*code {
                    "2067" => {
                        // SQLITE_CONSTRAINT_UNIQUE
                        (
                            StatusCode::BAD_REQUEST,
                            ErrorPage::build(&format!(
                                "category with name \"{name}\" already exists",
                                name = new_category.name
                            )),
                        )
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorPage::build(&format!(
                            "got error with unknown code: {}",
                            sqlite_error.to_string()
                        )),
                    ),
                }
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorPage::build(&format!(
                        "got error without code: {}",
                        sqlite_error.to_string()
                    )),
                )
            }
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&format!("got unknown error: {}", e.to_string())),
        ),
    })
    .await?;

    Ok(Redirect::to("/inventory/"))
}

async fn trip_state_set(
    State(state): State<AppState>,
    Path((trip_id, new_state)): Path<(Uuid, TripState)>,
) -> Result<Redirect, (StatusCode, Markup)> {
    let trip_id = trip_id.to_string();
    let result = query!(
        "UPDATE trips
        SET state = ?
        WHERE id = ?",
        new_state,
        trip_id,
    )
    .execute(&state.database_pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, ErrorPage::build(&e.to_string())))?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            ErrorPage::build(&format!("trip with id {id} not found", id = trip_id)),
        ))
    } else {
        Ok(Redirect::to(&format!("/trip/{id}/", id = trip_id)))
    }
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

    let trip_types: Vec<models::TripsType> = query_as!(
        DbTripsTypesRow,
        "SELECT
            id,
            name
        FROM trips_types",
    )
    .fetch(&state.database_pool)
    .map_ok(|row| row.try_into())
    .try_collect::<Vec<Result<models::TripsType, models::Error>>>()
    .await
    // we have two error handling lines here. these are distinct errors
    // this one is the SQL error that may arise during the query
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&e.to_string()),
        )
    })?
    .into_iter()
    .collect::<Result<Vec<models::TripsType>, models::Error>>()
    // and this one is the model mapping error that may arise e.g. during
    // reading of the rows
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorPage::build(&e.to_string()),
        )
    })?;

    Ok((
        StatusCode::OK,
        Root::build(
            &components::trip::TypeList::build(&state.client_state, trip_types),
            &TopLevelPage::Trips,
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
) -> Result<Redirect, (StatusCode, String)> {
    if new_trip_type.name.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "name cannot be empty".to_string(),
        ));
    }

    let id = Uuid::new_v4();
    let id_param = id.to_string();
    query!(
        "INSERT INTO trips_types
            (id, name)
        VALUES
            (?, ?)",
        id_param,
        new_trip_type.name,
    )
    .execute(&state.database_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref error) => {
            let sqlite_error = error.downcast_ref::<SqliteError>();
            if let Some(code) = sqlite_error.code() {
                match &*code {
                    "2067" => {
                        // SQLITE_CONSTRAINT_UNIQUE
                        (
                            StatusCode::BAD_REQUEST,
                            format!(
                                "trip type with name \"{name}\" already exists",
                                name = new_trip_type.name,
                            ),
                        )
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("got error with unknown code: {}", sqlite_error.to_string()),
                    ),
                }
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("got error without code: {}", sqlite_error.to_string()),
                )
            }
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("got unknown error: {}", e.to_string()),
        ),
    })?;

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
            ErrorPage::build("name cannot be empty"),
        ));
    }

    let id_param = trip_type_id.to_string();
    let result = query!(
        "UPDATE trips_types
        SET name = ?
        WHERE id = ?",
        trip_update.new_value,
        id_param,
    )
    .execute(&state.database_pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, ErrorPage::build(&e.to_string())))?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            ErrorPage::build(&format!(
                "tript type with id {id} not found",
                id = trip_type_id
            )),
        ))
    } else {
        Ok(Redirect::to("/trips/types/"))
    }
}
