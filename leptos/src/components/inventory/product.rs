use uuid::Uuid;

#[derive(Debug)]
pub struct Product {
    #[allow(dead_code)]
    pub id: Uuid,
    pub name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
}
