use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ScriptRun {
    pub id: Uuid,
    pub project_id: Uuid,
    pub workflow_step_id: Uuid,
    pub status: String,
    pub inputs: serde_json::Value,
    pub params: serde_json::Value,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScriptRun {
    pub workflow_step_id: Uuid,
    #[serde(default)]
    pub inputs: serde_json::Value,
    #[serde(default)]
    pub params: serde_json::Value,
}
