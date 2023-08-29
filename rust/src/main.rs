use packager::{auth, routing, sqlite, AppState, ClientState, StartError};

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    database_url: String,
    #[arg(long, default_value_t = 3000)]
    port: u16,
    #[arg(long)]
    bind: String,
    #[arg(long, name = "USERNAME")]
    disable_auth_and_assume_user: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), StartError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();

    let database_pool = sqlite::init_database_pool(&args.database_url).await?;
    sqlite::migrate(&database_pool).await?;

    let state = AppState {
        database_pool,
        client_state: ClientState::new(),
        auth_config: if let Some(assume_user) = args.disable_auth_and_assume_user {
            auth::AuthConfig::Disabled { assume_user }
        } else {
            auth::AuthConfig::Enabled
        },
    };

    // build our application with a route
    let app = routing::router(state);
    let addr = SocketAddr::from((
        IpAddr::from_str(&args.bind)
            .map_err(|error| format!("error parsing bind address {}: {}", &args.bind, error))
            .unwrap(),
        args.port,
    ));
    tracing::debug!("listening on {}", addr);
    axum::Server::try_bind(&addr)
        .map_err(|error| format!("error binding to {}: {}", addr, error))
        .unwrap()
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
