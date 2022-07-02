use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Duration {
    None,
    Days(i32),
}

impl Duration {
    pub fn is_none(d: &Duration) -> bool {
        matches!(d, Duration::None)
    }
}

#[derive(Debug)]
pub enum Period {
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

#[derive(Debug)]
pub enum ItemUsage {
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

#[derive(Debug)]
pub enum ItemSize {
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
    pub fn is_none(d: &ItemSize) -> bool {
        matches!(d, ItemSize::None)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparationStep {
    pub id: Uuid,
    pub name: String,

    #[serde(skip_serializing_if = "Duration::is_none")]
    pub start: Duration,
}

impl PreparationStep {
    pub fn new(id: Uuid, name: String, start: Duration) -> PreparationStep {
        PreparationStep { id, name, start }
    }
}

#[derive(Debug)]
pub enum Preparation {
    None,
    Steps(Vec<PreparationStep>),
}

impl Preparation {
    pub fn is_none(d: &Preparation) -> bool {
        matches!(d, Preparation::None)
    }
}

impl Serialize for Preparation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Preparation::None => {
                // serializer.serialize_unit_variant("Preparation", 0, "null")
                let steps: Vec<i32> = vec![];
                serializer.serialize_newtype_variant("Preparation", 1, "steps", &steps)
                // let mut s = serializer.serialize_seq(Some(0))?;
                // s.end()
            }
            Preparation::Steps(steps) => {
                serializer.serialize_newtype_variant("Preparation", 1, "steps", steps)
            }
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageItem {
    pub id: Uuid,
    pub name: String,

    #[serde(skip_serializing_if = "ItemSize::is_none")]
    size: ItemSize,
    pub count: i32,
    usage: ItemUsage,

    pub preparation: Preparation,
}

impl PackageItem {
    pub fn new(
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

    pub fn new_simple(id: Uuid, name: String) -> PackageItem {
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageList {
    pub id: Uuid,
    pub name: String,
    pub items: Vec<PackageItem>,
}

impl PackageList {
    pub fn new_from_items(id: Uuid, name: String, items: Vec<PackageItem>) -> PackageList {
        PackageList { id, name, items }
    }

    pub fn new(id: Uuid, name: String) -> PackageList {
        PackageList {
            id,
            name,
            items: Vec::new(),
        }
    }

    pub fn set_items(&mut self, items: Vec<PackageItem>) {
        self.items = items;
    }
}
