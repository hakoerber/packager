use axum::Router;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_meta::{provide_meta_context, MetaTags, Script, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;

    let router = packager_leptos::router(conf.leptos_options);

    let database_pool = match database::DB::init_database_pool(&args.database_url).await {
        Ok(pool) => pool,
        Err(err) => return <_ as Into<StartError>>::into(err).into(),
    };

    let state = AppState {
        database_pool,
        client_state: ClientState::new(),
        auth_config: if let Some(assume_user) = serve_args.disable_auth_and_assume_user {
            auth::Config::Disabled { assume_user }
        } else {
            auth::Config::Enabled
        },
    };

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    ()
}
