use std::{fmt, process::ExitCode};

use axum::Router;
use clap::{Parser, Subcommand, ValueEnum};
use database::Database as _;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_meta::{provide_meta_context, MetaTags, Script, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use packager_leptos::error::StartError;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub database_url: String,

    #[arg(long, name = "USERNAME")]
    pub disable_auth_and_assume_user: Option<String>,
}

enum MainError {
    Start(StartError),
}

impl From<StartError> for MainError {
    fn from(value: StartError) -> Self {
        Self::Start(value)
    }
}

impl fmt::Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Start(err) => write!(f, "{err}"),
        }
    }
}

struct MainResult(Result<(), MainError>);

impl std::process::Termination for MainResult {
    fn report(self) -> std::process::ExitCode {
        match self.0 {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {e}");
                ExitCode::FAILURE
            }
        }
    }
}

impl From<StartError> for MainResult {
    fn from(error: StartError) -> Self {
        Self(Err(error.into()))
    }
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> MainResult {
    let args = Args::parse();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;

    let database_pool = match database::DB::init_database_pool(&args.database_url).await {
        Ok(pool) => pool,
        Err(err) => return <_ as Into<packager_leptos::error::StartError>>::into(err).into(),
    };

    let state = packager_leptos::state::AppState {
        leptos_options: conf.leptos_options,
        database_pool,
        auth_config: if let Some(assume_user) = args.disable_auth_and_assume_user {
            packager_leptos::auth::Config::Disabled { assume_user }
        } else {
            packager_leptos::auth::Config::Enabled
        },
    };

    let router = packager_leptos::router(state);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();

    MainResult(Ok(()))
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    ()
}
