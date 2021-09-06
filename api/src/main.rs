#[derive(Debug)]
enum Period {
    Days(i32),
}

#[derive(Debug)]
enum ItemUsage {
    Singleton,
    Once,
    Periodic(Period),
}

#[derive(Debug)]
struct PackageItem {
    name: String,
    usage: ItemUsage,
}

impl PackageItem {
    fn new(name: String, usage: ItemUsage) -> PackageItem {
        PackageItem {
            name: name,
            usage: usage,
        }
    }
}

#[derive(Debug)]
enum TripItemStatus {
    Pending,
    Ready,
    Packed,
}

#[derive(Debug)]
struct TripItem<'a> {
    package_item: &'a PackageItem,
    status: TripItemStatus,
}

impl TripItem<'_> {
    fn from_package_item(package_item: &PackageItem) -> TripItem  {
        TripItem {
            package_item: package_item,
            status: TripItemStatus::Pending,
        }
    }
}

#[derive(Debug)]
struct PackageList {
    name: String,
    items: Vec<PackageItem>,
}

#[derive(Debug)]
struct TripList<'a> {
    items: Vec<TripItem<'a>>,
}

impl<'a> TripList<'a> {
    fn from_package_list(list: &'a PackageList) -> TripList<'a> {
        TripList {
            items: vec![TripItem::from_package_item(&list.items[0])],
        }

    }
}

#[derive(Debug)]
struct Trip<'a> {
    name: String,
    date: String,
    list: TripList<'a>,
}

impl<'a> Trip<'a> {
    fn from_package_list(name: String, date: String, list: &'a PackageList) -> Trip<'a> {
        Trip {
            name: name,
            date: date,
            list: TripList::from_package_list(list),
        }
    }
}

fn main() {
    let package_list_camping = PackageList {
        name: String::from("camping"),
        items: vec![
            PackageItem::new(String::from("Rucksack"), ItemUsage::Singleton),
            PackageItem::new(String::from("Schlafsack"), ItemUsage::Singleton),
            PackageItem::new(String::from("Zelt"), ItemUsage::Singleton),
            PackageItem::new(String::from("Luftmatratze"), ItemUsage::Singleton),
            PackageItem::new(String::from("Campingstuhl"), ItemUsage::Singleton),
            PackageItem::new(String::from("Regenjacke"), ItemUsage::Singleton),
            PackageItem::new(String::from("Tasse"), ItemUsage::Singleton),
            PackageItem::new(String::from("Trinkblase"), ItemUsage::Singleton),
            PackageItem::new(String::from("Trinkflasche"), ItemUsage::Singleton),
            PackageItem::new(String::from("Topf"), ItemUsage::Singleton),
            PackageItem::new(String::from("Messer"), ItemUsage::Singleton),
            PackageItem::new(String::from("Messer"), ItemUsage::Singleton),
            PackageItem::new(
                String::from("Unterhose"),
                ItemUsage::Periodic(Period::Days(1)),
            ),
        ],
    };

    println!("Contents of package list {:?}:", package_list_camping.name);
    for item in &package_list_camping.items {
        println!("{:?}", item);
    }

    println!("Now we're starting an actual trip!");

    let trip = Trip::from_package_list(
        String::from("Campingtrip"),
        String::from("2021-09-06"),
        &package_list_camping,
    );

    println!("Package list for trip {:?} at {:?}:", trip.name, trip.date);
    for item in &trip.list.items {
        println!("{:?}", item);
    }
}
