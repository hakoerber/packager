use serde::ser::{Serialize, SerializeStruct, Serializer};
use warp::Filter;

#[derive(Debug)]
enum Duration {
    None,
    Days(i32),
}

#[derive(Debug)]
enum Period {
    Daily(i32),
    Weekly(i32),
    Days(i32),
}

#[derive(Debug)]
enum ItemUsage {
    Singleton,
    Periodic(Period),
    Infinite,
}

#[derive(Debug)]
enum ItemSize {
    None,
    Pack(i32),
    Name(String),
    Grams(i32),
}

#[derive(Debug)]
struct PreparationStep {
    name: String,
    start: Duration,
}

impl PreparationStep {
    fn new(name: String, start: Duration) -> PreparationStep {
        PreparationStep { name, start }
    }
}

#[derive(Debug)]
enum Preparation {
    None,
    Steps(Vec<PreparationStep>),
}

#[derive(Debug)]
struct PackageItem {
    name: String,
    size: ItemSize,
    count: i32,
    usage: ItemUsage,
    preparation: Preparation,
}

impl PackageItem {
    fn new(
        name: String,
        size: ItemSize,
        count: i32,
        usage: ItemUsage,
        preparation: Preparation,
    ) -> PackageItem {
        PackageItem {
            name,
            size,
            count,
            usage,
            preparation,
        }
    }

    fn new_simple(name: String) -> PackageItem {
        PackageItem::new(
            name,
            ItemSize::None,
            1,
            ItemUsage::Singleton,
            Preparation::None,
        )
    }
}

impl Serialize for PackageItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PackageItem", 5)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("size", &self.size)?;
        state.serialize_field("count", &self.count)?;
        state.serialize_field("usage", &self.usage)?;
        state.serialize_field("preparation", &self.preparation)?;
        state.end()
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
    fn from_package_item(package_item: &PackageItem) -> TripItem {
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
struct PackageList {
    name: String,
    items: Vec<PackageItem>,
}

impl PackageList {
    fn new_from_items(name: String, items: Vec<PackageItem>) -> PackageList {
        PackageList { name, items }
    }
}

impl Serialize for PackageList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PackageList", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("items", &self.items)?;
        state.end()
    }
}

#[derive(Debug)]
struct TripList<'a> {
    items: Vec<TripItem<'a>>,
}

impl<'a> TripList<'a> {
    fn from_package_list(list: &'a PackageList) -> TripList<'a> {
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
    fn from_package_list(name: String, date: String, list: &'a PackageList) -> Trip<'a> {
        Trip {
            name,
            date,
            list: TripList::from_package_list(list),
        }
    }
}

fn get_lists() -> Vec<PackageList> {
    let lists = vec![
        PackageList::new_from_items(
            String::from("EDC"),
            vec![
                PackageItem::new_simple(String::from("Rucksack")),
                PackageItem::new_simple(String::from("Regenhülle für Rucksack")),
                PackageItem::new_simple(String::from("Normale Schuhe")),
                PackageItem::new_simple(String::from("Taschenmesser")),
                PackageItem::new(
                    String::from("Taschentücher"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Handy"),
                    ItemSize::None,
                    1,
                    ItemUsage::Infinite,
                    Preparation::Steps(vec![PreparationStep::new(
                        String::from("Aufladen"),
                        Duration::Days(1),
                    )]),
                ),
                PackageItem::new(
                    String::from("Kopfhörer"),
                    ItemSize::None,
                    1,
                    ItemUsage::Infinite,
                    Preparation::Steps(vec![PreparationStep::new(
                        String::from("Aufladen"),
                        Duration::Days(1),
                    )]),
                ),
                PackageItem::new(
                    String::from("Mundschutz"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Weekly(1)),
                    Preparation::None,
                ),
                PackageItem::new_simple(String::from("Ladekabel")),
            ],
        ),
        PackageList::new_from_items(
            String::from("Geld & Karten"),
            vec![
                PackageItem::new(
                    String::from("Bargeld"),
                    ItemSize::Name(String::from("Euro")),
                    100,
                    ItemUsage::Infinite,
                    Preparation::Steps(vec![PreparationStep::new(
                        String::from("Abheben"),
                        Duration::Days(1),
                    )]),
                ),
                PackageItem::new_simple(String::from("Kreditkarte")),
                PackageItem::new_simple(String::from("Pass")),
                PackageItem::new_simple(String::from("Krankenversicherungskarte")),
                PackageItem::new_simple(String::from("Krankenversicherungskarte (Zusatz)")),
                PackageItem::new_simple(String::from("Auslandskrankenversicherungsnachweis")),
                PackageItem::new_simple(String::from("Notfalltelefonnummernliste")),
                PackageItem::new_simple(String::from("ADAC-Karte")),
                PackageItem::new_simple(String::from("Impfausweiß (EU)")),
                PackageItem::new_simple(String::from("Führerschein")),
                PackageItem::new_simple(String::from("Internationaler Führerschein")),
                PackageItem::new_simple(String::from("Tagebuch")),
            ],
        ),
        PackageList::new_from_items(
            String::from("Waschzeug"),
            vec![
                PackageItem::new_simple(String::from("Waschbeutel")),
                PackageItem::new_simple(String::from("Sonnencreme")),
                PackageItem::new_simple(String::from("After-Sun")),
                PackageItem::new_simple(String::from("Nagelset")),
                PackageItem::new_simple(String::from("Rasurbox")),
                PackageItem::new_simple(String::from("Rasierer")),
                PackageItem::new(
                    String::from("Ersatzklingen"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new_simple(String::from("Zahnbürste")),
                PackageItem::new(
                    String::from("Zahnputztabletten"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Daily(2)),
                    Preparation::None,
                ),
                PackageItem::new_simple(String::from("Deo")),
                PackageItem::new_simple(String::from("Duschgel / Shampoo")),
            ],
        ),
        PackageList::new_from_items(
            String::from("Apotheke"),
            vec![
                PackageItem::new(
                    String::from("Blasenpflaster"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Erste-Hilfe-Set"),
                    ItemSize::None,
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Paracetamol"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Autan"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Pflaster"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Zeckenkarte"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
            ],
        ),
        PackageList::new_from_items(
            String::from("Badesachen"),
            vec![
                PackageItem::new_simple(String::from("Badehose")),
                PackageItem::new_simple(String::from("Badehandtuch")),
                PackageItem::new_simple(String::from("Surfshirt (Lang)")),
                PackageItem::new_simple(String::from("Wasserschuhe")),
            ],
        ),
        PackageList::new_from_items(
            String::from("Camping"),
            vec![
                PackageItem::new_simple(String::from("Schlafsack")),
                PackageItem::new_simple(String::from("Zelt")),
                PackageItem::new_simple(String::from("Luftmatratze")),
                PackageItem::new_simple(String::from("Campingstuhl")),
                PackageItem::new_simple(String::from("Panzertape")),
                PackageItem::new_simple(String::from("Tarp")),
                PackageItem::new_simple(String::from("Hängematte")),
                PackageItem::new_simple(String::from("Topf")),
                PackageItem::new_simple(String::from("Teller")),
                PackageItem::new_simple(String::from("Messer")),
                PackageItem::new_simple(String::from("Gabel")),
                PackageItem::new_simple(String::from("Löffel")),
                PackageItem::new_simple(String::from("Stirnlampe")),
                PackageItem::new_simple(String::from("Geschirrtuch")),
                PackageItem::new_simple(String::from("Spüllappen")),
                PackageItem::new_simple(String::from("Taschenlampe")),
                PackageItem::new_simple(String::from("Feuerzeug")),
                PackageItem::new_simple(String::from("Tasse")),
                PackageItem::new_simple(String::from("Grill")),
                PackageItem::new(
                    String::from("Grillkohle"),
                    ItemSize::Grams(1500),
                    1,
                    ItemUsage::Periodic(Period::Days(2)),
                    Preparation::None,
                ),
                PackageItem::new_simple(String::from("Campingkocher")),
                PackageItem::new(
                    String::from("Campinggas"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Periodic(Period::Days(3)),
                    Preparation::None,
                ),
                PackageItem::new_simple(String::from("Kaffeekochaufsatz")),
                PackageItem::new(
                    String::from("Küchenrolle"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Days(5)),
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Müllsäcke"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Teelichter"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Daily(3)),
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Klopapier"),
                    ItemSize::Name(String::from("Rolle")),
                    1,
                    ItemUsage::Periodic(Period::Weekly(1)),
                    Preparation::None,
                ),
            ],
        ),
        PackageList::new_from_items(
            String::from("Essen"),
            vec![PackageItem::new(
                String::from("Kaffee"),
                ItemSize::Grams(100),
                1,
                ItemUsage::Periodic(Period::Days(3)),
                Preparation::None,
            )],
        ),
        PackageList::new_from_items(
            String::from("Wanderzeug"),
            vec![
                PackageItem::new_simple(String::from("Wanderschuhe")),
                PackageItem::new(
                    String::from("Trinkblase"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::Steps(vec![PreparationStep::new(
                        String::from("Auffüllen"),
                        Duration::None,
                    )]),
                ),
            ],
        ),
        PackageList::new_from_items(
            String::from("Klamotten"),
            vec![
                PackageItem::new(
                    String::from("Cap"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Regenjacke"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Daunenjacke"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Pullover"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Lange Hose"),
                    ItemSize::None,
                    2,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Kurze Hose"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Jogginghose"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Socken"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Daily(1)),
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Unterhose"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Daily(1)),
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("T-Shirt"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Days(2)),
                    Preparation::None,
                ),
                PackageItem::new_simple(String::from("Schmutzwäschebeutel")),
            ],
        ),
        PackageList::new_from_items(
            String::from("Geld & Karten"),
            vec![
                PackageItem::new_simple(String::from("Fahrrad")),
                PackageItem::new_simple(String::from("Fahrradhelm")),
            ],
        ),
        PackageList::new_from_items(
            String::from("Geld & Karten"),
            vec![
                PackageItem::new_simple(String::from("Trinkflasche")),
                PackageItem::new_simple(String::from("Dyneemaschnur")),
                PackageItem::new_simple(String::from("Ladegerät")),
                PackageItem::new(
                    String::from("Powerbank"),
                    ItemSize::None,
                    1,
                    ItemUsage::Infinite,
                    Preparation::Steps(vec![PreparationStep::new(
                        String::from("Aufladen"),
                        Duration::Days(1),
                    )]),
                ),
                PackageItem::new(
                    String::from("Desinfektionsgel"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Weekly(1)),
                    Preparation::None,
                ),
                PackageItem::new(
                    String::from("Karabiner"),
                    ItemSize::None,
                    3,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new_simple(String::from("Ersatzbrille")),
                PackageItem::new_simple(String::from("Sonnenbrille")),
                PackageItem::new_simple(String::from("Ohrenstöpsel")),
            ],
        ),
    ];

    lists

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
}

#[tokio::main]
async fn main() {
    let accept_json = warp::header::exact("accept", "application/json");

    let root = warp::path::end().map(|| "Hi");
    let v1 = warp::path!("v1")
        .and(warp::get())
        .and(warp::path::end())
        .map(warp::reply);
    let lists = warp::path!("v1" / "lists")
        .and(warp::path::end())
        .and(warp::get())
        .and(accept_json)
        .map(|| warp::reply::json(&get_lists()));

    let routes = root.or(v1).or(lists);

    warp::serve(routes).run(([127, 0, 0, 1], 9000)).await;
}
