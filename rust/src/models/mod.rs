pub mod inventory;
pub mod trips;

mod error;
pub use error::{DatabaseError, Error, QueryError};

mod consts {
    use time::{format_description::FormatItem, macros::format_description};

    pub(super) const DATE_FORMAT: &[FormatItem<'static>] =
        format_description!("[year]-[month]-[day]");
}
