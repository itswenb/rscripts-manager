use sqlx::Row;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAGIC: &[u8] = b"RIPBKP1\n";
const TAG_DIR: u8 = 1;
const TAG_FILE: u8 = 2;
const TAG_END: u8 = 255;
const REQUIRED_DIRS: [&str; 4] = ["scripts", "projects", "data", "singularity_images"];
const DB_FILE: &str = "ripeline.db";
const COMPRESSION_LEVEL: i32 = 19;

const MERGE_TABLES: [&str; 10] = [
    "admin",
    "users",
    "settings",
    "projects",
    "pipeline_nodes",
    "project_flows",
    "project_flow_steps",
    "flow_runs",
    "step_runs",
    "audit_logs",
];

pub struct BackupReport {
    pub archive_path: PathBuf,
    pub files: usize,
    pub bytes: u64,
}

pub fn default_archive_path(data_dir: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let name = format!("ripeline-backup-{timestamp}.rpbk");
    Path::new(data_dir)
        .parent()
        .map(|parent| parent.join(&name))
        .unwrap_or_else(|| PathBuf::from(name))
}

pub fn create_backup(data_dir: &str, archive_path: &Path) -> Result<BackupReport, String> {
    let root = Path::new(data_dir);
    validate_data_dir(root)?;

    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create backup output directory: {err}"))?;
    }
    reject_archive_inside_data_dir(root, archive_path)?;

    let file = File::create(archive_path)
        .map_err(|err| format!("Failed to create backup archive: {err}"))?;
    let mut writer = io::BufWriter::new(file);
    writer
        .write_all(MAGIC)
        .map_err(|err| format!("Failed to write backup header: {err}"))?;
    let mut encoder = zstd::stream::write::Encoder::new(writer, COMPRESSION_LEVEL)
        .map_err(|err| format!("Failed to initialize backup compression: {err}"))?;

    let mut entries = Vec::new();
    collect_entries(root, root, &mut entries)?;
    entries.sort();

    let mut files = 0;
    let mut bytes = 0;
    for relative in entries {
        let source = root.join(&relative);
        let metadata = fs::symlink_metadata(&source)
            .map_err(|err| format!("Failed to inspect {}: {err}", source.display()))?;
        if metadata.file_type().is_dir() {
            write_dir_entry(&mut encoder, &relative)?;
        } else if metadata.file_type().is_file() {
            write_file_entry(&mut encoder, &relative, &source, metadata.len())?;
            files += 1;
            bytes += metadata.len();
        } else {
            return Err(format!(
                "Cannot back up special filesystem entry: {}",
                source.display()
            ));
        }
    }

    encoder
        .write_all(&[TAG_END])
        .map_err(|err| format!("Failed to finish backup stream: {err}"))?;
    let mut writer = encoder
        .finish()
        .map_err(|err| format!("Failed to finish backup compression: {err}"))?;
    writer
        .flush()
        .map_err(|err| format!("Failed to flush backup archive: {err}"))?;

    Ok(BackupReport {
        archive_path: archive_path.to_path_buf(),
        files,
        bytes,
    })
}

pub fn restore_archive(archive_path: &Path, data_dir: &Path) -> Result<PathBuf, String> {
    if !archive_path.is_file() {
        return Err(format!(
            "Restore archive does not exist: {}",
            archive_path.display()
        ));
    }

    let temp_dir = create_restore_temp(data_dir)?;
    extract_archive(archive_path, &temp_dir).inspect_err(|_| {
        let _ = fs::remove_dir_all(&temp_dir);
    })?;
    validate_restored_data_dir(&temp_dir).inspect_err(|_| {
        let _ = fs::remove_dir_all(&temp_dir);
    })?;
    copy_restored_files(&temp_dir, data_dir).inspect_err(|_| {
        let _ = fs::remove_dir_all(&temp_dir);
    })?;

    Ok(temp_dir.join(DB_FILE))
}

pub fn remove_restore_temp(restored_db: &Path) -> Result<(), String> {
    let Some(parent) = restored_db.parent() else {
        return Ok(());
    };
    if parent
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with(".ripeline-restore-"))
    {
        fs::remove_dir_all(parent)
            .map_err(|err| format!("Failed to remove restore temp directory: {err}"))?;
    }
    Ok(())
}

pub async fn merge_database(pool: &sqlx::SqlitePool, restored_db: &Path) -> Result<u64, String> {
    if !restored_db.is_file() {
        return Err(format!(
            "Restored database is missing: {}",
            restored_db.display()
        ));
    }

    let mut conn = pool
        .acquire()
        .await
        .map_err(|err| format!("Failed to acquire database connection: {err}"))?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(|err| format!("Failed to start restore transaction: {err}"))?;

    let result = async {
        sqlx::query("ATTACH DATABASE ? AS backup")
            .bind(restored_db.to_string_lossy().as_ref())
            .execute(&mut *conn)
            .await
            .map_err(|err| format!("Failed to attach restored database: {err}"))?;

        let mut merged_rows = 0;
        for table in MERGE_TABLES {
            merged_rows += merge_table(&mut conn, table).await?;
        }

        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(|err| format!("Failed to commit restore transaction: {err}"))?;
        let _ = sqlx::query("DETACH DATABASE backup")
            .execute(&mut *conn)
            .await;
        Ok::<u64, String>(merged_rows)
    }
    .await;

    if result.is_err() {
        let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
    }

    result
}

fn validate_data_dir(root: &Path) -> Result<(), String> {
    if !root.is_dir() {
        return Err(format!("DATA_DIR does not exist: {}", root.display()));
    }
    validate_restored_data_dir(root)
}

fn reject_archive_inside_data_dir(root: &Path, archive_path: &Path) -> Result<(), String> {
    let data_dir = fs::canonicalize(root)
        .map_err(|err| format!("Failed to resolve DATA_DIR {}: {err}", root.display()))?;
    let output_parent = archive_path.parent().unwrap_or_else(|| Path::new("."));
    let output_parent = fs::canonicalize(output_parent).map_err(|err| {
        format!(
            "Failed to resolve backup output directory {}: {err}",
            output_parent.display()
        )
    })?;

    if output_parent.starts_with(&data_dir) {
        return Err("Backup output must be outside DATA_DIR".to_string());
    }
    Ok(())
}

fn validate_restored_data_dir(root: &Path) -> Result<(), String> {
    for dir in REQUIRED_DIRS {
        let path = root.join(dir);
        if !path.is_dir() {
            return Err(format!("Backup is missing required directory: {dir}"));
        }
    }

    let db = root.join(DB_FILE);
    if !db.is_file() {
        return Err(format!("Backup is missing required database: {DB_FILE}"));
    }

    Ok(())
}

fn collect_entries(root: &Path, dir: &Path, entries: &mut Vec<PathBuf>) -> Result<(), String> {
    for item in
        fs::read_dir(dir).map_err(|err| format!("Failed to read {}: {err}", dir.display()))?
    {
        let item = item.map_err(|err| format!("Failed to read directory entry: {err}"))?;
        let path = item.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|err| format!("Failed to build backup path: {err}"))?
            .to_path_buf();
        entries.push(relative);

        let metadata = fs::symlink_metadata(&path)
            .map_err(|err| format!("Failed to inspect {}: {err}", path.display()))?;
        if metadata.file_type().is_dir() {
            collect_entries(root, &path, entries)?;
        }
    }
    Ok(())
}

fn write_dir_entry(writer: &mut impl Write, relative: &Path) -> Result<(), String> {
    writer
        .write_all(&[TAG_DIR])
        .map_err(|err| format!("Failed to write directory entry: {err}"))?;
    write_relative_path(writer, relative)
}

fn write_file_entry(
    writer: &mut impl Write,
    relative: &Path,
    source: &Path,
    len: u64,
) -> Result<(), String> {
    writer
        .write_all(&[TAG_FILE])
        .map_err(|err| format!("Failed to write file entry: {err}"))?;
    write_relative_path(writer, relative)?;
    writer
        .write_all(&len.to_le_bytes())
        .map_err(|err| format!("Failed to write file length: {err}"))?;

    let mut file =
        File::open(source).map_err(|err| format!("Failed to open {}: {err}", source.display()))?;
    io::copy(&mut file, writer)
        .map_err(|err| format!("Failed to archive {}: {err}", source.display()))?;
    Ok(())
}

fn write_relative_path(writer: &mut impl Write, relative: &Path) -> Result<(), String> {
    let path = relative_path_to_archive(relative)?;
    let bytes = path.as_bytes();
    let len = u32::try_from(bytes.len()).map_err(|_| format!("Path is too long: {path}"))?;
    writer
        .write_all(&len.to_le_bytes())
        .map_err(|err| format!("Failed to write path length: {err}"))?;
    writer
        .write_all(bytes)
        .map_err(|err| format!("Failed to write path: {err}"))?;
    Ok(())
}

fn extract_archive(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let mut file =
        File::open(archive_path).map_err(|err| format!("Failed to open restore archive: {err}"))?;
    let mut magic = [0_u8; MAGIC.len()];
    file.read_exact(&mut magic)
        .map_err(|err| format!("Failed to read backup header: {err}"))?;
    if magic != MAGIC {
        return Err("Restore archive is not a Ripeline backup".to_string());
    }

    let mut decoder = zstd::stream::read::Decoder::new(file)
        .map_err(|err| format!("Failed to initialize backup decompression: {err}"))?;

    loop {
        let mut tag = [0_u8; 1];
        decoder
            .read_exact(&mut tag)
            .map_err(|err| format!("Failed to read backup entry: {err}"))?;
        match tag[0] {
            TAG_DIR => {
                let relative = read_relative_path(&mut decoder)?;
                fs::create_dir_all(destination.join(relative))
                    .map_err(|err| format!("Failed to restore directory: {err}"))?;
            }
            TAG_FILE => {
                let relative = read_relative_path(&mut decoder)?;
                let len = read_u64(&mut decoder)?;
                let target = destination.join(relative);
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|err| format!("Failed to create restore directory: {err}"))?;
                }
                let mut output = File::create(&target)
                    .map_err(|err| format!("Failed to restore {}: {err}", target.display()))?;
                let mut limited = (&mut decoder).take(len);
                io::copy(&mut limited, &mut output)
                    .map_err(|err| format!("Failed to write {}: {err}", target.display()))?;
                if limited.limit() != 0 {
                    return Err(format!(
                        "Backup ended early while restoring {}",
                        target.display()
                    ));
                }
            }
            TAG_END => return Ok(()),
            other => return Err(format!("Invalid backup entry tag: {other}")),
        }
    }
}

fn read_relative_path(reader: &mut impl Read) -> Result<PathBuf, String> {
    let len = read_u32(reader)?;
    let mut bytes = vec![0_u8; len as usize];
    reader
        .read_exact(&mut bytes)
        .map_err(|err| format!("Failed to read backup path: {err}"))?;
    let path = String::from_utf8(bytes).map_err(|_| "Backup path is not UTF-8".to_string())?;
    archive_path_to_relative(&path)
}

fn read_u32(reader: &mut impl Read) -> Result<u32, String> {
    let mut bytes = [0_u8; 4];
    reader
        .read_exact(&mut bytes)
        .map_err(|err| format!("Failed to read integer: {err}"))?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(reader: &mut impl Read) -> Result<u64, String> {
    let mut bytes = [0_u8; 8];
    reader
        .read_exact(&mut bytes)
        .map_err(|err| format!("Failed to read integer: {err}"))?;
    Ok(u64::from_le_bytes(bytes))
}

fn relative_path_to_archive(relative: &Path) -> Result<String, String> {
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().to_string()),
            Component::CurDir => {}
            _ => {
                return Err(format!(
                    "Backup path must be relative and safe: {}",
                    relative.display()
                ));
            }
        }
    }

    if parts.is_empty() {
        return Err("Backup path cannot be empty".to_string());
    }
    Ok(parts.join("/"))
}

fn archive_path_to_relative(path: &str) -> Result<PathBuf, String> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        return Err(format!("Unsafe backup path: {path}"));
    }

    let mut relative = PathBuf::new();
    for part in path.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return Err(format!("Unsafe backup path: {path}"));
        }
        relative.push(part);
    }
    Ok(relative)
}

fn create_restore_temp(data_dir: &Path) -> Result<PathBuf, String> {
    for attempt in 0..100 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let temp_dir = data_dir.join(format!(
            ".ripeline-restore-{}-{timestamp}-{attempt}",
            std::process::id()
        ));
        match fs::create_dir(&temp_dir) {
            Ok(()) => return Ok(temp_dir),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(format!(
                    "Failed to create restore temp directory {}: {err}",
                    temp_dir.display()
                ));
            }
        }
    }
    Err("Failed to create a unique restore temp directory".to_string())
}

fn copy_restored_files(source: &Path, target: &Path) -> Result<(), String> {
    for item in fs::read_dir(source)
        .map_err(|err| format!("Failed to read restored data directory: {err}"))?
    {
        let item = item.map_err(|err| format!("Failed to read restored entry: {err}"))?;
        let path = item.path();
        let name = item.file_name();
        if is_sqlite_db_file(&name) {
            continue;
        }
        copy_entry(&path, &target.join(name))?;
    }
    Ok(())
}

fn copy_entry(source: &Path, target: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(source).map_err(|err| {
        format!(
            "Failed to inspect restored entry {}: {err}",
            source.display()
        )
    })?;
    if metadata.file_type().is_dir() {
        fs::create_dir_all(target).map_err(|err| {
            format!(
                "Failed to create restore target {}: {err}",
                target.display()
            )
        })?;
        for item in fs::read_dir(source).map_err(|err| {
            format!(
                "Failed to read restored directory {}: {err}",
                source.display()
            )
        })? {
            let item = item.map_err(|err| format!("Failed to read restored entry: {err}"))?;
            copy_entry(&item.path(), &target.join(item.file_name()))?;
        }
    } else if metadata.file_type().is_file() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "Failed to create restore target directory {}: {err}",
                    parent.display()
                )
            })?;
        }
        fs::copy(source, target).map_err(|err| {
            format!(
                "Failed to restore file {} to {}: {err}",
                source.display(),
                target.display()
            )
        })?;
    } else {
        return Err(format!(
            "Cannot restore special filesystem entry: {}",
            source.display()
        ));
    }
    Ok(())
}

fn is_sqlite_db_file(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_str(),
        Some("ripeline.db" | "ripeline.db-wal" | "ripeline.db-shm")
    )
}

async fn merge_table(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>,
    table: &str,
) -> Result<u64, String> {
    let target_columns = table_columns(conn, "main", table).await?;
    let source_columns = table_columns(conn, "backup", table).await?;
    if target_columns.is_empty() || source_columns.is_empty() {
        return Ok(0);
    }

    let columns = target_columns
        .into_iter()
        .filter(|column| source_columns.iter().any(|source| source == column))
        .collect::<Vec<_>>();
    if columns.is_empty() {
        return Ok(0);
    }

    let column_list = columns
        .iter()
        .map(|column| quote_ident(column))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT OR IGNORE INTO {table} ({columns}) SELECT {columns} FROM backup.{table}",
        table = quote_ident(table),
        columns = column_list
    );

    let result = sqlx::query(&sql)
        .execute(&mut **conn)
        .await
        .map_err(|err| format!("Failed to merge table {table}: {err}"))?;
    Ok(result.rows_affected())
}

async fn table_columns(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>,
    schema: &str,
    table: &str,
) -> Result<Vec<String>, String> {
    let sql = format!(
        "PRAGMA {schema}.table_info({table})",
        schema = quote_ident(schema),
        table = quote_string_literal(table)
    );
    let rows = sqlx::query(&sql)
        .fetch_all(&mut **conn)
        .await
        .map_err(|err| format!("Failed to inspect table {table}: {err}"))?;
    Ok(rows
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect())
}

fn quote_ident(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn quote_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
