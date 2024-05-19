use axum::{
    extract::{Extension, Path, Query, State},
    http::header::{self, HeaderMap, HeaderName},
    response::{IntoResponse, Redirect},
    Form,
};

use crate::components;
use crate::components::crud::*;
use crate::components::trips::todos;

use crate::view::Component;

use serde::Deserialize;
use uuid::Uuid;

use crate::htmx;
use crate::models;
use crate::view;
use crate::{AppState, Context, Error, RequestError, TopLevelPage};

use super::{get_referer, html};

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct InventoryQuery {
    edit_item: Option<Uuid>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NewItem {
    #[serde(rename = "new-item-name")]
    name: String,
    #[serde(rename = "new-item-weight")]
    weight: u32,
    // damn i just love how serde is integrated everywhere, just add a feature to the uuid in
    // cargo.toml and go
    #[serde(rename = "new-item-category-id")]
    category_id: Uuid,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NewItemName {
    #[serde(rename = "new-item-name")]
    name: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditItem {
    #[serde(rename = "edit-item-name")]
    name: String,
    #[serde(rename = "edit-item-weight")]
    weight: u32,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NewTrip {
    #[serde(rename = "new-trip-name")]
    name: String,
    #[serde(rename = "new-trip-start-date")]
    date_start: time::Date,
    #[serde(rename = "new-trip-end-date")]
    date_end: time::Date,
    #[serde(
        rename = "new-trip-copy-from",
        deserialize_with = "super::uuid_or_empty"
    )]
    copy_from: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TripQuery {
    edit: Option<models::trips::TripAttribute>,
    category: Option<Uuid>,
    edit_todo: Option<Uuid>,
    delete_todo: Option<Uuid>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CommentUpdate {
    #[serde(rename = "new-comment")]
    new_comment: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TripUpdate {
    #[serde(rename = "new-value")]
    new_value: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NewCategory {
    #[serde(rename = "new-category-name")]
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TripTypeQuery {
    edit: Option<Uuid>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NewTripType {
    #[serde(rename = "new-trip-type-name")]
    name: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TripTypeUpdate {
    #[serde(rename = "new-value")]
    new_value: String,
}

#[tracing::instrument]
pub async fn root(
    Extension(current_user): Extension<models::user::User>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if htmx::is_htmx(&headers) {
        view::root::Body::init(
            view::Parent::Root,
            view::root::BodyArgs {
                body: &view::home::Home::build(),
                active_page: None,
            },
        )
        .build(&Context::build(current_user))
    } else {
        view::Root::build(
            &Context::build(current_user),
            &view::home::Home::build(),
            None,
        )
    }
}

#[tracing::instrument]
pub async fn icon() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/svg+xml")],
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/luggage.svg")),
    )
}

#[tracing::instrument]
pub async fn debug(headers: HeaderMap) -> impl IntoResponse {
    let mut out = String::new();
    for (key, value) in headers.iter() {
        out.push_str(&format!("{}: {}\n", key, value.to_str().unwrap()));
    }
    out
}

#[tracing::instrument]
pub async fn inventory_active(
    Extension(current_user): Extension<models::user::User>,
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(inventory_query): Query<InventoryQuery>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    state.client_state.edit_item = inventory_query.edit_item;
    state.client_state.active_category_id = Some(id);

    let inventory = models::inventory::Inventory::load(&ctx, &state.database_pool).await?;

    let active_category: Option<&models::inventory::Category> = state
        .client_state
        .active_category_id
        .map(|id| {
            inventory
                .categories
                .iter()
                .find(|category| category.id == id)
                .ok_or(Error::Request(RequestError::NotFound {
                    message: format!("a category with id {id} does not exist"),
                }))
        })
        .transpose()?;

    Ok(view::Root::build(
        &ctx,
        &view::inventory::Inventory::build(
            active_category,
            &inventory.categories,
            state.client_state.edit_item,
        ),
        Some(&TopLevelPage::Inventory),
    ))
}

#[tracing::instrument]
pub async fn inventory_inactive(
    Extension(current_user): Extension<models::user::User>,
    State(mut state): State<AppState>,
    Query(inventory_query): Query<InventoryQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    state.client_state.edit_item = inventory_query.edit_item;
    state.client_state.active_category_id = None;
    let inventory = models::inventory::Inventory::load(&ctx, &state.database_pool).await?;

    if htmx::is_htmx(&headers) {
        Ok(view::root::Body::init(
            view::Parent::Root,
            view::root::BodyArgs {
                body: &view::inventory::Inventory::build(
                    None,
                    &inventory.categories,
                    state.client_state.edit_item,
                ),

                active_page: Some(&TopLevelPage::Inventory),
            },
        )
        .build(&ctx))
    } else {
        Ok(view::Root::build(
            &ctx,
            &view::inventory::Inventory::build(
                None,
                &inventory.categories,
                state.client_state.edit_item,
            ),
            Some(&TopLevelPage::Inventory),
        ))
    }
}

#[tracing::instrument]
pub async fn inventory_item_validate_name(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Form(new_item): Form<NewItemName>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    let exists =
        models::inventory::InventoryItem::name_exists(&ctx, &state.database_pool, &new_item.name)
            .await?;

    Ok(view::inventory::InventoryNewItemFormName::build(
        Some(&new_item.name),
        exists,
    ))
}

#[tracing::instrument]
pub async fn inventory_item_create(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(new_item): Form<NewItem>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    if new_item.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id = models::inventory::InventoryItem::save(
        &ctx,
        &state.database_pool,
        &new_item.name,
        new_item.category_id,
        new_item.weight,
    )
    .await?;

    if htmx::is_htmx(&headers) {
        let inventory = models::inventory::Inventory::load(&ctx, &state.database_pool).await?;

        // it's impossible to NOT find the item here, as we literally just added
        // it.
        let active_category: Option<&models::inventory::Category> = Some(
            inventory
                .categories
                .iter()
                .find(|category| category.id == new_item.category_id)
                .unwrap(),
        );

        Ok(view::inventory::Inventory::build(
            active_category,
            &inventory.categories,
            state.client_state.edit_item,
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

#[tracing::instrument]
pub async fn inventory_item_delete(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    let deleted = models::inventory::InventoryItem::delete(&ctx, &state.database_pool, id).await?;

    if deleted {
        Ok(Redirect::to(get_referer(&headers)?))
    } else {
        Err(Error::Request(RequestError::NotFound {
            message: format!("item with id {id} not found"),
        }))
    }
}

#[tracing::instrument]
pub async fn inventory_item_edit(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(edit_item): Form<EditItem>,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    if edit_item.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let id = models::inventory::InventoryItem::update(
        &ctx,
        &state.database_pool,
        id,
        &edit_item.name,
        edit_item.weight,
    )
    .await?;

    Ok(Redirect::to(&format!("/inventory/category/{id}/")))
}

#[tracing::instrument]
pub async fn inventory_item_cancel(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    let id = models::inventory::InventoryItem::find(&ctx, &state.database_pool, id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("item with id {id} not found"),
        }))?;

    Ok(Redirect::to(&format!(
        "/inventory/category/{id}/",
        id = id.category.id
    )))
}

#[tracing::instrument]
pub async fn trip_create(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Form(new_trip): Form<NewTrip>,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    if new_trip.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let new_id = models::trips::Trip::save(
        &ctx,
        &state.database_pool,
        &new_trip.name,
        new_trip.date_start,
        new_trip.date_end,
        new_trip.copy_from,
    )
    .await?;

    Ok(Redirect::to(&format!("/trips/{new_id}/")))
}

#[tracing::instrument]
pub async fn trips(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    let trips = models::trips::Trip::all(&ctx, &state.database_pool).await?;

    if htmx::is_htmx(&headers) {
        Ok(view::root::Body::init(
            view::Parent::Root,
            view::root::BodyArgs {
                body: &view::trip::TripManager::build(trips),
                active_page: Some(&TopLevelPage::Trips),
            },
        )
        .build(&ctx))
    } else {
        Ok(view::Root::build(
            &ctx,
            &view::trip::TripManager::build(trips),
            Some(&TopLevelPage::Trips),
        ))
    }
}

#[tracing::instrument]
pub async fn trip(
    Extension(current_user): Extension<models::user::User>,
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(trip_query): Query<TripQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    state.client_state.trip_edit_attribute = trip_query.edit;
    state.client_state.active_category_id = trip_query.category;

    if let Some(delete_todo) = trip_query.delete_todo {
        let deleted = components::trips::todos::Todo::delete(
            &ctx,
            &state.database_pool,
            &todos::Reference {
                id: components::trips::todos::Id::new(delete_todo),
                container: todos::Container { trip_id: id },
            },
        )
        .await?;

        return if deleted {
            Ok(Redirect::to(get_referer(&headers)?).into_response())
        } else {
            Err(Error::Request(RequestError::NotFound {
                message: format!("todo with id {id} not found"),
            }))
        };
    }

    let mut trip: models::trips::Trip = models::trips::Trip::find(&ctx, &state.database_pool, id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("trip with id {id} not found"),
        }))?;

    trip.load_trips_types(&ctx, &state.database_pool).await?;

    trip.load_todos(&ctx, &state.database_pool).await?;

    trip.sync_trip_items_with_inventory(&ctx, &state.database_pool)
        .await?;

    trip.load_categories(&ctx, &state.database_pool).await?;

    let active_category: Option<&models::trips::TripCategory> = state
        .client_state
        .active_category_id
        .map(|id| {
            trip.categories()
                .iter()
                .find(|category| category.category.id == id)
                .ok_or(Error::Request(RequestError::NotFound {
                    message: format!("an active category with id {id} does not exist"),
                }))
        })
        .transpose()?;

    Ok(view::Root::build(
        &ctx,
        &view::trip::Trip::build(
            &trip,
            state.client_state.trip_edit_attribute.as_ref(),
            active_category,
            trip_query.edit_todo,
        ),
        Some(&TopLevelPage::Trips),
    )
    .into_response())
}

#[tracing::instrument]
pub async fn trip_type_remove(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    let found =
        models::trips::Trip::trip_type_remove(&ctx, &state.database_pool, trip_id, type_id).await?;

    if found {
        Ok(Redirect::to(&format!("/trips/{trip_id}/")))
    } else {
        Err(Error::Request(RequestError::NotFound {
            message: format!("type {type_id} is not active for trip {trip_id}"),
        }))
    }
}

#[tracing::instrument]
pub async fn trip_type_add(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    models::trips::Trip::trip_type_add(&ctx, &state.database_pool, trip_id, type_id).await?;

    Ok(Redirect::to(&format!("/trips/{trip_id}/")))
}

#[tracing::instrument]
pub async fn trip_comment_set(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
    Form(comment_update): Form<CommentUpdate>,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    let found = models::trips::Trip::set_comment(
        &ctx,
        &state.database_pool,
        trip_id,
        &comment_update.new_comment,
    )
    .await?;

    if found {
        Ok(Redirect::to(&format!("/trips/{trip_id}/")))
    } else {
        Err(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))
    }
}

#[tracing::instrument]
pub async fn trip_edit_attribute(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, attribute)): Path<(Uuid, models::trips::TripAttributeUpdate)>,
    Form(trip_update): Form<TripUpdate>,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    if let models::trips::TripAttributeUpdate::Name(ref s) = attribute {
        if s.is_empty() {
            return Err(Error::Request(RequestError::EmptyFormElement {
                name: "name".to_string(),
            }));
        }
    }
    models::trips::Trip::set_attribute(&ctx, &state.database_pool, trip_id, attribute).await?;

    Ok(Redirect::to(&format!("/trips/{trip_id}/")))
}

#[tracing::instrument]
pub async fn trip_item_set_state(
    ctx: &Context,
    state: &AppState,
    trip_id: Uuid,
    item_id: Uuid,
    key: models::trips::TripItemStateKey,
    value: bool,
) -> Result<(), Error> {
    models::trips::TripItem::set_state(ctx, &state.database_pool, trip_id, item_id, key, value)
        .await?;
    Ok(())
}

#[tracing::instrument]
pub async fn trip_row(
    ctx: &Context,
    state: &AppState,
    trip_id: Uuid,
    item_id: Uuid,
) -> Result<impl IntoResponse, Error> {
    let item = models::trips::TripItem::find(ctx, &state.database_pool, trip_id, item_id)
        .await?
        .ok_or_else(|| {
            Error::Request(RequestError::NotFound {
                message: format!("item with id {item_id} not found for trip {trip_id}"),
            })
        })?;

    let item_row = view::trip::TripItemListRow::build(
        trip_id,
        &item,
        models::inventory::InventoryItem::get_category_max_weight(
            ctx,
            &state.database_pool,
            item.item.category_id,
        )
        .await?,
    );

    let category = models::trips::TripCategory::find(
        ctx,
        &state.database_pool,
        trip_id,
        item.item.category_id,
    )
    .await?
    .ok_or_else(|| {
        Error::Request(RequestError::NotFound {
            message: format!("category with id {} not found", item.item.category_id),
        })
    })?;

    // TODO biggest_category_weight?
    let category_row = view::trip::TripCategoryListRow::build(trip_id, &category, true, 0, true);

    Ok(html::concat(&item_row, &category_row))
}

#[tracing::instrument]
pub async fn trip_item_set_pick(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        trip_item_set_state(
            &ctx,
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Pick,
            true,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
pub async fn trip_item_set_pick_htmx(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    trip_item_set_state(
        &ctx,
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pick,
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
pub async fn trip_item_set_unpick(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        trip_item_set_state(
            &ctx,
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Pick,
            false,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
pub async fn trip_item_set_unpick_htmx(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    trip_item_set_state(
        &ctx,
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pick,
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
pub async fn trip_item_set_pack(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        trip_item_set_state(
            &ctx,
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Pack,
            true,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
pub async fn trip_item_set_pack_htmx(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    trip_item_set_state(
        &ctx,
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
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
pub async fn trip_item_set_unpack(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        trip_item_set_state(
            &ctx,
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Pack,
            false,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
pub async fn trip_item_set_unpack_htmx(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    trip_item_set_state(
        &ctx,
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
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
pub async fn trip_item_set_ready(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        trip_item_set_state(
            &ctx,
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Ready,
            true,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
pub async fn trip_item_set_ready_htmx(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    trip_item_set_state(
        &ctx,
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
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
pub async fn trip_item_set_unready(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    Ok::<_, Error>(
        trip_item_set_state(
            &ctx,
            &state,
            trip_id,
            item_id,
            models::trips::TripItemStateKey::Ready,
            false,
        )
        .await?,
    )
    .map(|_| -> Result<Redirect, Error> { Ok(Redirect::to(get_referer(&headers)?)) })?
}

#[tracing::instrument]
pub async fn trip_item_set_unready_htmx(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    trip_item_set_state(
        &ctx,
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
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
pub async fn trip_total_weight_htmx(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    let total_weight =
        models::trips::Trip::find_total_picked_weight(&ctx, &state.database_pool, trip_id).await?;
    Ok(view::trip::TripInfoTotalWeightRow::build(
        trip_id,
        total_weight,
    ))
}

#[tracing::instrument]
pub async fn inventory_category_create(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Form(new_category): Form<NewCategory>,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    if new_category.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id =
        models::inventory::Category::save(&ctx, &state.database_pool, &new_category.name).await?;

    Ok(Redirect::to("/inventory/"))
}

#[tracing::instrument]
pub async fn trip_state_set(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((trip_id, new_state)): Path<(Uuid, models::trips::TripState)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    let exists =
        models::trips::Trip::set_state(&ctx, &state.database_pool, trip_id, &new_state).await?;

    if !exists {
        return Err(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }));
    }

    if htmx::is_htmx(&headers) {
        Ok(view::trip::TripInfoStateRow::build(&new_state).into_response())
    } else {
        Ok(Redirect::to(&format!("/trips/{trip_id}/")).into_response())
    }
}

#[tracing::instrument]
pub async fn trips_types(
    Extension(current_user): Extension<models::user::User>,
    State(mut state): State<AppState>,
    Query(trip_type_query): Query<TripTypeQuery>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    state.client_state.trip_type_edit = trip_type_query.edit;

    let trip_types: Vec<models::trips::TripsType> =
        models::trips::TripsType::all(&ctx, &state.database_pool).await?;

    Ok(view::Root::build(
        &ctx,
        &view::trip::types::TypeList::build(&state.client_state, trip_types),
        Some(&TopLevelPage::Trips),
    ))
}

#[tracing::instrument]
pub async fn trip_type_create(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Form(new_trip_type): Form<NewTripType>,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    if new_trip_type.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id =
        models::trips::TripsType::save(&ctx, &state.database_pool, &new_trip_type.name).await?;

    Ok(Redirect::to("/trips/types/"))
}

#[tracing::instrument]
pub async fn trips_types_edit_name(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(trip_type_id): Path<Uuid>,
    Form(trip_update): Form<TripTypeUpdate>,
) -> Result<Redirect, Error> {
    let ctx = Context::build(current_user);
    if trip_update.new_value.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let exists = models::trips::TripsType::set_name(
        &ctx,
        &state.database_pool,
        trip_type_id,
        &trip_update.new_value,
    )
    .await?;

    if exists {
        Ok(Redirect::to("/trips/types/"))
    } else {
        Err(Error::Request(RequestError::NotFound {
            message: format!("trip type with id {trip_type_id} not found"),
        }))
    }
}

#[tracing::instrument]
pub async fn inventory_item(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    let item = models::inventory::InventoryItem::find(&ctx, &state.database_pool, id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("inventory item with id {id} not found"),
        }))?;

    Ok(view::Root::build(
        &ctx,
        &view::inventory::InventoryItem::build(&state.client_state, &item),
        Some(&TopLevelPage::Inventory),
    ))
}

#[tracing::instrument]
pub async fn trip_category_select(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, category_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    let mut trip = models::trips::Trip::find(&ctx, &state.database_pool, trip_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))?;

    trip.load_categories(&ctx, &state.database_pool).await?;

    let active_category = trip
        .categories()
        .iter()
        .find(|c| c.category.id == category_id)
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("category with id {category_id} not found"),
        }))?;

    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        htmx::ResponseHeaders::PushUrl.into(),
        format!("?category={category_id}").parse().unwrap(),
    );

    Ok((
        headers,
        view::trip::TripItems::build(Some(active_category), &trip),
    ))
}

#[tracing::instrument]
pub async fn inventory_category_select(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    let inventory = models::inventory::Inventory::load(&ctx, &state.database_pool).await?;

    let active_category: Option<&models::inventory::Category> = Some(
        inventory
            .categories
            .iter()
            .find(|category| category.id == category_id)
            .ok_or(Error::Request(RequestError::NotFound {
                message: format!("a category with id {category_id} not found"),
            }))?,
    );

    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        htmx::ResponseHeaders::PushUrl.into(),
        format!("/inventory/category/{category_id}/")
            .parse()
            .unwrap(),
    );

    Ok((
        headers,
        view::inventory::Inventory::build(
            active_category,
            &inventory.categories,
            state.client_state.edit_item,
        ),
    ))
}

#[tracing::instrument]
pub async fn trip_packagelist(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    let mut trip = models::trips::Trip::find(&ctx, &state.database_pool, trip_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))?;

    trip.load_categories(&ctx, &state.database_pool).await?;

    Ok(view::Root::build(
        &ctx,
        &view::trip::packagelist::TripPackageList::build(&trip),
        Some(&TopLevelPage::Trips),
    ))
}

#[tracing::instrument]
pub async fn trip_item_packagelist_set_pack_htmx(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    trip_item_set_state(
        &ctx,
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
        true,
    )
    .await?;

    let item = models::trips::TripItem::find(&ctx, &state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowReady::build(
        trip_id, &item,
    ))
}

#[tracing::instrument]
pub async fn trip_item_packagelist_set_unpack_htmx(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    trip_item_set_state(
        &ctx,
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
        false,
    )
    .await?;

    // note that this cannot fail due to a missing item, as trip_item_set_state would already
    // return 404. but error handling cannot hurt ;)
    let item = models::trips::TripItem::find(&ctx, &state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowReady::build(
        trip_id, &item,
    ))
}

#[tracing::instrument]
pub async fn trip_item_packagelist_set_ready_htmx(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    trip_item_set_state(
        &ctx,
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
        true,
    )
    .await?;

    let item = models::trips::TripItem::find(&ctx, &state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowUnready::build(
        trip_id, &item,
    ))
}

#[tracing::instrument]
pub async fn trip_item_packagelist_set_unready_htmx(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    trip_item_set_state(
        &ctx,
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
        false,
    )
    .await?;

    // note that this cannot fail due to a missing item, as trip_item_set_state would already
    // return 404. but error handling cannot hurt ;)
    let item = models::trips::TripItem::find(&ctx, &state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowUnready::build(
        trip_id, &item,
    ))
}
