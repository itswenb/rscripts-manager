use super::auth::is_authenticated;
use crate::AppState;
use askama::Template;
use axum::body::Body;
use axum::extract::{Multipart, Query, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub icon: &'static str,
    pub description: &'static str,
    pub is_special: bool,
}

pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub file_type: String,
    pub size: String,
    pub modified: String,
    pub icon: &'static str,
    pub description: &'static str,
    pub is_special: bool,
}

pub struct Breadcrumb {
    pub name: String,
    pub path: String,
}

#[derive(Template)]
#[template(path = "files.html")]
struct FilesTemplate {
    active_nav: &'static str,
    current_path: String,
    full_current_path: String,
    directories: Vec<DirEntry>,
    breadcrumbs: Vec<Breadcrumb>,
    entries: Vec<FileEntry>,
}

#[derive(Template)]
#[template(path = "fragments/file_table.html")]
struct FileTableFragment {
    current_path: String,
    full_current_path: String,
    breadcrumbs: Vec<Breadcrumb>,
    entries: Vec<FileEntry>,
}

#[derive(Template)]
#[template(path = "fragments/data_file_picker.html")]
struct DataFilePickerFragment {
    current_path: String,
    data_relative_prefix: String,
    breadcrumbs: Vec<Breadcrumb>,
    entries: Vec<FileEntry>,
}

fn full_current_path(base: &std::path::Path, rel: &str) -> String {
    if rel.is_empty() {
        base.to_string_lossy().into_owned()
    } else {
        base.join(rel).to_string_lossy().into_owned()
    }
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers.contains_key("hx-request")
}

fn is_nav_request(headers: &HeaderMap) -> bool {
    headers.get("hx-target").map(|v| v.as_bytes()) == Some(b"main-content")
}

#[derive(Deserialize)]
pub struct FileQuery {
    pub path: Option<String>,
}

#[derive(Serialize)]
pub struct FileListResponse {
    pub files: Vec<String>,
}

async fn build_file_data(
    base: &std::path::Path,
    rel: &str,
) -> (Vec<DirEntry>, Vec<Breadcrumb>, Vec<FileEntry>) {
    let dir = base.join(rel);

    let mut directories = Vec::new();
    if let Ok(mut rd) = tokio::fs::read_dir(base).await {
        while let Ok(Some(entry)) = rd.next_entry().await {
            if entry.metadata().await.map(|m| m.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().into_owned();
                let meta = special_dir_meta(&name);
                directories.push(DirEntry {
                    path: name.clone(),
                    name,
                    icon: meta.icon,
                    description: meta.description,
                    is_special: meta.is_special,
                });
            }
        }
    }
    directories.sort_by(|a, b| a.name.cmp(&b.name));

    let mut breadcrumbs = vec![Breadcrumb {
        name: "home".into(),
        path: "".into(),
    }];
    let mut acc = String::new();
    for part in rel.split('/').filter(|s| !s.is_empty()) {
        if !acc.is_empty() {
            acc.push('/');
        }
        acc.push_str(part);
        breadcrumbs.push(Breadcrumb {
            name: part.to_string(),
            path: acc.clone(),
        });
    }

    let mut entries = Vec::new();
    if let Ok(mut rd) = tokio::fs::read_dir(&dir).await {
        while let Ok(Some(entry)) = rd.next_entry().await {
            let name = entry.file_name().to_string_lossy().into_owned();
            let meta = entry.metadata().await.ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = meta
                .as_ref()
                .map(|m| format_size(m.len()))
                .unwrap_or_default();
            let modified = meta
                .as_ref()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let dt: chrono::DateTime<chrono::Local> = t.into();
                    dt.format("%m-%d %H:%M").to_string()
                })
                .unwrap_or_default();
            let file_type = if is_dir {
                "目录".into()
            } else {
                ext_type(&name)
            };
            let special = if rel.is_empty() && is_dir {
                special_dir_meta(&name)
            } else {
                FileMeta::default()
            };
            let icon = if is_dir { special.icon } else { "file-text" };
            entries.push(FileEntry {
                name,
                is_dir,
                file_type,
                size,
                modified,
                icon,
                description: special.description,
                is_special: special.is_special,
            });
        }
    }
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));

    (directories, breadcrumbs, entries)
}

#[derive(Clone, Copy)]
struct FileMeta {
    icon: &'static str,
    description: &'static str,
    is_special: bool,
}

impl Default for FileMeta {
    fn default() -> Self {
        Self {
            icon: "folder",
            description: "",
            is_special: false,
        }
    }
}

fn special_dir_meta(name: &str) -> FileMeta {
    match name {
        "scripts" => FileMeta {
            icon: "file-code-2",
            description: "存放 R 脚本节点源文件",
            is_special: true,
        },
        "projects" => FileMeta {
            icon: "folder-kanban",
            description: "保存项目运行空间和输出结果",
            is_special: true,
        },
        "data" => FileMeta {
            icon: "database",
            description: "存放流程输入数据文件",
            is_special: true,
        },
        "singularity_images" => FileMeta {
            icon: "box",
            description: "存放 Slurm 运行使用的 SIF 镜像",
            is_special: true,
        },
        _ => FileMeta::default(),
    }
}

fn is_selectable_data_file(name: &str) -> bool {
    matches!(
        name.rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "csv" | "tsv" | "txt" | "xls" | "xlsx" | "rds" | "rda" | "rdata"
    )
}

fn filter_picker_entries(entries: Vec<FileEntry>) -> Vec<FileEntry> {
    entries
        .into_iter()
        .filter(|entry| entry.is_dir || is_selectable_data_file(&entry.name))
        .collect()
}

fn data_relative_prefix(path: &str) -> String {
    path.strip_prefix("data/")
        .or_else(|| path.strip_prefix("data"))
        .unwrap_or(path)
        .trim_matches('/')
        .to_string()
}

pub async fn list(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Query(q): Query<FileQuery>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let base = std::path::PathBuf::from(&state.data_dir);
    let rel = q.path.clone().unwrap_or_default();
    let dir = base.join(&rel);
    if !dir.starts_with(&base) {
        return Err(Redirect::to("/files"));
    }

    let (directories, breadcrumbs, entries) = build_file_data(&base, &rel).await;
    let full_current_path = full_current_path(&base, &rel);

    if is_htmx(&headers) && !is_nav_request(&headers) {
        let tmpl = FileTableFragment {
            current_path: rel,
            full_current_path,
            breadcrumbs,
            entries,
        };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    } else {
        let tmpl = FilesTemplate {
            active_nav: "files",
            current_path: rel,
            full_current_path,
            directories,
            breadcrumbs,
            entries,
        };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    }
}

pub async fn upload(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let base = std::path::PathBuf::from(&state.data_dir);
    let mut upload_path = String::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("path") {
            upload_path = field.text().await.unwrap_or_default();
        } else if field.name() == Some("file") {
            if let Some(filename) = field.file_name().map(|s| s.to_string()) {
                let data = field.bytes().await.unwrap_or_default();
                let target = base.join(&upload_path).join(&filename);
                if target.starts_with(&base) {
                    if let Some(parent) = target.parent() {
                        tokio::fs::create_dir_all(parent).await.ok();
                    }
                    tokio::fs::write(target, data).await.ok();
                }
            }
        }
    }

    if is_htmx(&headers) {
        let (_, breadcrumbs, entries) = build_file_data(&base, &upload_path).await;
        let full_path = full_current_path(&base, &upload_path);
        let tmpl = FileTableFragment {
            current_path: upload_path,
            full_current_path: full_path,
            breadcrumbs,
            entries,
        };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    } else {
        Ok(Redirect::to(&format!("/files?path={upload_path}")).into_response())
    }
}

async fn list_files_recursive(
    base: &std::path::Path,
    rel: &str,
) -> Result<Vec<String>, std::io::Error> {
    let root = base.join(rel);
    let mut pending = vec![(root, rel.trim_matches('/').to_string())];
    let mut files = Vec::new();

    while let Some((dir, prefix)) = pending.pop() {
        let mut read_dir = tokio::fs::read_dir(&dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let name = entry.file_name().to_string_lossy().into_owned();
            let child_rel = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            let path = entry.path();
            let metadata = entry.metadata().await?;
            if metadata.is_dir() {
                pending.push((path, child_rel));
            } else if metadata.is_file() {
                files.push(child_rel);
            }
        }
    }

    files.sort();
    Ok(files)
}

pub async fn list_json(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(q): Query<FileQuery>,
) -> Result<Json<FileListResponse>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }

    let base = std::path::PathBuf::from(&state.data_dir);
    let rel = q.path.clone().unwrap_or_default();
    let dir = base.join(&rel);
    if !dir.starts_with(&base) || !dir.is_dir() {
        return Err(Redirect::to("/files"));
    }

    let files = list_files_recursive(&base, &rel).await.unwrap_or_default();
    Ok(Json(FileListResponse { files }))
}

pub async fn data_picker(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(q): Query<FileQuery>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }

    let base = std::path::PathBuf::from(&state.data_dir);
    let rel = q.path.clone().unwrap_or_else(|| "data".to_string());
    let dir = base.join(&rel);
    if !dir.starts_with(&base) || !dir.is_dir() {
        return Err(Redirect::to("/files"));
    }

    let (_, breadcrumbs, entries) = build_file_data(&base, &rel).await;
    let tmpl = DataFilePickerFragment {
        current_path: rel,
        data_relative_prefix: data_relative_prefix(
            &q.path.clone().unwrap_or_else(|| "data".to_string()),
        ),
        breadcrumbs,
        entries: filter_picker_entries(entries),
    };
    Ok(Html(tmpl.render().unwrap_or_default()).into_response())
}

#[derive(Serialize)]
pub struct ChunkResponse {
    pub ok: bool,
}

pub async fn upload_chunk(
    State(state): State<AppState>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Result<Json<ChunkResponse>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let base = std::path::PathBuf::from(&state.data_dir);
    let mut upload_path = String::new();
    let mut filename = String::new();
    let mut offset: u64 = 0;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("path") => upload_path = field.text().await.unwrap_or_default(),
            Some("filename") => filename = field.text().await.unwrap_or_default(),
            Some("offset") => offset = field.text().await.unwrap_or_default().parse().unwrap_or(0),
            Some("chunk") => {
                let data = field.bytes().await.unwrap_or_default();
                let target = base.join(&upload_path).join(&filename);
                if target.starts_with(&base) && !filename.is_empty() {
                    if let Some(parent) = target.parent() {
                        tokio::fs::create_dir_all(parent).await.ok();
                    }
                    let mut file = tokio::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&target)
                        .await
                        .map_err(|_| Redirect::to("/files"))?;
                    use tokio::io::AsyncSeekExt;
                    file.seek(std::io::SeekFrom::Start(offset)).await.ok();
                    file.write_all(&data).await.ok();
                }
            }
            _ => {}
        }
    }
    Ok(Json(ChunkResponse { ok: true }))
}

#[derive(Deserialize)]
pub struct MkdirForm {
    pub path: String,
    pub name: String,
}

pub async fn mkdir(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Form(form): Form<MkdirForm>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let base = std::path::PathBuf::from(&state.data_dir);
    let target = base.join(&form.path).join(&form.name);
    if target.starts_with(&base) {
        tokio::fs::create_dir_all(target).await.ok();
    }

    if is_htmx(&headers) {
        let (_, breadcrumbs, entries) = build_file_data(&base, &form.path).await;
        let full_path = full_current_path(&base, &form.path);
        let tmpl = FileTableFragment {
            current_path: form.path,
            full_current_path: full_path,
            breadcrumbs,
            entries,
        };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    } else {
        Ok(Redirect::to(&format!("/files?path={}", form.path)).into_response())
    }
}

#[derive(Deserialize)]
pub struct DeleteForm {
    pub path: String,
}

pub async fn delete(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Form(form): Form<DeleteForm>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let base = std::path::PathBuf::from(&state.data_dir);
    let target = base.join(&form.path);
    if target.starts_with(&base) && target != base {
        if target.is_dir() {
            tokio::fs::remove_dir_all(&target).await.ok();
        } else {
            tokio::fs::remove_file(&target).await.ok();
        }
    }
    let parent = std::path::Path::new(&form.path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    if is_htmx(&headers) {
        let (_, breadcrumbs, entries) = build_file_data(&base, &parent).await;
        let full_path = full_current_path(&base, &parent);
        let tmpl = FileTableFragment {
            current_path: parent,
            full_current_path: full_path,
            breadcrumbs,
            entries,
        };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    } else {
        Ok(Redirect::to(&format!("/files?path={parent}")).into_response())
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}

fn ext_type(name: &str) -> String {
    match name.rsplit('.').next().unwrap_or("") {
        "R" | "r" => "R Script".into(),
        "csv" => "CSV".into(),
        "tsv" | "txt" => "Text".into(),
        "png" | "jpg" | "jpeg" | "gif" => "Image".into(),
        "pdf" => "PDF".into(),
        "rds" | "rda" | "RData" => "R Data".into(),
        _ => "File".into(),
    }
}

#[derive(Deserialize)]
pub struct RenameForm {
    pub path: String,
    pub new_name: String,
}

pub async fn rename(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Form(form): Form<RenameForm>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let base = std::path::PathBuf::from(&state.data_dir);
    let source = base.join(&form.path);
    let parent_rel = std::path::Path::new(&form.path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    if let Some(parent) = source.parent() {
        let dest = parent.join(&form.new_name);
        if source.starts_with(&base) && dest.starts_with(&base) {
            tokio::fs::rename(source, dest).await.ok();
        }
    }

    if is_htmx(&headers) {
        let (_, breadcrumbs, entries) = build_file_data(&base, &parent_rel).await;
        let full_path = full_current_path(&base, &parent_rel);
        let tmpl = FileTableFragment {
            current_path: parent_rel,
            full_current_path: full_path,
            breadcrumbs,
            entries,
        };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    } else {
        Ok(Redirect::to("/files").into_response())
    }
}

#[derive(Serialize)]
pub struct FileContent {
    pub content: String,
    pub editable: bool,
    pub image: bool,
    pub pdf: bool,
    pub filename: String,
}

pub async fn read_file(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(q): Query<FileQuery>,
) -> Result<Json<FileContent>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let base = std::path::PathBuf::from(&state.data_dir);
    let rel = q.path.clone().unwrap_or_default();
    let target = base.join(&rel);
    if !target.starts_with(&base) || !target.is_file() {
        return Err(Redirect::to("/files"));
    }

    let filename = target
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let image = matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp"
    );
    let pdf = ext == "pdf";
    let editable = matches!(
        ext.as_str(),
        "r" | "rmd"
            | "txt"
            | "csv"
            | "tsv"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "ini"
            | "cfg"
            | "conf"
            | "sh"
            | "bash"
            | "py"
            | "md"
            | "xml"
            | "html"
            | "css"
            | "js"
            | "ts"
            | "sql"
            | "log"
            | "env"
            | "gitignore"
    );

    let content = if image {
        format!("/files/download?path={}", rel)
    } else if editable {
        tokio::fs::read_to_string(&target)
            .await
            .unwrap_or_else(|_| "(无法读取)".into())
    } else {
        String::new()
    };

    Ok(Json(FileContent {
        content,
        editable,
        image,
        pdf,
        filename,
    }))
}

#[derive(Deserialize)]
pub struct SaveForm {
    pub path: String,
    pub content: String,
}

pub async fn save_file(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(form): Json<SaveForm>,
) -> Result<Json<ChunkResponse>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let base = std::path::PathBuf::from(&state.data_dir);
    let target = base.join(&form.path);
    if !target.starts_with(&base) {
        return Err(Redirect::to("/files"));
    }
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    tokio::fs::write(&target, form.content.as_bytes())
        .await
        .ok();
    Ok(Json(ChunkResponse { ok: true }))
}

pub async fn download(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(q): Query<FileQuery>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let base = std::path::PathBuf::from(&state.data_dir);
    let rel = q.path.clone().unwrap_or_default();
    let target = base.join(&rel);
    if !target.starts_with(&base) || !target.is_file() {
        return Err(Redirect::to("/files"));
    }

    let filename = target
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    };

    let data = tokio::fs::read(&target).await.unwrap_or_default();
    Ok(Response::builder()
        .header("content-type", mime)
        .header(
            "content-disposition",
            format!("inline; filename=\"{filename}\""),
        )
        .body(Body::from(data))
        .unwrap())
}
