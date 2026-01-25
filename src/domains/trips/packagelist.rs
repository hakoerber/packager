use axum::{
    extract::{Extension, Path, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use uuid::Uuid;

use crate::{AppState, Context, RunError, RequestError, TopLevelPage};

use super::{model, view};
use crate::models::User;

#[tracing::instrument]
pub async fn base(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    let mut trip = model::Trip::find(&ctx, &state.database_pool, trip_id)
        .await?
        .ok_or(RunError::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))?;

    trip.load_categories(&ctx, &state.database_pool).await?;

    Ok(crate::view::Root::build(
        &ctx,
        &view::packagelist::TripPackageList::build(&trip),
        Some(&TopLevelPage::Trips),
    ))
}

#[tracing::instrument]
pub async fn set_item_pack_html(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    model::trip_item_set_state(
        &ctx,
        &state.database_pool,
        trip_id,
        item_id,
        model::TripItemStateKey::Pack,
        true,
    )
    .await?;

    let item = model::TripItem::find(&ctx, &state.database_pool, trip_id, item_id)
        .await?
        .ok_or(RunError::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::packagelist::TripPackageListRowReady::build(
        trip_id, &item,
    ))
}

#[tracing::instrument]
pub async fn set_item_unpack_htmx(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    model::trip_item_set_state(
        &ctx,
        &state.database_pool,
        trip_id,
        item_id,
        model::TripItemStateKey::Pack,
        false,
    )
    .await?;

    // note that this cannot fail due to a missing item, as trip_item_set_state would already
    // return 404. but error handling cannot hurt ;)
    let item = model::TripItem::find(&ctx, &state.database_pool, trip_id, item_id)
        .await?
        .ok_or(RunError::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::packagelist::TripPackageListRowReady::build(
        trip_id, &item,
    ))
}

#[tracing::instrument]
pub async fn set_item_ready_htmx(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    model::trip_item_set_state(
        &ctx,
        &state.database_pool,
        trip_id,
        item_id,
        model::TripItemStateKey::Ready,
        true,
    )
    .await?;

    let item = model::TripItem::find(&ctx, &state.database_pool, trip_id, item_id)
        .await?
        .ok_or(RunError::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::packagelist::TripPackageListRowUnready::build(
        trip_id, &item,
    ))
}

#[tracing::instrument]
pub async fn set_item_unready_html(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    model::trip_item_set_state(
        &ctx,
        &state.database_pool,
        trip_id,
        item_id,
        model::TripItemStateKey::Ready,
        false,
    )
    .await?;

    // note that this cannot fail due to a missing item, as trip_item_set_state would already
    // return 404. but error handling cannot hurt ;)
    let item = model::TripItem::find(&ctx, &state.database_pool, trip_id, item_id)
        .await?
        .ok_or(RunError::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::packagelist::TripPackageListRowUnready::build(
        trip_id, &item,
    ))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(base))
        .route("/item/{id}/pack", post(set_item_pack_html))
        .route("/item/{id}/unpack", post(set_item_unpack_htmx))
        .route("/item/{id}/ready", post(set_item_ready_htmx))
        .route("/item/{id}/unready", post(set_item_unready_html))
}
