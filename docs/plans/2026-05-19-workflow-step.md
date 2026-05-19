# WorkflowStep Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement admin-managed workflow step registration — define R scripts with input schemas, parameter schemas, and output expectations that users can execute.

**Architecture:** `workflow_steps` table stores registered R scripts with JSON schemas for inputs and parameters. Admin-only CRUD endpoints. Scripts must exist on the filesystem at a configured path. Validation ensures script file exists before registration.

**Tech Stack:** sqlx (JSON columns), serde_json, tokio::fs

---

### Task 1: Create workflow_steps migration

**Files:**

- Create: `migrations/004_create_workflow_steps.sql`

**Step 1: Write migration SQL**

```sql
CREATE TABLE workflow_steps (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL DEFAULT '',
  script_path TEXT NOT NULL,
  input_schema JSONB NOT NULL DEFAULT '[]'::jsonb,
  param_schema JSONB NOT NULL DEFAULT '[]'::jsonb,
  output_dir_name TEXT NOT NULL DEFAULT 'outputs',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER workflow_steps_updated_at
  BEFORE UPDATE ON workflow_steps
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at();
```

`input_schema` example: `[{"name": "data_file", "type": "file", "required": true}]`
`param_schema` example: `[{"name": "threshold", "type": "number", "default": 0.05}]`

**Step 2: Apply migration**

Run: `psql $DATABASE_URL -f migrations/004_create_workflow_steps.sql`
Expected: CREATE TABLE, CREATE TRIGGER

**Step 3: Commit**

```bash
git add migrations/004_create_workflow_steps.sql
git commit -m "feat: add workflow_steps table migration"
```

---

### Task 2: Add WorkflowStep model to core

**Files:**

- Create: `crates/core/src/models/workflow_step.rs`
- Modify: `crates/core/src/models/mod.rs`

**Step 1: Create model**

`crates/core/src/models/workflow_step.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub script_path: String,
    pub input_schema: serde_json::Value,
    pub param_schema: serde_json::Value,
    pub output_dir_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowStep {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub script_path: String,
    #[serde(default = "default_schema")]
    pub input_schema: serde_json::Value,
    #[serde(default = "default_schema")]
    pub param_schema: serde_json::Value,
    #[serde(default = "default_output_dir")]
    pub output_dir_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowStep {
    pub name: Option<String>,
    pub description: Option<String>,
    pub script_path: Option<String>,
    pub input_schema: Option<serde_json::Value>,
    pub param_schema: Option<serde_json::Value>,
    pub output_dir_name: Option<String>,
}

fn default_schema() -> serde_json::Value {
    serde_json::json!([])
}

fn default_output_dir() -> String {
    "outputs".into()
}
```

**Step 2: Register module**

In `crates/core/src/models/mod.rs`:

```rust
pub mod workflow_step;
pub use workflow_step::*;
```

**Step 3: Verify**

Run: `cargo build -p rflow-core`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/core/src/models/workflow_step.rs crates/core/src/models/mod.rs
git commit -m "feat: add WorkflowStep model"
```

---

### Task 3: Implement WorkflowStep CRUD handlers

**Files:**

- Create: `crates/api/src/routes/workflow_steps.rs`
- Modify: `crates/api/src/routes/mod.rs`

**Step 1: Write handlers**

`crates/api/src/routes/workflow_steps.rs`:

```rust
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use rflow_core::models::{CreateWorkflowStep, UpdateWorkflowStep, WorkflowStep};
use rflow_core::AppError;

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<Vec<WorkflowStep>>, ApiError> {
    let steps = sqlx::query_as::<_, WorkflowStep>(
        "SELECT * FROM workflow_steps ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(steps))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<CreateWorkflowStep>,
) -> Result<(StatusCode, Json<WorkflowStep>), ApiError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden.into());
    }
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name cannot be empty".into()).into());
    }

    // Verify script exists on filesystem
    let rscript_base = std::env::var("RSCRIPT_PATH").unwrap_or_else(|_| "/usr/bin/Rscript".into());
    let script_full = std::path::Path::new(&input.script_path);
    if !script_full.exists() {
        return Err(AppError::Validation(format!("script not found: {}", input.script_path)).into());
    }

    let step = sqlx::query_as::<_, WorkflowStep>(
        "INSERT INTO workflow_steps (name, description, script_path, input_schema, param_schema, output_dir_name) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING *",
    )
    .bind(input.name.trim())
    .bind(&input.description)
    .bind(&input.script_path)
    .bind(&input.input_schema)
    .bind(&input.param_schema)
    .bind(&input.output_dir_name)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(step)))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<Json<WorkflowStep>, ApiError> {
    let step = sqlx::query_as::<_, WorkflowStep>(
        "SELECT * FROM workflow_steps WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(step))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    Json(input): Json<UpdateWorkflowStep>,
) -> Result<Json<WorkflowStep>, ApiError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden.into());
    }
    let existing = sqlx::query_as::<_, WorkflowStep>(
        "SELECT * FROM workflow_steps WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let name = input.name.unwrap_or(existing.name);
    let description = input.description.unwrap_or(existing.description);
    let script_path = input.script_path.unwrap_or(existing.script_path);
    let input_schema = input.input_schema.unwrap_or(existing.input_schema);
    let param_schema = input.param_schema.unwrap_or(existing.param_schema);
    let output_dir_name = input.output_dir_name.unwrap_or(existing.output_dir_name);

    let step = sqlx::query_as::<_, WorkflowStep>(
        "UPDATE workflow_steps SET name=$1, description=$2, script_path=$3, input_schema=$4, param_schema=$5, output_dir_name=$6 \
         WHERE id=$7 RETURNING *",
    )
    .bind(name.trim())
    .bind(&description)
    .bind(&script_path)
    .bind(&input_schema)
    .bind(&param_schema)
    .bind(&output_dir_name)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(step))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<StatusCode, ApiError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden.into());
    }
    let result = sqlx::query("DELETE FROM workflow_steps WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound.into());
    }
    Ok(StatusCode::NO_CONTENT)
}
```

**Step 2: Register module**

In `crates/api/src/routes/mod.rs`:

```rust
pub mod workflow_steps;
```

**Step 3: Verify**

Run: `cargo build -p rflow-api`
Expected: PASS (with unused warnings)

**Step 4: Commit**

```bash
git add crates/api/src/routes/workflow_steps.rs crates/api/src/routes/mod.rs
git commit -m "feat: implement WorkflowStep CRUD handlers"
```

---

### Task 4: Wire up WorkflowStep routes

**Files:**

- Modify: `crates/api/src/main.rs`

**Step 1: Add routes to protected router**

In `main.rs`, add to the `protected` router:

```rust
.route(
    "/api/workflow-steps",
    get(routes::workflow_steps::list).post(routes::workflow_steps::create),
)
.route(
    "/api/workflow-steps/{id}",
    get(routes::workflow_steps::get)
        .patch(routes::workflow_steps::update)
        .delete(routes::workflow_steps::delete),
)
```

**Step 2: Verify**

Run: `cargo build -p rflow-api`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/api/src/main.rs
git commit -m "feat: wire up WorkflowStep routes"
```

---

## Execution Batches

| Batch | Tasks | Focus |
|-------|-------|-------|
| 1 | 1-2 | Migration + model |
| 2 | 3-4 | Handlers + routing |

## Notes

- Only `admin` role can create/update/delete workflow steps; all authenticated users can list/get.
- `script_path` is validated against the filesystem at creation time.
- `input_schema` and `param_schema` are stored as JSONB for flexibility — schema validation of the JSON structure itself can be added later.
- The `output_dir_name` field tells the runner where to expect outputs relative to the run directory.
