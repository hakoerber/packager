use axum::{
    extract::{Extension, Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};

use uuid::Uuid;

use crate::models;
use crate::{AppState, Context, RunError, RequestError, TopLevelPage};

use super::{model, view};

#[tracing::instrument]
pub async fn product(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, RunError> {
    let ctx = Context::build(current_user);
    let product = model::Product::find(&ctx, &state.database_pool, id)
        .await?
        .ok_or(RunError::Request(RequestError::NotFound {
            message: format!("product with id {id} not found"),
        }))?;

    Ok(crate::view::Root::build(
        &ctx,
        &view::Product::build(&product),
        Some(&TopLevelPage::Inventory),
    ))
}

pub fn router() -> Router<AppState> {
    Router::new().nest(
        (&TopLevelPage::Products.path()).into(),
        Router::new()
            .route("/{id}", get(product))
            .nest("/{id}/comments/", super::comments::routes::router()),
    )
}
