use leptos::{logging::log, prelude::*, task::spawn_local};

#[cfg(feature = "ssr")]
use axum::{
    body::Body,
    extract::{Request, State},
    middleware,
    response::Response,
    Router,
};
#[cfg(feature = "ssr")]
use leptos_axum::{generate_route_list, LeptosRoutes};

use leptos_meta::{provide_meta_context, MetaTags, Script, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

#[cfg(feature = "ssr")]
use crate::{components::user::User, context::Context, state::AppState};

mod components;

#[cfg(feature = "ssr")]
pub mod state;

#[cfg(feature = "ssr")]
mod context;

#[cfg(feature = "ssr")]
pub mod error;

#[cfg(feature = "ssr")]
pub mod auth;

#[cfg(feature = "ssr")]
mod telemetry;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" />

        <Title text="Welcome to Packager" />

        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=TripList />
                </Routes>
            </main>
        </Router>
    }
}

#[server]
pub async fn get_trips() -> Result<Vec<crate::components::trip::Trip>, ServerFnError> {
    let state = expect_context::<AppState>();
    let user = expect_context::<User>();

    let trips =
        crate::components::trip::Trip::findall(&Context::build(user), &state.database_pool).await?;

    Ok(trips)
}

#[component]
fn TripList() -> impl IntoView {
    let trips = Resource::new(|| (), |_| get_trips());

    view! {
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || {
                trips.with(|trips| trips.as_ref().map(|result| {
                    match result {
                        Ok(trips) => view! {
                            <ul>
                                {trips
                                    .into_iter()
                                    .map(|trip| view! {
                                        <li>{trip.name.clone()}</li>
                                    })
                                    .collect_view()}
                            </ul>
                        }.into_any(),

                        Err(e) => view! {
                            <p>{e.to_string()}</p>
                        }.into_any(),
                    }
                }))
            }}
        </Suspense>
    }
}

#[cfg(feature = "ssr")]
async fn leptos_handler(State(state): State<AppState>, req: Request<Body>) -> Response {
    let user = req
        .extensions()
        .get::<User>()
        .cloned()
        .expect("auth middleware inserts user");

    let options = state.leptos_options.clone();

    leptos_axum::render_app_to_stream_with_context(
        move || {
            provide_context(state.clone());
            provide_context(user.clone());
        },
        move || shell(options.clone()),
    )(req)
    .await
}

#[cfg(feature = "ssr")]
pub fn router(state: AppState) -> Router {
    let routes = generate_route_list(App);

    let protected = Router::new()
        .leptos_routes_with_handler(routes, leptos_handler)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize,
        ));

    Router::new()
        .merge(protected)
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(state)
}
