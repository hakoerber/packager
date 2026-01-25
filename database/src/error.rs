use std::{fmt, net::SocketAddr};

use sqlx::error::DatabaseError as _;

#[derive(Debug)]
pub enum InitError {
    DatabaseInit { message: String },
    DatabaseMigration { message: String },
    AddrParse { input: String, message: String },
    Bind { addr: SocketAddr, message: String },
    UrlParse { url: String, message: String },
}

impl std::error::Error for InitError {}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
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

impl From<sqlx::Error> for InitError {
    fn from(value: sqlx::Error) -> Self {
        Self::DatabaseInit {
            message: value.to_string(),
        }
    }
}

impl From<sqlx::migrate::MigrateError> for InitError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self::DatabaseMigration {
            message: value.to_string(),
        }
    }
}

pub enum DataError {
    /// Errors we can receive **from** the database that are caused by connection
    /// problems or schema problems (e.g. we get a return value that does not fit our enum,
    /// or a wrongly formatted date)
    Sql {
        description: String,
    },
    Uuid {
        description: String,
    },
    Enum {
        description: String,
    },
    TimeParse {
        description: String,
    },
    Overflow {
        description: String,
    },
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Sql { description } => {
                write!(f, "SQL error: {description}")
            }
            Self::Uuid { description } => {
                write!(f, "UUID error: {description}")
            }
            Self::Enum { description } => {
                write!(f, "Enum error: {description}")
            }
            Self::TimeParse { description } => {
                write!(f, "Date parse error: {description}")
            }
            Self::Overflow { description } => {
                write!(f, "Overflow error: {description}")
            }
        }
    }
}

pub enum QueryError {
    /// Errors that are caused by wrong input data, e.g. ids that cannot be found, or
    /// inserts that violate unique constraints
    Duplicate {
        description: String,
    },
    NotFound {
        description: String,
    },
    ReferenceNotFound {
        description: String,
    },
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Duplicate { description } => {
                write!(f, "Duplicate data entry: {description}")
            }
            Self::NotFound { description } => {
                write!(f, "not found: {description}")
            }
            Self::ReferenceNotFound { description } => {
                write!(f, "SQL foreign key reference was not found: {description}")
            }
        }
    }
}

pub enum Error {
    Database(DataError),
    Query(QueryError),
}

impl From<DataError> for Error {
    fn from(value: DataError) -> Self {
        Self::Database(value)
    }
}

impl From<QueryError> for Error {
    fn from(value: QueryError) -> Self {
        Self::Query(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Database(error) => write!(f, "{error}"),
            Self::Query(error) => write!(f, "{error}"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // defer to Display
        write!(f, "SQL error: {self}")
    }
}

impl From<uuid::Error> for Error {
    fn from(value: uuid::Error) -> Self {
        Self::Database(DataError::Uuid {
            description: value.to_string(),
        })
    }
}

impl From<time::error::Format> for Error {
    fn from(value: time::error::Format) -> Self {
        Self::Database(DataError::TimeParse {
            description: value.to_string(),
        })
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::Query(QueryError::NotFound {
                description: value.to_string(),
            }),
            sqlx::Error::Database(ref error) => {
                let error = error.downcast_ref::<sqlx::postgres::PgDatabaseError>();
                if error.is_unique_violation() {
                    Self::Query(QueryError::Duplicate {
                        description: "item with unique constraint already exists".to_string(),
                    })
                } else {
                    Self::Database(DataError::Sql {
                        description: format!("got unknown error: {error}"),
                    })
                }
            }
            _ => Self::Database(DataError::Sql {
                description: format!("got unknown error: {value}"),
            }),
        }
    }
}

impl From<time::error::Parse> for Error {
    fn from(value: time::error::Parse) -> Self {
        Self::Database(DataError::TimeParse {
            description: value.to_string(),
        })
    }
}

impl From<(String, url::ParseError)> for InitError {
    fn from((url, error): (String, url::ParseError)) -> Self {
        Self::UrlParse {
            url,
            message: error.to_string(),
        }
    }
}

impl std::error::Error for Error {}
