use axum::{
    extract::{Extension, Path, Query, State},
    http::header::{self, HeaderMap, HeaderName},
    response::{IntoResponse, Redirect},
    Form,
};

use serde::Deserialize;
use uuid::Uuid;

use crate::models;
use crate::view;
use crate::{html, AppState, Context, Error, HtmxEvents, RequestError, TopLevelPage};

use super::{get_referer, is_htmx, HtmxResponseHeaders};

#[derive(Deserialize, Default)]
pub struct InventoryQuery {
    edit_item: Option<Uuid>,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct NewItemName {
    #[serde(rename = "new-item-name")]
    name: String,
}

#[derive(Deserialize)]
pub struct EditItem {
    #[serde(rename = "edit-item-name")]
    name: String,
    #[serde(rename = "edit-item-weight")]
    weight: u32,
}

#[derive(Deserialize)]
pub struct NewTrip {
    #[serde(rename = "new-trip-name")]
    name: String,
    #[serde(rename = "new-trip-start-date")]
    date_start: time::Date,
    #[serde(rename = "new-trip-end-date")]
    date_end: time::Date,
}

#[derive(Debug, Deserialize)]
pub struct TripQuery {
    edit: Option<models::trips::TripAttribute>,
    category: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CommentUpdate {
    #[serde(rename = "new-comment")]
    new_comment: String,
}

#[derive(Deserialize)]
pub struct TripUpdate {
    #[serde(rename = "new-value")]
    new_value: String,
}

#[derive(Deserialize)]
pub struct NewCategory {
    #[serde(rename = "new-category-name")]
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct TripTypeQuery {
    edit: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct NewTripType {
    #[serde(rename = "new-trip-type-name")]
    name: String,
}

#[derive(Deserialize)]
pub struct TripTypeUpdate {
    #[serde(rename = "new-value")]
    new_value: String,
}

pub async fn root(Extension(current_user): Extension<models::user::User>) -> impl IntoResponse {
    view::Root::build(
        &Context::build(current_user),
        &view::home::Home::build(),
        None,
    )
}

pub async fn icon() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/svg+xml")],
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/luggage.svg")),
    )
}

pub async fn debug(headers: HeaderMap) -> impl IntoResponse {
    let out = {
        let mut out = String::new();
        for (key, value) in headers.iter() {
            out.push_str(&format!("{}: {}\n", key, value.to_str().unwrap()));
        }
        out
    };
    out
}
pub async fn inventory_active(
    Extension(current_user): Extension<models::user::User>,
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(inventory_query): Query<InventoryQuery>,
) -> Result<impl IntoResponse, Error> {
    state.client_state.edit_item = inventory_query.edit_item;
    state.client_state.active_category_id = Some(id);

    let inventory = models::inventory::Inventory::load(&state.database_pool).await?;

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
        &Context::build(current_user),
        &view::inventory::Inventory::build(
            active_category,
            &inventory.categories,
            state.client_state.edit_item,
        ),
        Some(&TopLevelPage::Inventory),
    ))
}

pub async fn inventory_inactive(
    Extension(current_user): Extension<models::user::User>,
    State(mut state): State<AppState>,
    Query(inventory_query): Query<InventoryQuery>,
) -> Result<impl IntoResponse, Error> {
    state.client_state.edit_item = inventory_query.edit_item;
    state.client_state.active_category_id = None;

    let inventory = models::inventory::Inventory::load(&state.database_pool).await?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::inventory::Inventory::build(
            None,
            &inventory.categories,
            state.client_state.edit_item,
        ),
        Some(&TopLevelPage::Inventory),
    ))
}

pub async fn inventory_item_validate_name(
    State(state): State<AppState>,
    Form(new_item): Form<NewItemName>,
) -> Result<impl IntoResponse, Error> {
    let exists =
        models::inventory::InventoryItem::name_exists(&state.database_pool, &new_item.name).await?;

    Ok(view::inventory::InventoryNewItemFormName::build(
        Some(&new_item.name),
        exists,
    ))
}

pub async fn inventory_item_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(new_item): Form<NewItem>,
) -> Result<impl IntoResponse, Error> {
    if new_item.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id = models::inventory::InventoryItem::save(
        &state.database_pool,
        &new_item.name,
        new_item.category_id,
        new_item.weight,
    )
    .await?;

    if is_htmx(&headers) {
        let inventory = models::inventory::Inventory::load(&state.database_pool).await?;

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
pub async fn inventory_item_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Redirect, Error> {
    let deleted = models::inventory::InventoryItem::delete(&state.database_pool, id).await?;

    if !deleted {
        Err(Error::Request(RequestError::NotFound {
            message: format!("item with id {id} not found"),
        }))
    } else {
        Ok(Redirect::to(get_referer(&headers)?))
    }
}

pub async fn inventory_item_edit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(edit_item): Form<EditItem>,
) -> Result<Redirect, Error> {
    if edit_item.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let id = models::inventory::InventoryItem::update(
        &state.database_pool,
        id,
        &edit_item.name,
        edit_item.weight,
    )
    .await?;

    Ok(Redirect::to(&format!("/inventory/category/{id}/", id = id)))
}

pub async fn inventory_item_cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Redirect, Error> {
    let id = models::inventory::InventoryItem::find(&state.database_pool, id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("item with id {id} not found"),
        }))?;

    Ok(Redirect::to(&format!(
        "/inventory/category/{id}/",
        id = id.category.id
    )))
}

pub async fn trip_create(
    State(state): State<AppState>,
    Form(new_trip): Form<NewTrip>,
) -> Result<Redirect, Error> {
    if new_trip.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let new_id = models::trips::Trip::save(
        &state.database_pool,
        &new_trip.name,
        new_trip.date_start,
        new_trip.date_end,
    )
    .await?;

    Ok(Redirect::to(&format!("/trips/{new_id}/")))
}

pub async fn trips(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, Error> {
    let trips = models::trips::Trip::all(&state.database_pool).await?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::trip::TripManager::build(trips),
        Some(&TopLevelPage::Trips),
    ))
}

pub async fn trip(
    Extension(current_user): Extension<models::user::User>,
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(trip_query): Query<TripQuery>,
) -> Result<impl IntoResponse, Error> {
    state.client_state.trip_edit_attribute = trip_query.edit;
    state.client_state.active_category_id = trip_query.category;

    let mut trip: models::trips::Trip = models::trips::Trip::find(&state.database_pool, id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("trip with id {id} not found"),
        }))?;

    trip.load_trips_types(&state.database_pool).await?;

    trip.sync_trip_items_with_inventory(&state.database_pool)
        .await?;

    trip.load_categories(&state.database_pool).await?;

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
        &Context::build(current_user),
        &view::trip::Trip::build(
            &trip,
            state.client_state.trip_edit_attribute,
            active_category,
        ),
        Some(&TopLevelPage::Trips),
    ))
}

pub async fn trip_type_remove(
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, Error> {
    let found =
        models::trips::Trip::trip_type_remove(&state.database_pool, trip_id, type_id).await?;

    if !found {
        Err(Error::Request(RequestError::NotFound {
            message: format!("type {type_id} is not active for trip {trip_id}"),
        }))
    } else {
        Ok(Redirect::to(&format!("/trips/{trip_id}/")))
    }
}

pub async fn trip_type_add(
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, Error> {
    models::trips::Trip::trip_type_add(&state.database_pool, trip_id, type_id).await?;

    Ok(Redirect::to(&format!("/trips/{trip_id}/")))
}

pub async fn trip_comment_set(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
    Form(comment_update): Form<CommentUpdate>,
) -> Result<Redirect, Error> {
    let found = models::trips::Trip::set_comment(
        &state.database_pool,
        trip_id,
        &comment_update.new_comment,
    )
    .await?;

    if !found {
        Err(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))
    } else {
        Ok(Redirect::to(&format!("/trips/{id}/", id = trip_id)))
    }
}

pub async fn trip_edit_attribute(
    State(state): State<AppState>,
    Path((trip_id, attribute)): Path<(Uuid, models::trips::TripAttribute)>,
    Form(trip_update): Form<TripUpdate>,
) -> Result<Redirect, Error> {
    if attribute == models::trips::TripAttribute::Name {
        if trip_update.new_value.is_empty() {
            return Err(Error::Request(RequestError::EmptyFormElement {
                name: "name".to_string(),
            }));
        }
    }
    models::trips::Trip::set_attribute(
        &state.database_pool,
        trip_id,
        attribute,
        &trip_update.new_value,
    )
    .await?;

    Ok(Redirect::to(&format!("/trips/{trip_id}/")))
}

pub async fn trip_item_set_state(
    state: &AppState,
    trip_id: Uuid,
    item_id: Uuid,
    key: models::trips::TripItemStateKey,
    value: bool,
) -> Result<(), Error> {
    models::trips::TripItem::set_state(&state.database_pool, trip_id, item_id, key, value).await?;
    Ok(())
}

pub async fn trip_row(
    state: &AppState,
    trip_id: Uuid,
    item_id: Uuid,
) -> Result<impl IntoResponse, Error> {
    let item = models::trips::TripItem::find(&state.database_pool, trip_id, item_id)
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
            &state.database_pool,
            item.item.category_id,
        )
        .await?,
    );

    let category =
        models::trips::TripCategory::find(&state.database_pool, trip_id, item.item.category_id)
            .await?
            .ok_or_else(|| {
                Error::Request(RequestError::NotFound {
                    message: format!("category with id {} not found", item.item.category_id),
                })
            })?;

    // TODO biggest_category_weight?
    let category_row = view::trip::TripCategoryListRow::build(trip_id, &category, true, 0, true);

    Ok(html::concat(item_row, category_row))
}

pub async fn trip_item_set_pick(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
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

pub async fn trip_item_set_pick_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pick,
        true,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

pub async fn trip_item_set_unpick(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
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

pub async fn trip_item_set_unpick_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pick,
        false,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

pub async fn trip_item_set_pack(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
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

pub async fn trip_item_set_pack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
        true,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

pub async fn trip_item_set_unpack(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
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

pub async fn trip_item_set_unpack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
        false,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

pub async fn trip_item_set_ready(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
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

pub async fn trip_item_set_ready_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
        true,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

pub async fn trip_item_set_unready(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Redirect, Error> {
    Ok::<_, Error>(
        trip_item_set_state(
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

pub async fn trip_item_set_unready_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
        false,
    )
    .await?;
    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::Trigger.into(),
        HtmxEvents::TripItemEdited.into(),
    );
    Ok((headers, trip_row(&state, trip_id, item_id).await?))
}

pub async fn trip_total_weight_htmx(
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let total_weight =
        models::trips::Trip::find_total_picked_weight(&state.database_pool, trip_id).await?;
    Ok(view::trip::TripInfoTotalWeightRow::build(
        trip_id,
        total_weight,
    ))
}

pub async fn inventory_category_create(
    State(state): State<AppState>,
    Form(new_category): Form<NewCategory>,
) -> Result<Redirect, Error> {
    if new_category.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id =
        models::inventory::Category::save(&state.database_pool, &new_category.name).await?;

    Ok(Redirect::to("/inventory/"))
}

pub async fn trip_state_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((trip_id, new_state)): Path<(Uuid, models::trips::TripState)>,
) -> Result<impl IntoResponse, Error> {
    let exists = models::trips::Trip::set_state(&state.database_pool, trip_id, &new_state).await?;

    if !exists {
        return Err(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }));
    }

    if is_htmx(&headers) {
        Ok(view::trip::TripInfoStateRow::build(&new_state).into_response())
    } else {
        Ok(Redirect::to(&format!("/trips/{id}/", id = trip_id)).into_response())
    }
}
pub async fn trips_types(
    Extension(current_user): Extension<models::user::User>,
    State(mut state): State<AppState>,
    Query(trip_type_query): Query<TripTypeQuery>,
) -> Result<impl IntoResponse, Error> {
    state.client_state.trip_type_edit = trip_type_query.edit;

    let trip_types: Vec<models::trips::TripsType> =
        models::trips::TripsType::all(&state.database_pool).await?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::trip::types::TypeList::build(&state.client_state, trip_types),
        Some(&TopLevelPage::Trips),
    ))
}
pub async fn trip_type_create(
    State(state): State<AppState>,
    Form(new_trip_type): Form<NewTripType>,
) -> Result<Redirect, Error> {
    if new_trip_type.name.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id = models::trips::TripsType::save(&state.database_pool, &new_trip_type.name).await?;

    Ok(Redirect::to("/trips/types/"))
}
pub async fn trips_types_edit_name(
    State(state): State<AppState>,
    Path(trip_type_id): Path<Uuid>,
    Form(trip_update): Form<TripTypeUpdate>,
) -> Result<Redirect, Error> {
    if trip_update.new_value.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let exists = models::trips::TripsType::set_name(
        &state.database_pool,
        trip_type_id,
        &trip_update.new_value,
    )
    .await?;

    if !exists {
        return Err(Error::Request(RequestError::NotFound {
            message: format!("trip type with id {trip_type_id} not found"),
        }));
    } else {
        Ok(Redirect::to("/trips/types/"))
    }
}

pub async fn inventory_item(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let item = models::inventory::InventoryItem::find(&state.database_pool, id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("inventory item with id {id} not found"),
        }))?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::inventory::InventoryItem::build(&state.client_state, &item),
        Some(&TopLevelPage::Inventory),
    ))
}

pub async fn trip_category_select(
    State(state): State<AppState>,
    Path((trip_id, category_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let mut trip = models::trips::Trip::find(&state.database_pool, trip_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))?;

    trip.load_categories(&state.database_pool).await?;

    let active_category = trip
        .categories()
        .iter()
        .find(|c| c.category.id == category_id)
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("category with id {category_id} not found"),
        }))?;

    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        HtmxResponseHeaders::PushUrl.into(),
        format!("?={category_id}").parse().unwrap(),
    );

    Ok((
        headers,
        view::trip::TripItems::build(Some(active_category), &trip),
    ))
}

pub async fn inventory_category_select(
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let inventory = models::inventory::Inventory::load(&state.database_pool).await?;

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
        HtmxResponseHeaders::PushUrl.into(),
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

pub async fn trip_packagelist(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let mut trip = models::trips::Trip::find(&state.database_pool, trip_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))?;

    trip.load_categories(&state.database_pool).await?;

    Ok(view::Root::build(
        &Context::build(current_user),
        &view::trip::packagelist::TripPackageList::build(&trip),
        Some(&TopLevelPage::Trips),
    ))
}

pub async fn trip_item_packagelist_set_pack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
        true,
    )
    .await?;

    let item = models::trips::TripItem::find(&state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowReady::build(
        trip_id, &item,
    ))
}

pub async fn trip_item_packagelist_set_unpack_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Pack,
        false,
    )
    .await?;

    // note that this cannot fail due to a missing item, as trip_item_set_state would already
    // return 404. but error handling cannot hurt ;)
    let item = models::trips::TripItem::find(&state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowReady::build(
        trip_id, &item,
    ))
}

pub async fn trip_item_packagelist_set_ready_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
        true,
    )
    .await?;

    let item = models::trips::TripItem::find(&state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowUnready::build(
        trip_id, &item,
    ))
}

pub async fn trip_item_packagelist_set_unready_htmx(
    State(state): State<AppState>,
    Path((trip_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    trip_item_set_state(
        &state,
        trip_id,
        item_id,
        models::trips::TripItemStateKey::Ready,
        false,
    )
    .await?;

    // note that this cannot fail due to a missing item, as trip_item_set_state would already
    // return 404. but error handling cannot hurt ;)
    let item = models::trips::TripItem::find(&state.database_pool, trip_id, item_id)
        .await?
        .ok_or(Error::Request(RequestError::NotFound {
            message: format!("an item with id {item_id} does not exist"),
        }))?;

    Ok(view::trip::packagelist::TripPackageListRowUnready::build(
        trip_id, &item,
    ))
}
