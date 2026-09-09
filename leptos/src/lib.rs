pub mod app;

mod components;

#[cfg(feature = "ssr")]
mod state;

#[cfg(feature = "ssr")]
mod context;

#[cfg(feature = "ssr")]
mod error;

#[cfg(feature = "ssr")]
mod auth;

#[cfg(feature = "ssr")]
mod telemetry;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
