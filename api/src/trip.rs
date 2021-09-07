use uuid::Uuid;
use serde::{Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TripItemStatus {
    Pending,
    Ready,
    Packed,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TripItem<'a> {
    pub id: Uuid,

    pub package_item: &'a super::PackageItem,
    pub status: TripItemStatus,
}

impl TripItem<'_> {
    pub fn from_package_item(id: Uuid, package_item: &super::PackageItem) -> TripItem {
        TripItem {
            id,
            package_item,
            status: TripItemStatus::Pending,
        }
    }

    pub fn set_status(&mut self, status: TripItemStatus) {
        self.status = status;
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TripList<'a> {
    pub id: Uuid,
    pub items: Vec<TripItem<'a>>,
}

impl<'a> TripList<'a> {
    pub fn from_package_list(id: Uuid, list: &'a super::PackageList) -> TripList<'a> {
        let mut items = Vec::new();
        for item in &list.items {
            items.push(TripItem::from_package_item(Uuid::new_v4(), item));
        }

        TripList { id, items }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trip {
    pub id: Uuid,
    pub name: String,
    pub date: String,
    pub package_list_ids: Vec<Uuid>,
}

impl Trip {
    pub fn from_package_list(
        id: Uuid,
        name: String,
        date: String,
        package_list_ids: Vec<Uuid>,
    ) -> Trip {
        Trip {
            id,
            name,
            date,
            package_list_ids,
        }
    }
}
