use axum::{
    extract::{Extension, Path, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use uuid::Uuid;

use crate::{models, AppState, Context, Error, RequestError, TopLevelPage};

#[tracing::instrument]
pub(crate) async fn comment_create(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Form(new_comment): Form<super::model::NewComment>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);

    if new_comment.content.is_empty() {
        return Err(Error::Request(RequestError::EmptyFormElement {
            name: "content".to_string(),
        }));
    }

    let _new_id =
        super::model::Comment::new(&ctx, &state.database_pool, product_id, new_comment).await?;

    Ok(Redirect::to(&format!("/products/{product_id}")))
}

#[tracing::instrument]
pub(crate) async fn comment_delete(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((product_id, comment_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);

    let deleted =
        super::model::Comment::delete(&ctx, &state.database_pool, product_id, comment_id).await?;

    if deleted {
        Ok(Redirect::to(&format!("/products/{product_id}")))
    } else {
        Err(Error::Request(RequestError::NotFound {
            message: format!("comment with id {comment_id} not found"),
        }))
    }
}

#[tracing::instrument]
pub(crate) async fn comment_edit(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((product_id, comment_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);
    let comment = super::model::Comment::find(&ctx, &state.database_pool, product_id, comment_id)
        .await?
        .ok_or_else(|| {
            crate::Error::Request(RequestError::NotFound {
                message: format!("comment with id {comment_id} not found"),
            })
        })?;

    Ok(crate::view::Root::build(
        &ctx,
        &super::view::EditComment::build(product_id, &comment),
        Some(&TopLevelPage::Inventory),
    ))
}

#[tracing::instrument]
pub(crate) async fn comment_edit_save(
    Extension(current_user): Extension<models::user::User>,
    State(state): State<AppState>,
    Path((product_id, comment_id)): Path<(Uuid, Uuid)>,
    Form(update_comment): Form<super::model::UpdateComment>,
) -> Result<impl IntoResponse, Error> {
    let ctx = Context::build(current_user);

    super::model::Comment::update(
        &ctx,
        &state.database_pool,
        product_id,
        comment_id,
        update_comment,
    )
    .await?;

    Ok(Redirect::to(&format!("/products/{product_id}")))
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/new", post(comment_create))
        .route("/{id}/delete", get(comment_delete))
        .route("/{id}/edit", get(comment_edit))
        .route("/{id}/edit/save", post(comment_edit_save))
}
