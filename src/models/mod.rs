pub mod user;

pub(crate) use user::User;

#[derive(Debug, Clone)]
pub(crate) enum Currency {
    Eur(rust_decimal::Decimal),
}
