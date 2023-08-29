use std::net::{IpAddr, SocketAddr};
use std::process::ExitCode;
use std::str::FromStr;

use clap::{Parser, Subcommand};

use packager::{auth, models, routing, sqlite, AppState, ClientState, Error};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    database_url: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Serve(Serve),
    #[command(subcommand)]
    Admin(Admin),
    Migrate,
}

#[derive(Parser, Debug)]
struct Serve {
    #[arg(long, default_value_t = 3000)]
    port: u16,
    #[arg(long)]
    bind: String,
    #[arg(long, name = "USERNAME")]
    disable_auth_and_assume_user: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Admin {
    #[command(subcommand)]
    User(UserCommand),
}

#[derive(Subcommand, Debug)]
enum UserCommand {
    Create(UserCreate),
}

#[derive(Parser, Debug)]
struct UserCreate {
    #[arg(long)]
    username: String,
    #[arg(long)]
    fullname: String,
}

struct MainResult(Result<(), Error>);

impl std::process::Termination for MainResult {
    fn report(self) -> std::process::ExitCode {
        match self.0 {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {e}");
                ExitCode::FAILURE
            }
        }
    }
}

impl From<Error> for MainResult {
    fn from(error: Error) -> Self {
        Self(Err(error))
    }
}

fn init_tracing() {
    use std::io::stdout;
    use tracing::Level;
    use tracing_subscriber::{
        filter::Targets,
        fmt::{format::Format, Layer},
        prelude::*,
        registry::Registry,
    };
    // default is the Full format, there is no way to specify this, but it can be
    // overridden via builder methods
    let console_format = Format::default()
        .with_ansi(true)
        .with_target(true)
        .with_level(true)
        .json();

    let console_layer = Layer::default()
        .event_format(console_format)
        .with_writer(stdout);

    let console_level = Level::DEBUG;

    let console_filter = Targets::new().with_target(env!("CARGO_PKG_NAME"), console_level);

    let console_layer = if true {
        console_layer.boxed()
    } else {
        console_layer.with_filter(console_filter).boxed()
    };

    let registry = Registry::default()
        // just an example, you can actuall pass Options here for layers that might be
        // set/unset at runtime
        .with(Some(console_layer))
        .with(None::<Layer<_>>);

    tracing::subscriber::set_global_default(registry).unwrap();
}

#[tokio::main]
async fn main() -> MainResult {
    init_tracing();
    let args = Args::parse();
    match args.command {
        Command::Serve(serve_args) => {
            if let Err(e) = sqlite::migrate(&args.database_url).await {
                return <_ as Into<Error>>::into(e).into();
            }

            let database_pool = match sqlite::init_database_pool(&args.database_url).await {
                Ok(pool) => pool,
                Err(e) => return <_ as Into<Error>>::into(e).into(),
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

            // build our application with a route
            let app = routing::router(state);
            let addr = SocketAddr::from((
                IpAddr::from_str(&serve_args.bind)
                    .map_err(|error| {
                        format!("error parsing bind address {}: {}", &serve_args.bind, error)
                    })
                    .unwrap(),
                serve_args.port,
            ));
            tracing::debug!("listening on {}", addr);
            if let Err(e) = axum::Server::try_bind(&addr)
                .map_err(|error| format!("error binding to {}: {}", addr, error))
                .unwrap()
                .serve(app.into_make_service())
                .await
            {
                return <hyper::Error as Into<Error>>::into(e).into();
            }
        }
        Command::Admin(admin_command) => match admin_command {
            Admin::User(cmd) => match cmd {
                UserCommand::Create(user) => {
                    let database_pool = match sqlite::init_database_pool(&args.database_url).await {
                        Ok(pool) => pool,
                        Err(e) => return <_ as Into<Error>>::into(e).into(),
                    };

                    let id = match models::user::create(
                        &database_pool,
                        models::user::NewUser {
                            username: &user.username,
                            fullname: &user.fullname,
                        },
                    )
                    .await
                    .map_err(|error| match error {
                        models::Error::Query(models::QueryError::Duplicate { description: _ }) => {
                            Error::Command(packager::CommandError::UserExists {
                                username: user.username.clone(),
                            })
                        }
                        _ => Error::Model(error),
                    }) {
                        Ok(id) => id,
                        Err(e) => {
                            return e.into();
                        }
                    };

                    println!(
                        "User \"{}\" created successfully (id {})",
                        &user.username, id
                    )
                }
            },
        },
        Command::Migrate => {
            if let Err(e) = sqlite::migrate(&args.database_url).await {
                return <_ as Into<Error>>::into(e).into();
            }

            println!("Migrations successfully applied");
        }
    }

    MainResult(Ok(()))
}
