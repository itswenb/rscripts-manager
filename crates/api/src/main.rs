use axum::{middleware as axum_mw, routing::get, Router};
use casbin::{CoreApi, Enforcer, MgmtApi};
use sqlx_adapter::SqlxAdapter;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod auth;
mod error;
mod middleware;
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

    let adapter = SqlxAdapter::new(&database_url, 8).await.unwrap();
    let enforcer = Enforcer::new("config/casbin_model.conf", adapter).await.unwrap();
    let enforcer = Arc::new(RwLock::new(enforcer));

    {
        let mut e = enforcer.write().await;
        let policies = e.get_policy();
        if policies.is_empty() {
            for act in ["GET", "POST", "PATCH", "DELETE"] {
                e.add_policy(vec![
                    "admin".into(), "/api/*".into(), act.into(),
                ]).await.unwrap();
            }
            tracing::info!("Seeded default admin policies");
        }
    }

    let state = AppState { pool, enforcer };

    let protected = Router::new()
        .route(
            "/api/projects",
            get(routes::projects::list).post(routes::projects::create),
        )
        .route(
            "/api/projects/{id}",
            get(routes::projects::get)
                .patch(routes::projects::update)
                .delete(routes::projects::delete),
        )
        .route_layer(axum_mw::from_fn_with_state(state.clone(), middleware::casbin_auth));

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(protected)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:4001".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("API server listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
