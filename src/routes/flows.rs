use axum::extract::{State, Path};
use axum::response::{Html, Redirect};
use axum_extra::extract::cookie::CookieJar;
use crate::AppState;
use super::auth::is_authenticated;

pub async fn start_run(State(state): State<AppState>, jar: CookieJar, Path(flow_id): Path<String>) -> Result<Html<String>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    Ok(Html(format!("TODO: start run for flow {flow_id}")))
}

pub async fn pause(State(state): State<AppState>, jar: CookieJar, Path(flow_id): Path<String>) -> Result<Html<String>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    Ok(Html(format!("TODO: pause flow {flow_id}")))
}

pub async fn resume(State(state): State<AppState>, jar: CookieJar, Path(flow_id): Path<String>) -> Result<Html<String>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    Ok(Html(format!("TODO: resume flow {flow_id}")))
}

pub async fn reset_to_step(State(state): State<AppState>, jar: CookieJar, Path((flow_id, step)): Path<(String, i32)>) -> Result<Html<String>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    Ok(Html(format!("TODO: reset flow {flow_id} to step {step}")))
}

pub async fn status(State(state): State<AppState>, jar: CookieJar, Path(flow_id): Path<String>) -> Result<Html<String>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    Ok(Html(format!("TODO: status for flow {flow_id}")))
}
