#[derive(Clone, Debug)]
pub struct AppState {
    pub database_pool: database::Pool,
    pub auth_config: super::auth::Config,
}
