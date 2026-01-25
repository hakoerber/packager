use axum::{
    extract::{Extension, Path, Query, State},
    http::header::{HeaderMap, HeaderName},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};

use crate::routing::get_referer;
use crate::view::Component;

use serde::Deserialize;
use uuid::Uuid;

use crate::htmx;
use crate::models;
use crate::{AppState, Context, RunError, RequestError, TopLevelPage};

use super::{model, view};

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
pub struct NewCategory {
    #[serde(rename = "new-category-name")]
    name: String,
}

#[tracing::instrument]
pub async fn active(
    Extension(current_user): Extension<models::user::User>,
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(inventory_query): Query<InventoryQuery>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    state.client_state.edit_item = inventory_query.edit_item;
    state.client_state.active_category_id = Some(id);

    let inventory = model::Inventory::load(&ctx, &state.database_pool).await?;

    let active_category: Option<&model::Category> = state
        .client_state
        .active_category_id
        .map(|id| {
            inventory
                .categories
                .iter()
                .find(|category| category.id == id)
                .ok_or(RunError::Request(RequestError::NotFound {
                    message: format!("a category with id {id} does not exist"),
                }))
        })
        .transpose()?;

    Ok(crate::view::Root::build(
        &ctx,
        &view::Inventory::build(
            active_category,
            &inventory.categories,
            state.client_state.edit_item,
        ),
        Some(&TopLevelPage::Inventory),
    ))
}

#[tracing::instrument]
pub async fn inactive(
    Extension(current_user): Extension<models::user::User>,
    State(mut state): State<AppState>,
    Query(inventory_query): Query<InventoryQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    state.client_state.edit_item = inventory_query.edit_item;
    state.client_state.active_category_id = None;
    let inventory = model::Inventory::load(&ctx, &state.database_pool).await?;

    if htmx::is_htmx(&headers) {
        Ok(crate::view::root::Body::init(
            crate::view::Parent::Root,
            crate::view::root::BodyArgs {
                body: &view::Inventory::build(
                    None,
                    &inventory.categories,
                    state.client_state.edit_item,
                ),

                active_page: Some(&TopLevelPage::Inventory),
            },
        )
        .build(&ctx))
    } else {
        Ok(crate::view::Root::build(
            &ctx,
            &view::Inventory::build(None, &inventory.categories, state.client_state.edit_item),
            Some(&TopLevelPage::Inventory),
        ))
    }
}

#[tracing::instrument]
pub async fn item_validate_name(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Form(new_item): Form<NewItemName>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    let exists =
        model::InventoryItem::name_exists(&ctx, &state.database_pool, &new_item.name).await?;

    Ok(view::InventoryNewItemFormName::build(
        Some(&new_item.name),
        exists,
    ))
}

#[tracing::instrument]
pub async fn create_item(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(new_item): Form<NewItem>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    if new_item.name.is_empty() {
        return Err(RunError::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id = model::InventoryItem::save(
        &ctx,
        &state.database_pool,
        &new_item.name,
        new_item.category_id,
        new_item.weight,
    )
    .await?;

    if htmx::is_htmx(&headers) {
        let inventory = model::Inventory::load(&ctx, &state.database_pool).await?;

        // it's impossible to NOT find the item here, as we literally just added
        // it.
        let active_category: Option<&model::Category> = Some(
            inventory
                .categories
                .iter()
                .find(|category| category.id == new_item.category_id)
                .unwrap(),
        );

        Ok(view::Inventory::build(
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
pub async fn item_delete(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Redirect, RunError> {
    let ctx = Context::build(current_user);
    let deleted = model::InventoryItem::delete(&ctx, &state.database_pool, id).await?;

    if deleted {
        Ok(Redirect::to(get_referer(&headers)?))
    } else {
        Err(RunError::Request(RequestError::NotFound {
            message: format!("item with id {id} not found"),
        }))
    }
}

#[tracing::instrument]
pub async fn item_edit(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(edit_item): Form<EditItem>,
) -> Result<Redirect, RunError> {
    let ctx = Context::build(current_user);
    if edit_item.name.is_empty() {
        return Err(RunError::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let id = model::InventoryItem::update(
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
pub async fn item_cancel(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Redirect, RunError> {
    let ctx = Context::build(current_user);
    let id = model::InventoryItem::find(&ctx, &state.database_pool, id)
        .await?
        .ok_or(RunError::Request(RequestError::NotFound {
            message: format!("item with id {id} not found"),
        }))?;

    Ok(Redirect::to(&format!(
        "/inventory/category/{id}/",
        id = id.category.id
    )))
}

#[tracing::instrument]
pub async fn create_category(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Form(new_category): Form<NewCategory>,
) -> Result<Redirect, RunError> {
    let ctx = Context::build(current_user);
    if new_category.name.is_empty() {
        return Err(RunError::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id = model::Category::save(&ctx, &state.database_pool, &new_category.name).await?;

    Ok(Redirect::to("/inventory/"))
}

#[tracing::instrument]
pub async fn item(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    let item = model::InventoryItem::find(&ctx, &state.database_pool, id)
        .await?
        .ok_or(RunError::Request(RequestError::NotFound {
            message: format!("inventory item with id {id} not found"),
        }))?;

    Ok(crate::view::Root::build(
        &ctx,
        &view::InventoryItem::build(&state.client_state, &item),
        Some(&TopLevelPage::Inventory),
    ))
}

#[tracing::instrument]
pub async fn select_category(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    let inventory = model::Inventory::load(&ctx, &state.database_pool).await?;

    let active_category: Option<&model::Category> = Some(
        inventory
            .categories
            .iter()
            .find(|category| category.id == category_id)
            .ok_or(RunError::Request(RequestError::NotFound {
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
        view::Inventory::build(
            active_category,
            &inventory.categories,
            state.client_state.edit_item,
        ),
    ))
}

pub fn router() -> Router<AppState> {
    Router::new().nest(
        (&TopLevelPage::Inventory.path()).into(),
        Router::new()
            .route("/", get(inactive))
            .route("/categories/{id}/select", post(select_category))
            .route("/category/", post(create_category))
            .route("/category/{id}/", get(active))
            .route("/item/", post(create_item))
            .route("/item/{id}/", get(item))
            .route("/item/{id}/cancel", get(item_cancel))
            .route("/item/{id}/delete", get(item_delete))
            .route("/item/{id}/edit", post(item_edit))
            .route("/item/name/validate", post(item_validate_name)),
    )
}
