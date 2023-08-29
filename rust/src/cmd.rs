use clap::{Parser, Subcommand, ValueEnum};

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum BoolArg {
    True,
    False,
}

impl From<BoolArg> for bool {
    fn from(arg: BoolArg) -> bool {
        match arg {
            BoolArg::True => true,
            BoolArg::False => false,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub database_url: String,

    #[arg(long, value_enum, default_value_t = BoolArg::False)]
    pub enable_opentelemetry: BoolArg,

    #[arg(long, value_enum, default_value_t = BoolArg::False)]
    pub enable_tokio_console: BoolArg,

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
