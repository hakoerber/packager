use std::convert;
use std::fmt;

use sqlx::error::DatabaseError as _;

pub enum DatabaseError {
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
}

impl fmt::Display for DatabaseError {
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
        }
    }
}

pub enum QueryError {
    /// Errors that are caused by wrong input data, e.g. ids that cannot be found, or
    /// inserts that violate unique constraints
    Constraint {
        description: String,
    },
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
            Self::Constraint { description } => {
                write!(f, "SQL constraint error: {description}")
            }
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
    Database(DatabaseError),
    Query(QueryError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Database(error) => write!(f, "{}", error),
            Self::Query(error) => write!(f, "{}", error),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // defer to Display
        write!(f, "SQL error: {self}")
    }
}

impl convert::From<uuid::Error> for Error {
    fn from(value: uuid::Error) -> Self {
        Error::Database(DatabaseError::Uuid {
            description: value.to_string(),
        })
    }
}

impl convert::From<time::error::Format> for Error {
    fn from(value: time::error::Format) -> Self {
        Error::Database(DatabaseError::TimeParse {
            description: value.to_string(),
        })
    }
}

impl convert::From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Error::Query(QueryError::NotFound {
                description: value.to_string(),
            }),
            sqlx::Error::Database(ref error) => {
                let sqlite_error = error.downcast_ref::<sqlx::sqlite::SqliteError>();
                if let Some(code) = sqlite_error.code() {
                    match &*code {
                        // SQLITE_CONSTRAINT_FOREIGNKEY
                        "787" => Error::Query(QueryError::Constraint {
                            description: format!("foreign key reference not found"),
                        }),
                        // SQLITE_CONSTRAINT_UNIQUE
                        "2067" => Error::Query(QueryError::Constraint {
                            description: format!("item with unique constraint already exists",),
                        }),
                        _ => Error::Database(DatabaseError::Sql {
                            description: format!(
                                "got error with unknown code: {}",
                                sqlite_error.to_string()
                            ),
                        }),
                    }
                } else {
                    Error::Database(DatabaseError::Sql {
                        description: format!(
                            "got error without code: {}",
                            sqlite_error.to_string()
                        ),
                    })
                }
            }
            _ => Error::Database(DatabaseError::Sql {
                description: format!("got unknown error: {}", value.to_string()),
            }),
        }
    }
}

impl convert::From<time::error::Parse> for Error {
    fn from(value: time::error::Parse) -> Self {
        Error::Database(DatabaseError::TimeParse {
            description: value.to_string(),
        })
    }
}

impl std::error::Error for Error {}
