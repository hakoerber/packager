use axum::{
    extract::{Extension, Path, State},
    response::{IntoResponse, Redirect},
    routing::post,
    Form, Router,
};
use uuid::Uuid;

use crate::{models, AppState, Context, Error, RequestError};

#[tracing::instrument]
pub(crate) async fn new_comment(
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

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/new", post(new_comment))
}
