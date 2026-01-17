use axum::{
    extract::Extension,
    http::header::{self, HeaderMap},
    response::IntoResponse,
};

use crate::{
    htmx, models,
    view::{self, Component},
    Context,
};

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
