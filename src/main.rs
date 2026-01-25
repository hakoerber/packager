use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::process::ExitCode;
use std::str::FromStr;

use packager::{
    AppState, ClientState, CommandError, DatabaseError, QueryError, StartError, auth, cli, models,
    routing, telemetry,
};
use tokio::net::TcpListener;

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
            MainError::Start(start_error) => write!(f, "{start_error}"),
            MainError::Command(command_error) => write!(f, "{command_error}"),
        }
    }
}

struct MainResult(Result<(), MainError>);

use database::Database as _;

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

#[tokio::main]
async fn main() -> MainResult {
    let args = match cli::Args::get() {
        Ok(args) => args,
        Err(e) => return e.into(),
    };

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
                match args.command {
                    cli::Command::Serve(serve_args) => {
                        if let Err(err) = database::DB::migrate(&args.database_url).await {
                            let command_error: CommandError = err.into();
                            return command_error.into();
                        }

                        let database_pool =
                            match database::DB::init_database_pool(&args.database_url).await {
                                Ok(pool) => pool,
                                Err(err) => return <_ as Into<StartError>>::into(err).into(),
                            };

                        let state = AppState {
                            database_pool,
                            client_state: ClientState::new(),
                            auth_config: if let Some(assume_user) =
                                serve_args.disable_auth_and_assume_user
                            {
                                auth::Config::Disabled { assume_user }
                            } else {
                                auth::Config::Enabled
                            },
                        };

                        // build our application with a route
                        let app = routing::router(state);
                        let app = telemetry::tracing::init_request_tracing(app);

                        let mut join_set = tokio::task::JoinSet::new();

                        let app = if args.enable_prometheus.into() {
                            // we `require_if()` prometheus port & bind when `enable_prometheus` is set, so
                            // this cannot fail

                            let bind = args.prometheus_bind.unwrap();
                            let port = args.prometheus_port.unwrap();

                            let ip = IpAddr::from_str(&bind);

                            let addr = match ip {
                                Err(e) => return <_ as Into<StartError>>::into((bind, e)).into(),
                                Ok(ip) => SocketAddr::from((ip, port)),
                            };

                            let (app, task) = telemetry::metrics::prometheus_server(app, addr);
                            join_set.spawn(task);
                            app
                        } else {
                            app
                        };

                        join_set.spawn(async move {
                            let addr = SocketAddr::from((
                                IpAddr::from_str(&serve_args.bind)
                                    .map_err(|e| (serve_args.bind, e))?,
                                serve_args.port,
                            ));

                            tracing::debug!("listening on {}", addr);

                            axum::serve(
                                TcpListener::bind(&addr)
                                    .await
                                    .map_err(|e| StartError::Bind {
                                        addr,
                                        message: e.to_string(),
                                    })?,
                                app,
                            )
                            .await
                            // Error = Infallible
                            .unwrap();
                            unreachable!()
                        });

                        // now we wait for all tasks. none of them are supposed to finish

                        // EXPECT: join_set cannot be empty as it will always at least contain the main_handle
                        let result = join_set
                            .join_next()
                            .await
                            .expect("join_set is empty, this is a bug");

                        // EXPECT: We never expect a JoinError, as all threads run infinitely
                        let result: Result<(), StartError> = result.expect("thread panicked");

                        // If we get an Ok(()), something weird happened
                        let result = result.expect_err("thread ran to completion");

                        return result.into();
                    }
                    cli::Command::Admin(admin_command) => match admin_command {
                        cli::Admin::User(cmd) => match cmd {
                            cli::UserCommand::Create(user) => {
                                let database_pool = match database::DB::init_database_pool(
                                    &args.database_url,
                                )
                                .await
                                {
                                    Ok(pool) => pool,
                                    Err(e) => return <_ as Into<StartError>>::into(e).into(),
                                };

                                let id = match models::user::create(
                                    &database_pool,
                                    models::user::NewUser {
                                        username: &user.username,
                                        fullname: &user.fullname,
                                    },
                                )
                                .await
                                .map_err(|error: DatabaseError| match error {
                                    DatabaseError::Query(QueryError::Duplicate {
                                        description: _,
                                    }) => CommandError::UserExists {
                                        username: user.username.clone(),
                                    },
                                    err => CommandError::Database(err),
                                }) {
                                    Ok(id) => id,
                                    Err(e) => {
                                        return e.into();
                                    }
                                };

                                println!(
                                    "User \"{}\" created successfully (id {})",
                                    &user.username, id
                                );
                            }
                        },
                    },
                    cli::Command::Migrate => {
                        if let Err(e) = database::DB::migrate(&args.database_url).await {
                            return <_ as Into<StartError>>::into(e).into();
                        }

                        println!("Migrations successfully applied");
                    }
                }
                MainResult(Ok(()))
            })
        },
    )
    .await
}
