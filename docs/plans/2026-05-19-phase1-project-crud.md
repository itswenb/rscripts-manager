# Phase 1: Project CRUD + Database Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement Project CRUD API with PostgreSQL persistence, migration infrastructure, and typed domain models.

**Architecture:** Direct sqlx queries in Axum handlers (no repository trait). AppError with thiserror in core crate. PostgreSQL trigger for updated_at. Offset-based pagination.

**Tech Stack:** Rust, Axum 0.8, SQLx 0.8, PostgreSQL 16, tokio, thiserror, uuid, chrono

---

### Task 1: Database Migration Infrastructure

**Model hint:** `codex`

**Files:**
- Create: `migrations/001_create_projects.sql`
- Modify: `crates/core/Cargo.toml`
- Modify: `crates/core/src/db.rs`

**Step 1: Create migration file**

Create `migrations/001_create_projects.sql`:
```sql
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE projects (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER projects_updated_at
  BEFORE UPDATE ON projects
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at();
```

**Step 2: Implement db module**

Write `crates/core/src/db.rs`:
```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Failed to connect to database")
}
```

**Step 3: Run migration**

```bash
cp .env.example .env
docker compose up -d
sqlx database create --database-url postgres://rflow:rflow@localhost:5432/rflow
sqlx migrate run --source migrations --database-url postgres://rflow:rflow@localhost:5432/rflow
```
Expected: Migration applied successfully

**Step 4: Verify table exists**

Run: `psql postgres://rflow:rflow@localhost:5432/rflow -c "\d projects"`
Expected: Shows projects table with all columns

**Step 5: Commit**

```bash
git add migrations/ crates/core/src/db.rs
git commit -m "feat: add projects migration and db pool setup"
```

---

### Task 2: Domain Models

**Model hint:** `codex`

**Files:**
- Create: `crates/core/src/models/mod.rs`
- Create: `crates/core/src/models/project.rs`
- Modify: `crates/core/src/lib.rs`

**Step 1: Create models directory structure**

Replace `crates/core/src/models.rs` with `crates/core/src/models/mod.rs`:
```rust
pub mod project;
pub use project::*;
```

**Step 2: Create Project model**

Write `crates/core/src/models/project.rs`:
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProject {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
```

**Step 3: Verify compilation**

Run: `cargo build -p rflow-core`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/core/src/models/
git rm crates/core/src/models.rs 2>/dev/null || true
git commit -m "feat: add Project domain models"
```

---

### Task 3: Error Handling

**Model hint:** `codex`

**Files:**
- Create: `crates/core/src/error.rs`
- Modify: `crates/core/src/lib.rs`

**Step 1: Create error module**

Write `crates/core/src/error.rs`:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("validation: {0}")]
    Validation(String),
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("internal: {0}")]
    Internal(String),
}
```

**Step 2: Update lib.rs**

Modify `crates/core/src/lib.rs`:
```rust
pub mod db;
pub mod error;
pub mod models;

pub use error::AppError;
```

**Step 3: Verify compilation**

Run: `cargo build -p rflow-core`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/core/src/error.rs crates/core/src/lib.rs
git commit -m "feat: add AppError with thiserror"
```

---

### Task 4: API State and Error Response

**Model hint:** `codex`

**Files:**
- Create: `crates/api/src/state.rs`
- Create: `crates/api/src/error.rs`

**Step 1: Create AppState**

Write `crates/api/src/state.rs`:
```rust
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}
```

**Step 2: Create API error response layer**

Write `crates/api/src/error.rs`:
```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rflow_core::AppError;
use serde_json::json;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error".to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

注意：IntoResponse 是为外部类型实现外部 trait，需要用 newtype wrapper 或者把 impl 放在定义 AppError 的 crate 中。更好的方案是在 api crate 中创建一个 wrapper：

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rflow_core::AppError;
use serde_json::json;

pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        Self(e)
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        Self(AppError::Database(e))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error".to_string()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

**Step 3: Verify compilation**

Run: `cargo build -p rflow-api`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/api/src/state.rs crates/api/src/error.rs
git commit -m "feat: add AppState and ApiError response layer"
```

---

### Task 5: Project CRUD Route Handlers

**Model hint:** `codex`

**Files:**
- Create: `crates/api/src/routes/mod.rs`
- Create: `crates/api/src/routes/projects.rs`

**Step 1: Create routes module**

Write `crates/api/src/routes/mod.rs`:
```rust
pub mod projects;
```

**Step 2: Implement Project CRUD handlers**

Write `crates/api/src/routes/projects.rs`:
```rust
use axum::extract::{Path, Query, State};
use axum::Json;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;
use rflow_core::models::{CreateProject, PaginationParams, Project, UpdateProject};
use rflow_core::AppError;

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<Project>>, ApiError> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);
    let projects = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, updated_at FROM projects ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(projects))
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateProject>,
) -> Result<(axum::http::StatusCode, Json<Project>), ApiError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name cannot be empty".into()).into());
    }
    let project = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (name, description) VALUES ($1, $2) RETURNING id, name, description, created_at, updated_at"
    )
    .bind(input.name.trim())
    .bind(&input.description)
    .fetch_one(&state.pool)
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(project)))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Project>, ApiError> {
    let project = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(project))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateProject>,
) -> Result<Json<Project>, ApiError> {
    let existing = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let name = input.name.unwrap_or(existing.name);
    let description = input.description.unwrap_or(existing.description);

    let project = sqlx::query_as::<_, Project>(
        "UPDATE projects SET name = $1, description = $2 WHERE id = $3 RETURNING id, name, description, created_at, updated_at"
    )
    .bind(name.trim())
    .bind(&description)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(project))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound.into());
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}
```

**Step 3: Verify compilation**

Run: `cargo build -p rflow-api`
Expected: Compiles (may need to add `mod routes;` to main.rs)

**Step 4: Commit**

```bash
git add crates/api/src/routes/
git commit -m "feat: implement Project CRUD handlers"
```

---

### Task 6: Wire Up Router and Main

**Model hint:** `codex`

**Files:**
- Modify: `crates/api/src/main.rs`

**Step 1: Update main.rs to wire everything together**

Rewrite `crates/api/src/main.rs`:
```rust
use axum::{routing::{get, post, patch, delete}, Router};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod error;
mod routes;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://rflow:rflow@localhost:5432/rflow".into());

    let pool = rflow_core::db::create_pool(&database_url).await;
    let state = AppState { pool };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/projects", get(routes::projects::list).post(routes::projects::create))
        .route("/api/projects/{id}", get(routes::projects::get).patch(routes::projects::update).delete(routes::projects::delete))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4001").await.unwrap();
    tracing::info!("API server listening on :4001");
    axum::serve(listener, app).await.unwrap();
}
```

**Step 2: Verify full build**

Run: `cargo build --workspace`
Expected: Compiles successfully

**Step 3: Start server and test CRUD**

```bash
# Terminal 1
cargo run -p rflow-api

# Terminal 2
# Create
curl -s -X POST http://localhost:4001/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name":"Test Project","description":"A test"}' | jq .

# List
curl -s http://localhost:4001/api/projects | jq .

# Get (use id from create response)
curl -s http://localhost:4001/api/projects/<UUID> | jq .

# Update
curl -s -X PATCH http://localhost:4001/api/projects/<UUID> \
  -H "Content-Type: application/json" \
  -d '{"name":"Updated Name"}' | jq .

# Delete
curl -s -X DELETE http://localhost:4001/api/projects/<UUID> -w "%{http_code}"
```
Expected: 201 for create, 200 for list/get/update, 204 for delete

**Step 4: Commit**

```bash
git add crates/api/src/main.rs
git commit -m "feat: wire up Project CRUD routes in API server"
```

---

### Task 7: Integration Test

**Model hint:** `codex`

**Files:**
- Create: `crates/api/tests/projects_test.rs`
- Modify: `crates/api/Cargo.toml` (add dev-dependencies)

**Step 1: Add test dependencies**

Add to `crates/api/Cargo.toml`:
```toml
[dev-dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full", "test-util"] }
```

**Step 2: Write integration test**

Create `crates/api/tests/projects_test.rs`:
```rust
use reqwest::Client;
use serde_json::{json, Value};

async fn base_url() -> String {
    std::env::var("TEST_API_URL").unwrap_or_else(|_| "http://localhost:4001".into())
}

#[tokio::test]
async fn test_project_crud() {
    let client = Client::new();
    let base = base_url().await;

    // Create
    let res = client
        .post(format!("{base}/api/projects"))
        .json(&json!({"name": "Integration Test Project", "description": "testing"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let project: Value = res.json().await.unwrap();
    let id = project["id"].as_str().unwrap();

    // Get
    let res = client.get(format!("{base}/api/projects/{id}")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let fetched: Value = res.json().await.unwrap();
    assert_eq!(fetched["name"], "Integration Test Project");

    // Update
    let res = client
        .patch(format!("{base}/api/projects/{id}"))
        .json(&json!({"name": "Updated Name"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Updated Name");

    // List
    let res = client.get(format!("{base}/api/projects")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let list: Vec<Value> = res.json().await.unwrap();
    assert!(!list.is_empty());

    // Delete
    let res = client.delete(format!("{base}/api/projects/{id}")).send().await.unwrap();
    assert_eq!(res.status(), 204);

    // Verify deleted
    let res = client.get(format!("{base}/api/projects/{id}")).send().await.unwrap();
    assert_eq!(res.status(), 404);
}
```

**Step 3: Run test (requires server running)**

```bash
cargo run -p rflow-api &
sleep 2
cargo test -p rflow-api -- --nocapture
kill %1
```
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/api/tests/ crates/api/Cargo.toml
git commit -m "test: add Project CRUD integration test"
```

---

## Summary

| Task | Description | Key Verification |
|------|-------------|-----------------|
| 1 | Migration + db pool | `sqlx migrate run` succeeds, table exists |
| 2 | Domain models | `cargo build -p rflow-core` passes |
| 3 | Error handling | Compiles with thiserror |
| 4 | AppState + ApiError | `cargo build -p rflow-api` passes |
| 5 | CRUD handlers | Compiles |
| 6 | Wire router | Full CRUD via curl works |
| 7 | Integration test | `cargo test -p rflow-api` passes |
