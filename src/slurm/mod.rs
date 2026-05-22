use crate::models::{ClusterRuntimeMode, RuntimeConfig, RuntimeMode};
use crate::runtime;

#[derive(serde::Serialize, Clone, Default)]
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
            let mut s = ClusterStatus {
                reachable: true,
                ..Default::default()
            };
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

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn build_script_command(
    run_dir: &std::path::Path,
    script_path: &str,
    extra_args: &[String],
    runtime: &RuntimeConfig,
    node_sif: Option<&str>,
) -> String {
    let mut parts = Vec::new();
    let run_dir_q = shell_quote(&run_dir.display().to_string());
    let script_q = shell_quote(script_path);
    let arg_qs = extra_args
        .iter()
        .map(|a| shell_quote(a))
        .collect::<Vec<_>>()
        .join(" ");

    match runtime.cluster.mode {
        ClusterRuntimeMode::Bundled => {
            parts.push(format!("Rscript {} {}", script_q, run_dir_q));
        }
        ClusterRuntimeMode::Module => {
            let module = runtime.cluster.module_name.trim();
            if !module.is_empty() {
                parts.push(format!("module load {}", shell_quote(module)));
            }
            parts.push(format!("Rscript {} {}", script_q, run_dir_q));
        }
        ClusterRuntimeMode::Singularity => {
            // Per-node SIF overrides global sif_path
            let image = node_sif
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(runtime.cluster.sif_path.trim());
            let singularity_args = runtime.cluster.singularity_args.trim();
            let mut singularity = String::from("singularity exec");
            if !singularity_args.is_empty() {
                singularity.push(' ');
                singularity.push_str(singularity_args);
            }
            if !image.is_empty() {
                singularity.push(' ');
                singularity.push_str(&shell_quote(image));
            }
            singularity.push_str(&format!(" Rscript {} {}", script_q, run_dir_q));
            parts.push(singularity);
        }
    }

    if !arg_qs.is_empty() {
        parts.push(arg_qs);
    }

    parts.join(" ")
}

pub async fn resolve_auto(runtime: &mut RuntimeConfig) -> Result<(), String> {
    if runtime.mode != RuntimeMode::Auto {
        return Ok(());
    }
    let det = runtime::detect().await;
    let has_sif = !runtime.cluster.sif_path.trim().is_empty();
    let has_module = !runtime.cluster.module_name.trim().is_empty();

    if det.sinfo && det.singularity && has_sif {
        runtime.mode = RuntimeMode::ClusterSingularity;
        runtime.cluster.mode = ClusterRuntimeMode::Singularity;
    } else if det.sinfo && det.module && has_module {
        runtime.mode = RuntimeMode::ClusterModule;
        runtime.cluster.mode = ClusterRuntimeMode::Module;
    } else if det.sinfo {
        runtime.mode = RuntimeMode::ClusterBundled;
        runtime.cluster.mode = ClusterRuntimeMode::Bundled;
    } else if det.host_rscript {
        runtime.mode = RuntimeMode::Host;
    } else {
        return Err("无可用运行环境（自动检测失败）".into());
    }
    Ok(())
}

pub async fn validate_runtime(runtime: &RuntimeConfig) -> Result<(), String> {
    let det = runtime::detect().await;
    match runtime.mode {
        RuntimeMode::Host => {
            if !det.host_rscript {
                return Err("当前宿主机未检测到 Rscript".into());
            }
        }
        RuntimeMode::ClusterBundled => {
            if !det.sinfo {
                return Err("未检测到 SLURM (sinfo)".into());
            }
        }
        RuntimeMode::ClusterModule => {
            if !det.sinfo {
                return Err("未检测到 SLURM (sinfo)".into());
            }
            if !det.module {
                return Err("未检测到 module 系统".into());
            }
            if runtime.cluster.module_name.trim().is_empty() {
                return Err("未配置 module 名称".into());
            }
        }
        RuntimeMode::ClusterSingularity => {
            if !det.sinfo {
                return Err("未检测到 SLURM (sinfo)".into());
            }
            if !det.singularity {
                return Err("未检测到 singularity".into());
            }
            // Global sif_path is optional — individual nodes can override via per-node SIF
        }
        RuntimeMode::Auto => {}
    }
    Ok(())
}

pub async fn submit_job(
    run_dir: &std::path::Path,
    script_path: &str,
    job_name: &str,
    extra_args: &[String],
    runtime: &RuntimeConfig,
    node_sif: Option<&str>,
) -> Result<String, std::io::Error> {
    let command = build_script_command(run_dir, script_path, extra_args, runtime, node_sif);
    let use_cluster = matches!(
        runtime.mode,
        RuntimeMode::ClusterSingularity | RuntimeMode::ClusterModule | RuntimeMode::ClusterBundled
    );

    if use_cluster && has_slurm().await {
        let output = tokio::process::Command::new("sbatch")
            .arg("--job-name")
            .arg(job_name)
            .arg("--output")
            .arg(run_dir.join("stdout.log"))
            .arg("--error")
            .arg(run_dir.join("stderr.log"))
            .arg("--wrap")
            .arg(format!("bash -lc {}", shell_quote(&command)))
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let job_id = stdout.trim().rsplit(' ').next().unwrap_or("").to_string();
        Ok(job_id)
    } else {
        let stdout_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(run_dir.join("stdout.log"))?;
        let stderr_file = std::fs::File::create(run_dir.join("stderr.log"))?;
        let child = tokio::process::Command::new("bash")
            .arg("-lc")
            .arg(&command)
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
            let code = tokio::fs::read_to_string(&exit_file)
                .await
                .unwrap_or_default();
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
                tokio::process::Command::new("kill")
                    .arg(pid.to_string())
                    .output()
                    .await?;
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
