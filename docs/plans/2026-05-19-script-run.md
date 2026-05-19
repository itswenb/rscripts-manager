# ScriptRun + OutputFile Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement R script execution engine — submit runs against registered workflow steps, execute via Rscript, track status, capture stdout/stderr, and catalog generated output files.

**Architecture:** `script_runs` table tracks execution state (pending → running → completed/failed). `output_files` table catalogs generated outputs per run. The `rrunner` crate handles actual Rscript subprocess execution. The `worker` crate polls for pending runs and dispatches them. Run directories follow the pattern `{DATA_DIR}/projects/{project_id}/runs/{run_id}/`.

**Tech Stack:** tokio::process (Command), serde_json, sqlx, uuid

---

### Task 1: Create script_runs and output_files migrations

**Files:**

- Create: `migrations/005_create_script_runs.sql`

**Step 1: Write migration SQL**

```sql
CREATE TABLE script_runs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  workflow_step_id UUID NOT NULL REFERENCES workflow_steps(id),
  status TEXT NOT NULL DEFAULT 'pending',
  inputs JSONB NOT NULL DEFAULT '{}'::jsonb,
  params JSONB NOT NULL DEFAULT '{}'::jsonb,
  stdout TEXT,
  stderr TEXT,
  started_at TIMESTAMPTZ,
  finished_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_script_runs_project ON script_runs(project_id);
CREATE INDEX idx_script_runs_status ON script_runs(status);

CREATE TRIGGER script_runs_updated_at
  BEFORE UPDATE ON script_runs
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at();

CREATE TABLE output_files (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  run_id UUID NOT NULL REFERENCES script_runs(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  size_bytes BIGINT NOT NULL DEFAULT 0,
  mime_type TEXT,
  storage_path TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_output_files_run ON output_files(run_id);
```

Status values: `pending`, `running`, `completed`, `failed`.

**Step 2: Apply migration**

Run: `psql $DATABASE_URL -f migrations/005_create_script_runs.sql`
Expected: CREATE TABLE (x2), CREATE INDEX (x3), CREATE TRIGGER

**Step 3: Commit**

```bash
git add migrations/005_create_script_runs.sql
git commit -m "feat: add script_runs and output_files migrations"
```

---

### Task 2: Add ScriptRun and OutputFile models to core

**Files:**

- Create: `crates/core/src/models/script_run.rs`
- Create: `crates/core/src/models/output_file.rs`
- Modify: `crates/core/src/models/mod.rs`

**Step 1: Create ScriptRun model**

`crates/core/src/models/script_run.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ScriptRun {
    pub id: Uuid,
    pub project_id: Uuid,
    pub workflow_step_id: Uuid,
    pub status: String,
    pub inputs: serde_json::Value,
    pub params: serde_json::Value,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScriptRun {
    pub workflow_step_id: Uuid,
    #[serde(default)]
    pub inputs: serde_json::Value,
    #[serde(default)]
    pub params: serde_json::Value,
}
```

**Step 2: Create OutputFile model**

`crates/core/src/models/output_file.rs`:

```rust
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
```

**Step 3: Register modules**

In `crates/core/src/models/mod.rs`:

```rust
pub mod output_file;
pub mod script_run;
pub use output_file::*;
pub use script_run::*;
```

**Step 4: Verify**

Run: `cargo build -p rflow-core`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/core/src/models/script_run.rs crates/core/src/models/output_file.rs crates/core/src/models/mod.rs
git commit -m "feat: add ScriptRun and OutputFile models"
```

---

### Task 3: Implement ScriptRun API handlers

**Files:**

- Create: `crates/api/src/routes/runs.rs`
- Modify: `crates/api/src/routes/mod.rs`

**Step 1: Write handlers**

`crates/api/src/routes/runs.rs`:

```rust
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use rflow_core::models::{CreateScriptRun, OutputFile, ScriptRun};
use rflow_core::AppError;

pub async fn create(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    _auth: AuthUser,
    Json(input): Json<CreateScriptRun>,
) -> Result<(StatusCode, Json<ScriptRun>), ApiError> {
    // Verify workflow step exists
    let _step = sqlx::query("SELECT id FROM workflow_steps WHERE id = $1")
        .bind(input.workflow_step_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::Validation("workflow step not found".into()))?;

    let run = sqlx::query_as::<_, ScriptRun>(
        "INSERT INTO script_runs (project_id, workflow_step_id, inputs, params) \
         VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(project_id)
    .bind(input.workflow_step_id)
    .bind(&input.inputs)
    .bind(&input.params)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(run)))
}

pub async fn list(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<Json<Vec<ScriptRun>>, ApiError> {
    let runs = sqlx::query_as::<_, ScriptRun>(
        "SELECT * FROM script_runs WHERE project_id = $1 ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(runs))
}

pub async fn get(
    State(state): State<AppState>,
    Path((_project_id, run_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<Json<ScriptRun>, ApiError> {
    let run = sqlx::query_as::<_, ScriptRun>(
        "SELECT * FROM script_runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(run))
}

pub async fn list_outputs(
    State(state): State<AppState>,
    Path((_project_id, run_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<Json<Vec<OutputFile>>, ApiError> {
    let outputs = sqlx::query_as::<_, OutputFile>(
        "SELECT * FROM output_files WHERE run_id = $1 ORDER BY name",
    )
    .bind(run_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(outputs))
}
```

**Step 2: Register module**

In `crates/api/src/routes/mod.rs`:

```rust
pub mod runs;
```

**Step 3: Verify**

Run: `cargo build -p rflow-api`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/api/src/routes/runs.rs crates/api/src/routes/mod.rs
git commit -m "feat: implement ScriptRun API handlers"
```

---

### Task 4: Wire up ScriptRun routes

**Files:**

- Modify: `crates/api/src/main.rs`

**Step 1: Add routes**

In `main.rs`, add to the `protected` router:

```rust
.route(
    "/api/projects/{project_id}/runs",
    get(routes::runs::list).post(routes::runs::create),
)
.route(
    "/api/projects/{project_id}/runs/{run_id}",
    get(routes::runs::get),
)
.route(
    "/api/projects/{project_id}/runs/{run_id}/outputs",
    get(routes::runs::list_outputs),
)
```

**Step 2: Verify**

Run: `cargo build -p rflow-api`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/api/src/main.rs
git commit -m "feat: wire up ScriptRun routes"
```

---

### Task 5: Implement rrunner crate — R script executor

**Files:**

- Modify: `crates/rrunner/src/lib.rs`
- Modify: `crates/rrunner/Cargo.toml`

**Step 1: Add dependencies to rrunner**

`crates/rrunner/Cargo.toml`:

```toml
[package]
name = "rflow-rrunner"
version = "0.1.0"
edition.workspace = true

[dependencies]
tokio = { workspace = true }
serde_json.workspace = true
thiserror.workspace = true
```

**Step 2: Implement executor**

`crates/rrunner/src/lib.rs`:

```rust
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
```

**Step 3: Verify**

Run: `cargo build -p rflow-rrunner`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/rrunner/
git commit -m "feat: implement rrunner R script executor"
```

---

### Task 6: Implement worker — poll and execute pending runs

**Files:**

- Modify: `crates/worker/Cargo.toml`
- Modify: `crates/worker/src/main.rs`

**Step 1: Add dependencies**

`crates/worker/Cargo.toml`:

```toml
[package]
name = "rflow-worker"
version = "0.1.0"
edition.workspace = true

[dependencies]
rflow-core = { path = "../core" }
rflow-rrunner = { path = "../rrunner" }
tokio.workspace = true
sqlx.workspace = true
serde_json.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
uuid.workspace = true
chrono.workspace = true
```

**Step 2: Implement worker loop**

`crates/worker/src/main.rs`:

```rust
use chrono::Utc;
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
    let rscript_path = std::env::var("RSCRIPT_PATH")
        .unwrap_or_else(|_| "/usr/bin/Rscript".into());
    let data_dir = std::env::var("DATA_DIR")
        .unwrap_or_else(|_| "./data".into());

    tracing::info!("Worker started, polling for pending runs...");

    loop {
        if let Err(e) = poll_and_execute(&pool, &rscript_path, &data_dir).await {
            tracing::error!("Worker error: {e}");
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

async fn poll_and_execute(pool: &PgPool, rscript_path: &str, data_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let row = sqlx::query_as::<_, (Uuid, Uuid, String, serde_json::Value, serde_json::Value)>(
        "UPDATE script_runs SET status = 'running', started_at = now() \
         WHERE id = (SELECT id FROM script_runs WHERE status = 'pending' ORDER BY created_at LIMIT 1 FOR UPDATE SKIP LOCKED) \
         RETURNING id, project_id, workflow_step_id::text, inputs, params",
    )
    .fetch_optional(pool)
    .await?;

    let Some((run_id, project_id, step_id_str, inputs, params)) = row else {
        return Ok(());
    };

    let step_id: Uuid = step_id_str.parse()?;
    let step = sqlx::query_as::<_, (String,)>(
        "SELECT script_path FROM workflow_steps WHERE id = $1",
    )
    .bind(step_id)
    .fetch_one(pool)
    .await?;

    let run_dir = PathBuf::from(format!("{data_dir}/projects/{project_id}/runs/{run_id}"));
    tokio::fs::create_dir_all(&run_dir).await?;

    // Write inputs.json and params.json
    tokio::fs::write(run_dir.join("inputs.json"), serde_json::to_string_pretty(&inputs)?).await?;
    tokio::fs::write(run_dir.join("params.json"), serde_json::to_string_pretty(&params)?).await?;

    let result = rflow_rrunner::execute_script(rscript_path, &step.0, &run_dir).await?;

    let status = if result.success { "completed" } else { "failed" };

    // Save stdout/stderr logs
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
```

**Step 3: Verify**

Run: `cargo build -p rflow-worker`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/worker/
git commit -m "feat: implement worker polling loop for script execution"
```

---

## Execution Batches

| Batch | Tasks | Focus |
|-------|-------|-------|
| 1 | 1-2 | Migrations + models |
| 2 | 3-4 | API handlers + routing |
| 3 | 5-6 | rrunner executor + worker loop |

## Notes

- Worker uses `FOR UPDATE SKIP LOCKED` for safe concurrent polling (multiple workers possible).
- Run directory structure: `{DATA_DIR}/projects/{project_id}/runs/{run_id}/{inputs.json, params.json, outputs/, stdout.log, stderr.log}`.
- The worker is a separate binary (`rflow-worker`) that runs alongside the API server.
- Output files are cataloged after execution completes — the worker scans the outputs directory.
- Status transitions: `pending` → `running` → `completed`/`failed`. No retry logic in MVP.
