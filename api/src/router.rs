use serde::{Deserialize, Serialize};
use std::convert::Infallible;

use warp;
use warp::http::StatusCode;
use warp::Filter;

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

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct JsonListItem {
    name: String,
    count: i32,
}

pub fn new() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let accept_json = warp::header::exact("accept", "application/json");
    let content_json = warp::header::exact("content-type", "application/json");
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(&[
            warp::http::Method::GET,
            warp::http::Method::POST,
            warp::http::Method::DELETE,
        ])
        .allow_headers(vec!["accept", "content-type"]);

    fn json_body() -> impl Filter<Extract = (JsonListItem,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    let root = warp::path::end().map(|| "Hi");

    let v1 = warp::path!("v1")
        .and(warp::get())
        .and(warp::path::end())
        .map(warp::reply);

    let lists = warp::path!("v1" / "lists")
        .and(warp::path::end())
        .and(warp::get())
        .and(accept_json)
        .map(|| warp::reply::json(&super::get_lists()))
        .with(&cors);

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
                Err(_) => Err(warp::reject::custom(InvalidUuid)),
            }
        })
        .with(&cors);

    let list_items = warp::path!("v1" / "lists" / String / "items")
        .and(warp::path::end())
        .and(warp::get())
        .and(accept_json)
        .and_then(|list_id: String| async move {
            match Uuid::parse_str(&list_id) {
                Ok(uuid) => {
                    let items = &super::get_packagelist_items(uuid);
                    Ok(warp::reply::json(items))
                }
                Err(_) => Err(warp::reject::custom(InvalidUuid)),
            }
        })
        .with(&cors);

    let preparation = warp::path!("v1" / "lists" / String / "items" / String / "preparation")
        .and(warp::path::end())
        .and(warp::get())
        .and(accept_json)
        .and_then(|list_id: String, item_id: String| async move {
            match Uuid::parse_str(&list_id) {
                Ok(list_id) => match Uuid::parse_str(&item_id) {
                    Err(_) => Err(warp::reject::custom(InvalidUuid)),
                    Ok(item_id) => {
                        let items = &super::get_preparation(list_id, item_id);
                        Ok(warp::reply::json(items))
                    }
                },
                Err(_) => Err(warp::reject::custom(InvalidUuid)),
            }
        })
        .with(&cors);

    let new_item = warp::path!("v1" / "lists" / String / "items")
        .and(warp::path::end())
        .and(warp::post())
        .and(accept_json)
        .and(content_json)
        .and(json_body())
        .and_then(|list_id: String, item: JsonListItem| async move {
            match Uuid::parse_str(&list_id) {
                Ok(list_id) => {
                    let new_item = &super::new_item(list_id, item.name, item.count);
                    Ok(warp::reply::json(new_item))
                }
                Err(_) => Err(warp::reject::custom(InvalidUuid)),
            }
        })
        .with(&cors);

    let trips = warp::path!("v1" / "trips")
        .and(warp::path::end())
        .and(warp::get())
        .and(accept_json)
        .map(|| warp::reply::json(&super::get_trips()))
        .with(&cors);

    root.or(v1)
        .or(lists)
        .or(list)
        .or(new_item)
        .or(list_items)
        .or(preparation)
        .or(trips)
        .recover(handle_rejection)
        .boxed()
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
    } else if err.find::<warp::filters::body::BodyDeserializeError>().is_some() {
        message = "BAD_REQUEST";
        code = StatusCode::BAD_REQUEST;
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
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
