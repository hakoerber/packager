use axum::{extract::Path, http::StatusCode, response::Html, routing::get, Router};
use sqlx::sqlite::SqlitePoolOptions;

use futures::TryStreamExt;
use uuid::Uuid;

use std::net::SocketAddr;

mod components;
mod models;

use crate::components::*;
use crate::models::*;

pub struct State {
    pub has_active_category: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            has_active_category: false,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/trips/", get(trips))
        .route("/inventory/", get(inventory_inactive))
        .route("/inventory/category/:id", get(inventory_active));

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
        Html::from(Root::build(Home::build().into(), TopLevelPage::None).to_string()),
    )
}

async fn inventory_active(
    Path(id): Path<String>,
) -> Result<(StatusCode, Html<String>), (StatusCode, Html<String>)> {
    inventory(Some(id)).await
}

async fn inventory_inactive() -> Result<(StatusCode, Html<String>), (StatusCode, Html<String>)> {
    inventory(None).await
}

async fn inventory(
    active_id: Option<String>,
) -> Result<(StatusCode, Html<String>), (StatusCode, Html<String>)> {
    let mut state: State = State::new();
    state.has_active_category = active_id.is_some();

    let active_id = active_id
        .map(|id| Uuid::try_parse(&id))
        .transpose()
        .map_err(|e| (StatusCode::BAD_REQUEST, Html::from(e.to_string())))?;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:///home/hannes-private/sync/items/items.sqlite")
        .await
        .unwrap();

    let mut categories = sqlx::query("SELECT id,name,description FROM inventoryitemcategories")
        .fetch(&pool)
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
                Inventory::build(state, categories)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Html::from(e.to_string())))?
                    .into(),
                TopLevelPage::Inventory,
            )
            .to_string(),
        ),
    ))
}

async fn trips() -> Result<(StatusCode, Html<String>), (StatusCode, Html<String>)> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:///home/hannes-private/sync/items/items.sqlite")
        .await
        .unwrap();

    let trips = sqlx::query("SELECT * FROM trips")
        .fetch(&pool)
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
        Html::from(Root::build(TripList::build(trips).into(), TopLevelPage::Trips).to_string()),
    ))
}
