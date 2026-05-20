use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use rand_core::OsRng;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::audit;
use crate::state::AppState;
use rflow_core::models::{CreateUser, PaginationParams, UpdateUser, User};
use rflow_core::AppError;

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<User>>, ApiError> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);
    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, role, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(users))
}

pub async fn create(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(input): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    if input.username.trim().is_empty() {
        return Err(AppError::Validation("username cannot be empty".into()).into());
    }
    if input.password.len() < 4 {
        return Err(AppError::Validation("password too short".into()).into());
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(input.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_string();

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password_hash, role) VALUES ($1, $2, $3) RETURNING id, username, password_hash, role, created_at, updated_at",
    )
    .bind(input.username.trim())
    .bind(&hash)
    .bind(&input.role)
    .fetch_one(&state.pool)
    .await?;

    audit::log(&state.pool, &auth_user, "create", "user", Some(&user.id.to_string()), None).await;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, ApiError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, role, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(user))
}

pub async fn update(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateUser>,
) -> Result<Json<User>, ApiError> {
    let existing = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, role, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let hash = if let Some(ref pw) = input.password {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(pw.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_string()
    } else {
        existing.password_hash
    };

    let role = input.role.unwrap_or(existing.role);

    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET password_hash = $1, role = $2 WHERE id = $3 RETURNING id, username, password_hash, role, created_at, updated_at",
    )
    .bind(&hash)
    .bind(&role)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    audit::log(&state.pool, &auth_user, "update", "user", Some(&id.to_string()), None).await;
    Ok(Json(user))
}

pub async fn delete(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound.into());
    }
    audit::log(&state.pool, &auth_user, "delete", "user", Some(&id.to_string()), None).await;
    Ok(StatusCode::NO_CONTENT)
}
