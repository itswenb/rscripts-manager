use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct PipelineNode {
    pub id: String,
    pub name: String,
    pub script_path: String,
    pub params_schema: String,
    pub inputs_schema: String,
    pub outputs_schema: String,
    pub default_sif: String,
    pub created_at: NaiveDateTime,
}
