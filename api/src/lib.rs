pub mod packagelist;

use uuid::Uuid;

pub use packagelist::Duration;
pub use packagelist::ItemSize;
pub use packagelist::ItemUsage;
pub use packagelist::PackageItem;
pub use packagelist::PackageList;
pub use packagelist::Period;
pub use packagelist::Preparation;
pub use packagelist::PreparationStep;

pub mod router;

pub mod db;

pub mod trip;
pub use trip::Trip;
pub use trip::TripParameters;
pub use trip::TripState;

pub fn get_list(id: Uuid) -> Option<packagelist::PackageList> {
    self::db::get_list(id).unwrap()
}

pub fn get_lists() -> Vec<packagelist::PackageList> {
    self::db::get_lists().unwrap()
}

pub fn get_packagelist_items(id: Uuid) -> Vec<packagelist::PackageItem> {
    self::db::get_list_items(id).unwrap()
}

pub fn get_preparation(list_id: Uuid, item_id: Uuid) -> Vec<packagelist::PreparationStep> {
    self::db::get_preparation(list_id, item_id).unwrap()
}

pub fn new_item(list_id: Uuid, item_name: String, item_count: i32) -> packagelist::PackageItem {
    let item = PackageItem::new(
        Uuid::new_v4(),
        item_name,
        ItemSize::None,
        item_count,
        ItemUsage::Singleton,
        Preparation::None,
    );
    self::db::save_item(list_id, item).unwrap()
}

pub fn get_trips() -> Vec<Trip> {
    self::db::get_trips().unwrap()
}
