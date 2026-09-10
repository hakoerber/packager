use std::{fmt, process::ExitCode};

use clap::{Parser, ValueEnum};
use leptos::{logging::log, prelude::*};

use database::Database as _;
use packager_leptos::{
    error::{CommandError, StartError},
    telemetry,
};

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum BoolArg {
    True,
    False,
}

impl From<BoolArg> for bool {
    fn from(arg: BoolArg) -> Self {
        arg.bool()
    }
}

impl BoolArg {
    fn bool(self) -> bool {
        match self {
            Self::True => true,
            Self::False => false,
        }
    }
}

// this is required because the required_if* functions match against the
// *raw* value, before parsing is done
impl From<BoolArg> for clap::builder::OsStr {
    fn from(arg: BoolArg) -> Self {
        match arg {
            BoolArg::True => "true".into(),
            BoolArg::False => "false".into(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub database_url: String,

    #[arg(long, name = "USERNAME")]
    pub disable_auth_and_assume_user: Option<String>,

    #[arg(long, value_enum, default_value_t = BoolArg::False)]
    pub enable_opentelemetry: BoolArg,

    #[arg(long, value_enum, default_value_t = BoolArg::False)]
    pub enable_tokio_console: BoolArg,

    #[arg(long, value_enum, default_value_t = BoolArg::False)]
    pub enable_prometheus: BoolArg,
}

enum MainError {
    Start(StartError),
    Command(CommandError),
}

impl From<StartError> for MainError {
    fn from(value: StartError) -> Self {
        Self::Start(value)
    }
}

impl From<CommandError> for MainError {
    fn from(value: CommandError) -> Self {
        Self::Command(value)
    }
}

impl fmt::Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Start(err) => write!(f, "{err}"),
            Self::Command(err) => write!(f, "{err}"),
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

impl From<CommandError> for MainResult {
    fn from(error: CommandError) -> Self {
        Self(Err(error.into()))
    }
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> MainResult {
    let args = Args::parse();

    telemetry::tracing::init(
        if args.enable_opentelemetry.into() {
            telemetry::tracing::OpenTelemetryConfig::Enabled
        } else {
            telemetry::tracing::OpenTelemetryConfig::Disabled
        },
        if args.enable_tokio_console.into() {
            telemetry::tracing::TokioConsoleConfig::Enabled
        } else {
            telemetry::tracing::TokioConsoleConfig::Disabled
        },
        args,
        |args| {
            Box::pin(async move {
                if let Err(err) = database::DB::migrate(&args.database_url).await {
                    let command_error: CommandError = err.into();
                    return command_error.into();
                }

                let conf = get_configuration(None).unwrap();
                let addr = conf.leptos_options.site_addr;

                let database_pool = match database::DB::init_database_pool(&args.database_url).await
                {
                    Ok(pool) => pool,
                    Err(err) => {
                        return <_ as Into<packager_leptos::error::StartError>>::into(err).into()
                    }
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
                let router = telemetry::tracing::init_request_tracing(router);

                log!("listening on http://{}", &addr);
                let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
                axum::serve(listener, router.into_make_service())
                    .await
                    .unwrap();

                MainResult(Ok(()))
            })
        },
    )
    .await
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    ()
}
