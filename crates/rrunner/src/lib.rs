use std::path::Path;
use tokio::process::Command;

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
    let inputs_path = run_dir.join("inputs.json");
    let params_path = run_dir.join("params.json");
    let output_dir = run_dir.join("outputs");

    tokio::fs::create_dir_all(&output_dir).await?;

    let output = Command::new(rscript_path)
        .arg(script_path)
        .arg("--inputs")
        .arg(&inputs_path)
        .arg("--params")
        .arg(&params_path)
        .arg("--output")
        .arg(&output_dir)
        .current_dir(run_dir)
        .output()
        .await?;

    Ok(RunResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}
