use serde::Serialize;

#[derive(Serialize, Clone, Default)]
pub struct ClusterStatus {
    pub idle: u32,
    pub alloc: u32,
    pub down: u32,
    pub reachable: bool,
}

pub async fn cluster_status() -> ClusterStatus {
    let output = tokio::process::Command::new("sinfo")
        .args(["--noheader", "--format=%T"])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let mut s = ClusterStatus { reachable: true, ..Default::default() };
            for line in text.lines() {
                match line.trim() {
                    "idle" | "idle*" => s.idle += 1,
                    "mixed" | "allocated" | "alloc" => s.alloc += 1,
                    _ => s.down += 1,
                }
            }
            s
        }
        _ => ClusterStatus::default(),
    }
}

async fn has_slurm() -> bool {
    tokio::process::Command::new("sinfo")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub async fn submit_job(
    run_dir: &std::path::Path,
    script_path: &str,
    job_name: &str,
    extra_args: &[String],
) -> Result<String, std::io::Error> {
    if has_slurm().await {
        let args_str = extra_args.iter().map(|a| format!("'{}'", a.replace('\'', "'\\''"))).collect::<Vec<_>>().join(" ");
        let cmd = if args_str.is_empty() {
            format!("cd {} && Rscript {} {}", run_dir.display(), script_path, run_dir.display())
        } else {
            format!("cd {} && Rscript {} {} {}", run_dir.display(), script_path, run_dir.display(), args_str)
        };
        let output = tokio::process::Command::new("sbatch")
            .arg("--job-name")
            .arg(job_name)
            .arg("--output")
            .arg(run_dir.join("stdout.log"))
            .arg("--error")
            .arg(run_dir.join("stderr.log"))
            .arg("--wrap")
            .arg(&cmd)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let job_id = stdout.trim().rsplit(' ').next().unwrap_or("").to_string();
        Ok(job_id)
    } else {
        let stdout_file = std::fs::OpenOptions::new().create(true).append(true).open(run_dir.join("stdout.log"))?;
        let stderr_file = std::fs::File::create(run_dir.join("stderr.log"))?;
        let mut cmd = tokio::process::Command::new("Rscript");
        cmd.arg(script_path);
        cmd.arg(run_dir.as_os_str());
        for arg in extra_args {
            cmd.arg(arg);
        }
        let child = cmd
            .current_dir(run_dir)
            .stdout(std::process::Stdio::from(stdout_file))
            .stderr(std::process::Stdio::from(stderr_file))
            .spawn()?;
        let pid = child.id().unwrap_or(0);
        let exit_file = run_dir.join(".exit_code");
        let pid_file = run_dir.join(".pid");
        let _ = tokio::fs::write(&pid_file, pid.to_string()).await;
        tokio::spawn(async move {
            let result = child.wait_with_output().await;
            let code = result.map(|o| o.status.code().unwrap_or(1)).unwrap_or(1);
            let _ = tokio::fs::write(exit_file, code.to_string()).await;
        });
        Ok(format!("local:{}", run_dir.display()))
    }
}

pub async fn job_status(job_id: &str) -> Result<String, std::io::Error> {
    if let Some(run_dir) = job_id.strip_prefix("local:") {
        let exit_file = std::path::Path::new(run_dir).join(".exit_code");
        if exit_file.exists() {
            let code = tokio::fs::read_to_string(&exit_file).await.unwrap_or_default();
            if code.trim() == "0" {
                Ok("COMPLETED".to_string())
            } else {
                Ok("FAILED".to_string())
            }
        } else {
            Ok("RUNNING".to_string())
        }
    } else {
        let output = tokio::process::Command::new("sacct")
            .args(["-j", job_id, "--format=State", "--noheader", "--parsable2"])
            .output()
            .await?;
        let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(status)
    }
}

pub async fn cancel_job(job_id: &str) -> Result<(), std::io::Error> {
    if let Some(run_dir) = job_id.strip_prefix("local:") {
        let pid_file = std::path::Path::new(run_dir).join(".pid");
        if let Ok(pid_str) = tokio::fs::read_to_string(&pid_file).await {
            let pid: u32 = pid_str.trim().parse().unwrap_or(0);
            if pid > 0 {
                tokio::process::Command::new("kill").arg(pid.to_string()).output().await?;
            }
        }
    } else {
        tokio::process::Command::new("scancel")
            .arg(job_id)
            .output()
            .await?;
    }
    Ok(())
}
