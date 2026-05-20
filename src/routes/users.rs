use axum::extract::State;
use axum::response::{Html, Redirect, IntoResponse, Response};
use axum::Form;
use axum::http::HeaderMap;
use axum_extra::extract::cookie::CookieJar;
use askama::Template;
use serde::Deserialize;
use crate::AppState;
use crate::models::User;
use super::auth::is_authenticated;

#[derive(Template)]
#[template(path = "users.html")]
struct UsersTemplate {
    active_nav: &'static str,
    users: Vec<User>,
}

#[derive(Template)]
#[template(path = "fragments/user_list.html")]
struct UserListFragment {
    users: Vec<User>,
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers.contains_key("hx-request")
}

fn is_nav_request(headers: &HeaderMap) -> bool {
    headers.get("hx-target").map(|v| v.as_bytes()) == Some(b"main-content")
}

pub async fn list(State(state): State<AppState>, jar: CookieJar, headers: HeaderMap) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&state.pool).await.unwrap_or_default();

    if is_htmx(&headers) && !is_nav_request(&headers) {
        Ok(Html(UserListFragment { users }.render().unwrap_or_default()).into_response())
    } else {
        Ok(Html(UsersTemplate { active_nav: "users", users }.render().unwrap_or_default()).into_response())
    }
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub role: String,
}

pub async fn create(State(state): State<AppState>, jar: CookieJar, headers: HeaderMap, Form(form): Form<CreateUser>) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let id = uuid::Uuid::new_v4().to_string();
    let hash = crate::hash_password(&form.password);
    sqlx::query("INSERT INTO users (id, username, password_hash, role) VALUES (?, ?, ?, ?)")
        .bind(&id).bind(&form.username).bind(&hash).bind(&form.role)
        .execute(&state.pool).await.ok();

    crate::audit(&state.pool, "admin", "create_user", &form.username, "").await;

    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&state.pool).await.unwrap_or_default();

    if is_htmx(&headers) {
        Ok(Html(UserListFragment { users }.render().unwrap_or_default()).into_response())
    } else {
        Ok(Redirect::to("/users").into_response())
    }
}

pub async fn delete(State(state): State<AppState>, jar: CookieJar, axum::extract::Path(id): axum::extract::Path<String>) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    sqlx::query("DELETE FROM users WHERE id = ?").bind(&id).execute(&state.pool).await.ok();
    crate::audit(&state.pool, "admin", "delete_user", &id, "").await;
    Ok(Html("").into_response())
}
