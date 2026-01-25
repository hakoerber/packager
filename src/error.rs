use std::{convert::Infallible, fmt, net::SocketAddr};

use crate::view;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub use database::{Error as DatabaseError, QueryError};

#[derive(Debug)]
pub enum RequestError {
    EmptyFormElement { name: String },
    RefererNotFound,
    RefererInvalid { message: String },
    NotFound { message: String },
    Auth { inner: AuthError },
    Transport { inner: hyper::Error },
}

impl std::error::Error for RequestError {}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyFormElement { name } => write!(f, "Form element {name} cannot be empty"),
            Self::RefererNotFound => write!(f, "Referer header not found"),
            Self::RefererInvalid { message } => write!(f, "Referer header invalid: {message}"),
            Self::NotFound { message } => write!(f, "Not found: {message}"),
            Self::Auth { inner } => {
                write!(f, "Authentication failed: {inner}")
            }
            Self::Transport { inner } => {
                write!(f, "HTTP error: {inner}")
            }
        }
    }
}

#[derive(Debug)]
pub enum DataError {
    NotFound { description: String },
}

impl std::error::Error for DataError {}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotFound { description } => {
                write!(f, "{description}")
            }
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    AuthenticationUserNotFound { username: String },
    AuthenticationHeaderMissing,
    AuthenticationHeaderInvalid { message: String },
}

impl AuthError {
    #[must_use]
    pub fn to_prom_metric_name(&self) -> &'static str {
        match self {
            Self::AuthenticationUserNotFound { username: _ } => "user_not_found",
            Self::AuthenticationHeaderMissing => "header_missing",
            Self::AuthenticationHeaderInvalid { message: _ } => "header_invalid",
        }
    }

    pub fn trace(&self) {
        match self {
            Self::AuthenticationUserNotFound { username } => {
                tracing::info!(username, "auth failed, user not found");
            }
            Self::AuthenticationHeaderMissing => {
                tracing::info!("auth failed, auth header missing");
            }
            Self::AuthenticationHeaderInvalid { message } => {
                tracing::info!(message, "auth failed, auth header invalid");
            }
        }
    }
}

impl<'a> AuthError {
    #[must_use]
    pub fn to_prom_labels(&'a self) -> Vec<(&'static str, String)> {
        match self {
            Self::AuthenticationUserNotFound { username } => vec![("username", username.clone())],
            Self::AuthenticationHeaderMissing
            | Self::AuthenticationHeaderInvalid { message: _ } => vec![],
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AuthenticationUserNotFound { username } => {
                write!(f, "User \"{username}\" not found")
            }
            Self::AuthenticationHeaderMissing => write!(f, "Authentication header not found"),
            Self::AuthenticationHeaderInvalid { message } => {
                write!(f, "Authentication header invalid: {message}")
            }
        }
    }
}

impl From<AuthError> for RunError {
    fn from(e: AuthError) -> Self {
        Self::Request(RequestError::Auth { inner: e })
    }
}

impl std::error::Error for AuthError {}

#[derive(Debug)]
pub enum RunError {
    Request(RequestError),
    Database(database::Error),
    Data(DataError),
}

impl std::error::Error for RunError {}

impl From<Infallible> for RunError {
    fn from(_value: Infallible) -> Self {
        unreachable!()
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Request(request_error) => write!(f, "Request error: {request_error}"),
            Self::Database(db_error) => write!(f, "{db_error}"),
            Self::Data(data_error) => write!(f, "{data_error}"),
        }
    }
}

impl From<database::Error> for RunError {
    fn from(value: database::Error) -> Self {
        Self::Database(value)
    }
}

impl From<sqlx::Error> for RunError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<hyper::Error> for RunError {
    fn from(value: hyper::Error) -> Self {
        Self::Request(RequestError::Transport { inner: value })
    }
}

impl IntoResponse for RunError {
    fn into_response(self) -> Response {
        match self {
            Self::Database(ref db_error) => match db_error {
                database::error::Error::Database(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    view::ErrorPage::build(&self.to_string()),
                ),
                database::error::Error::Query(error) => match error {
                    database::error::QueryError::NotFound { description } => {
                        (StatusCode::NOT_FOUND, view::ErrorPage::build(description))
                    }
                    _ => (
                        StatusCode::BAD_REQUEST,
                        view::ErrorPage::build(&error.to_string()),
                    ),
                },
            },
            Self::Request(request_error) => match request_error {
                RequestError::RefererNotFound => (
                    StatusCode::BAD_REQUEST,
                    view::ErrorPage::build("no referer header found"),
                ),
                RequestError::RefererInvalid { message } => (
                    StatusCode::BAD_REQUEST,
                    view::ErrorPage::build(&format!("referer could not be converted: {message}")),
                ),
                RequestError::EmptyFormElement { name } => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    view::ErrorPage::build(&format!("empty form element: {name}")),
                ),
                RequestError::NotFound { message } => (
                    StatusCode::NOT_FOUND,
                    view::ErrorPage::build(&format!("not found: {message}")),
                ),
                RequestError::Auth { inner: e } => (
                    StatusCode::UNAUTHORIZED,
                    view::ErrorPage::build(&format!("authentication failed: {e}")),
                ),
                RequestError::Transport { inner } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    view::ErrorPage::build(&inner.to_string()),
                ),
            },
            Self::Data(data_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                view::ErrorPage::build(&data_error.to_string()),
            ),
        }
        .into_response()
    }
}

#[derive(Debug)]
pub enum StartError {
    Bind { addr: SocketAddr, message: String },
    Call { message: String },
    Exec(tokio::task::JoinError),
    DatabaseInit(database::InitError),
    AddrParse { input: String, message: String },
}

impl std::error::Error for StartError {}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bind { message, addr } => {
                write!(f, "error binding network interface {addr}: {message}")
            }
            Self::Call { message } => {
                write!(f, "invalid invocation: {message}")
            }
            Self::Exec(join_error) => write!(f, "{join_error}"),
            Self::DatabaseInit(start_error) => write!(f, "{start_error}"),
            Self::AddrParse { message, input } => {
                write!(f, "error parsing \"{input}\": {message}")
            }
        }
    }
}

impl From<tokio::task::JoinError> for StartError {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::Exec(value)
    }
}

impl From<database::InitError> for StartError {
    fn from(value: database::InitError) -> Self {
        Self::DatabaseInit(value)
    }
}

impl From<(String, std::net::AddrParseError)> for StartError {
    fn from((input, error): (String, std::net::AddrParseError)) -> Self {
        Self::AddrParse {
            input,
            message: error.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum CommandError {
    Start(StartError),
    Database(database::Error),
    UserExists { username: String },
}

impl std::error::Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Start(start_error) => {
                write!(f, "{start_error}")
            }
            Self::Database(db_error) => write!(f, "{db_error}"),
            Self::UserExists { username } => {
                write!(f, "user \"{username}\" already exists")
            }
        }
    }
}

impl From<tokio::task::JoinError> for CommandError {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::Start(value.into())
    }
}

impl From<database::InitError> for CommandError {
    fn from(value: database::InitError) -> Self {
        Self::Start(value.into())
    }
}

impl From<(String, std::net::AddrParseError)> for CommandError {
    fn from((input, error): (String, std::net::AddrParseError)) -> Self {
        Self::Start(StartError::AddrParse {
            input,
            message: error.to_string(),
        })
    }
}

impl From<StartError> for CommandError {
    fn from(value: StartError) -> Self {
        Self::Start(value)
    }
}
