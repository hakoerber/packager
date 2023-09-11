use axum::{extract::State, middleware::Next, response::IntoResponse};
use futures::FutureExt;
use tracing::Instrument;

use hyper::Request;

use crate::models::user::User;

use super::models;
use super::{AppState, AuthError, Error};

#[derive(Clone, Debug)]
pub enum Config {
    Enabled,
    Disabled { assume_user: String },
}

#[tracing::instrument(name = "check_auth", skip(state, request, next))]
pub async fn authorize<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, Error> {
    let user = async {
        let auth: Result<Result<User, AuthError>, Error> = match state.auth_config {
            Config::Disabled { assume_user } => {
                let user =
                    match models::user::User::find_by_name(&state.database_pool, &assume_user)
                        .await?
                    {
                        Some(user) => Ok(user),
                        None => Err(AuthError::AuthenticationUserNotFound {
                            username: assume_user,
                        }),
                    };
                Ok(user)
            }
            Config::Enabled => match request.headers().get("x-auth-username") {
                None => Ok(Err(AuthError::AuthenticationHeaderMissing)),
                Some(username) => match username.to_str() {
                    Err(e) => Ok(Err(AuthError::AuthenticationHeaderInvalid {
                        message: e.to_string(),
                    })),
                    Ok(username) => {
                        match models::user::User::find_by_name(&state.database_pool, username)
                            .await?
                        {
                            Some(user) => Ok(Ok(user)),
                            None => Ok(Err(AuthError::AuthenticationUserNotFound {
                                username: username.to_string(),
                            })),
                        }
                    }
                },
            },
        };

        auth
    }
    .instrument(tracing::debug_span!("authorize"))
    .inspect(|r| {
        if let Ok(auth) = r {
            match auth {
                Ok(user) => tracing::debug!(?user, "auth successful"),
                Err(e) => e.trace(),
            }
        }
    })
    .map(|r| {
        r.map(|auth| {
            metrics::counter!(
                format!("packager_auth_{}_total", {
                    match auth {
                        Ok(_) => "success".to_string(),
                        Err(ref e) => format!("failure_{}", e.to_prom_metric_name()),
                    }
                }),
                1,
                &match &auth {
                    Ok(user) => vec![("username", user.username.clone())],
                    Err(e) => e.to_prom_labels(),
                }
            );
            auth
        })
    })
    // outer result: failure of the process, e.g. database connection failed
    // inner result: auth rejected, with AuthError
    .await??;

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}
