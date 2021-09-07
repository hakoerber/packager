use std::convert::Infallible;
use serde::Serialize;
use warp;
use warp::http::StatusCode;
use uuid::Uuid;

#[derive(Debug)]
struct InvalidUuid;

impl warp::reject::Reject for InvalidUuid {}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    success: bool,
    message: String,
}

pub fn new() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let accept_json = warp::header::exact("accept", "application/json");
    let cors = warp::cors().allow_any_origin();

    let root = warp::path::end()
        .map(|| "Hi")
        .recover(handle_rejection);

    let v1 = warp::path!("v1")
        .and(warp::get())
        .and(warp::path::end())
        .map(warp::reply)
        .recover(handle_rejection);

    let lists = warp::path!("v1" / "lists")
        .and(warp::path::end())
        .and(warp::get())
        .and(accept_json)
        .map(|| warp::reply::json(&super::get_lists()))
        .with(&cors)
        .recover(handle_rejection);

    let list = warp::path!("v1" / "lists" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(accept_json)
        .and_then(|id: String| async move {
            match Uuid::parse_str(&id) {
                Ok(uuid) => {
                    let list = &super::get_list(uuid);
                    match list {
                        Some(l) => Ok(warp::reply::json(l)),
                        None => Err(warp::reject::not_found()),
                    }
                }
                Err(e) => Err(warp::reject::custom(InvalidUuid)),
            }
        })
        .with(&cors)
        .recover(handle_rejection);

    let routes = root.or(v1).or(lists).or(list).boxed();

    routes
}

// See https://github.com/seanmonstar/warp/blob/master/examples/rejections.rs
async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        message = "NOT_FOUND";
        code = StatusCode::NOT_FOUND;
    } else if let Some(InvalidUuid) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "INVALID_UUID";
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        message = "BAD_REQUEST";
        code = StatusCode::BAD_REQUEST;
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        message = "METHOD_NOT_ALLOWED";
        code = StatusCode::METHOD_NOT_ALLOWED;
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        message = "UNHANDLED_REJECTION";
        code = StatusCode::INTERNAL_SERVER_ERROR;
    }

    let json = warp::reply::json(&ErrorMessage {
        success: false,
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}
