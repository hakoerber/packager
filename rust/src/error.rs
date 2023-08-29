use std::fmt;

use crate::models;
use crate::view;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum RequestError {
    EmptyFormElement { name: String },
    RefererNotFound,
    RefererInvalid { message: String },
    NotFound { message: String },
    AuthenticationUserNotFound { username: String },
    AuthenticationHeaderMissing,
    AuthenticationHeaderInvalid { message: String },
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
            Self::AuthenticationUserNotFound { username } => {
                write!(f, "User \"{username}\" not found")
            }
            Self::AuthenticationHeaderMissing => write!(f, "Authentication header not found"),
            Self::AuthenticationHeaderInvalid { message } => {
                write!(f, "Authentication header invalid: {message}")
            }
            Self::Transport { inner } => {
                write!(f, "HTTP error: {inner}")
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Model(models::Error),
    Request(RequestError),
    Start(StartError),
    Command(CommandError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Model(model_error) => write!(f, "Model error: {model_error}"),
            Self::Request(request_error) => write!(f, "Request error: {request_error}"),
            Self::Start(start_error) => write!(f, "{start_error}"),
            Self::Command(command_error) => write!(f, "{command_error}"),
        }
    }
}

impl From<models::Error> for Error {
    fn from(value: models::Error) -> Self {
        Self::Model(value)
    }
}

impl From<StartError> for Error {
    fn from(value: StartError) -> Self {
        Self::Start(value)
    }
}

impl From<hyper::Error> for Error {
    fn from(value: hyper::Error) -> Self {
        Self::Request(RequestError::Transport { inner: value })
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Model(ref model_error) => match model_error {
                models::Error::Database(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    view::ErrorPage::build(&self.to_string()),
                ),
                models::Error::Query(error) => match error {
                    models::QueryError::NotFound { description } => {
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
                RequestError::AuthenticationUserNotFound { username: _ } => (
                    StatusCode::BAD_REQUEST,
                    view::ErrorPage::build(&request_error.to_string()),
                ),
                RequestError::AuthenticationHeaderMissing
                | RequestError::AuthenticationHeaderInvalid { message: _ } => (
                    StatusCode::UNAUTHORIZED,
                    view::ErrorPage::build(&request_error.to_string()),
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
        }
        .into_response()
    }
}

#[derive(Debug)]
pub enum StartError {
    DatabaseInitError { message: String },
    DatabaseMigrationError { message: String },
}

impl std::error::Error for StartError {}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DatabaseInitError { message } => {
                write!(f, "database initialization error: {message}")
            }
            Self::DatabaseMigrationError { message } => {
                write!(f, "database migration error: {message}")
            }
        }
    }
}

impl From<sqlx::Error> for StartError {
    fn from(value: sqlx::Error) -> Self {
        Self::DatabaseInitError {
            message: value.to_string(),
        }
    }
}

impl From<sqlx::migrate::MigrateError> for StartError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self::DatabaseMigrationError {
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
