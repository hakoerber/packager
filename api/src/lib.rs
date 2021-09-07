use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Duration {
    None,
    Days(i32),
}

impl Duration {
    fn is_none(d: &Duration) -> bool {
        matches!(d, Duration::None)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Period {
    Daily(i32),
    Weekly(i32),
    Days(i32),
}

impl Serialize for Period {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Period::Daily(i) => {
                let mut s = serializer.serialize_struct("period", 2)?;
                s.serialize_field("type", "daily")?;
                s.serialize_field("value", &i)?;
                s.end()
            }
            Period::Weekly(i) => {
                let mut s = serializer.serialize_struct("period", 2)?;
                s.serialize_field("type", "weekly")?;
                s.serialize_field("value", &i)?;
                s.end()
            }
            Period::Days(i) => {
                let mut s = serializer.serialize_struct("period", 2)?;
                s.serialize_field("type", "days")?;
                s.serialize_field("value", &i)?;
                s.end()
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ItemUsage {
    Singleton,
    Periodic(Period),
    Infinite,
}

impl Serialize for ItemUsage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ItemUsage::Singleton => {
                let mut s = serializer.serialize_struct("usage", 1)?;
                s.serialize_field("type", "singleton")?;
                s.end()
            }
            ItemUsage::Periodic(p) => {
                let mut s = serializer.serialize_struct("usage", 2)?;
                s.serialize_field("type", "peridoic")?;
                s.serialize_field("value", &p)?;
                s.end()
            }
            ItemUsage::Infinite => {
                let mut s = serializer.serialize_struct("size", 1)?;
                s.serialize_field("type", "infinite")?;
                s.end()
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ItemSize {
    None,
    Pack(i32),
    Name(String),
    Gram(i32),
}

impl Serialize for ItemSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ItemSize::None => {
                let mut s = serializer.serialize_struct("size", 1)?;
                s.serialize_field("type", "none")?;
                s.end()
            }
            ItemSize::Pack(i) => {
                let mut s = serializer.serialize_struct("size", 2)?;
                s.serialize_field("type", "pack")?;
                s.serialize_field("value", &i)?;
                s.end()
            }
            ItemSize::Name(n) => {
                let mut s = serializer.serialize_struct("size", 2)?;
                s.serialize_field("type", "name")?;
                s.serialize_field("value", &n)?;
                s.end()
            }
            ItemSize::Gram(i) => {
                let mut s = serializer.serialize_struct("size", 2)?;
                s.serialize_field("type", "gram")?;
                s.serialize_field("value", &i)?;
                s.end()
            }
        }
    }
}

impl ItemSize {
    fn is_none(d: &ItemSize) -> bool {
        matches!(d, ItemSize::None)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PreparationStep {
    name: String,

    #[serde(skip_serializing_if = "Duration::is_none")]
    start: Duration,
}

impl PreparationStep {
    fn new(name: String, start: Duration) -> PreparationStep {
        PreparationStep { name, start }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Preparation {
    None,
    Steps(Vec<PreparationStep>),
}

impl Preparation {
    fn is_none(d: &Preparation) -> bool {
        matches!(d, Preparation::None)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageItem {
    id: Uuid,
    name: String,

    #[serde(skip_serializing_if = "ItemSize::is_none")]
    size: ItemSize,
    count: i32,
    usage: ItemUsage,

    #[serde(skip_serializing_if = "Preparation::is_none")]
    preparation: Preparation,
}

impl PackageItem {
    fn new(
        id: Uuid,
        name: String,
        size: ItemSize,
        count: i32,
        usage: ItemUsage,
        preparation: Preparation,
    ) -> PackageItem {
        PackageItem {
            id,
            name,
            size,
            count,
            usage,
            preparation,
        }
    }

    fn new_simple(id: Uuid, name: String) -> PackageItem {
        PackageItem::new(
            id,
            name,
            ItemSize::None,
            1,
            ItemUsage::Singleton,
            Preparation::None,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageList {
    id: Uuid,
    pub name: String,
    pub items: Vec<PackageItem>,
}

impl PackageList {
    fn new_from_items(id: Uuid, name: String, items: Vec<PackageItem>) -> PackageList {
        PackageList { id, name, items }
    }
}

pub fn get_list(id: Uuid) -> Option<PackageList> {
    println!("Looking for id {}", id);
    for list in get_lists() {
        println!("Have {}", list.id);
        if list.id == id {
            println!("Found!");
            return Some(list);
        }
    }
    println!("Not Found!");
    None
}

pub fn get_lists() -> Vec<PackageList> {
    vec![
        PackageList::new_from_items(
            Uuid::parse_str("5f95d8c7-c4da-44bc-af30-2d10c479de8a").unwrap(),
            String::from("EDC"),
            vec![
                PackageItem::new_simple(
                    Uuid::parse_str("48703b81-037a-401f-8f46-56c242bb16c3").unwrap(),
                    String::from("Rucksack"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("39b26f98-0cbc-46d5-ac17-43d61ba1a503").unwrap(),
                    String::from("Regenhülle für Rucksack"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("a5934361-bc5d-4092-bad4-95c15c875dca").unwrap(),
                    String::from("Normale Schuhe"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("4b0a6dbb-652b-464a-b413-93dd4f010ce3").unwrap(),
                    String::from("Taschenmesser"),
                ),
                PackageItem::new(
                    Uuid::parse_str("9823e841-64d1-43ab-a05f-95606b89482c").unwrap(),
                    String::from("Taschentücher"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("a7eac657-e870-4181-a644-16fe229d917a").unwrap(),
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
                    Uuid::parse_str("c0d97e46-a9fe-4d20-a3e9-8cf7c69d2fbf").unwrap(),
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
                    Uuid::parse_str("9b04e117-1a61-4643-8238-55c401a2dd00").unwrap(),
                    String::from("Mundschutz"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Weekly(1)),
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("5efe30c7-896d-4c32-9976-81ac93ff6aa0").unwrap(),
                    String::from("Kokain"),
                    ItemSize::Gram(100),
                    1,
                    ItemUsage::Periodic(Period::Weekly(1)),
                    Preparation::Steps(vec![PreparationStep::new(
                        String::from("Beim Dealer kaufen"),
                        Duration::Days(1),
                    )]),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("8fb8d7ce-7766-4203-bc38-058fe2440519").unwrap(),
                    String::from("Ladekabel"),
                ),
            ],
        ),
        PackageList::new_from_items(
            Uuid::parse_str("3ea0714e-3115-46c4-8ae0-f2a37398cc7a").unwrap(),
            String::from("Geld & Karten"),
            vec![
                PackageItem::new(
                    Uuid::parse_str("18ebb1bc-e01f-4dc7-bda7-919dd8f069c7").unwrap(),
                    String::from("Bargeld"),
                    ItemSize::Name(String::from("Euro")),
                    100,
                    ItemUsage::Infinite,
                    Preparation::Steps(vec![PreparationStep::new(
                        String::from("Abheben"),
                        Duration::Days(1),
                    )]),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("2aba4c22-9c3d-4b19-95b5-265f1846b8f9").unwrap(),
                    String::from("Kreditkarte"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("aeec841e-0691-4af9-b719-16a6752c33d6").unwrap(),
                    String::from("Pass"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("1a598c16-a238-4d02-baef-03208beb9509").unwrap(),
                    String::from("Krankenversicherungskarte"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("584955e2-3bf4-459c-8c87-631414842571").unwrap(),
                    String::from("Krankenversicherungskarte (Zusatz)"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("571dd7b6-0420-4f8f-8e83-fb430c6f1c23").unwrap(),
                    String::from("Auslandskrankenversicherungsnachweis"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("25937e62-4ade-495b-9535-c4db1176fbab").unwrap(),
                    String::from("Notfalltelefonnummernliste"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("6464f872-00f6-4621-9d37-ff6b75c8d79a").unwrap(),
                    String::from("ADAC-Karte"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("0b9298b7-6bdb-4fdc-a171-1953654b160e").unwrap(),
                    String::from("Impfausweiß (EU)"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("fac04399-e092-4aeb-bf61-edd081890fa6").unwrap(),
                    String::from("Führerschein"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("17ef3d3a-010f-44c1-acae-ef0ea9a985b8").unwrap(),
                    String::from("Internationaler Führerschein"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("b256cd9f-8c5c-45f2-9e08-04cc62800cfe").unwrap(),
                    String::from("Tagebuch"),
                ),
            ],
        ),
        PackageList::new_from_items(
            Uuid::parse_str("4b2c2d4e-3014-48aa-95e0-39f352cb6494").unwrap(),
            String::from("Waschzeug"),
            vec![
                PackageItem::new_simple(
                    Uuid::parse_str("c91e2295-7fa6-4165-bb6c-0de60069c410").unwrap(),
                    String::from("Waschbeutel"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("005639bb-b336-4e7c-bb88-aeaca20fc488").unwrap(),
                    String::from("Sonnencreme"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("9a907ab6-3e49-4cc0-af86-637821127354").unwrap(),
                    String::from("After-Sun"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("7ed5be40-929e-4ae2-a723-61e6514e9592").unwrap(),
                    String::from("Nagelset"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("40f9bc11-1e24-4a60-b183-7f9a2eab0e42").unwrap(),
                    String::from("Rasurbox"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("c72126c8-5639-4952-b4a7-d588d636a6f0").unwrap(),
                    String::from("Rasierer"),
                ),
                PackageItem::new(
                    Uuid::parse_str("9f6e9c09-e550-411c-a89d-7f79608b1bb6").unwrap(),
                    String::from("Ersatzklingen"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("547fb1cd-568b-44d2-9618-0f4de7ca2ca5").unwrap(),
                    String::from("Zahnbürste"),
                ),
                PackageItem::new(
                    Uuid::parse_str("09259ebe-407b-4808-9a19-db3b95f77846").unwrap(),
                    String::from("Zahnputztabletten"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Daily(2)),
                    Preparation::None,
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("10342277-9984-4c9a-ad07-155e148f91fd").unwrap(),
                    String::from("Deo"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("e323ca57-8487-4b3f-a3f9-6dd8e26ff0e4").unwrap(),
                    String::from("Duschgel / Shampoo"),
                ),
            ],
        ),
        PackageList::new_from_items(
            Uuid::parse_str("4ba0462f-5c75-4926-9ca2-ea93ef5b66ef").unwrap(),
            String::from("Apotheke"),
            vec![
                PackageItem::new(
                    Uuid::parse_str("bdeea898-479b-41e9-8d33-1c79743b3011").unwrap(),
                    String::from("Blasenpflaster"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("2175fa92-0ef2-48a1-bea0-f70268b0ad68").unwrap(),
                    String::from("Erste-Hilfe-Set"),
                    ItemSize::None,
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("bad9f320-eaf8-4240-bfa6-890b755eb03a").unwrap(),
                    String::from("Paracetamol"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("b9d3f29d-4448-42ba-aeff-ba179542b4ae").unwrap(),
                    String::from("Autan"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("bfeb6d0f-5122-4f80-94e9-be54a1d1c838").unwrap(),
                    String::from("Pflaster"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("3f08c3e2-a0d9-40fb-b3fb-f4bb79815b9d").unwrap(),
                    String::from("Zeckenkarte"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
            ],
        ),
        PackageList::new_from_items(
            Uuid::parse_str("9aa4ee9f-2f00-42d2-980e-8fd1ede0283c").unwrap(),
            String::from("Badesachen"),
            vec![
                PackageItem::new_simple(
                    Uuid::parse_str("879b9d32-0de5-45f3-bffb-0c1e73b5a7b8").unwrap(),
                    String::from("Badehose"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("5610ab1f-308d-4ef9-b8e4-3721fa4c4172").unwrap(),
                    String::from("Badehandtuch"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("e7e3c053-a269-4f0d-b6bd-99b48bf4573a").unwrap(),
                    String::from("Surfshirt (Lang)"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("360c20f5-8142-48d1-add4-a8f2a48a242b").unwrap(),
                    String::from("Wasserschuhe"),
                ),
            ],
        ),
        PackageList::new_from_items(
            Uuid::parse_str("1e0728d1-9dd9-48ff-a206-a73b604b9748").unwrap(),
            String::from("Camping"),
            vec![
                PackageItem::new_simple(
                    Uuid::parse_str("fb6eec8c-a1ad-420f-b8de-9695e9ccb67d").unwrap(),
                    String::from("Schlafsack"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("69a06abd-34f4-4991-9a41-b3ec50bcbbd7").unwrap(),
                    String::from("Zelt"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("22169568-1976-43ab-8c40-37c7e6193e18").unwrap(),
                    String::from("Luftmatratze"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("dd10738b-0162-4e0a-9db9-ea5e16ee5566").unwrap(),
                    String::from("Campingstuhl"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("a88c14c1-8194-4fea-be6c-b532517bbf97").unwrap(),
                    String::from("Panzertape"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("fb67fbf1-cd04-4b39-b0a3-bff4bf07f38e").unwrap(),
                    String::from("Tarp"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("6de5825b-0e39-4693-9664-2edc7353db3b").unwrap(),
                    String::from("Hängematte"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("c6f14fb7-c598-4a47-8b12-b96ff3856a21").unwrap(),
                    String::from("Topf"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("2fb43b9c-c6ca-4fda-9e09-e1a83efdc4f7").unwrap(),
                    String::from("Teller"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("a203f34c-e912-4479-ad8e-e16150122cad").unwrap(),
                    String::from("Messer"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("63837c0a-7a8e-40ae-aced-818066bd9e89").unwrap(),
                    String::from("Gabel"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("437a45d9-4fc5-459e-b790-7628945d7c38").unwrap(),
                    String::from("Löffel"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("a461859f-b430-42da-bf52-8cdb3eb42d13").unwrap(),
                    String::from("Stirnlampe"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("f109c12f-10dd-4249-a02e-c28aaa20a8f6").unwrap(),
                    String::from("Geschirrtuch"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("1f2ffc7e-060a-4277-a15d-119b85481438").unwrap(),
                    String::from("Spüllappen"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("bf6d49fe-25cc-4114-a374-3c4772cf5f3a").unwrap(),
                    String::from("Taschenlampe"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("03994e35-f0f0-4ccd-bc24-0a331ffbba96").unwrap(),
                    String::from("Feuerzeug"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("fc3ecdab-eb3a-4d27-8f95-cd78d37e063e").unwrap(),
                    String::from("Tasse"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("8b55efae-6e18-4d7f-a5b7-83ba15c50995").unwrap(),
                    String::from("Grill"),
                ),
                PackageItem::new(
                    Uuid::parse_str("b6d821e5-3285-4bf2-bd71-be9154f94937").unwrap(),
                    String::from("Grillkohle"),
                    ItemSize::Gram(1500),
                    1,
                    ItemUsage::Periodic(Period::Days(2)),
                    Preparation::None,
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("d2338ac1-9995-4319-8ca5-12654a266266").unwrap(),
                    String::from("Campingkocher"),
                ),
                PackageItem::new(
                    Uuid::parse_str("4ff62940-f691-4982-b1fd-54a2850e06f4").unwrap(),
                    String::from("Campinggas"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Periodic(Period::Days(3)),
                    Preparation::None,
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("a024c67a-6e61-4749-b2a4-1ec8ca132184").unwrap(),
                    String::from("Kaffeekochaufsatz"),
                ),
                PackageItem::new(
                    Uuid::parse_str("054e1610-1e8e-421e-95ad-c76e88cb45ab").unwrap(),
                    String::from("Küchenrolle"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Days(5)),
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("181310a8-946c-4075-9f95-c7a82d0269fd").unwrap(),
                    String::from("Müllsäcke"),
                    ItemSize::Pack(1),
                    1,
                    ItemUsage::Infinite,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("c823ce3f-922e-4300-95ad-21ff221cd896").unwrap(),
                    String::from("Teelichter"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Daily(3)),
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("e86bfb7c-a840-4b88-b216-afd3c173754b").unwrap(),
                    String::from("Klopapier"),
                    ItemSize::Name(String::from("Rolle")),
                    1,
                    ItemUsage::Periodic(Period::Weekly(1)),
                    Preparation::None,
                ),
            ],
        ),
        PackageList::new_from_items(
            Uuid::parse_str("e5ecb3cc-a893-44cd-b4f8-4b41af003c96").unwrap(),
            String::from("Essen"),
            vec![PackageItem::new(
                Uuid::parse_str("6ebf5d87-6b97-4d80-8e03-25fa62e641d9").unwrap(),
                String::from("Kaffee"),
                ItemSize::Gram(100),
                1,
                ItemUsage::Periodic(Period::Days(3)),
                Preparation::None,
            )],
        ),
        PackageList::new_from_items(
            Uuid::parse_str("4e9042db-1db8-4912-8179-d0a3ebb80876").unwrap(),
            String::from("Wanderzeug"),
            vec![
                PackageItem::new_simple(
                    Uuid::parse_str("2eccd528-17a7-429f-b97a-70faae1b4dac").unwrap(),
                    String::from("Wanderschuhe"),
                ),
                PackageItem::new(
                    Uuid::parse_str("d8c52786-d50e-4a9f-b8c6-e9396c5be789").unwrap(),
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
            Uuid::parse_str("5b272c8e-bf54-4af6-94e0-63071c92a8e3").unwrap(),
            String::from("Klamotten"),
            vec![
                PackageItem::new(
                    Uuid::parse_str("d18215d2-9cf9-48c7-894b-b9680ba39879").unwrap(),
                    String::from("Cap"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("83c03986-cc34-4500-9cee-1cc1778f5cf3").unwrap(),
                    String::from("Regenjacke"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("00c116ba-840d-47c4-a081-5e256941d3b9").unwrap(),
                    String::from("Daunenjacke"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("35834bec-6c63-41df-825c-05b943e07bc8").unwrap(),
                    String::from("Pullover"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("127704a8-1e4d-4c49-b74f-34130ed08f83").unwrap(),
                    String::from("Lange Hose"),
                    ItemSize::None,
                    2,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("e15a6772-3d5d-454d-8f7d-bf9066ba5d23").unwrap(),
                    String::from("Kurze Hose"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("09992e6d-408a-43e4-beee-1925e727415f").unwrap(),
                    String::from("Jogginghose"),
                    ItemSize::None,
                    1,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("799f0fd4-2be7-44b5-b286-889242900be3").unwrap(),
                    String::from("Socken"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Daily(1)),
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("f57f3299-a46a-4ed3-aff0-ffc5d2627a6a").unwrap(),
                    String::from("Unterhose"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Daily(1)),
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("c100f7be-ceb7-46a6-aded-380a201bfe45").unwrap(),
                    String::from("T-Shirt"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Days(2)),
                    Preparation::None,
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("22908312-a8d2-4005-893c-12e56d9af022").unwrap(),
                    String::from("Schmutzwäschebeutel"),
                ),
            ],
        ),
        PackageList::new_from_items(
            Uuid::parse_str("929bc029-ec8f-4294-9484-ec32f0170f5c").unwrap(),
            String::from("Fahrrad"),
            vec![
                PackageItem::new_simple(
                    Uuid::parse_str("d91ed4fd-ef9b-4ad5-849f-1481dbbc95b0").unwrap(),
                    String::from("Fahrrad"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("2f2a09d7-2d97-4ff8-9fb1-ace86eaf6de5").unwrap(),
                    String::from("Fahrradhelm"),
                ),
            ],
        ),
        PackageList::new_from_items(
            Uuid::parse_str("0103e348-b4e8-4cc9-95e6-4ef85b7c76ab").unwrap(),
            String::from("Misc"),
            vec![
                PackageItem::new_simple(
                    Uuid::parse_str("990c7121-7342-48a2-9abb-4d0e274b6759").unwrap(),
                    String::from("Trinkflasche"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("0ead2b93-9b81-4e32-a856-0a562557598c").unwrap(),
                    String::from("Dyneemaschnur"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("be5d49bc-9c06-4747-b954-9ca003694018").unwrap(),
                    String::from("Ladegerät"),
                ),
                PackageItem::new(
                    Uuid::parse_str("aab7b03b-19f9-4149-a23d-3c14d938cca4").unwrap(),
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
                    Uuid::parse_str("c54deb17-fe62-4300-8297-c5dfc4773a02").unwrap(),
                    String::from("Desinfektionsgel"),
                    ItemSize::None,
                    1,
                    ItemUsage::Periodic(Period::Weekly(1)),
                    Preparation::None,
                ),
                PackageItem::new(
                    Uuid::parse_str("3fb87c86-7155-4df5-8f7b-68c288ba1147").unwrap(),
                    String::from("Karabiner"),
                    ItemSize::None,
                    3,
                    ItemUsage::Singleton,
                    Preparation::None,
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("c965176c-9014-4e8f-8e99-8cd2e37a64ac").unwrap(),
                    String::from("Ersatzbrille"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("b0909ce6-5a8c-42fa-a384-de108f19ee5d").unwrap(),
                    String::from("Sonnenbrille"),
                ),
                PackageItem::new_simple(
                    Uuid::parse_str("7ec347d4-0c28-48bb-9c32-558a1988a164").unwrap(),
                    String::from("Ohrenstöpsel"),
                ),
            ],
        ),
    ]
}
