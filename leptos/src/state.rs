use axum::extract::FromRef;
use leptos::config::LeptosOptions;

#[derive(Clone, Debug)]
pub struct AppState {
    pub database_pool: database::Pool,
    pub auth_config: super::auth::Config,
    pub leptos_options: LeptosOptions,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}
