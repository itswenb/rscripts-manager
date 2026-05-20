mod models;
mod routes;
mod rparser;
mod slurm;

use axum::extract::DefaultBodyLimit;
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub data_dir: String,
    pub secret: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenv::dotenv().expect(".env file not found");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let port: u16 = std::env::var("PORT")
        .expect("PORT must be set in .env")
        .parse()
        .expect("PORT must be a valid number");
    let data_dir = expand_tilde(&std::env::var("DATA_DIR").expect("DATA_DIR must be set in .env"));
    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");
    std::fs::create_dir_all(format!("{data_dir}/scripts")).expect("Failed to create scripts dir");
    std::fs::create_dir_all(format!("{data_dir}/projects")).expect("Failed to create projects dir");
    std::fs::create_dir_all(format!("{data_dir}/data")).expect("Failed to create data dir");
    let secret = std::env::var("SECRET").expect("SECRET must be set in .env");

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
        data_dir,
        secret,
    };

    let app = Router::new()
        .merge(routes::router(state.clone()))
        .nest_service("/static", ServeDir::new("static"))
        .layer(DefaultBodyLimit::max(512 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind");

    tracing::info!("Ripeline running on http://0.0.0.0:{port}");
    axum::serve(listener, app).await.unwrap();
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
        let username = std::env::var("ADMIN_USER").expect("ADMIN_USER must be set in .env");
        let password = std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set in .env");
        let hash = hash_password(&password);
        sqlx::query("INSERT INTO admin (id, username, password_hash) VALUES (1, ?, ?)")
            .bind(&username)
            .bind(&hash)
            .execute(pool)
            .await
            .expect("Failed to init admin");
        tracing::info!("Admin account initialized: {username}");
    }
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
