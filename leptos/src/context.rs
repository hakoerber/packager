use super::components::user::User;

#[derive(Clone, Debug)]
pub struct Context {
    pub user: User,
}

#[cfg(feature = "ssr")]
impl Context {
    fn build(user: User) -> Self {
        Self { user }
    }
}
