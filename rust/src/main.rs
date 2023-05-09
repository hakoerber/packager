#![allow(unused_imports)]
use axum::{
    extract::Path,
    extract::State,
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
    Pool, Sqlite,
};

use serde::Deserialize;

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
}

impl ClientState {
    pub fn new() -> Self {
        ClientState {
            active_category_id: None,
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
                .filename("/home/hannes-private/sync/items/items.sqlite")
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
        .route("/inventory/", get(inventory_inactive))
        .route("/inventory/item/", post(inventory_item_create))
        .route("/inventory/category/:id", get(inventory_active))
        .route("/inventory/item/:id/delete", get(inventory_item_delete))
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

async fn root() -> (StatusCode, Html<String>) {
    (
        StatusCode::OK,
        Html::from(Root::build(Home::build().into(), TopLevelPage::None).into_string()),
    )
}

async fn inventory_active(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<(StatusCode, Html<String>), (StatusCode, Html<String>)> {
    inventory(state, Some(id)).await
}

async fn inventory_inactive(
    State(state): State<AppState>,
) -> Result<(StatusCode, Html<String>), (StatusCode, Html<String>)> {
    inventory(state, None).await
}

async fn inventory(
    mut state: AppState,
    active_id: Option<String>,
) -> Result<(StatusCode, Html<String>), (StatusCode, Html<String>)> {
    let active_id = active_id
        .map(|id| Uuid::try_parse(&id))
        .transpose()
        .map_err(|e| (StatusCode::BAD_REQUEST, Html::from(e.to_string())))?;

    state.client_state.active_category_id = active_id;

    let mut categories = query("SELECT id,name,description FROM inventoryitemcategories")
        .fetch(&state.database_pool)
        .map_ok(|row| row.try_into())
        .try_collect::<Vec<Result<Category, models::Error>>>()
        .await
        // we have two error handling lines here. these are distinct errors
        // this one is the SQL error that may arise during the query
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Html::from(e.to_string())))?
        .into_iter()
        .collect::<Result<Vec<Category>, models::Error>>()
        // and this one is the model mapping error that may arise e.g. during
        // reading of the rows
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Html::from(e.to_string())))?;

    for category in &mut categories {
        category
            .populate_items()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Html::from(e.to_string())))?;

        if let Some(active_id) = active_id {
            if category.id == active_id {
                category.active = true;
            }
        }
    }

    Ok((
        StatusCode::OK,
        Html::from(
            Root::build(
                Inventory::build(state.client_state, categories)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Html::from(e.to_string())))?
                    .into(),
                TopLevelPage::Inventory,
            )
            .into_string(),
        ),
    ))
}

async fn trips(
    State(state): State<AppState>,
) -> Result<(StatusCode, Html<String>), (StatusCode, Html<String>)> {
    let trips = query("SELECT * FROM trips")
        .fetch(&state.database_pool)
        .map_ok(|row| row.try_into())
        .try_collect::<Vec<Result<Trip, models::Error>>>()
        .await
        // we have two error handling lines here. these are distinct errors
        // this one is the SQL error that may arise during the query
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Html::from(e.to_string())))?
        .into_iter()
        .collect::<Result<Vec<Trip>, models::Error>>()
        // and this one is the model mapping error that may arise e.g. during
        // reading of the rows
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Html::from(e.to_string())))?;

    Ok((
        StatusCode::OK,
        Html::from(Root::build(TripList::build(trips).into(), TopLevelPage::Trips).into_string()),
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
        "/inventory/category/{id}",
        id = new_item.category_id
    )))
}

async fn inventory_item_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Redirect, (StatusCode, String)> {
    query(
        "DELETE FROM inventoryitems
        WHERE id = ?",
    )
    .bind(id.to_string())
    .execute(&state.database_pool)
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

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

// async fn htmx_inventory_category_items(
//     Path(id): Path<String>,
// ) -> Result<(StatusCode, Html<String>), (StatusCode, Html<String>)> {
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
//         LEFT JOIN inventoryitems AS i
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
