use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use rflow_core::models::{CreateScriptRun, OutputFile, ScriptRun};
use rflow_core::AppError;

pub async fn create(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    _auth: AuthUser,
    Json(input): Json<CreateScriptRun>,
) -> Result<(StatusCode, Json<ScriptRun>), ApiError> {
    let _step = sqlx::query("SELECT id FROM workflow_steps WHERE id = $1")
        .bind(input.workflow_step_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::Validation("workflow step not found".into()))?;

    let run = sqlx::query_as::<_, ScriptRun>(
        "INSERT INTO script_runs (project_id, workflow_step_id, inputs, params) \
         VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(project_id)
    .bind(input.workflow_step_id)
    .bind(&input.inputs)
    .bind(&input.params)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(run)))
}

pub async fn list(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<Json<Vec<ScriptRun>>, ApiError> {
    let runs = sqlx::query_as::<_, ScriptRun>(
        "SELECT * FROM script_runs WHERE project_id = $1 ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(runs))
}

pub async fn get(
    State(state): State<AppState>,
    Path((_project_id, run_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<Json<ScriptRun>, ApiError> {
    let run = sqlx::query_as::<_, ScriptRun>("SELECT * FROM script_runs WHERE id = $1")
        .bind(run_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(run))
}

pub async fn list_outputs(
    State(state): State<AppState>,
    Path((_project_id, run_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<Json<Vec<OutputFile>>, ApiError> {
    let outputs = sqlx::query_as::<_, OutputFile>(
        "SELECT * FROM output_files WHERE run_id = $1 ORDER BY name",
    )
    .bind(run_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(outputs))
}
