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
        if let Err(e) = poll_pipeline_runs(&pool, &rscript_path, &data_dir).await {
            tracing::error!("Pipeline worker error: {e}");
        }
        if let Err(e) = poll_script_runs(&pool, &rscript_path, &data_dir).await {
            tracing::error!("Script run worker error: {e}");
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

async fn poll_pipeline_runs(
    pool: &PgPool,
    rscript_path: &str,
    data_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Find a pipeline run that is pending or running (has more steps to execute)
    let row = sqlx::query_as::<_, (Uuid, Uuid, Uuid)>(
        "UPDATE pipeline_runs SET status = 'running', started_at = COALESCE(started_at, now()) \
         WHERE id = (SELECT id FROM pipeline_runs WHERE status IN ('pending', 'running') \
         AND id IN (SELECT pipeline_run_id FROM pipeline_step_runs WHERE status = 'pending') \
         ORDER BY created_at LIMIT 1 FOR UPDATE SKIP LOCKED) \
         RETURNING id, pipeline_id, project_id",
    )
    .fetch_optional(pool)
    .await?;

    let Some((run_id, pipeline_id, project_id)) = row else {
        return Ok(());
    };

    // Get the next pending step
    let step_run = sqlx::query_as::<_, (Uuid, i32, String)>(
        "UPDATE pipeline_step_runs SET status = 'running', started_at = now() \
         WHERE id = (SELECT id FROM pipeline_step_runs WHERE pipeline_run_id = $1 AND status = 'pending' ORDER BY step_order LIMIT 1 FOR UPDATE SKIP LOCKED) \
         RETURNING id, step_order, script_path",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await?;

    let Some((step_run_id, step_order, script_path)) = step_run else {
        return Ok(());
    };

    // Get param_values for this step, merged with run-level param_overrides
    let step_params = sqlx::query_as::<_, (serde_json::Value,)>(
        "SELECT param_values FROM pipeline_steps WHERE pipeline_id = $1 AND step_order = $2",
    )
    .bind(pipeline_id)
    .bind(step_order)
    .fetch_optional(pool)
    .await?
    .map(|r| r.0)
    .unwrap_or(serde_json::Value::Object(Default::default()));

    let run_meta = sqlx::query_as::<_, (serde_json::Value, serde_json::Value)>(
        "SELECT input_files, param_overrides FROM pipeline_runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await?;

    // Merge params: step defaults + run overrides
    let params = if let (serde_json::Value::Object(mut base), serde_json::Value::Object(overrides)) =
        (step_params, run_meta.1)
    {
        for (k, v) in overrides {
            base.insert(k, v);
        }
        serde_json::Value::Object(base)
    } else {
        serde_json::Value::Object(Default::default())
    };

    let run_dir = PathBuf::from(format!(
        "{data_dir}/pipelines/{project_id}/{run_id}/step_{step_order}"
    ));
    tokio::fs::create_dir_all(&run_dir).await?;

    // Write params
    tokio::fs::write(
        run_dir.join("params.json"),
        serde_json::to_string_pretty(&params)?,
    )
    .await?;

    // For inputs: step 0 uses run-level input_files, later steps use previous step's outputs
    let inputs = if step_order > 0 {
        let prev_dir = PathBuf::from(format!(
            "{data_dir}/pipelines/{project_id}/{run_id}/step_{}/outputs",
            step_order - 1
        ));
        if prev_dir.exists() {
            let mut input_files = serde_json::Map::new();
            let mut entries = tokio::fs::read_dir(&prev_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.metadata().await?.is_file() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    let path = entry.path().to_string_lossy().into_owned();
                    input_files.insert(name, serde_json::Value::String(path));
                }
            }
            serde_json::Value::Object(input_files)
        } else {
            serde_json::Value::Object(Default::default())
        }
    } else {
        // First step: use input_files from the pipeline run
        run_meta.0
    };

    tokio::fs::write(
        run_dir.join("inputs.json"),
        serde_json::to_string_pretty(&inputs)?,
    )
    .await?;

    let result = rflow_rrunner::execute_script(rscript_path, &script_path, &run_dir).await?;
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
                let mime = mime_guess::from_path(&name).first_or_octet_stream().to_string();
                sqlx::query(
                    "INSERT INTO step_output_files (step_run_id, name, size_bytes, mime_type, storage_path) VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(step_run_id)
                .bind(&name)
                .bind(meta.len() as i64)
                .bind(&mime)
                .bind(&path)
                .execute(pool)
                .await?;
            }
        }
    }

    // Update step run status
    sqlx::query(
        "UPDATE pipeline_step_runs SET status = $1, stdout = $2, stderr = $3, finished_at = now() WHERE id = $4",
    )
    .bind(status)
    .bind(&result.stdout)
    .bind(&result.stderr)
    .bind(step_run_id)
    .execute(pool)
    .await?;

    // Update pipeline run current_step
    sqlx::query("UPDATE pipeline_runs SET current_step = $1 WHERE id = $2")
        .bind(step_order)
        .bind(run_id)
        .execute(pool)
        .await?;

    // If step failed, mark the whole pipeline as failed
    if !result.success {
        sqlx::query("UPDATE pipeline_runs SET status = 'failed', finished_at = now() WHERE id = $1")
            .bind(run_id)
            .execute(pool)
            .await?;
    } else {
        // Check if all steps are done
        let pending = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM pipeline_step_runs WHERE pipeline_run_id = $1 AND status = 'pending'",
        )
        .bind(run_id)
        .fetch_one(pool)
        .await?;

        if pending.0 == 0 {
            sqlx::query("UPDATE pipeline_runs SET status = 'completed', finished_at = now() WHERE id = $1")
                .bind(run_id)
                .execute(pool)
                .await?;
        }
    }

    tracing::info!("Pipeline run {run_id} step {step_order} finished: {status}");
    Ok(())
}

async fn poll_script_runs(
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
