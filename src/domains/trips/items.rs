use axum::{
    extract::{Extension, Path, State},
    http::header::{HeaderMap, HeaderName},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};

use uuid::Uuid;

use crate::{
    htmx,
    routing::{get_referer, html},
    AppState, Context, Error, RequestError,
};

use super::{model, view};
use crate::models::User;

#[tracing::instrument]
async fn trip_row(
    ctx: &Context,
    state: &AppState,
    trip_id: Uuid,
    item_id: Uuid,
) -> Result<impl IntoResponse, Error> {
    let item = model::TripItem::find(ctx, &state.database_pool, trip_id, item_id)
        .await?
        .ok_or_else(|| {
            Error::Request(RequestError::NotFound {
                message: format!("item with id {item_id} not found for trip {trip_id}"),
            })
        })?;

    let item_row = view::TripItemListRow::build(
        trip_id,
        &item,
        crate::domains::inventory::InventoryItem::get_category_max_weight(
            ctx,
            &state.database_pool,
            item.item.category_id,
        )
        .await?,
    );

    let category =
        model::TripCategory::find(ctx, &state.database_pool, trip_id, item.item.category_id)
            .await?
            .ok_or_else(|| {
                Error::Request(RequestError::NotFound {
                    message: format!("category with id {} not found", item.item.category_id),
                })
            })?;

    // TODO biggest_category_weight?
    let category_row = view::TripCategoryListRow::build(trip_id, &category, true, 0, true);

    Ok(html::concat(&item_row, &category_row))
}

#[tracing::instrument]
async fn set_item_pick(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        model::trip_item_set_state(
            &ctx,
            &state.database_pool,
            trip_id,
            item_id,
            model::TripItemStateKey::Pick,
            true,
        )
        .await?,
    )
    .map(|()| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
async fn set_item_pick_htmx(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    model::trip_item_set_state(
        &ctx,
        &state.database_pool,
        trip_id,
        item_id,
        model::TripItemStateKey::Pick,
        true,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        htmx::ResponseHeaders::Trigger.into(),
        htmx::Event::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&ctx, &state, trip_id, item_id).await?))
}

#[tracing::instrument]
async fn set_item_unpick(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        model::trip_item_set_state(
            &ctx,
            &state.database_pool,
            trip_id,
            item_id,
            model::TripItemStateKey::Pick,
            false,
        )
        .await?,
    )
    .map(|()| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
async fn set_item_unpick_htmx(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    model::trip_item_set_state(
        &ctx,
        &state.database_pool,
        trip_id,
        item_id,
        model::TripItemStateKey::Pick,
        false,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        htmx::ResponseHeaders::Trigger.into(),
        htmx::Event::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&ctx, &state, trip_id, item_id).await?))
}

#[tracing::instrument]
async fn set_item_pack(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        model::trip_item_set_state(
            &ctx,
            &state.database_pool,
            trip_id,
            item_id,
            model::TripItemStateKey::Pack,
            true,
        )
        .await?,
    )
    .map(|()| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
async fn set_item_pack_htmx(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
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
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        htmx::ResponseHeaders::Trigger.into(),
        htmx::Event::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&ctx, &state, trip_id, item_id).await?))
}

#[tracing::instrument]
async fn set_item_unpack(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        model::trip_item_set_state(
            &ctx,
            &state.database_pool,
            trip_id,
            item_id,
            model::TripItemStateKey::Pack,
            false,
        )
        .await?,
    )
    .map(|()| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
async fn set_item_unpack_htmx(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
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
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        htmx::ResponseHeaders::Trigger.into(),
        htmx::Event::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&ctx, &state, trip_id, item_id).await?))
}

#[tracing::instrument]
async fn set_item_ready(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        model::trip_item_set_state(
            &ctx,
            &state.database_pool,
            trip_id,
            item_id,
            model::TripItemStateKey::Ready,
            true,
        )
        .await?,
    )
    .map(|()| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
async fn set_item_ready_htmx(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
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
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        htmx::ResponseHeaders::Trigger.into(),
        htmx::Event::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&ctx, &state, trip_id, item_id).await?))
}

#[tracing::instrument]
async fn set_item_unready(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        model::trip_item_set_state(
            &ctx,
            &state.database_pool,
            trip_id,
            item_id,
            model::TripItemStateKey::Ready,
            false,
        )
        .await?,
    )
    .map(|()| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
async fn set_item_unready_htmx(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
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
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        htmx::ResponseHeaders::Trigger.into(),
        htmx::Event::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&ctx, &state, trip_id, item_id).await?))
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/:id/items/:id/pick",
            get(set_item_pick).post(set_item_pick_htmx),
        )
        .route(
            "/:id/items/:id/unpick",
            get(set_item_unpick).post(set_item_unpick_htmx),
        )
        .route(
            "/:id/items/:id/pack",
            get(set_item_pack).post(set_item_pack_htmx),
        )
        .route(
            "/:id/items/:id/unpack",
            get(set_item_unpack).post(set_item_unpack_htmx),
        )
        .route(
            "/:id/items/:id/ready",
            get(set_item_ready).post(set_item_ready_htmx),
        )
        .route(
            "/:id/items/:id/unready",
            get(set_item_unready).post(set_item_unready_htmx),
        )
}
