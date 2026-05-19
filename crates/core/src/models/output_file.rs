use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct OutputFile {
    pub id: Uuid,
    pub run_id: Uuid,
    pub name: String,
    pub size_bytes: i64,
    pub mime_type: Option<String>,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}
