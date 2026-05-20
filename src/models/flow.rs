use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub mode: RuntimeMode,
    pub local: LocalRuntimeConfig,
    pub cluster: ClusterRuntimeConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    #[default]
    Host,
    ClusterSingularity,
    ClusterModule,
    ClusterBundled,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalRuntimeConfig {
    pub data_dir: String,
    pub scripts_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRuntimeConfig {
    pub mode: ClusterRuntimeMode,
    pub module_name: String,
    pub sif_dir: String,
    pub sif_path: String,
    pub singularity_args: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ClusterRuntimeMode {
    #[default]
    Bundled,
    Module,
    Singularity,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            mode: RuntimeMode::Host,
            local: LocalRuntimeConfig {
                data_dir: String::new(),
                scripts_dir: String::new(),
            },
            cluster: ClusterRuntimeConfig {
                mode: ClusterRuntimeMode::Bundled,
                module_name: String::new(),
                sif_dir: String::new(),
                sif_path: String::new(),
                singularity_args: String::new(),
            },
        }
    }
}
