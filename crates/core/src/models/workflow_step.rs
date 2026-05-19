use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub script_path: String,
    pub input_schema: serde_json::Value,
    pub param_schema: serde_json::Value,
    pub output_dir_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowStep {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub script_path: String,
    #[serde(default = "default_schema")]
    pub input_schema: serde_json::Value,
    #[serde(default = "default_schema")]
    pub param_schema: serde_json::Value,
    #[serde(default = "default_output_dir")]
    pub output_dir_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowStep {
    pub name: Option<String>,
    pub description: Option<String>,
    pub script_path: Option<String>,
    pub input_schema: Option<serde_json::Value>,
    pub param_schema: Option<serde_json::Value>,
    pub output_dir_name: Option<String>,
}

fn default_schema() -> serde_json::Value {
    serde_json::json!([])
}

fn default_output_dir() -> String {
    "outputs".into()
}
