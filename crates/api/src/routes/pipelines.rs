use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use rflow_core::models::{
    CreatePipeline, Pipeline, PipelineRun, PipelineStep, PipelineStepRun, StepOutputFile,
};
use rflow_core::AppError;

#[derive(Serialize)]
pub struct PipelineWithSteps {
    #[serde(flatten)]
    pub pipeline: Pipeline,
    pub steps: Vec<PipelineStep>,
}

pub async fn list(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<Json<Vec<Pipeline>>, ApiError> {
    let pipelines = sqlx::query_as::<_, Pipeline>(
        "SELECT * FROM pipelines WHERE project_id = $1 ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(pipelines))
}

pub async fn create(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    _auth: AuthUser,
    Json(input): Json<CreatePipeline>,
) -> Result<(StatusCode, Json<PipelineWithSteps>), ApiError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name cannot be empty".into()).into());
    }
    if input.steps.is_empty() {
        return Err(AppError::Validation("pipeline must have at least one step".into()).into());
    }

    let pipeline = sqlx::query_as::<_, Pipeline>(
        "INSERT INTO pipelines (project_id, name, description) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(project_id)
    .bind(input.name.trim())
    .bind(input.description.as_deref().unwrap_or(""))
    .fetch_one(&state.pool)
    .await?;

    let mut steps = Vec::new();
    for (i, s) in input.steps.into_iter().enumerate() {
        let step = sqlx::query_as::<_, PipelineStep>(
            "INSERT INTO pipeline_steps (pipeline_id, step_order, script_path, label, param_values) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(pipeline.id)
        .bind(i as i32)
        .bind(&s.script_path)
        .bind(s.label.as_deref().unwrap_or(""))
        .bind(s.param_values.as_ref().unwrap_or(&serde_json::Value::Object(Default::default())))
        .fetch_one(&state.pool)
        .await?;
        steps.push(step);
    }

    Ok((StatusCode::CREATED, Json(PipelineWithSteps { pipeline, steps })))
}

pub async fn get(
    State(state): State<AppState>,
    Path((_project_id, pipeline_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<Json<PipelineWithSteps>, ApiError> {
    let pipeline = sqlx::query_as::<_, Pipeline>(
        "SELECT * FROM pipelines WHERE id = $1",
    )
    .bind(pipeline_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let steps = sqlx::query_as::<_, PipelineStep>(
        "SELECT * FROM pipeline_steps WHERE pipeline_id = $1 ORDER BY step_order",
    )
    .bind(pipeline_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PipelineWithSteps { pipeline, steps }))
}

pub async fn delete(
    State(state): State<AppState>,
    Path((_project_id, pipeline_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM pipelines WHERE id = $1")
        .bind(pipeline_id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound.into());
    }
    Ok(StatusCode::NO_CONTENT)
}

// --- Pipeline Runs ---

#[derive(Deserialize)]
pub struct StartRunInput {
    #[serde(default)]
    pub input_files: Vec<Uuid>,
    #[serde(default)]
    pub param_overrides: serde_json::Value,
}

pub async fn start_run(
    State(state): State<AppState>,
    Path((_project_id, pipeline_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
    body: Option<Json<StartRunInput>>,
) -> Result<(StatusCode, Json<PipelineRun>), ApiError> {
    let input = body.map(|b| b.0).unwrap_or(StartRunInput {
        input_files: vec![],
        param_overrides: serde_json::Value::Object(Default::default()),
    });

    let pipeline = sqlx::query_as::<_, Pipeline>(
        "SELECT * FROM pipelines WHERE id = $1",
    )
    .bind(pipeline_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let steps = sqlx::query_as::<_, PipelineStep>(
        "SELECT * FROM pipeline_steps WHERE pipeline_id = $1 ORDER BY step_order",
    )
    .bind(pipeline_id)
    .fetch_all(&state.pool)
    .await?;

    if steps.is_empty() {
        return Err(AppError::Validation("pipeline has no steps".into()).into());
    }

    // Build input_files JSON: map file asset IDs to their storage paths
    let mut input_files_map = serde_json::Map::new();
    for file_id in &input.input_files {
        let file = sqlx::query_as::<_, rflow_core::models::FileAsset>(
            "SELECT * FROM file_assets WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(file_id)
        .fetch_optional(&state.pool)
        .await?;
        if let Some(f) = file {
            input_files_map.insert(f.name.clone(), serde_json::Value::String(f.storage_path));
        }
    }

    let run = sqlx::query_as::<_, PipelineRun>(
        "INSERT INTO pipeline_runs (pipeline_id, project_id, input_files, param_overrides) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(pipeline_id)
    .bind(pipeline.project_id)
    .bind(serde_json::Value::Object(input_files_map))
    .bind(&input.param_overrides)
    .fetch_one(&state.pool)
    .await?;

    for step in &steps {
        sqlx::query(
            "INSERT INTO pipeline_step_runs (pipeline_run_id, step_order, script_path) VALUES ($1, $2, $3)",
        )
        .bind(run.id)
        .bind(step.step_order)
        .bind(&step.script_path)
        .execute(&state.pool)
        .await?;
    }

    Ok((StatusCode::CREATED, Json(run)))
}

pub async fn list_runs(
    State(state): State<AppState>,
    Path((_project_id, pipeline_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<Json<Vec<PipelineRun>>, ApiError> {
    let runs = sqlx::query_as::<_, PipelineRun>(
        "SELECT * FROM pipeline_runs WHERE pipeline_id = $1 ORDER BY created_at DESC",
    )
    .bind(pipeline_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(runs))
}

#[derive(Serialize)]
pub struct PipelineRunDetail {
    #[serde(flatten)]
    pub run: PipelineRun,
    pub step_runs: Vec<PipelineStepRun>,
}

pub async fn get_run(
    State(state): State<AppState>,
    Path((_project_id, _pipeline_id, run_id)): Path<(Uuid, Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<Json<PipelineRunDetail>, ApiError> {
    let run = sqlx::query_as::<_, PipelineRun>(
        "SELECT * FROM pipeline_runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let step_runs = sqlx::query_as::<_, PipelineStepRun>(
        "SELECT * FROM pipeline_step_runs WHERE pipeline_run_id = $1 ORDER BY step_order",
    )
    .bind(run_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PipelineRunDetail { run, step_runs }))
}

pub async fn list_step_outputs(
    State(state): State<AppState>,
    Path((_project_id, _pipeline_id, _run_id, step_run_id)): Path<(Uuid, Uuid, Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<Json<Vec<StepOutputFile>>, ApiError> {
    let outputs = sqlx::query_as::<_, StepOutputFile>(
        "SELECT * FROM step_output_files WHERE step_run_id = $1 ORDER BY name",
    )
    .bind(step_run_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(outputs))
}

pub async fn download_step_output(
    State(state): State<AppState>,
    Path(output_id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<axum::response::Response, ApiError> {
    let output = sqlx::query_as::<_, StepOutputFile>(
        "SELECT * FROM step_output_files WHERE id = $1",
    )
    .bind(output_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let file = tokio::fs::File::open(&output.storage_path).await
        .map_err(|_| AppError::NotFound)?;
    let stream = tokio_util::io::ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    let mime = mime_guess::from_path(&output.name)
        .first_or_octet_stream()
        .to_string();

    Ok(axum::response::Response::builder()
        .header("content-type", &mime)
        .header("content-disposition", format!("inline; filename=\"{}\"", output.name))
        .body(body)
        .unwrap())
}
