pub mod user;

pub use user::User;

#[derive(Debug, Clone)]
pub enum Currency {
    Eur(rust_decimal::Decimal),
}
