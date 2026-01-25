use crate::{DataError, Error};

pub type Money = sqlx::postgres::types::PgMoney;

pub fn try_into_date_range(
    start: time::Date,
    end: time::Date,
) -> Result<sqlx::postgres::types::PgRange<time::Date>, Error> {
    Ok(sqlx::postgres::types::PgRange {
        start: core::ops::Bound::Included(start),
        end: core::ops::Bound::Excluded(end.next_day().ok_or_else(|| {
            Error::Database(DataError::Overflow {
                description: "upper bound is Date::MAX, would overflow exclusive upper bound"
                    .into(),
            })
        })?),
    })
}

pub fn try_into_enum<F, R>(value: &str, f: F) -> Result<R, Error>
where
    F: FnOnce(&str) -> Option<R>,
{
    match f(value) {
        Some(r) => Ok(r),
        None => Err(Error::Database(DataError::Enum {
            description: format!("{value} is not a valid value for enum"),
        })),
    }
}
