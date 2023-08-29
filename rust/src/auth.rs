use axum::{extract::State, middleware::Next, response::IntoResponse};

use hyper::Request;

use super::models;
use super::{AppState, Error, RequestError};

#[derive(Clone, Debug)]
pub enum Config {
    Enabled,
    Disabled { assume_user: String },
}

#[tracing::instrument(skip(state, request, next))]
pub async fn authorize<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, Error> {
    let current_user = match state.auth_config {
        Config::Disabled { assume_user } => {
            match models::user::User::find_by_name(&state.database_pool, &assume_user).await? {
                Some(user) => user,
                None => {
                    return Err(Error::Request(RequestError::AuthenticationUserNotFound {
                        username: assume_user,
                    }))
                }
            }
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

            match models::user::User::find_by_name(&state.database_pool, &username).await? {
                Some(user) => user,
                None => {
                    return Err(Error::Request(RequestError::AuthenticationUserNotFound {
                        username,
                    }))
                }
            }
        }
    };

    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}
