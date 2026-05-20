use serde::Serialize;

#[derive(Serialize, Default, Clone)]
pub struct RuntimeDetection {
    pub host_rscript: bool,
    pub sinfo: bool,
    pub singularity: bool,
    pub module: bool,
}

pub async fn detect() -> RuntimeDetection {
    let (host_rscript, sinfo, singularity, module) = tokio::join!(
        cmd_ok("which", &["Rscript"]),
        cmd_ok("sinfo", &["--version"]),
        cmd_ok("singularity", &["--version"]),
        bash_ok("type module"),
    );

    RuntimeDetection {
        host_rscript,
        sinfo,
        singularity,
        module,
    }
}

async fn cmd_ok(program: &str, args: &[&str]) -> bool {
    tokio::process::Command::new(program)
        .args(args)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

async fn bash_ok(cmd: &str) -> bool {
    tokio::process::Command::new("bash")
        .args(["-lc", cmd])
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub async fn list_sif(dir: &str) -> Vec<String> {
    let dir = expand_tilde(dir);
    let mut out = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return out;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("sif") {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                out.push(name.to_string());
            }
        }
    }
    out.sort();
    out
}

pub async fn list_r_modules() -> Vec<String> {
    let output = tokio::process::Command::new("bash")
        .args(["-lc", "module avail R 2>&1"])
        .output()
        .await;
    let Ok(output) = output else {
        return Vec::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let text = format!("{}\n{}", stdout, stderr);
    let mut out = Vec::new();
    for line in text.lines() {
        for tok in line.split_whitespace() {
            if tok.starts_with("R/") || tok == "R" {
                out.push(tok.trim_end_matches('(').trim_end_matches(')').to_string());
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn expand_tilde(path: &str) -> String {
    if path == "~" {
        return std::env::var("HOME").unwrap_or_else(|_| path.to_string());
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return std::path::PathBuf::from(home)
                .join(rest)
                .to_string_lossy()
                .to_string();
        }
    }
    path.to_string()
}
