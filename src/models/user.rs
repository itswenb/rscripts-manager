use sqlx::FromRow;

#[derive(FromRow, Clone)]
#[allow(dead_code)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: String,
}

#[derive(FromRow, Clone)]
#[allow(dead_code)]
pub struct AuditLog {
    pub id: i64,
    pub user: String,
    pub action: String,
    pub target: String,
    pub detail: String,
    pub created_at: String,
}
