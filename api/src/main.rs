use std::convert::Infallible;

use serde::Serialize;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::Filter;

use packager;

#[derive(Debug)]
enum TripItemStatus {
    Pending,
    Ready,
    Packed,
}

#[derive(Debug)]
struct TripItem<'a> {
    package_item: &'a packager::PackageItem,
    status: TripItemStatus,
}

impl TripItem<'_> {
    fn from_package_item(package_item: &packager::PackageItem) -> TripItem {
        TripItem {
            package_item,
            status: TripItemStatus::Pending,
        }
    }

    fn set_status(&mut self, status: TripItemStatus) {
        self.status = status;
    }
}

#[derive(Debug)]
struct TripList<'a> {
    items: Vec<TripItem<'a>>,
}

impl<'a> TripList<'a> {
    fn from_package_list(list: &'a packager::PackageList) -> TripList<'a> {
        let mut items = Vec::new();
        for item in &list.items {
            items.push(TripItem::from_package_item(item));
        }

        TripList { items }
    }
}

#[derive(Debug)]
struct Trip<'a> {
    name: String,
    date: String,
    list: TripList<'a>,
}

impl<'a> Trip<'a> {
    fn from_package_list(name: String, date: String, list: &'a packager::PackageList) -> Trip<'a> {
        Trip {
            name,
            date,
            list: TripList::from_package_list(list),
        }
    }
}

// for list in &lists {
//     println!("Contents of package list {:?}:", list.name);
//     for item in &list.items {
//         println!("\t{:?}", item);
//     }
// }

// println!("\nNow we're starting an actual trip!");

// let mut trip = Trip::from_package_list(
//     String::from("Campingtrip"),
//     String::from("2021-09-06"),
//     &lists[0],
// );

// println!(
//     "\nPackage list for trip {:?} at {:?}:",
//     trip.name, trip.date
// );
// for item in &trip.list.items {
//     println!("{:?}", item);
// }

// trip.list.items[0].set_status(TripItemStatus::Ready);
// trip.list.items[1].set_status(TripItemStatus::Packed);
// for item in &trip.list.items {
//     println!("{:?}", item);
// }

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    success: bool,
    message: String,
}

#[derive(Debug)]
struct InvalidUuid;

impl warp::reject::Reject for InvalidUuid {}

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

#[tokio::main]
async fn main() {
    let accept_json = warp::header::exact("accept", "application/json");
    let cors = warp::cors().allow_any_origin();

    let root = warp::path::end().map(|| "Hi");
    let v1 = warp::path!("v1")
        .and(warp::get())
        .and(warp::path::end())
        .map(warp::reply);
    let lists = warp::path!("v1" / "lists")
        .and(warp::path::end())
        .and(warp::get())
        .and(accept_json)
        .map(|| warp::reply::json(&packager::get_lists()))
        .with(&cors);
    let list = warp::path!("v1" / "lists" / String)
        .and(warp::path::end())
        .and(warp::get())
        .and(accept_json)
        .and_then(|id: String| async move {
            match Uuid::parse_str(&id) {
                Ok(uuid) => {
                    let list = &packager::get_list(uuid);
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

    let routes = root.or(v1).or(lists).or(list);

    warp::serve(routes).run(([127, 0, 0, 1], 9000)).await;
}
