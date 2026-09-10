use std::fmt;

pub mod inventory;
pub mod todo;
pub mod trip;
pub mod user;

pub enum Component {
    Inventory,
    User,
    Trip,
    Todo,
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Inventory => "inventory",
                Self::User => "user",
                Self::Trip => "trips",
                Self::Todo => "todo",
            }
        )
    }
}
