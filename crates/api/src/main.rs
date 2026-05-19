use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod auth;
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
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:4001".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("API server listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
