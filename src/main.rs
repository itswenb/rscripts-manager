mod backup;
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
use std::process::ExitCode;
use tracing_subscriber::EnvFilter;

include!(concat!(env!("OUT_DIR"), "/static_assets.rs"));

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub data_dir: String,
    pub secret: String,
}

#[derive(Debug, PartialEq, Eq)]
enum CliCommand {
    Run,
    Reset { password: Option<String> },
    Backup { output: Option<PathBuf> },
    Restore { archive: PathBuf },
    Help,
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let _ = dotenv::dotenv();

    let command = match parse_cli(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("Error: {message}");
            eprintln!();
            print_help();
            return ExitCode::FAILURE;
        }
    };

    match runner(command).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Error: {message}");
            ExitCode::FAILURE
        }
    }
}

async fn runner(command: CliCommand) -> Result<(), String> {
    match command {
        CliCommand::Run => run_server().await,
        CliCommand::Reset { password } => reset_admin_password(password).await,
        CliCommand::Backup { output } => backup_data_dir(output).await,
        CliCommand::Restore { archive } => restore_data_dir(archive).await,
        CliCommand::Help => {
            print_help();
            Ok(())
        }
    }
}

async fn run_server() -> Result<(), String> {
    let port: u16 = env_or_default("PORT", "9000")
        .parse()
        .map_err(|_| "PORT must be a valid number".to_string())?;
    let data_dir = prepare_data_dir()?;
    let secret = std::env::var("SECRET").unwrap_or_else(|_| {
        tracing::warn!("SECRET is not set; using an ephemeral session secret for this process");
        random_secret()
    });

    let pool = connect_database(&data_dir).await?;

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
        .map_err(|err| format!("Failed to bind 0.0.0.0:{port}: {err}"))?;

    print_startup_banner(port, &data_dir);
    tracing::info!("Ripeline listening on http://0.0.0.0:{port}");
    axum::serve(listener, app)
        .await
        .map_err(|err| format!("Server stopped with an error: {err}"))
}

async fn reset_admin_password(password: Option<String>) -> Result<(), String> {
    let data_dir = prepare_data_dir()?;
    let pool = connect_database(&data_dir).await?;
    let username = env_or_default("ADMIN_USER", "admin");
    let password = password.unwrap_or_else(|| env_or_default("ADMIN_PASSWORD", "admin"));
    let hash = hash_password(&password);

    let result = sqlx::query(
        "INSERT INTO admin (id, username, password_hash) VALUES (1, ?, ?)
         ON CONFLICT(id) DO UPDATE SET password_hash = excluded.password_hash",
    )
    .bind(&username)
    .bind(&hash)
    .execute(&pool)
    .await
    .map_err(|err| format!("Failed to reset admin password: {err}"))?;

    if result.rows_affected() == 0 {
        return Err("Admin password was not changed".to_string());
    }

    let username = sqlx::query_scalar::<_, String>("SELECT username FROM admin WHERE id = 1")
        .fetch_one(&pool)
        .await
        .map_err(|err| format!("Failed to load admin username after reset: {err}"))?;

    println!("Admin password reset for user: {username}");
    Ok(())
}

fn parse_cli(args: impl IntoIterator<Item = String>) -> Result<CliCommand, String> {
    let args = args.into_iter().collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        None | Some("help" | "--help" | "-h") => Ok(CliCommand::Help),
        Some("run") => {
            if args.len() == 1 {
                Ok(CliCommand::Run)
            } else {
                Err(format!("unexpected argument for run: {}", args[1]))
            }
        }
        Some("reset") => parse_reset_args(&args[1..]),
        Some("backup") => parse_backup_args(&args[1..]),
        Some("restore") => parse_restore_args(&args[1..]),
        Some(command) => Err(format!("unknown command: {command}")),
    }
}

fn parse_reset_args(args: &[String]) -> Result<CliCommand, String> {
    let mut password = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--password" | "-p" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "reset requires a value after --password".to_string())?;
                if value.is_empty() {
                    return Err("reset password cannot be empty".to_string());
                }
                password = Some(value.clone());
                index += 2;
            }
            arg => return Err(format!("unexpected argument for reset: {arg}")),
        }
    }

    Ok(CliCommand::Reset { password })
}

fn parse_backup_args(args: &[String]) -> Result<CliCommand, String> {
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--output" | "-o" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "backup requires a value after --output".to_string())?;
                if value.is_empty() {
                    return Err("backup output path cannot be empty".to_string());
                }
                output = Some(PathBuf::from(expand_tilde(value)));
                index += 2;
            }
            arg => return Err(format!("unexpected argument for backup: {arg}")),
        }
    }

    Ok(CliCommand::Backup { output })
}

fn parse_restore_args(args: &[String]) -> Result<CliCommand, String> {
    match args {
        [archive] if !archive.is_empty() => Ok(CliCommand::Restore {
            archive: PathBuf::from(expand_tilde(archive)),
        }),
        [] => Err("restore requires an archive path".to_string()),
        [..] => Err("restore accepts exactly one archive path".to_string()),
    }
}

fn print_help() {
    println!(
        r#"Ripeline

Usage:
  ripeline run
  ripeline reset [--password <password>]
  ripeline backup [--output <archive.rpbk>]
  ripeline restore <archive.rpbk>
  ripeline help

Commands:
  run      Start the Ripeline web server.
  reset    Reset the administrator account password.
  backup   Compress the current DATA_DIR into a backup archive.
  restore  Restore an archive into DATA_DIR and merge SQLite rows.
  help     Show this guide.

Environment:
  PORT             Server port for run. Default: 9000
  DATA_DIR         Data directory. Default: ~/.ripeline
  DATABASE_URL     SQLite URL. Default: sqlite:$DATA_DIR/ripeline.db?mode=rwc
  SECRET           Session signing secret.
  ADMIN_USER       Admin username. Default: admin
  ADMIN_PASSWORD   Initial or reset password when --password is omitted. Default: admin
"#
    );
}

async fn backup_data_dir(output: Option<PathBuf>) -> Result<(), String> {
    let data_dir = current_data_dir();
    let archive_path = output.unwrap_or_else(|| backup::default_archive_path(&data_dir));
    let report = backup::create_backup(&data_dir, &archive_path)?;

    println!(
        "Backup created: {} ({} files, {} bytes)",
        report.archive_path.display(),
        report.files,
        report.bytes
    );
    Ok(())
}

async fn restore_data_dir(archive: PathBuf) -> Result<(), String> {
    let data_dir = prepare_data_dir()?;
    let restored_db = backup::restore_archive(&archive, std::path::Path::new(&data_dir))?;
    let pool = connect_database(&data_dir).await?;
    let merged_rows = match backup::merge_database(&pool, &restored_db).await {
        Ok(rows) => rows,
        Err(err) => {
            let _ = backup::remove_restore_temp(&restored_db);
            return Err(err);
        }
    };
    backup::remove_restore_temp(&restored_db)?;

    println!(
        "Restore completed into {data_dir}; merged {merged_rows} database rows from {}",
        archive.display()
    );
    Ok(())
}

fn prepare_data_dir() -> Result<String, String> {
    let data_dir = current_data_dir();
    std::fs::create_dir_all(&data_dir)
        .map_err(|err| format!("Failed to create data dir: {err}"))?;
    std::fs::create_dir_all(format!("{data_dir}/scripts"))
        .map_err(|err| format!("Failed to create scripts dir: {err}"))?;
    std::fs::create_dir_all(format!("{data_dir}/projects"))
        .map_err(|err| format!("Failed to create projects dir: {err}"))?;
    std::fs::create_dir_all(format!("{data_dir}/data"))
        .map_err(|err| format!("Failed to create data dir: {err}"))?;
    std::fs::create_dir_all(format!("{data_dir}/singularity_images"))
        .map_err(|err| format!("Failed to create singularity images dir: {err}"))?;
    Ok(data_dir)
}

fn current_data_dir() -> String {
    expand_tilde(&env_or_default("DATA_DIR", "~/.ripeline"))
}

async fn connect_database(data_dir: &str) -> Result<sqlx::SqlitePool, String> {
    let db_url = std::env::var("DATABASE_URL")
        .map(|url| normalize_database_url(&url))
        .unwrap_or_else(|_| default_database_url(data_dir));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|err| format!("Failed to connect to database: {err}"))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|err| format!("Failed to run migrations: {err}"))?;

    Ok(pool)
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

fn normalize_database_url(url: &str) -> String {
    let Some(path) = url.strip_prefix("sqlite:") else {
        return url.to_string();
    };

    let (path_part, query_part) = path.split_once('?').unwrap_or((path, ""));
    let expanded_path = expand_tilde(path_part);
    if query_part.is_empty() {
        format!("sqlite:{expanded_path}")
    } else {
        format!("sqlite:{expanded_path}?{query_part}")
    }
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

pub fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
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

#[cfg(test)]
mod tests {
    use super::{parse_cli, CliCommand};
    use std::path::PathBuf;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn no_args_show_help() {
        assert_eq!(parse_cli(args(&[])), Ok(CliCommand::Help));
    }

    #[test]
    fn run_command_starts_server() {
        assert_eq!(parse_cli(args(&["run"])), Ok(CliCommand::Run));
    }

    #[test]
    fn reset_accepts_password_argument() {
        assert_eq!(
            parse_cli(args(&["reset", "--password", "secret"])),
            Ok(CliCommand::Reset {
                password: Some("secret".to_string())
            })
        );
    }

    #[test]
    fn backup_accepts_output_argument() {
        assert_eq!(
            parse_cli(args(&["backup", "--output", "backup.rpbk"])),
            Ok(CliCommand::Backup {
                output: Some(PathBuf::from("backup.rpbk"))
            })
        );
    }

    #[test]
    fn restore_accepts_archive_argument() {
        assert_eq!(
            parse_cli(args(&["restore", "backup.rpbk"])),
            Ok(CliCommand::Restore {
                archive: PathBuf::from("backup.rpbk")
            })
        );
    }

    #[test]
    fn unknown_command_is_rejected() {
        assert!(parse_cli(args(&["serve"])).is_err());
    }
}
