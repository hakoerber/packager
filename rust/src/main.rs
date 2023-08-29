#![allow(unused_imports)]
use axum::{
    extract::{Path, Query, State},
    headers,
    headers::Header,
    http::{header::HeaderMap, StatusCode},
    response::{Html, Redirect},
    routing::{get, post},
    Form, Router,
};

use sqlx::{
    error::DatabaseError,
    query,
    sqlite::{SqliteConnectOptions, SqliteError, SqlitePoolOptions},
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

#[derive(Clone)]
pub struct ClientState {
    pub active_category_id: Option<Uuid>,
    pub edit_item: Option<Uuid>,
}

impl ClientState {
    pub fn new() -> Self {
        ClientState {
            active_category_id: None,
            edit_item: None,
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
            SqliteConnectOptions::new()
                .filename(std::env::var("SQLITE_DATABASE").expect("env SQLITE_DATABASE not found"))
                .pragma("foreign_keys", "1"),
        )
        .await
        .unwrap();

    let state = AppState {
        database_pool,
        client_state: ClientState::new(),
    };

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/trips/", get(trips))
        .route("/trip/", post(trip_create))
        .route("/trip/:id/", get(trip))
        .route("/trip/:id/type/:id/add", get(trip_type_add))
        .route("/trip/:id/type/:id/remove", get(trip_type_remove))
        .route("/inventory/", get(inventory_inactive))
        .route("/inventory/item/", post(inventory_item_create))
        .route("/inventory/category/:id/", get(inventory_active))
        .route("/inventory/item/:id/delete", get(inventory_item_delete))
        .route("/inventory/item/:id/edit", post(inventory_item_edit))
        .route("/inventory/item/:id/cancel", get(inventory_item_cancel))
        // .route(
        //     "/inventory/category/:id/items",
        //     post(htmx_inventory_category_items),
        // );
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
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
        Root::build(Home::build(), &TopLevelPage::None),
    )
}

#[derive(Deserialize)]
struct InventoryQuery {
    edit_item: Option<Uuid>,
}

impl Default for InventoryQuery {
    fn default() -> Self {
        Self { edit_item: None }
    }
}

async fn inventory_active(
    Path(id): Path<Uuid>,
    State(mut state): State<AppState>,
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

    let mut categories = query("SELECT id,name,description FROM inventoryitemcategories")
        .fetch(&state.database_pool)
        .map_ok(std::convert::TryInto::try_into)
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
            Inventory::build(state.client_state, categories)
                .await
                .map_err(|e| match e {
                    Error::NotFoundError { description } => {
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

async fn inventory_item_create(
    State(state): State<AppState>,
    Form(new_item): Form<NewItem>,
) -> Result<Redirect, (StatusCode, String)> {
    query(
        "INSERT INTO inventoryitems
            (id, name, description, weight, category_id)
        VALUES
            (?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&new_item.name)
    .bind("")
    .bind(new_item.weight)
    .bind(new_item.category_id.to_string())
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
    let results = query(
        "DELETE FROM inventoryitems
        WHERE id = ?",
    )
    .bind(id.to_string())
    .execute(&state.database_pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

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

//     let items = query(&format!(
//     //TODO bind this stuff!!!!!!! no sql injection pls
//         "SELECT
//             i.id, i.name, i.description, i.weight, i.category_id
//         FROM inventoryitemcategories AS c
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
) -> Result<Redirect, (StatusCode, String)> {
    let id = Item::update(&state.database_pool, id, &edit_item.name, edit_item.weight)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("item with id {id} not found", id = id),
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
    start_date: time::Date,
    #[serde(rename = "new-trip-end-date")]
    end_date: time::Date,
}

async fn trip_create(
    State(state): State<AppState>,
    Form(new_trip): Form<NewTrip>,
) -> Result<Redirect, (StatusCode, String)> {
    let id = Uuid::new_v4();
    query(
        "INSERT INTO trips
            (id, name, start_date, end_date)
        VALUES
            (?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(&new_trip.name)
    .bind(new_trip.start_date)
    .bind(new_trip.end_date)
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

    Ok(Redirect::to(&format!("/trips/{id}/", id = id.to_string())))
}

async fn trips(
    State(state): State<AppState>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    let trips: Vec<models::Trip> = query("SELECT * FROM trips")
        .fetch(&state.database_pool)
        .map_ok(std::convert::TryInto::try_into)
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
        Root::build(TripManager::build(trips), &TopLevelPage::Trips),
    ))
}

async fn trip(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<(StatusCode, Markup), (StatusCode, Markup)> {
    let mut trip: models::Trip =
        query("SELECT id,name,start_date,end_date,state,location,temp_min,temp_max FROM trips WHERE id = ?")
            .bind(id.to_string())
            .fetch_one(&state.database_pool)
            .map_ok(std::convert::TryInto::try_into)
            .await
            .map_err(|e: sqlx::Error| match e {
                sqlx::Error::RowNotFound => (
                    StatusCode::NOT_FOUND,
                    ErrorPage::build(&format!("trip with id {} not found", id)),
                ),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, ErrorPage::build(&e.to_string())),
            })?
            .map_err(|e: Error| (StatusCode::INTERNAL_SERVER_ERROR, ErrorPage::build(&e.to_string())))?;

    trip.load_triptypes(&state.database_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorPage::build(&e.to_string()),
            )
        })?;

    Ok((
        StatusCode::OK,
        Root::build(components::Trip::build(&trip), &TopLevelPage::Trips),
    ))
}

async fn trip_type_remove(
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> Result<Redirect, (StatusCode, Markup)> {
    let results = query(
        "DELETE FROM trips_to_triptypes AS ttt
        WHERE ttt.trip_id = ?
            AND ttt.trip_type_id = ?
        ",
    )
    .bind(trip_id.to_string())
    .bind(type_id.to_string())
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
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> Result<Redirect, (StatusCode, Markup)> {
    query(
        "INSERT INTO trips_to_triptypes
        (trip_id, trip_type_id) VALUES (?, ?)",
    )
    .bind(trip_id.to_string())
    .bind(type_id.to_string())
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
