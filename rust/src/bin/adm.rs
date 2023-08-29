use std::fmt;
use std::process::exit;

use clap::{Parser, Subcommand};

use packager::{models, sqlite, StartError};

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

#[derive(Debug)]
enum Error {
    Generic { message: String },
    UserExists { username: String },
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Generic { message } => write!(f, "{}", message),
            Self::UserExists { username } => write!(f, "user \"{username}\" already exists"),
        }
    }
}

impl From<StartError> for Error {
    fn from(starterror: StartError) -> Self {
        Self::Generic {
            message: starterror.to_string(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let database_pool = sqlite::init_database_pool(&args.database_url).await?;

    match args.command {
        Command::User(cmd) => match cmd {
            UserCommand::Create(user) => {
                let id = match models::user::create(
                    &database_pool,
                    models::user::NewUser {
                        username: &user.username,
                        fullname: &user.fullname,
                    },
                )
                .await
                {
                    Ok(id) => id,
                    Err(error) => {
                        if let models::Error::Query(models::QueryError::Duplicate {
                            description: _,
                        }) = error
                        {
                            println!(
                                "Error: {}",
                                Error::UserExists {
                                    username: user.username,
                                }
                                .to_string()
                            );
                            exit(1);
                        }
                        return Err(error.into());
                    }
                };
                println!(
                    "User \"{}\" created successfully (id {})",
                    user.username, id
                )
            }
        },
    }

    Ok(())
}
