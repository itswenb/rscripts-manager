use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Pipeline {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipelineStep {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub step_order: i32,
    pub script_path: String,
    pub label: String,
    pub param_values: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipelineRun {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub project_id: Uuid,
    pub status: String,
    pub current_step: i32,
    pub input_files: serde_json::Value,
    pub param_overrides: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipelineStepRun {
    pub id: Uuid,
    pub pipeline_run_id: Uuid,
    pub step_order: i32,
    pub script_path: String,
    pub status: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StepOutputFile {
    pub id: Uuid,
    pub step_run_id: Uuid,
    pub name: String,
    pub size_bytes: i64,
    pub mime_type: Option<String>,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePipeline {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<CreatePipelineStep>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePipelineStep {
    pub script_path: String,
    pub label: Option<String>,
    pub param_values: Option<serde_json::Value>,
}
