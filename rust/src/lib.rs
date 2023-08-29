use axum::{extract::State, http::header::HeaderValue, middleware::Next, response::IntoResponse};

use hyper::Request;

use uuid::Uuid;

use std::fmt;

pub mod error;
pub mod models;
pub mod routing;
pub mod sqlite;

mod html;
mod view;

pub use error::{Error, RequestError, StartError};

#[derive(Clone)]
pub enum AuthConfig {
    Enabled,
    Disabled { assume_user: String },
}

#[derive(Clone)]
pub struct AppState {
    pub database_pool: sqlite::Pool<sqlite::Sqlite>,
    pub client_state: ClientState,
    pub auth_config: AuthConfig,
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

enum HtmxEvents {
    TripItemEdited,
}

impl From<HtmxEvents> for HeaderValue {
    fn from(val: HtmxEvents) -> Self {
        HeaderValue::from_static(val.to_str())
    }
}

impl HtmxEvents {
    fn to_str(&self) -> &'static str {
        match self {
            Self::TripItemEdited => "TripItemEdited",
        }
    }
}

async fn authorize<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, Error> {
    let current_user = match state.auth_config {
        AuthConfig::Disabled { assume_user } => {
            match models::user::User::find_by_name(&state.database_pool, &assume_user).await? {
                Some(user) => user,
                None => {
                    return Err(Error::Request(RequestError::AuthenticationUserNotFound {
                        username: assume_user,
                    }))
                }
            }
        }
        AuthConfig::Enabled => {
            let Some(username) = request.headers().get("x-auth-username") else {
                return Err(Error::Request(RequestError::AuthenticationHeaderMissing));
            };

            let username = username
                .to_str()
                .map_err(|error| {
                    Error::Request(RequestError::AuthenticationHeaderInvalid {
                        message: error.to_string(),
                    })
                })?
                .to_string();

            match models::user::User::find_by_name(&state.database_pool, &username).await? {
                Some(user) => user,
                None => {
                    return Err(Error::Request(RequestError::AuthenticationUserNotFound {
                        username,
                    }))
                }
            }
        }
    };

    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}
