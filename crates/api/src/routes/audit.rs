use axum::extract::{Query, State};
use axum::Json;
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use rflow_core::models::{AuditLog, AuditLogQuery};

pub async fn log(
    pool: &PgPool,
    user: &AuthUser,
    action: &str,
    resource_type: &str,
    resource_id: Option<&str>,
    details: Option<serde_json::Value>,
) {
    let _ = sqlx::query(
        "INSERT INTO audit_logs (user_id, username, action, resource_type, resource_id, details) \
         VALUES ((SELECT id FROM users WHERE username = $1), $1, $2, $3, $4, $5)",
    )
    .bind(&user.username)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(details)
    .execute(pool)
    .await;
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<AuditLogQuery>,
) -> Result<Json<Vec<AuditLog>>, ApiError> {
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);

    let logs = sqlx::query_as::<_, AuditLog>(
        "SELECT id, user_id, username, action, resource_type, resource_id, details, created_at \
         FROM audit_logs \
         WHERE ($3::TEXT IS NULL OR username = $3) \
         AND ($4::TEXT IS NULL OR action = $4) \
         AND ($5::TEXT IS NULL OR resource_type = $5) \
         ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .bind(&params.username)
    .bind(&params.action)
    .bind(&params.resource_type)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(logs))
}
