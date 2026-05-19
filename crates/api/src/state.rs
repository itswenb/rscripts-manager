use casbin::Enforcer;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub enforcer: Arc<RwLock<Enforcer>>,
}
