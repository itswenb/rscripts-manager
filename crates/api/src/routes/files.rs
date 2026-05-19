use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use rflow_core::models::{CreateDirectory, FileAsset, MoveAsset};
use rflow_core::AppError;

#[derive(Debug, serde::Deserialize)]
pub struct UploadParams {
    pub parent_id: Option<Uuid>,
}

pub async fn upload(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(params): Query<UploadParams>,
    _auth: AuthUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Vec<FileAsset>>), ApiError> {
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".into());
    let upload_dir = format!("{data_dir}/projects/{project_id}/uploads");
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
    tokio::fs::create_dir_all(&dir_path)
        .await
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

    let file = tokio::fs::File::open(&asset.storage_path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let stream = ReaderStream::new(file);

    let content_type = asset
        .mime_type
        .unwrap_or_else(|| "application/octet-stream".into());

    Ok(Response::builder()
        .header("content-type", content_type)
        .header(
            "content-disposition",
            format!("attachment; filename=\"{}\"", asset.name),
        )
        .body(Body::from_stream(stream))
        .unwrap())
}

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

    let content = tokio::fs::read_to_string(&asset.storage_path)
        .await
        .map_err(|_| AppError::Validation("file is not text".into()))?;

    let preview: String = content.lines().take(100).collect::<Vec<_>>().join("\n");
    Ok(preview)
}
