use packager;
use uuid::Uuid;

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

#[tokio::main]
async fn main() {
//     let accept_json = warp::header::exact("accept", "application/json");
//     let cors = warp::cors().allow_any_origin();

//     let root = warp::path::end()
//         .map(|| "Hi")
//         .recover(packager::router::handle_rejection);

//     let v1 = warp::path!("v1")
//         .and(warp::get())
//         .and(warp::path::end())
//         .map(warp::reply)
//         .recover(packager::router::handle_rejection);

//     let lists = warp::path!("v1" / "lists")
//         .and(warp::path::end())
//         .and(warp::get())
//         .and(accept_json)
//         .map(|| warp::reply::json(&packager::get_lists()))
//         .with(&cors)
//         .recover(packager::router::handle_rejection);

//     let list = warp::path!("v1" / "lists" / String)
//         .and(warp::path::end())
//         .and(warp::get())
//         .and(accept_json)
//         .and_then(|id: String| async move {
//             match Uuid::parse_str(&id) {
//                 Ok(uuid) => {
//                     let list = &packager::get_list(uuid);
//                     match list {
//                         Some(l) => Ok(warp::reply::json(l)),
//                         None => Err(warp::reject::not_found()),
//                     }
//                 }
//                 Err(e) => Err(warp::reject::custom(packager::router::InvalidUuid)),
//             }
//         })
//         .with(&cors)
//         .recover(packager::router::handle_rejection);

//     let router = root.or(v1).or(lists).or(list);

    let router = packager::router::new();

    warp::serve(router).run(([127, 0, 0, 1], 9000)).await;
}
