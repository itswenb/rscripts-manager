use super::auth::is_authenticated;
use askama::Template;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Form, Json};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(sqlx::FromRow)]
struct AdminRow {
    username: String,
    password_hash: String,
    avatar_base64: String,
}

#[derive(Clone)]
struct ProfileView {
    username: String,
    avatar_base64: String,
    avatar_initial: String,
}

#[derive(Template)]
#[template(path = "users.html")]
struct UsersTemplate {
    active_nav: &'static str,
    profile: ProfileView,
}

#[derive(Serialize)]
pub struct ProfileSummary {
    username: String,
    avatar_base64: String,
    avatar_initial: String,
}

#[derive(Serialize)]
pub struct UpdateProfileResponse {
    ok: bool,
    username: String,
    message: String,
}

#[derive(Deserialize)]
pub struct UpdateProfileForm {
    username: String,
    #[serde(default)]
    current_password: String,
    #[serde(default)]
    new_password: String,
    #[serde(default)]
    confirm_password: String,
}

#[derive(Deserialize)]
pub struct UpdateAvatarRequest {
    avatar_base64: String,
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers.contains_key("hx-request")
}

fn is_nav_request(headers: &HeaderMap) -> bool {
    headers.get("hx-target").map(|v| v.as_bytes()) == Some(b"main-content")
}

pub async fn list(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }

    let Some(profile) = load_profile_view(&state.pool).await else {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
    };

    let tmpl = UsersTemplate {
        active_nav: "users",
        profile,
    };

    let rendered = tmpl.render().unwrap_or_default();
    if is_htmx(&headers) && !is_nav_request(&headers) {
        Ok(Html(rendered).into_response())
    } else {
        Ok(Html(rendered).into_response())
    }
}

pub async fn summary(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<ProfileSummary>, StatusCode> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    load_profile_summary(&state.pool)
        .await
        .map(Json)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn update_profile(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<UpdateProfileForm>,
) -> Result<Json<UpdateProfileResponse>, (StatusCode, Json<serde_json::Value>)> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(error_json(
            StatusCode::UNAUTHORIZED,
            "未登录，无法修改账户信息",
        ));
    }

    let response = apply_profile_update(&state.pool, form).await?;
    Ok(Json(response))
}

pub async fn update_avatar(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<UpdateAvatarRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(error_json(StatusCode::UNAUTHORIZED, "未登录，无法修改头像"));
    }

    let Some(admin) = load_admin_row(&state.pool).await else {
        return Err(error_json(
            StatusCode::INTERNAL_SERVER_ERROR,
            "未找到当前管理员账户",
        ));
    };

    let avatar_base64 = body.avatar_base64.trim();
    if avatar_base64.is_empty() {
        return Err(error_json(StatusCode::BAD_REQUEST, "头像数据不能为空"));
    }

    if avatar_base64.len() > 350 * 1024 {
        return Err(error_json(
            StatusCode::BAD_REQUEST,
            "头像过大，请选择更小的图片",
        ));
    }

    if !avatar_base64
        .bytes()
        .all(|byte| matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/' | b'='))
    {
        return Err(error_json(StatusCode::BAD_REQUEST, "头像数据格式不正确"));
    }

    sqlx::query("UPDATE admin SET avatar_base64 = ? WHERE id = 1")
        .bind(avatar_base64)
        .execute(&state.pool)
        .await
        .map_err(|_| error_json(StatusCode::INTERNAL_SERVER_ERROR, "保存头像失败"))?;

    crate::audit(
        &state.pool,
        &admin.username,
        "update_avatar",
        &admin.username,
        "",
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn load_admin_row(pool: &sqlx::SqlitePool) -> Option<AdminRow> {
    sqlx::query_as::<_, AdminRow>(
        "SELECT username, password_hash, avatar_base64 FROM admin WHERE id = 1",
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None)
}

async fn load_profile_view(pool: &sqlx::SqlitePool) -> Option<ProfileView> {
    let admin = load_admin_row(pool).await?;
    Some(profile_view_from_row(admin))
}

async fn load_profile_summary(pool: &sqlx::SqlitePool) -> Option<ProfileSummary> {
    let admin = load_admin_row(pool).await?;
    let profile = profile_view_from_row(admin);
    Some(ProfileSummary {
        username: profile.username,
        avatar_base64: profile.avatar_base64,
        avatar_initial: profile.avatar_initial,
    })
}

fn profile_view_from_row(admin: AdminRow) -> ProfileView {
    let avatar_initial = admin
        .username
        .chars()
        .next()
        .map(|ch| ch.to_uppercase().collect::<String>())
        .unwrap_or_else(|| "U".into());

    ProfileView {
        username: admin.username,
        avatar_base64: admin.avatar_base64,
        avatar_initial,
    }
}

fn error_json(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({ "ok": false, "message": message })),
    )
}

async fn apply_profile_update(
    pool: &sqlx::SqlitePool,
    form: UpdateProfileForm,
) -> Result<UpdateProfileResponse, (StatusCode, Json<serde_json::Value>)> {
    let Some(admin) = load_admin_row(pool).await else {
        return Err(error_json(
            StatusCode::INTERNAL_SERVER_ERROR,
            "未找到当前管理员账户",
        ));
    };

    let username = form.username.trim();
    let current_password = form.current_password;
    let new_password = form.new_password;
    let confirm_password = form.confirm_password;

    if username.is_empty() {
        return Err(error_json(StatusCode::BAD_REQUEST, "用户名不能为空"));
    }

    let username_changed = username != admin.username;
    let password_changed = !new_password.is_empty() || !confirm_password.is_empty();

    if !username_changed && !password_changed {
        return Ok(UpdateProfileResponse {
            ok: true,
            username: admin.username,
            message: "没有需要保存的修改".into(),
        });
    }

    if current_password.is_empty() {
        return Err(error_json(StatusCode::BAD_REQUEST, "请输入当前密码"));
    }

    if !crate::verify_password(&current_password, &admin.password_hash) {
        return Err(error_json(StatusCode::BAD_REQUEST, "当前密码不正确"));
    }

    let password_hash = if password_changed {
        if new_password.is_empty() {
            return Err(error_json(StatusCode::BAD_REQUEST, "新密码不能为空"));
        }
        if new_password != confirm_password {
            return Err(error_json(
                StatusCode::BAD_REQUEST,
                "两次输入的新密码不一致",
            ));
        }
        crate::hash_password(&new_password)
    } else {
        admin.password_hash
    };

    sqlx::query("UPDATE admin SET username = ?, password_hash = ? WHERE id = 1")
        .bind(username)
        .bind(&password_hash)
        .execute(pool)
        .await
        .map_err(|_| error_json(StatusCode::INTERNAL_SERVER_ERROR, "保存账户信息失败"))?;

    crate::audit(
        pool,
        username,
        "update_profile",
        username,
        if password_changed {
            "username,password"
        } else {
            "username"
        },
    )
    .await;

    Ok(UpdateProfileResponse {
        ok: true,
        username: username.to_string(),
        message: "账户信息已保存".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::{apply_profile_update, UpdateProfileForm};
    use crate::verify_password;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_url(label: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "ripeline-users-{label}-{}-{nanos}.db",
            std::process::id()
        ));
        format!("sqlite:{}?mode=rwc", path.display())
    }

    async fn test_pool(label: &str) -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&temp_db_url(label))
            .await
            .expect("connect sqlite db");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("run migrations");
        pool
    }

    #[tokio::test]
    async fn apply_profile_update_replaces_admin_password_hash() {
        let pool = test_pool("profile-update").await;
        let old_hash = crate::hash_password("old-password");

        sqlx::query("INSERT INTO admin (id, username, password_hash) VALUES (1, ?, ?)")
            .bind("admin")
            .bind(&old_hash)
            .execute(&pool)
            .await
            .expect("seed admin");

        let response = apply_profile_update(
            &pool,
            UpdateProfileForm {
                username: "admin".into(),
                current_password: "old-password".into(),
                new_password: "new-password".into(),
                confirm_password: "new-password".into(),
            },
        )
        .await
        .expect("update profile");

        assert!(response.ok);
        assert_eq!(response.username, "admin");

        let stored_hash =
            sqlx::query_scalar::<_, String>("SELECT password_hash FROM admin WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("load updated hash");
        assert!(verify_password("new-password", &stored_hash));
        assert!(!verify_password("old-password", &stored_hash));

        pool.close().await;
    }
}
