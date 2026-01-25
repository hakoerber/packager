use axum::{
    Form, Router,
    extract::{Extension, Path, Query, State},
    http::header::{HeaderMap, HeaderName},
    response::{IntoResponse, Redirect},
    routing::{get, post},
};

use serde::Deserialize;
use time::Date;
use uuid::Uuid;

use crate::{
    AppState, Context, RunError, RequestError, TopLevelPage,
    domains::{crud::Delete as _, route::Router as _, trips::todos},
    htmx,
    routing::{get_referer, uuid_or_empty},
    view::Component,
};

use super::{model, view};
use crate::models::User;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NewTrip {
    #[serde(rename = "new-trip-name")]
    name: String,
    #[serde(rename = "new-trip-start-date")]
    date_start: Date,
    #[serde(rename = "new-trip-end-date")]
    date_end: Date,
    #[serde(rename = "new-trip-copy-from", deserialize_with = "uuid_or_empty")]
    copy_from: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TripQuery {
    edit: Option<model::TripAttribute>,
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
pub async fn create(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Form(new_trip): Form<NewTrip>,
) -> Result<Redirect, RunError> {
    let ctx = Context::build(current_user);
    if new_trip.name.is_empty() {
        return Err(RunError::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let new_id = model::Trip::save(
        &ctx,
        &state.database_pool,
        &new_trip.name,
        model::TripDate {
            start: new_trip.date_start,
            end: new_trip.date_end,
        },
        new_trip.copy_from,
    )
    .await?;

    Ok(Redirect::to(&format!("/trips/{new_id}/")))
}

#[tracing::instrument]
pub async fn trips(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    let trips = model::Trip::all(&ctx, &state.database_pool).await?;

    if htmx::is_htmx(&headers) {
        Ok(crate::view::root::Body::init(
            crate::view::Parent::Root,
            crate::view::root::BodyArgs {
                body: &view::TripManager::build(&trips),
                active_page: Some(&TopLevelPage::Trips),
            },
        )
        .build(&ctx))
    } else {
        Ok(crate::view::Root::build(
            &ctx,
            &view::TripManager::build(&trips),
            Some(&TopLevelPage::Trips),
        ))
    }
}

#[tracing::instrument]
pub async fn trip(
    Extension(current_user): Extension<User>,
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(trip_query): Query<TripQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    state.client_state.trip_edit_attribute = trip_query.edit;
    state.client_state.active_category_id = trip_query.category;

    if let Some(delete_todo) = trip_query.delete_todo {
        let deleted = todos::Todo::delete(
            &ctx,
            &state.database_pool,
            &todos::Reference {
                id: todos::Id::new(delete_todo),
                container: todos::Container { trip_id: id },
            },
        )
        .await?;

        return if deleted {
            Ok(Redirect::to(get_referer(&headers)?).into_response())
        } else {
            Err(RunError::Request(RequestError::NotFound {
                message: format!("todo with id {id} not found"),
            }))
        };
    }

    let mut trip = model::Trip::find(&ctx, &state.database_pool, id)
        .await?
        .ok_or(RunError::Request(RequestError::NotFound {
            message: format!("trip with id {id} not found"),
        }))?;

    trip.load_trip_types(&ctx, &state.database_pool).await?;

    trip.load_todos(&ctx, &state.database_pool).await?;

    trip.sync_trip_items_with_inventory(&ctx, &state.database_pool)
        .await?;

    trip.load_categories(&ctx, &state.database_pool).await?;

    let active_category: Option<&model::TripCategory> = state
        .client_state
        .active_category_id
        .map(|id| {
            trip.categories()
                .iter()
                .find(|category| category.category.id == id)
                .ok_or(RunError::Request(RequestError::NotFound {
                    message: format!("an active category with id {id} does not exist"),
                }))
        })
        .transpose()?;

    Ok(crate::view::Root::build(
        &ctx,
        &view::Trip::build(
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
pub async fn remove_type(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, RunError> {
    let ctx = Context::build(current_user);
    let found = model::Trip::trip_type_remove(&ctx, &state.database_pool, trip_id, type_id).await?;

    if found {
        Ok(Redirect::to(&format!("/trips/{trip_id}/")))
    } else {
        Err(RunError::Request(RequestError::NotFound {
            message: format!("type {type_id} is not active for trip {trip_id}"),
        }))
    }
}

#[tracing::instrument]
pub async fn add_type(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, type_id)): Path<(Uuid, Uuid)>,
) -> Result<Redirect, RunError> {
    let ctx = Context::build(current_user);
    model::Trip::trip_type_add(&ctx, &state.database_pool, trip_id, type_id).await?;

    Ok(Redirect::to(&format!("/trips/{trip_id}/")))
}

#[tracing::instrument]
pub async fn set_comment(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
    Form(comment_update): Form<CommentUpdate>,
) -> Result<Redirect, RunError> {
    let ctx = Context::build(current_user);
    let found = model::Trip::set_comment(
        &ctx,
        &state.database_pool,
        trip_id,
        &comment_update.new_comment,
    )
    .await?;

    if found {
        Ok(Redirect::to(&format!("/trips/{trip_id}/")))
    } else {
        Err(RunError::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))
    }
}

#[tracing::instrument]
pub async fn total_weight_htmx(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path(trip_id): Path<Uuid>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    let total_weight =
        model::Trip::find_total_picked_weight(&ctx, &state.database_pool, trip_id).await?;
    Ok(view::TripInfoTotalWeightRow::build(trip_id, total_weight))
}

#[tracing::instrument]
pub async fn set_state(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((trip_id, new_state)): Path<(Uuid, model::TripState)>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    let exists = model::Trip::set_state(&ctx, &state.database_pool, trip_id, &new_state).await?;

    if !exists {
        return Err(RunError::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }));
    }

    if htmx::is_htmx(&headers) {
        Ok(view::TripInfoStateRow::build(&new_state).into_response())
    } else {
        Ok(Redirect::to(&format!("/trips/{trip_id}/")).into_response())
    }
}

#[tracing::instrument]
pub async fn trip_types(
    Extension(current_user): Extension<User>,
    State(mut state): State<AppState>,
    Query(trip_type_query): Query<TripTypeQuery>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    state.client_state.trip_type_edit = trip_type_query.edit;

    let trip_types: Vec<model::TripsType> =
        model::TripsType::all(&ctx, &state.database_pool).await?;

    Ok(crate::view::Root::build(
        &ctx,
        &view::types::TypeList::build(&state.client_state, trip_types),
        Some(&TopLevelPage::Trips),
    ))
}

#[tracing::instrument]
pub async fn create_type(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Form(new_trip_type): Form<NewTripType>,
) -> Result<Redirect, RunError> {
    let ctx = Context::build(current_user);
    if new_trip_type.name.is_empty() {
        return Err(RunError::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let _new_id = model::TripsType::save(&ctx, &state.database_pool, &new_trip_type.name).await?;

    Ok(Redirect::to("/trips/types/"))
}

#[tracing::instrument]
pub async fn edit_type_name(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path(trip_type_id): Path<Uuid>,
    Form(trip_update): Form<TripTypeUpdate>,
) -> Result<Redirect, RunError> {
    let ctx = Context::build(current_user);
    if trip_update.new_value.is_empty() {
        return Err(RunError::Request(RequestError::EmptyFormElement {
            name: "name".to_string(),
        }));
    }

    let exists = model::TripsType::set_name(
        &ctx,
        &state.database_pool,
        trip_type_id,
        &trip_update.new_value,
    )
    .await?;

    if exists {
        Ok(Redirect::to("/trips/types/"))
    } else {
        Err(RunError::Request(RequestError::NotFound {
            message: format!("trip type with id {trip_type_id} not found"),
        }))
    }
}

#[tracing::instrument]
pub async fn select_category(
    Extension(current_user): Extension<User>,
    State(state): State<AppState>,
    Path((trip_id, category_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    let mut trip = model::Trip::find(&ctx, &state.database_pool, trip_id)
        .await?
        .ok_or(RunError::Request(RequestError::NotFound {
            message: format!("trip with id {trip_id} not found"),
        }))?;

    trip.load_categories(&ctx, &state.database_pool).await?;

    let active_category = trip
        .categories()
        .iter()
        .find(|c| c.category.id == category_id)
        .ok_or(RunError::Request(RequestError::NotFound {
            message: format!("category with id {category_id} not found"),
        }))?;

    let mut headers = HeaderMap::new();
    headers.insert::<HeaderName>(
        htmx::ResponseHeaders::PushUrl.into(),
        format!("?category={category_id}").parse().unwrap(),
    );

    Ok((
        headers,
        view::TripItems::build(Some(active_category), &trip),
    ))
}

pub fn router() -> Router<AppState> {
    Router::new().nest(
        (&TopLevelPage::Trips.path()).into(),
        Router::new()
            .route("/", get(trips).post(create))
            .route("/types/", get(trip_types).post(create_type))
            .route("/types/{id}/edit/name/submit", post(edit_type_name))
            .route("/{id}/", get(trip))
            .route("/{id}/comment/submit", post(set_comment))
            .route("/{id}/categories/{id}/select", post(select_category))
            .route("/{id}/state/{id}", post(set_state))
            .route("/{id}/total_weight", get(total_weight_htmx))
            .route("/{id}/type/{id}/add", get(add_type))
            .route("/{id}/type/{id}/remove", get(remove_type))
            .nest("/{id}/packagelist/", super::packagelist::router())
            .nest("/{id}/edit/", model::routes::router())
            .nest("/{id}/items/", super::items::router())
            .nest("/{id}/todo/", todos::Todo::router()),
    )
}
