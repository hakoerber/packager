use std::env;

#[tokio::main]
async fn main() {
    // let lists = packager::get_lists();

    // for list in &lists {
    //     println!("Contents of package list {:?}:", list.name);
    //     for item in &list.items {
    //         println!("\t{:?}", item);
    //     }
    // }

    // println!("\nNow we're starting an actual trip!");

    // let mut trip = packager::trip::Trip::from_package_list(
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

    // trip.list.items[0].set_status(packager::trip::TripItemStatus::Ready);
    // trip.list.items[1].set_status(packager::trip::TripItemStatus::Packed);
    // for item in &trip.list.items {
    //     println!("{:?}", item);
    // }
    let args: Vec<String> = env::args().skip(1).collect();
    match args.get(0) {
        None => (),
        Some(cmd) => match cmd.as_ref() {
            "--load-example-data" => {
                packager::db::load().unwrap();
            }
            _ => panic!("Unknown argument: \"{}\"", cmd),
        },
    };

    let router = packager::router::new();

    println!("Initialization done, listening for connections");
    warp::serve(router).run(([127, 0, 0, 1], 9000)).await;
}
