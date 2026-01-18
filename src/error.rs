use std::{fmt, net::SocketAddr};

use crate::{db, view};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub use crate::db::error::DataError;
pub use crate::db::error::{Error as DatabaseError, QueryError};

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

impl From<AuthError> for Error {
    fn from(e: AuthError) -> Self {
        Self::Request(RequestError::Auth { inner: e })
    }
}

impl std::error::Error for AuthError {}

#[derive(Debug)]
pub enum Error {
    Request(RequestError),
    Start(StartError),
    Command(CommandError),
    Exec(tokio::task::JoinError),
    Database(crate::db::error::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Request(request_error) => write!(f, "Request error: {request_error}"),
            Self::Start(start_error) => write!(f, "{start_error}"),
            Self::Command(command_error) => write!(f, "{command_error}"),
            Self::Exec(join_error) => write!(f, "{join_error}"),
            Self::Database(db_error) => write!(f, "{db_error}"),
        }
    }
}

impl From<StartError> for Error {
    fn from(value: StartError) -> Self {
        Self::Start(value)
    }
}

impl From<crate::db::error::Error> for Error {
    fn from(value: crate::db::error::Error) -> Self {
        Self::Database(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<hyper::Error> for Error {
    fn from(value: hyper::Error) -> Self {
        Self::Request(RequestError::Transport { inner: value })
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::Exec(value)
    }
}

impl From<(String, std::net::AddrParseError)> for Error {
    fn from((input, error): (String, std::net::AddrParseError)) -> Self {
        Self::Start(StartError::AddrParse {
            input,
            message: error.to_string(),
        })
    }
}

impl From<(String, url::ParseError)> for StartError {
    fn from((url, error): (String, url::ParseError)) -> Self {
        Self::UrlParse {
            url,
            message: error.to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Database(ref db_error) => match db_error {
                db::error::Error::Database(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    view::ErrorPage::build(&self.to_string()),
                ),
                db::error::Error::Query(error) => match error {
                    db::error::QueryError::NotFound { description } => {
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
            Self::Start(start_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                view::ErrorPage::build(&start_error.to_string()),
            ),
            Self::Command(command_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                view::ErrorPage::build(&command_error.to_string()),
            ),
            Self::Exec(join_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                view::ErrorPage::build(&join_error.to_string()),
            ),
        }
        .into_response()
    }
}

#[derive(Debug)]
pub enum StartError {
    Call { message: String },
    DatabaseInit { message: String },
    DatabaseMigration { message: String },
    AddrParse { input: String, message: String },
    Bind { addr: SocketAddr, message: String },
    UrlParse { url: String, message: String },
}

impl std::error::Error for StartError {}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Call { message } => {
                write!(f, "invalid invocation: {message}")
            }
            Self::DatabaseInit { message } => {
                write!(f, "database initialization error: {message}")
            }
            Self::DatabaseMigration { message } => {
                write!(f, "database migration error: {message}")
            }
            Self::AddrParse { message, input } => {
                write!(f, "error parsing \"{input}\": {message}")
            }
            Self::Bind { message, addr } => {
                write!(f, "error binding network interface {addr}: {message}")
            }
            Self::UrlParse { url, message } => {
                write!(f, "error parsing url {url}: {message}")
            }
        }
    }
}

impl From<sqlx::Error> for StartError {
    fn from(value: sqlx::Error) -> Self {
        Self::DatabaseInit {
            message: value.to_string(),
        }
    }
}

impl From<sqlx::migrate::MigrateError> for StartError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self::DatabaseMigration {
            message: value.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum CommandError {
    UserExists { username: String },
}

impl std::error::Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UserExists { username } => {
                write!(f, "user \"{username}\" already exists")
            }
        }
    }
}
