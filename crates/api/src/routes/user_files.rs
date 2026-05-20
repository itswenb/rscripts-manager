use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::audit;
use crate::state::AppState;
use rflow_core::models::{CreateDirectory, FileAsset};
use rflow_core::AppError;

#[derive(Debug, serde::Deserialize)]
pub struct FileListParams {
    pub parent_id: Option<Uuid>,
}

pub async fn list_my_files(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<FileListParams>,
) -> Result<Json<Vec<FileAsset>>, ApiError> {
    let user_id = get_user_id(&state, &auth).await?;
    let files = if let Some(parent_id) = params.parent_id {
        sqlx::query_as::<_, FileAsset>(
            "SELECT * FROM file_assets WHERE owner_id = $1 AND parent_id = $2 AND deleted_at IS NULL ORDER BY is_directory DESC, name",
        )
        .bind(user_id)
        .bind(parent_id)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, FileAsset>(
            "SELECT * FROM file_assets WHERE owner_id = $1 AND parent_id IS NULL AND deleted_at IS NULL ORDER BY is_directory DESC, name",
        )
        .bind(user_id)
        .fetch_all(&state.pool)
        .await?
    };
    Ok(Json(files))
}

pub async fn list_public_files(
    State(state): State<AppState>,
    Query(params): Query<FileListParams>,
) -> Result<Json<Vec<FileAsset>>, ApiError> {
    let files = if let Some(parent_id) = params.parent_id {
        sqlx::query_as::<_, FileAsset>(
            "SELECT * FROM file_assets WHERE is_public = true AND parent_id = $2 AND deleted_at IS NULL ORDER BY is_directory DESC, name",
        )
        .bind(parent_id)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, FileAsset>(
            "SELECT * FROM file_assets WHERE is_public = true AND parent_id IS NULL AND deleted_at IS NULL ORDER BY is_directory DESC, name",
        )
        .fetch_all(&state.pool)
        .await?
    };
    Ok(Json(files))
}

pub async fn upload_my_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<FileListParams>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Vec<FileAsset>>), ApiError> {
    let user_id = get_user_id(&state, &auth).await?;
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".into());
    let upload_dir = format!("{data_dir}/users/{user_id}");
    tokio::fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut assets = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        let file_name = field
            .file_name()
            .ok_or_else(|| AppError::Validation("missing filename".into()))?
            .to_string();

        let data = field
            .bytes()
            .await
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

        tokio::fs::write(&storage_path, &data)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mime = mime_guess::from_path(&file_name)
            .first_raw()
            .map(|s| s.to_string());

        let asset = sqlx::query_as::<_, FileAsset>(
            "INSERT INTO file_assets (id, owner_id, parent_id, name, is_directory, size_bytes, mime_type, storage_path, is_public)
             VALUES ($1, $2, $3, $4, false, $5, $6, $7, false)
             RETURNING *",
        )
        .bind(file_id)
        .bind(user_id)
        .bind(params.parent_id)
        .bind(&file_name)
        .bind(data.len() as i64)
        .bind(&mime)
        .bind(&storage_path)
        .fetch_one(&state.pool)
        .await?;

        assets.push(asset);
    }

    for a in &assets {
        audit::log(&state.pool, &auth, "upload", "file", Some(&a.id.to_string()), None).await;
    }
    Ok((StatusCode::CREATED, Json(assets)))
}

pub async fn create_my_directory(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<CreateDirectory>,
) -> Result<(StatusCode, Json<FileAsset>), ApiError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name cannot be empty".into()).into());
    }
    let user_id = get_user_id(&state, &auth).await?;
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".into());
    let dir_path = format!("{data_dir}/users/{user_id}/{}", Uuid::new_v4());
    tokio::fs::create_dir_all(&dir_path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let asset = sqlx::query_as::<_, FileAsset>(
        "INSERT INTO file_assets (owner_id, parent_id, name, is_directory, storage_path, is_public)
         VALUES ($1, $2, $3, true, $4, false)
         RETURNING *",
    )
    .bind(user_id)
    .bind(input.parent_id)
    .bind(input.name.trim())
    .bind(&dir_path)
    .fetch_one(&state.pool)
    .await?;

    audit::log(&state.pool, &auth, "create", "directory", Some(&asset.id.to_string()), None).await;
    Ok((StatusCode::CREATED, Json(asset)))
}

pub async fn move_to_public(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(asset_id): Path<Uuid>,
) -> Result<Json<FileAsset>, ApiError> {
    let user_id = get_user_id(&state, &auth).await?;
    let asset = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(asset_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if asset.owner_id != Some(user_id) {
        return Err(AppError::Forbidden.into());
    }

    let updated = sqlx::query_as::<_, FileAsset>(
        "UPDATE file_assets SET is_public = true, parent_id = NULL WHERE id = $1 RETURNING *",
    )
    .bind(asset_id)
    .fetch_one(&state.pool)
    .await?;

    audit::log(&state.pool, &auth, "move_to_public", "file", Some(&asset_id.to_string()), None).await;
    Ok(Json(updated))
}

pub async fn delete_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(asset_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let asset = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(asset_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let user_id = get_user_id(&state, &auth).await?;
    let role = get_user_role(&state, &auth).await?;

    if asset.is_public && role != "admin" {
        return Err(AppError::Forbidden.into());
    }
    if !asset.is_public && asset.owner_id != Some(user_id) {
        return Err(AppError::Forbidden.into());
    }

    sqlx::query("UPDATE file_assets SET deleted_at = now() WHERE id = $1")
        .bind(asset_id)
        .execute(&state.pool)
        .await?;

    audit::log(&state.pool, &auth, "delete", "file", Some(&asset_id.to_string()), None).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn download_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(asset_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let asset = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE id = $1 AND deleted_at IS NULL AND is_directory = false",
    )
    .bind(asset_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let user_id = get_user_id(&state, &auth).await?;
    if !asset.is_public && asset.owner_id != Some(user_id) {
        return Err(AppError::Forbidden.into());
    }

    audit::log(&state.pool, &auth, "download", "file", Some(&asset_id.to_string()), None).await;

    let file = tokio::fs::File::open(&asset.storage_path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let stream = ReaderStream::new(file);
    let content_type = asset.mime_type.unwrap_or_else(|| "application/octet-stream".into());

    Ok(Response::builder()
        .header("content-type", content_type)
        .header("content-disposition", format!("attachment; filename=\"{}\"", asset.name))
        .body(Body::from_stream(stream))
        .unwrap())
}

#[derive(Debug, serde::Deserialize)]
pub struct RenameInput {
    pub name: String,
}

pub async fn rename_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(asset_id): Path<Uuid>,
    Json(input): Json<RenameInput>,
) -> Result<Json<FileAsset>, ApiError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name cannot be empty".into()).into());
    }
    let user_id = get_user_id(&state, &auth).await?;
    let role = get_user_role(&state, &auth).await?;
    let asset = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(asset_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if asset.is_public && role != "admin" {
        return Err(AppError::Forbidden.into());
    }
    if !asset.is_public && asset.owner_id != Some(user_id) {
        return Err(AppError::Forbidden.into());
    }

    let updated = sqlx::query_as::<_, FileAsset>(
        "UPDATE file_assets SET name = $1 WHERE id = $2 RETURNING *",
    )
    .bind(input.name.trim())
    .bind(asset_id)
    .fetch_one(&state.pool)
    .await?;

    audit::log(&state.pool, &auth, "rename", "file", Some(&asset_id.to_string()), None).await;
    Ok(Json(updated))
}

#[derive(Debug, serde::Deserialize)]
pub struct MoveInput {
    pub parent_id: Option<Uuid>,
}

pub async fn move_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(asset_id): Path<Uuid>,
    Json(input): Json<MoveInput>,
) -> Result<Json<FileAsset>, ApiError> {
    let user_id = get_user_id(&state, &auth).await?;
    let asset = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(asset_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if asset.owner_id != Some(user_id) {
        return Err(AppError::Forbidden.into());
    }

    // Verify target parent belongs to same user or is null (root)
    if let Some(parent_id) = input.parent_id {
        let parent = sqlx::query_as::<_, FileAsset>(
            "SELECT * FROM file_assets WHERE id = $1 AND is_directory = true AND deleted_at IS NULL",
        )
        .bind(parent_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::Validation("target folder not found".into()))?;

        if parent.owner_id != Some(user_id) && !parent.is_public {
            return Err(AppError::Forbidden.into());
        }
    }

    let updated = sqlx::query_as::<_, FileAsset>(
        "UPDATE file_assets SET parent_id = $1 WHERE id = $2 RETURNING *",
    )
    .bind(input.parent_id)
    .bind(asset_id)
    .fetch_one(&state.pool)
    .await?;

    audit::log(&state.pool, &auth, "move", "file", Some(&asset_id.to_string()), None).await;
    Ok(Json(updated))
}

pub async fn copy_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(asset_id): Path<Uuid>,
    Json(input): Json<MoveInput>,
) -> Result<(StatusCode, Json<FileAsset>), ApiError> {
    let user_id = get_user_id(&state, &auth).await?;
    let asset = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE id = $1 AND deleted_at IS NULL AND is_directory = false",
    )
    .bind(asset_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Can copy own files or public files
    if !asset.is_public && asset.owner_id != Some(user_id) {
        return Err(AppError::Forbidden.into());
    }

    let new_id = Uuid::new_v4();
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".into());
    let ext = std::path::Path::new(&asset.name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let storage_name = if ext.is_empty() {
        new_id.to_string()
    } else {
        format!("{new_id}.{ext}")
    };
    let new_path = format!("{data_dir}/users/{user_id}/{storage_name}");

    tokio::fs::copy(&asset.storage_path, &new_path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let copy = sqlx::query_as::<_, FileAsset>(
        "INSERT INTO file_assets (id, owner_id, parent_id, name, is_directory, size_bytes, mime_type, storage_path, is_public)
         VALUES ($1, $2, $3, $4, false, $5, $6, $7, false)
         RETURNING *",
    )
    .bind(new_id)
    .bind(user_id)
    .bind(input.parent_id)
    .bind(&asset.name)
    .bind(asset.size_bytes)
    .bind(&asset.mime_type)
    .bind(&new_path)
    .fetch_one(&state.pool)
    .await?;

    audit::log(&state.pool, &auth, "copy", "file", Some(&copy.id.to_string()), None).await;
    Ok((StatusCode::CREATED, Json(copy)))
}

async fn get_user_id(state: &AppState, auth: &AuthUser) -> Result<Uuid, ApiError> {
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE username = $1")
        .bind(&auth.username)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(row.0)
}

async fn get_user_role(state: &AppState, auth: &AuthUser) -> Result<String, ApiError> {
    let row: (String,) = sqlx::query_as("SELECT role FROM users WHERE username = $1")
        .bind(&auth.username)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(row.0)
}
