#[derive(Debug, Clone)]
pub enum Currency {
    Eur(rust_decimal::Decimal),
}
