use crate::Error;

#[cfg(feature = "prometheus")]
use crate::StartError;

use clap::{Parser, Subcommand, ValueEnum};

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

    #[cfg(feature = "otel")]
    #[arg(long, value_enum, default_value_t = BoolArg::False)]
    pub enable_opentelemetry: BoolArg,

    #[cfg(feature = "tokio-console")]
    #[arg(long, value_enum, default_value_t = BoolArg::False)]
    pub enable_tokio_console: BoolArg,

    #[cfg(feature = "prometheus")]
    #[arg(long, value_enum, default_value_t = BoolArg::False)]
    pub enable_prometheus: BoolArg,

    #[cfg(feature = "prometheus")]
    #[arg(long, value_enum, required_if_eq("enable_prometheus", BoolArg::True))]
    pub prometheus_port: Option<u16>,

    #[cfg(feature = "prometheus")]
    #[arg(long, value_enum, required_if_eq("enable_prometheus", BoolArg::True))]
    pub prometheus_bind: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Serve(Serve),
    #[command(subcommand)]
    Admin(Admin),
    Migrate,
}

#[derive(Parser, Debug)]
pub struct Serve {
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
    #[arg(long)]
    pub bind: String,
    #[arg(long, name = "USERNAME")]
    pub disable_auth_and_assume_user: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Admin {
    #[command(subcommand)]
    User(UserCommand),
}

#[derive(Subcommand, Debug)]
pub enum UserCommand {
    Create(UserCreate),
}

#[derive(Parser, Debug)]
pub struct UserCreate {
    #[arg(long)]
    pub username: String,
    #[arg(long)]
    pub fullname: String,
}

impl Args {
    pub fn get() -> Result<Self, Error> {
        let args = Self::parse();

        #[cfg(feature = "prometheus")]
        if !args.enable_prometheus.bool()
            && (args.prometheus_port.is_some() || args.prometheus_bind.is_some())
        {
            return Err(Error::Start(StartError::CallError {
                message: "do not set prometheus options when prometheus is not enabled".to_string(),
            }));
        }

        Ok(args)
    }
}
