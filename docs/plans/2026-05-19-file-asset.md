# FileAsset Management Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement file asset management for projects — upload, directory management, move/rename, delete (soft), download, and preview.

**Architecture:** `file_assets` table tracks every file/directory with parent_id for tree structure. Physical files stored at `{DATA_DIR}/projects/{project_id}/uploads/`. Multipart upload via `axum-multipart`. Soft delete moves to `trash/` directory. Preview returns first N lines for text files.

**Tech Stack:** axum-multipart (or `axum::extract::Multipart`), tokio::fs, uuid, mime_guess

---

### Task 1: Create file_assets migration

**Files:**

- Create: `migrations/003_create_file_assets.sql`

**Step 1: Write migration SQL**

```sql
CREATE TABLE file_assets (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  parent_id UUID REFERENCES file_assets(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  is_directory BOOLEAN NOT NULL DEFAULT false,
  size_bytes BIGINT NOT NULL DEFAULT 0,
  mime_type TEXT,
  storage_path TEXT NOT NULL,
  deleted_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_file_assets_project ON file_assets(project_id);
CREATE INDEX idx_file_assets_parent ON file_assets(parent_id);
CREATE UNIQUE INDEX idx_file_assets_unique_name ON file_assets(project_id, parent_id, name) WHERE deleted_at IS NULL;

CREATE TRIGGER file_assets_updated_at
  BEFORE UPDATE ON file_assets
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at();
```

**Step 2: Apply migration**

Run: `psql $DATABASE_URL -f migrations/003_create_file_assets.sql`
Expected: CREATE TABLE, CREATE INDEX (x3), CREATE TRIGGER

**Step 3: Commit**

```bash
git add migrations/003_create_file_assets.sql
git commit -m "feat: add file_assets table migration"
```

---

### Task 2: Add FileAsset model to core

**Files:**

- Create: `crates/core/src/models/file_asset.rs`
- Modify: `crates/core/src/models/mod.rs`

**Step 1: Create model**

`crates/core/src/models/file_asset.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct FileAsset {
    pub id: Uuid,
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub is_directory: bool,
    pub size_bytes: i64,
    pub mime_type: Option<String>,
    pub storage_path: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDirectory {
    pub name: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct MoveAsset {
    pub parent_id: Option<Uuid>,
    pub name: Option<String>,
}
```

**Step 2: Register module**

In `crates/core/src/models/mod.rs`:

```rust
pub mod file_asset;
pub use file_asset::*;
```

**Step 3: Verify**

Run: `cargo build -p rflow-core`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/core/src/models/file_asset.rs crates/core/src/models/mod.rs
git commit -m "feat: add FileAsset model"
```

---

### Task 3: Add multipart dependency and file routes module

**Files:**

- Modify: `Cargo.toml` (workspace root)
- Modify: `crates/api/Cargo.toml`
- Create: `crates/api/src/routes/files.rs`
- Modify: `crates/api/src/routes/mod.rs`

**Step 1: Add dependencies**

In root `Cargo.toml` `[workspace.dependencies]`:

```toml
tokio-util = { version = "0.7", features = ["io"] }
```

In `crates/api/Cargo.toml` `[dependencies]`:

```toml
tokio-util.workspace = true
```

Note: Axum 0.8 has built-in `axum::extract::Multipart` — no extra crate needed for upload.

**Step 2: Create empty routes file**

`crates/api/src/routes/files.rs`:

```rust
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use rflow_core::models::{CreateDirectory, FileAsset, MoveAsset};
use rflow_core::AppError;
```

**Step 3: Register module**

In `crates/api/src/routes/mod.rs`:

```rust
pub mod files;
```

**Step 4: Verify**

Run: `cargo build -p rflow-api`
Expected: PASS (with unused warnings)

**Step 5: Commit**

```bash
git add Cargo.toml crates/api/Cargo.toml crates/api/src/routes/files.rs crates/api/src/routes/mod.rs Cargo.lock
git commit -m "feat: scaffold file routes module"
```

---

### Task 4: Implement file upload handler

**Files:**

- Modify: `crates/api/src/routes/files.rs`

**Step 1: Write upload handler**

Append to `crates/api/src/routes/files.rs`:

```rust
pub async fn upload(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(params): Query<UploadParams>,
    _auth: AuthUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Vec<FileAsset>>), ApiError> {
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".into());
    let upload_dir = format!("{data_dir}/projects/{project_id}/uploads");
    tokio::fs::create_dir_all(&upload_dir).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut assets = Vec::new();

    while let Some(field) = multipart.next_field().await
        .map_err(|e| AppError::Internal(e.to_string()))? {
        let file_name = field.file_name()
            .ok_or_else(|| AppError::Validation("missing filename".into()))?
            .to_string();

        let data = field.bytes().await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let file_id = Uuid::new_v4();
        let ext = std::path::Path::new(&file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let storage_name = if ext.is_empty() {
            file_id.to_string()
        } else {
            format!("{file_id}.{ext}")
        };
        let storage_path = format!("{upload_dir}/{storage_name}");

        tokio::fs::write(&storage_path, &data).await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mime = mime_guess::from_path(&file_name)
            .first_raw()
            .map(|s| s.to_string());

        let asset = sqlx::query_as::<_, FileAsset>(
            "INSERT INTO file_assets (id, project_id, parent_id, name, is_directory, size_bytes, mime_type, storage_path)
             VALUES ($1, $2, $3, $4, false, $5, $6, $7)
             RETURNING *",
        )
        .bind(file_id)
        .bind(project_id)
        .bind(params.parent_id)
        .bind(&file_name)
        .bind(data.len() as i64)
        .bind(&mime)
        .bind(&storage_path)
        .fetch_one(&state.pool)
        .await?;

        assets.push(asset);
    }

    Ok((StatusCode::CREATED, Json(assets)))
}

#[derive(Debug, serde::Deserialize)]
pub struct UploadParams {
    pub parent_id: Option<Uuid>,
}
```

**Step 2: Add `mime_guess` dependency**

In root `Cargo.toml`:

```toml
mime_guess = "2"
```

In `crates/api/Cargo.toml`:

```toml
mime_guess.workspace = true
```

**Step 3: Verify**

Run: `cargo build -p rflow-api`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/api/src/routes/files.rs Cargo.toml crates/api/Cargo.toml Cargo.lock
git commit -m "feat: implement file upload handler"
```

---

### Task 5: Implement list, create directory, and get handlers

**Files:**

- Modify: `crates/api/src/routes/files.rs`

**Step 1: Add list handler**

```rust
#[derive(Debug, serde::Deserialize)]
pub struct ListParams {
    pub parent_id: Option<Uuid>,
}

pub async fn list(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(params): Query<ListParams>,
    _auth: AuthUser,
) -> Result<Json<Vec<FileAsset>>, ApiError> {
    let assets = if let Some(parent_id) = params.parent_id {
        sqlx::query_as::<_, FileAsset>(
            "SELECT * FROM file_assets WHERE project_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY is_directory DESC, name",
        )
        .bind(project_id)
        .bind(parent_id)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, FileAsset>(
            "SELECT * FROM file_assets WHERE project_id = $1 AND parent_id IS NULL AND deleted_at IS NULL ORDER BY is_directory DESC, name",
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await?
    };
    Ok(Json(assets))
}
```

**Step 2: Add create directory handler**

```rust
pub async fn create_directory(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    _auth: AuthUser,
    Json(input): Json<CreateDirectory>,
) -> Result<(StatusCode, Json<FileAsset>), ApiError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name cannot be empty".into()).into());
    }

    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".into());
    let dir_path = format!("{data_dir}/projects/{project_id}/uploads/{}", Uuid::new_v4());
    tokio::fs::create_dir_all(&dir_path).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let asset = sqlx::query_as::<_, FileAsset>(
        "INSERT INTO file_assets (project_id, parent_id, name, is_directory, storage_path)
         VALUES ($1, $2, $3, true, $4)
         RETURNING *",
    )
    .bind(project_id)
    .bind(input.parent_id)
    .bind(input.name.trim())
    .bind(&dir_path)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(asset)))
}
```

**Step 3: Add get handler**

```rust
pub async fn get(
    State(state): State<AppState>,
    Path((_project_id, asset_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<Json<FileAsset>, ApiError> {
    let asset = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(asset_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(asset))
}
```

**Step 4: Verify**

Run: `cargo build -p rflow-api`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/api/src/routes/files.rs
git commit -m "feat: add list, create_directory, and get file handlers"
```

---

### Task 6: Implement move/rename and soft delete

**Files:**

- Modify: `crates/api/src/routes/files.rs`

**Step 1: Add move/rename handler**

```rust
pub async fn move_asset(
    State(state): State<AppState>,
    Path((_project_id, asset_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
    Json(input): Json<MoveAsset>,
) -> Result<Json<FileAsset>, ApiError> {
    let existing = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(asset_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let new_name = input.name.unwrap_or(existing.name);
    let new_parent = input.parent_id.or(existing.parent_id);

    let asset = sqlx::query_as::<_, FileAsset>(
        "UPDATE file_assets SET name = $1, parent_id = $2 WHERE id = $3 RETURNING *",
    )
    .bind(new_name.trim())
    .bind(new_parent)
    .bind(asset_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(asset))
}
```

**Step 2: Add soft delete handler**

```rust
pub async fn delete(
    State(state): State<AppState>,
    Path((_project_id, asset_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query(
        "UPDATE file_assets SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(asset_id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound.into());
    }
    Ok(StatusCode::NO_CONTENT)
}
```

**Step 3: Verify**

Run: `cargo build -p rflow-api`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/api/src/routes/files.rs
git commit -m "feat: add move/rename and soft delete for file assets"
```

---

### Task 7: Implement download and preview

**Files:**

- Modify: `crates/api/src/routes/files.rs`

**Step 1: Add download handler**

```rust
use axum::body::Body;
use axum::response::Response;
use tokio_util::io::ReaderStream;

pub async fn download(
    State(state): State<AppState>,
    Path((_project_id, asset_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<Response, ApiError> {
    let asset = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE id = $1 AND deleted_at IS NULL AND is_directory = false",
    )
    .bind(asset_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let file = tokio::fs::File::open(&asset.storage_path).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let stream = ReaderStream::new(file);

    let content_type = asset.mime_type.unwrap_or_else(|| "application/octet-stream".into());

    Ok(Response::builder()
        .header("content-type", content_type)
        .header("content-disposition", format!("attachment; filename=\"{}\"", asset.name))
        .body(Body::from_stream(stream))
        .unwrap())
}
```

**Step 2: Add preview handler**

```rust
pub async fn preview(
    State(state): State<AppState>,
    Path((_project_id, asset_id)): Path<(Uuid, Uuid)>,
    _auth: AuthUser,
) -> Result<String, ApiError> {
    let asset = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE id = $1 AND deleted_at IS NULL AND is_directory = false",
    )
    .bind(asset_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let content = tokio::fs::read_to_string(&asset.storage_path).await
        .map_err(|_| AppError::Validation("file is not text".into()))?;

    let preview: String = content.lines().take(100).collect::<Vec<_>>().join("\n");
    Ok(preview)
}
```

**Step 3: Verify**

Run: `cargo build -p rflow-api`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/api/src/routes/files.rs
git commit -m "feat: add download and preview handlers"
```

---

### Task 8: Wire up file routes in main.rs

**Files:**

- Modify: `crates/api/src/main.rs`

**Step 1: Add file routes to protected router**

```rust
use axum::routing::{get, post, patch, delete};

let protected = Router::new()
    // ... existing project routes ...
    .route("/api/projects/{project_id}/files", get(routes::files::list).post(routes::files::upload))
    .route("/api/projects/{project_id}/files/directory", post(routes::files::create_directory))
    .route("/api/projects/{project_id}/files/{asset_id}", get(routes::files::get).patch(routes::files::move_asset).delete(routes::files::delete))
    .route("/api/projects/{project_id}/files/{asset_id}/download", get(routes::files::download))
    .route("/api/projects/{project_id}/files/{asset_id}/preview", get(routes::files::preview))
    .route_layer(axum_mw::from_fn_with_state(state.clone(), middleware::casbin_auth));
```

**Step 2: Verify**

Run: `cargo build -p rflow-api`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/api/src/main.rs
git commit -m "feat: wire up file asset routes"
```

---

### Task 9: Update Casbin policy for file routes

**Files:**

- Modify: `crates/api/src/main.rs` (seeding section)
- Modify: `config/casbin_policy.csv`

**Step 1: Policy already covers `/api/*`**

The existing policy `p, admin, /api/*, GET|POST|PATCH|DELETE` already covers file routes since they're under `/api/projects/{id}/files/*` and the matcher uses `keyMatch2`.

No code change needed — just verify the matcher handles nested paths.

**Step 2: Commit (if any changes)**

No commit needed for this task.

---

## Execution Batches

| Batch | Tasks | Focus |
|-------|-------|-------|
| 1 | 1-3 | Migration + model + scaffold |
| 2 | 4-5 | Upload + list/directory/get |
| 3 | 6-7 | Move/delete + download/preview |
| 4 | 8-9 | Route wiring + policy verification |

## Notes

- Files are stored with UUID names on disk to avoid path conflicts; original name preserved in DB.
- Soft delete sets `deleted_at` timestamp; queries filter `WHERE deleted_at IS NULL`.
- The unique index on `(project_id, parent_id, name)` prevents duplicate filenames in the same directory.
- Large file upload should eventually use streaming/chunked upload, but for MVP the full-body approach is acceptable.
- `DATA_DIR` env var controls the root storage path.
