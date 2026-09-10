use super::components::user::User;

#[derive(Clone, Debug)]
pub struct Context {
    pub user: User,
}

#[cfg(feature = "ssr")]
impl Context {
    pub fn build(user: User) -> Self {
        Self { user }
    }
}
