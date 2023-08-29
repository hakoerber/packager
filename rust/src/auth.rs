use axum::{extract::State, middleware::Next, response::IntoResponse};
use tracing::Instrument;

use hyper::Request;

use super::models;
use super::{AppState, Error, RequestError};

#[derive(Clone, Debug)]
pub enum Config {
    Enabled,
    Disabled { assume_user: String },
}

// #[tracing::instrument(name = "check_auth", skip(state, request, next))]
pub async fn authorize<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, Error> {
    let current_user = async {
        let user = match state.auth_config {
            Config::Disabled { assume_user } => {
                let user =
                    match models::user::User::find_by_name(&state.database_pool, &assume_user)
                        .await?
                    {
                        Some(user) => user,
                        None => {
                            return Err(Error::Request(RequestError::AuthenticationUserNotFound {
                                username: assume_user,
                            }))
                        }
                    };
                tracing::info!(?user, "auth disabled, requested user exists");
                user
            }
            Config::Enabled => {
                let Some(username) = request.headers().get("x-auth-username") else {
                return Err(Error::Request(RequestError::AuthenticationHeaderMissing));
            };
                let username = username
                    .to_str()
                    .map_err(|error| {
                        Error::Request(RequestError::AuthenticationHeaderInvalid {
                            message: error.to_string(),
                        })
                    })?
                    .to_string();

                let user = match models::user::User::find_by_name(&state.database_pool, &username)
                    .await?
                {
                    Some(user) => user,
                    None => {
                        tracing::warn!(username, "auth rejected, user not found");
                        return Err(Error::Request(RequestError::AuthenticationUserNotFound {
                            username,
                        }));
                    }
                };
                tracing::info!(?user, "auth successful");
                user
            }
        };
        Ok(user)
    }
    // .instrument(tracing::debug_span!("authorize"))
    .await?;

    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}
