use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;
use rflow_core::models::{CreateWorkflowStep, UpdateWorkflowStep, WorkflowStep};
use rflow_core::AppError;

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkflowStep>>, ApiError> {
    let steps = sqlx::query_as::<_, WorkflowStep>(
        "SELECT * FROM workflow_steps ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(steps))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateWorkflowStep>,
) -> Result<(StatusCode, Json<WorkflowStep>), ApiError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name cannot be empty".into()).into());
    }

    let script_path = std::path::Path::new(&input.script_path);
    if !script_path.exists() {
        return Err(AppError::Validation(format!("script not found: {}", input.script_path)).into());
    }

    let step = sqlx::query_as::<_, WorkflowStep>(
        "INSERT INTO workflow_steps (name, description, script_path, input_schema, param_schema, output_dir_name) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
    )
    .bind(input.name.trim())
    .bind(&input.description)
    .bind(&input.script_path)
    .bind(&input.input_schema)
    .bind(&input.param_schema)
    .bind(&input.output_dir_name)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(step)))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkflowStep>, ApiError> {
    let step = sqlx::query_as::<_, WorkflowStep>(
        "SELECT * FROM workflow_steps WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(step))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateWorkflowStep>,
) -> Result<Json<WorkflowStep>, ApiError> {
    let existing = sqlx::query_as::<_, WorkflowStep>(
        "SELECT * FROM workflow_steps WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let name = input.name.unwrap_or(existing.name);
    let description = input.description.unwrap_or(existing.description);
    let script_path = input.script_path.unwrap_or(existing.script_path);
    let input_schema = input.input_schema.unwrap_or(existing.input_schema);
    let param_schema = input.param_schema.unwrap_or(existing.param_schema);
    let output_dir_name = input.output_dir_name.unwrap_or(existing.output_dir_name);

    let step = sqlx::query_as::<_, WorkflowStep>(
        "UPDATE workflow_steps SET name=$1, description=$2, script_path=$3, input_schema=$4, param_schema=$5, output_dir_name=$6 \
         WHERE id=$7 RETURNING *",
    )
    .bind(name.trim())
    .bind(&description)
    .bind(&script_path)
    .bind(&input_schema)
    .bind(&param_schema)
    .bind(&output_dir_name)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(step))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM workflow_steps WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound.into());
    }
    Ok(StatusCode::NO_CONTENT)
}
