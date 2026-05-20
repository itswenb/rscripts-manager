use std::path::Path;
use tokio::process::Command;

pub mod parser;
pub use parser::{parse_script, ScriptMeta, PortDef, ParamDef};

#[derive(Debug)]
pub struct RunResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub async fn execute_script(
    rscript_path: &str,
    script_path: &str,
    run_dir: &Path,
) -> Result<RunResult, std::io::Error> {
    let output_dir = run_dir.join("outputs");
    tokio::fs::create_dir_all(&output_dir).await?;

    // Copy rflow.R helper if available
    let helper_src = Path::new("templates/rflow.R");
    if helper_src.exists() {
        let _ = tokio::fs::copy(helper_src, run_dir.join("rflow.R")).await;
    }

    let output = Command::new(rscript_path)
        .arg(script_path)
        .env("RFLOW_RUN_DIR", run_dir)
        .current_dir(run_dir)
        .output()
        .await?;

    Ok(RunResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}
