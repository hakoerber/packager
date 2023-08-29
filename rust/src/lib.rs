use uuid::Uuid;

use std::fmt;

pub mod auth;
pub mod error;
pub mod htmx;
pub mod models;
pub mod routing;
pub mod sqlite;

mod view;

pub use error::{CommandError, Error, RequestError, StartError};

#[derive(Clone)]
pub struct AppState {
    pub database_pool: sqlite::Pool<sqlite::Sqlite>,
    pub client_state: ClientState,
    pub auth_config: auth::Config,
}

#[derive(Clone)]
pub struct Context {
    user: models::user::User,
}

impl Context {
    fn build(user: models::user::User) -> Self {
        Self { user }
    }
}

#[derive(Clone)]
pub struct ClientState {
    pub active_category_id: Option<Uuid>,
    pub edit_item: Option<Uuid>,
    pub trip_edit_attribute: Option<models::trips::TripAttribute>,
    pub trip_type_edit: Option<Uuid>,
}

impl ClientState {
    pub fn new() -> Self {
        ClientState {
            active_category_id: None,
            edit_item: None,
            trip_edit_attribute: None,
            trip_type_edit: None,
        }
    }
}

impl Default for ClientState {
    fn default() -> Self {
        Self::new()
    }
}

struct UriPath(String);

impl fmt::Display for UriPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> Into<&'a str> for &'a UriPath {
    fn into(self) -> &'a str {
        self.0.as_str()
    }
}

#[derive(PartialEq, Eq)]
pub enum TopLevelPage {
    Inventory,
    Trips,
}

impl TopLevelPage {
    fn id(&self) -> &'static str {
        match self {
            Self::Inventory => "inventory",
            Self::Trips => "trips",
        }
    }

    fn path(&self) -> UriPath {
        UriPath(
            match self {
                Self::Inventory => "/inventory/",
                Self::Trips => "/trips/",
            }
            .to_string(),
        )
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Inventory => "Inventory",
            Self::Trips => "Trips",
        }
    }
}
