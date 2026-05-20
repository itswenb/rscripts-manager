use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct ProjectFlow {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub graph_data: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct ProjectFlowStep {
    pub id: String,
    pub flow_id: String,
    pub node_id: String,
    pub step_order: i32,
    pub param_values: String,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct FlowRun {
    pub id: String,
    pub flow_id: String,
    pub status: String,
    pub current_step: i32,
    pub created_at: NaiveDateTime,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct StepRun {
    pub id: String,
    pub flow_run_id: String,
    pub step_order: i32,
    pub status: String,
    pub slurm_job_id: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
}
