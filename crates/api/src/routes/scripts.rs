use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use rflow_core::models::FileAsset;
use rflow_rrunner::{parse_script, ScriptMeta};

#[derive(Serialize)]
pub struct ScriptInfo {
    pub id: Uuid,
    pub name: String,
    pub storage_path: String,
    pub meta: ScriptMeta,
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ScriptInfo>>, ApiError> {
    let user_id = get_user_id(&state, &auth).await?;

    // Find the "scripts" directory for this user
    let scripts_dir = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE owner_id = $1 AND parent_id IS NULL AND name = 'scripts' AND is_directory = true AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?;

    let Some(scripts_dir) = scripts_dir else {
        return Ok(Json(vec![]));
    };

    // Get all .R files in the scripts directory
    let files = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE owner_id = $1 AND parent_id = $2 AND is_directory = false AND deleted_at IS NULL AND name LIKE '%.R' ORDER BY name",
    )
    .bind(user_id)
    .bind(scripts_dir.id)
    .fetch_all(&state.pool)
    .await?;

    let mut scripts = Vec::new();
    for file in files {
        let content = match tokio::fs::read_to_string(&file.storage_path).await {
            Ok(c) => c,
            Err(_) => continue,
        };
        let meta = parse_script(&content);
        scripts.push(ScriptInfo {
            id: file.id,
            name: file.name,
            storage_path: file.storage_path,
            meta,
        });
    }

    Ok(Json(scripts))
}

pub async fn get(
    State(state): State<AppState>,
    Path(asset_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<ScriptInfo>, ApiError> {
    let user_id = get_user_id(&state, &auth).await?;

    let file = sqlx::query_as::<_, FileAsset>(
        "SELECT * FROM file_assets WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL",
    )
    .bind(asset_id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(rflow_core::AppError::NotFound)?;

    let content = tokio::fs::read_to_string(&file.storage_path).await
        .map_err(|_| rflow_core::AppError::NotFound)?;
    let meta = parse_script(&content);

    Ok(Json(ScriptInfo {
        id: file.id,
        name: file.name,
        storage_path: file.storage_path,
        meta,
    }))
}

async fn get_user_id(state: &AppState, auth: &AuthUser) -> Result<Uuid, ApiError> {
    let row = sqlx::query_as::<_, (Uuid,)>(
        "SELECT id FROM users WHERE username = $1",
    )
    .bind(&auth.username)
    .fetch_one(&state.pool)
    .await?;
    Ok(row.0)
}
