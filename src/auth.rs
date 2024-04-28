use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use futures::FutureExt;
use tracing::Instrument;

use crate::models::user::User;

use super::models;
use super::{AppState, AuthError, Error};

#[derive(Clone, Debug)]
pub enum Config {
    Enabled,
    Disabled { assume_user: String },
}

#[tracing::instrument(name = "check_auth", skip(state, request, next))]
pub async fn authorize(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, Error> {
    // We must not access `request` inside the async block above, otherwise there will be
    // errors like the following:
    //
    // the trait `tower::Service<http::Request<axum::body::Body>>` is not implemented for
    // `FromFn<fn(State<AppState>, Request<Body>, Next) -> impl Future<Output =
    // Result<impl IntoResponse, Error>> {authorize}, AppState, Route, _>
    //
    // I am honestly not sure about the reason
    let username_header = request.headers().get("x-auth-username");

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
            Config::Enabled => match username_header {
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
                        Err(ref e) => {
                            format!("failure_{}", e.to_prom_metric_name())
                        }
                    }
                }),
                &match &auth {
                    Ok(user) => vec![("username", user.username.clone())],
                    Err(e) => e.to_prom_labels(),
                }
            )
            .increment(1);
            auth
        })
    })
    // outer result: failure of the process, e.g. database connection failed
    // inner result: auth rejected, with AuthError
    .await??;

    request.extensions_mut().insert(user);
    Ok::<http::Response<axum::body::Body>, Error>(next.run(request).await)
}
