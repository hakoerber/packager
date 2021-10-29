use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TripItemStatus {
    Pending,
    Ready,
    Packed,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TripParameters {
    pub days: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TripPackageList {
    pub id: Uuid,
    pub name: String,
}

impl TripPackageList {
    fn construct(item: (Uuid, String)) -> TripPackageList {
        TripPackageList {
            id: item.0,
            name: item.1,
        }
    }

    fn construct_vec(items: Vec<(Uuid, String)>) -> Vec<TripPackageList> {
        items.into_iter().map(TripPackageList::construct).collect()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TripState {
    Planned,
    Packing,
    Active,
    Finished,
}

impl rusqlite::types::FromSql for TripState {
    fn column_result(value: rusqlite::types::ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(TripState::Planned),
            2 => Ok(TripState::Packing),
            3 => Ok(TripState::Active),
            4 => Ok(TripState::Finished),
            v => Err(rusqlite::types::FromSqlError::OutOfRange(v)),
        }
    }
}

impl rusqlite::types::ToSql for TripState {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput> {
        let v = rusqlite::types::Value::Integer(match self {
            TripState::Planned => 1,
            TripState::Packing => 2,
            TripState::Active => 3,
            TripState::Finished => 4,
        });
        rusqlite::Result::Ok(rusqlite::types::ToSqlOutput::Owned(v))
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trip {
    pub id: Uuid,
    pub name: String,
    pub date: String,
    pub parameters: TripParameters,
    pub package_lists: Vec<TripPackageList>,
    pub state: TripState,
}

impl Trip {
    pub fn new(
        id: Uuid,
        name: String,
        date: String,
        parameters: TripParameters,
        state: TripState,
    ) -> Trip {
        Trip {
            id,
            name,
            date,
            parameters,
            package_lists: vec![],
            state,
        }
    }

    pub fn from_package_list(
        id: Uuid,
        name: String,
        date: String,
        parameters: TripParameters,
        package_lists: Vec<(Uuid, String)>,
        state: TripState,
    ) -> Trip {
        let lists = TripPackageList::construct_vec(package_lists);
        Trip {
            id,
            name,
            date,
            parameters,
            package_lists: lists,
            state,
        }
    }

    pub fn set_package_lists(&mut self, package_lists: Vec<(Uuid, String)>) {
        let v = TripPackageList::construct_vec(package_lists);
        self.package_lists = v;
    }
}
