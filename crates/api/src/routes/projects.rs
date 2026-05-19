use axum::extract::{Path, Query, State};
use axum::Json;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;
use rflow_core::models::{CreateProject, PaginationParams, Project, UpdateProject};
use rflow_core::AppError;

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<Project>>, ApiError> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);
    let projects = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, updated_at FROM projects ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(projects))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateProject>,
) -> Result<(axum::http::StatusCode, Json<Project>), ApiError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name cannot be empty".into()).into());
    }
    let project = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (name, description) VALUES ($1, $2) RETURNING id, name, description, created_at, updated_at",
    )
    .bind(input.name.trim())
    .bind(&input.description)
    .fetch_one(&state.pool)
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(project)))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Project>, ApiError> {
    let project = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(project))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateProject>,
) -> Result<Json<Project>, ApiError> {
    let existing = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let name = input.name.unwrap_or(existing.name);
    let description = input.description.unwrap_or(existing.description);

    let project = sqlx::query_as::<_, Project>(
        "UPDATE projects SET name = $1, description = $2 WHERE id = $3 RETURNING id, name, description, created_at, updated_at",
    )
    .bind(name.trim())
    .bind(&description)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(project))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound.into());
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}
