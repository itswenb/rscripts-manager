mod models;
mod routes;
mod rparser;
mod runtime;
mod slurm;

use axum::extract::DefaultBodyLimit;
use axum::extract::Path;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{body::Body, Router};
use sqlx::sqlite::SqlitePoolOptions;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

include!(concat!(env!("OUT_DIR"), "/static_assets.rs"));

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub data_dir: String,
    pub secret: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let _ = dotenv::dotenv();

    let port: u16 = env_or_default("PORT", "9000")
        .parse()
        .expect("PORT must be a valid number");
    let data_dir = expand_tilde(&env_or_default("DATA_DIR", "~/.ripeline"));
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| default_database_url(&data_dir));
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");
    std::fs::create_dir_all(format!("{data_dir}/scripts")).expect("Failed to create scripts dir");
    std::fs::create_dir_all(format!("{data_dir}/projects")).expect("Failed to create projects dir");
    std::fs::create_dir_all(format!("{data_dir}/data")).expect("Failed to create data dir");
    let secret = std::env::var("SECRET").unwrap_or_else(|_| {
        tracing::warn!("SECRET is not set; using an ephemeral session secret for this process");
        random_secret()
    });

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    init_admin(&pool).await;

    let state = AppState {
        pool,
        data_dir: data_dir.clone(),
        secret,
    };

    let app = Router::new()
        .merge(routes::router(state.clone()))
        .route("/static/{*path}", get(static_asset))
        .layer(DefaultBodyLimit::max(512 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind");

    print_startup_banner(port, &data_dir);
    tracing::info!("Ripeline listening on http://0.0.0.0:{port}");
    axum::serve(listener, app).await.unwrap();
}

fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn default_database_url(data_dir: &str) -> String {
    format!(
        "sqlite:{}/ripeline.db?mode=rwc",
        data_dir.trim_end_matches('/')
    )
}

fn print_startup_banner(port: u16, data_dir: &str) {
    let local_url = format!("http://localhost:{port}");
    let network_url = format!("http://0.0.0.0:{port}");
    let linked_local_url = terminal_link(&local_url, &local_url);

    println!();
    println!("  Ripeline is running");
    println!("  Local:   {linked_local_url}");
    println!("  Network: {network_url}");
    println!("  Data:    {data_dir}");
    println!();
}

fn terminal_link(label: &str, url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{label}\x1b]8;;\x1b\\")
}

fn expand_tilde(path: &str) -> String {
    if path == "~" {
        return std::env::var("HOME").unwrap_or_else(|_| path.to_string());
    }

    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest).to_string_lossy().to_string();
        }
    }

    path.to_string()
}

async fn init_admin(pool: &sqlx::SqlitePool) {
    let exists = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM admin")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    if exists == 0 {
        let username = env_or_default("ADMIN_USER", "admin");
        let password = env_or_default("ADMIN_PASSWORD", "admin");
        let hash = hash_password(&password);
        sqlx::query("INSERT INTO admin (id, username, password_hash) VALUES (1, ?, ?)")
            .bind(&username)
            .bind(&hash)
            .execute(pool)
            .await
            .expect("Failed to init admin");
        tracing::info!("Admin account initialized: {username}");
        if std::env::var("ADMIN_PASSWORD").is_err() {
            tracing::warn!(
                "ADMIN_PASSWORD is not set; initialized admin with the default password"
            );
        }
    }
}

async fn static_asset(Path(path): Path<String>) -> Response {
    let clean_path = path.trim_start_matches('/');
    if clean_path.is_empty() || clean_path.split('/').any(|segment| segment == "..") {
        return StatusCode::NOT_FOUND.into_response();
    }

    let Some((_, bytes)) = STATIC_ASSETS
        .iter()
        .find(|(asset_path, _)| *asset_path == clean_path)
    else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let mut response = Body::from(*bytes).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(content_type(clean_path)),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    response
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or_default() {
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "wasm" => "application/wasm",
        "ttf" => "font/ttf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "map" => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn random_secret() -> String {
    use argon2::password_hash::rand_core::{OsRng, RngCore};

    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn hash_password(password: &str) -> String {
    use argon2::password_hash::rand_core::OsRng;
    use argon2::password_hash::SaltString;
    use argon2::{Argon2, PasswordHasher};
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

pub async fn audit(pool: &sqlx::SqlitePool, user: &str, action: &str, target: &str, detail: &str) {
    sqlx::query("INSERT INTO audit_logs (user, action, target, detail) VALUES (?, ?, ?, ?)")
        .bind(user)
        .bind(action)
        .bind(target)
        .bind(detail)
        .execute(pool)
        .await
        .ok();
}
