use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use casbin::CoreApi;

use crate::auth::AuthUser;
use crate::state::AppState;

pub async fn casbin_auth(
    State(state): State<AppState>,
    auth_user: AuthUser,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    let enforcer = state.enforcer.read().await;
    let authorized = enforcer
        .enforce(vec![auth_user.username, path, method])
        .unwrap_or(false);

    if !authorized {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
