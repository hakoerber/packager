use axum::{
    extract::{Extension, Path, Query, State},
    http::header::{self, HeaderMap, HeaderName},
    response::{IntoResponse, Redirect},
    Form,
};

use crate::view::Component;

use serde::Deserialize;
use uuid::Uuid;

use crate::htmx;
use crate::models;
use crate::view;
use crate::{AppState, Context, Error, RequestError, TopLevelPage};

use super::get_referer;

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
    for (key, value) in &headers {
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
