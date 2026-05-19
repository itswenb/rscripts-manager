use sqlx::PgPool;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://rflow:rflow@localhost:5432/rflow".into());
    let pool = rflow_core::db::create_pool(&database_url).await;
    let rscript_path =
        std::env::var("RSCRIPT_PATH").unwrap_or_else(|_| "/usr/bin/Rscript".into());
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".into());

    tracing::info!("Worker started, polling for pending runs...");

    loop {
        if let Err(e) = poll_and_execute(&pool, &rscript_path, &data_dir).await {
            tracing::error!("Worker error: {e}");
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

async fn poll_and_execute(
    pool: &PgPool,
    rscript_path: &str,
    data_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let row = sqlx::query_as::<_, (Uuid, Uuid, Uuid, serde_json::Value, serde_json::Value)>(
        "UPDATE script_runs SET status = 'running', started_at = now() \
         WHERE id = (SELECT id FROM script_runs WHERE status = 'pending' ORDER BY created_at LIMIT 1 FOR UPDATE SKIP LOCKED) \
         RETURNING id, project_id, workflow_step_id, inputs, params",
    )
    .fetch_optional(pool)
    .await?;

    let Some((run_id, project_id, step_id, inputs, params)) = row else {
        return Ok(());
    };

    let step = sqlx::query_as::<_, (String,)>(
        "SELECT script_path FROM workflow_steps WHERE id = $1",
    )
    .bind(step_id)
    .fetch_one(pool)
    .await?;

    let run_dir = PathBuf::from(format!("{data_dir}/projects/{project_id}/runs/{run_id}"));
    tokio::fs::create_dir_all(&run_dir).await?;

    tokio::fs::write(
        run_dir.join("inputs.json"),
        serde_json::to_string_pretty(&inputs)?,
    )
    .await?;
    tokio::fs::write(
        run_dir.join("params.json"),
        serde_json::to_string_pretty(&params)?,
    )
    .await?;

    let result = rflow_rrunner::execute_script(rscript_path, &step.0, &run_dir).await?;

    let status = if result.success { "completed" } else { "failed" };

    tokio::fs::write(run_dir.join("stdout.log"), &result.stdout).await?;
    tokio::fs::write(run_dir.join("stderr.log"), &result.stderr).await?;

    // Catalog output files
    let output_dir = run_dir.join("outputs");
    if output_dir.exists() {
        let mut entries = tokio::fs::read_dir(&output_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let meta = entry.metadata().await?;
            if meta.is_file() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let path = entry.path().to_string_lossy().into_owned();
                sqlx::query(
                    "INSERT INTO output_files (run_id, name, size_bytes, storage_path) VALUES ($1, $2, $3, $4)",
                )
                .bind(run_id)
                .bind(&name)
                .bind(meta.len() as i64)
                .bind(&path)
                .execute(pool)
                .await?;
            }
        }
    }

    sqlx::query(
        "UPDATE script_runs SET status = $1, stdout = $2, stderr = $3, finished_at = now() WHERE id = $4",
    )
    .bind(status)
    .bind(&result.stdout)
    .bind(&result.stderr)
    .bind(run_id)
    .execute(pool)
    .await?;

    tracing::info!("Run {run_id} finished with status: {status}");
    Ok(())
}
