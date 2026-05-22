use super::auth::is_authenticated;
use crate::models::{Project, ProjectFlow, RuntimeConfig};
use crate::AppState;
use askama::Template;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Form, Json};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "projects.html")]
struct ProjectsTemplate {
    active_nav: &'static str,
    projects: Vec<Project>,
}

#[derive(Template)]
#[template(path = "fragments/project_list.html")]
struct ProjectListFragment {
    projects: Vec<Project>,
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers.contains_key("hx-request")
}

fn is_nav_request(headers: &HeaderMap) -> bool {
    headers.get("hx-target").map(|v| v.as_bytes()) == Some(b"main-content")
}

pub async fn list(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let projects = sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY created_at DESC")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    if is_htmx(&headers) && !is_nav_request(&headers) {
        let tmpl = ProjectListFragment { projects };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    } else {
        let tmpl = ProjectsTemplate {
            active_nav: "projects",
            projects,
        };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    }
}

#[derive(Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Form(form): Form<CreateProject>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let project_dir = format!("{}/projects/{}", state.data_dir, form.name);
    tokio::fs::create_dir_all(&project_dir).await.ok();
    sqlx::query("INSERT INTO projects (id, name, description) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&form.name)
        .bind(form.description.as_deref().unwrap_or(""))
        .execute(&state.pool)
        .await
        .ok();

    if is_htmx(&headers) {
        let projects =
            sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY created_at DESC")
                .fetch_all(&state.pool)
                .await
                .unwrap_or_default();
        let tmpl = ProjectListFragment { projects };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    } else {
        Ok(Redirect::to("/projects").into_response())
    }
}

#[derive(Template)]
#[template(path = "project_detail.html")]
struct ProjectDetailTemplate {
    active_nav: &'static str,
    project: Project,
}

pub async fn detail(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);

    match project {
        Some(p) => {
            let tmpl = ProjectDetailTemplate {
                active_nav: "projects",
                project: p,
            };
            Ok(Html(tmpl.render().unwrap_or_default()).into_response())
        }
        None => Ok(Redirect::to("/projects").into_response()),
    }
}

pub async fn get_flow(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let flow = sqlx::query_as::<_, ProjectFlow>(
        "SELECT * FROM project_flows WHERE project_id = ? LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let graph_data = flow
        .map(|f| f.graph_data)
        .unwrap_or_else(|| "{}".to_string());
    let json: serde_json::Value =
        serde_json::from_str(&graph_data).unwrap_or(serde_json::json!({}));
    Ok(Json(json))
}

#[derive(Deserialize)]
pub struct SaveFlow {
    pub graph_data: serde_json::Value,
}

pub async fn save_flow(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
    Json(body): Json<SaveFlow>,
) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let graph_str = serde_json::to_string(&body.graph_data).unwrap_or_default();

    let existing = sqlx::query_scalar::<_, String>(
        "SELECT id FROM project_flows WHERE project_id = ? LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    if let Some(flow_id) = existing {
        sqlx::query("UPDATE project_flows SET graph_data = ? WHERE id = ?")
            .bind(&graph_str)
            .bind(&flow_id)
            .execute(&state.pool)
            .await
            .ok();
    } else {
        let flow_id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO project_flows (id, project_id, name, graph_data) VALUES (?, ?, 'default', ?)")
            .bind(&flow_id).bind(&id).bind(&graph_str)
            .execute(&state.pool).await.ok();
    }

    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Serialize)]
pub struct NodeOutput {
    files: Vec<String>,
    path: String,
}

pub async fn node_output(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((id, node_index)): Path<(String, String)>,
) -> Result<Json<NodeOutput>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);

    let Some(project) = project else {
        return Err(Redirect::to("/projects"));
    };
    let output_dir = format!(
        "{}/projects/{}/{}",
        state.data_dir, project.name, node_index
    );
    let mut files = Vec::new();
    if let Ok(mut rd) = tokio::fs::read_dir(&output_dir).await {
        while let Ok(Some(entry)) = rd.next_entry().await {
            files.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(Json(NodeOutput {
        files,
        path: output_dir,
    }))
}

pub async fn node_output_file(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((id, node_index, filename)): Path<(String, String, String)>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);
    let Some(project) = project else {
        return Err(Redirect::to("/projects"));
    };
    let file_path = format!(
        "{}/projects/{}/{}/{}",
        state.data_dir, project.name, node_index, filename
    );
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Ok((axum::http::StatusCode::NOT_FOUND, "not found").into_response());
    }
    let content_type = match path.extension().and_then(|e| e.to_str()) {
        Some("pdf") => "application/pdf",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };
    let bytes = tokio::fs::read(&file_path).await.unwrap_or_default();
    let mut headers = HeaderMap::new();
    headers.insert("content-type", content_type.parse().unwrap());
    Ok((headers, bytes).into_response())
}

pub async fn run_flow(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let flow = sqlx::query_as::<_, ProjectFlow>(
        "SELECT * FROM project_flows WHERE project_id = ? LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);
    let Some(flow) = flow else {
        return Ok(Json(serde_json::json!({"error": "no flow"})));
    };
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);
    let Some(project) = project else {
        return Err(Redirect::to("/projects"));
    };

    let graph: serde_json::Value = serde_json::from_str(&flow.graph_data).unwrap_or_default();
    if let Err(e) = preflight_graph(&graph, &state.data_dir, &project.name, None).await {
        return Ok(Json(serde_json::json!({"error": e})));
    }

    // 加载运行时配置（来自全局 settings）并做检测/校验
    let mut runtime = crate::routes::settings::load_runtime_config(&state.pool).await;
    if let Err(e) = crate::slurm::resolve_auto(&mut runtime).await {
        return Ok(Json(serde_json::json!({"error": e})));
    }
    if let Err(e) = crate::slurm::validate_runtime(&runtime).await {
        return Ok(Json(serde_json::json!({"error": e})));
    }

    let run_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO flow_runs (id, flow_id, status) VALUES (?, ?, 'running')")
        .bind(&run_id)
        .bind(&flow.id)
        .execute(&state.pool)
        .await
        .ok();

    let data_dir = state.data_dir.clone();
    let pool = state.pool.clone();
    tokio::spawn(async move {
        execute_flow(graph, project.name, data_dir, run_id, pool, runtime, None).await;
    });

    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn run_node(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((id, node_id)): Path<(String, i64)>,
) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let flow = sqlx::query_as::<_, ProjectFlow>(
        "SELECT * FROM project_flows WHERE project_id = ? LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);
    let Some(flow) = flow else {
        return Ok(Json(serde_json::json!({"error": "no flow"})));
    };
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);
    let Some(project) = project else {
        return Err(Redirect::to("/projects"));
    };

    let graph: serde_json::Value = serde_json::from_str(&flow.graph_data).unwrap_or_default();
    if let Err(e) = preflight_graph(&graph, &state.data_dir, &project.name, Some(node_id)).await {
        return Ok(Json(serde_json::json!({"error": e})));
    }

    let mut runtime = crate::routes::settings::load_runtime_config(&state.pool).await;
    if let Err(e) = crate::slurm::resolve_auto(&mut runtime).await {
        return Ok(Json(serde_json::json!({"error": e})));
    }
    if let Err(e) = crate::slurm::validate_runtime(&runtime).await {
        return Ok(Json(serde_json::json!({"error": e})));
    }

    let run_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO flow_runs (id, flow_id, status) VALUES (?, ?, 'running')")
        .bind(&run_id)
        .bind(&flow.id)
        .execute(&state.pool)
        .await
        .ok();

    let data_dir = state.data_dir.clone();
    let pool = state.pool.clone();
    tokio::spawn(async move {
        execute_flow(
            graph,
            project.name,
            data_dir,
            run_id,
            pool,
            runtime,
            Some(node_id),
        )
        .await;
    });

    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Clone, Copy)]
enum LogKind {
    Section,
    Info,
    Command,
    Success,
    Error,
}

impl LogKind {
    fn sgr(self) -> &'static str {
        match self {
            Self::Section => "1;36",
            Self::Info => "0;94",
            Self::Command => "1;35",
            Self::Success => "1;32",
            Self::Error => "1;31",
        }
    }
}

fn ansi_wrap(text: impl AsRef<str>, sgr: &str) -> String {
    format!("\x1b[{sgr}m{}\x1b[0m", text.as_ref())
}

fn format_log_message(kind: LogKind, message: impl AsRef<str>) -> String {
    ansi_wrap(message, kind.sgr())
}

fn contains_ansi(text: &str) -> bool {
    text.contains("\x1b[")
}

const NON_SCRIPT_TYPES: [&str; 3] = ["datasource/File", "constant/Value", "viewer/Preview"];

fn is_script_node(node: &serde_json::Value) -> bool {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    !NON_SCRIPT_TYPES.contains(&node_type)
}

fn node_id(node: &serde_json::Value) -> i64 {
    node.get("id").and_then(|i| i.as_i64()).unwrap_or(0)
}

fn node_title(node: &serde_json::Value) -> &str {
    node.get("title")
        .or(node.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("node")
}

fn port_name(port: &serde_json::Value) -> &str {
    let raw_name = port.get("name").and_then(|n| n.as_str()).unwrap_or("input");
    raw_name.split(" :").next().unwrap_or(raw_name)
}

fn sorted_script_nodes(nodes: &[serde_json::Value]) -> Vec<&serde_json::Value> {
    let mut script_nodes: Vec<&serde_json::Value> =
        nodes.iter().filter(|node| is_script_node(node)).collect();
    script_nodes.sort_by_key(|node| node_id(node));
    script_nodes
}

fn resolve_data_path(data_dir: &str, file_path: &str) -> String {
    if std::path::Path::new(file_path).is_absolute() {
        file_path.to_string()
    } else {
        format!("{}/{}", data_dir, file_path)
    }
}

async fn file_exists(path: &str) -> bool {
    tokio::fs::metadata(path)
        .await
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
}

fn build_execution_order(
    script_nodes: &[&serde_json::Value],
    links: &[serde_json::Value],
) -> Result<Vec<usize>, String> {
    let script_ids: Vec<i64> = script_nodes.iter().map(|node| node_id(node)).collect();
    let mut deps: Vec<Vec<usize>> = vec![Vec::new(); script_nodes.len()];
    for (i, node) in script_nodes.iter().enumerate() {
        if let Some(inputs) = node.get("inputs").and_then(|x| x.as_array()) {
            for input in inputs {
                let link_id = input.get("link").and_then(|l| l.as_i64());
                if let Some(lid) = link_id {
                    if let Some(link) = links
                        .iter()
                        .find(|l| l.get(0).and_then(|v| v.as_i64()) == Some(lid))
                    {
                        let origin_id = link.get(1).and_then(|v| v.as_i64()).unwrap_or(0);
                        if let Some(dep_idx) = script_ids.iter().position(|&id| id == origin_id) {
                            deps[i].push(dep_idx);
                        }
                    }
                }
            }
        }
    }

    let mut in_degree: Vec<usize> = deps.iter().map(Vec::len).collect();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); script_nodes.len()];
    for (i, d) in deps.iter().enumerate() {
        for &dep in d {
            adj[dep].push(i);
        }
    }

    let mut queue = std::collections::VecDeque::new();
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            queue.push_back(i);
        }
    }
    let mut exec_order = Vec::new();
    while let Some(idx) = queue.pop_front() {
        exec_order.push(idx);
        for &next in &adj[idx] {
            in_degree[next] -= 1;
            if in_degree[next] == 0 {
                queue.push_back(next);
            }
        }
    }

    if exec_order.len() != script_nodes.len() {
        return Err("流程图存在循环依赖，无法执行".to_string());
    }

    Ok(exec_order)
}

async fn preflight_graph(
    graph: &serde_json::Value,
    data_dir: &str,
    project_name: &str,
    target_node_id: Option<i64>,
) -> Result<(), String> {
    let nodes = graph
        .get("nodes")
        .and_then(|n| n.as_array())
        .cloned()
        .unwrap_or_default();
    let links = graph
        .get("links")
        .and_then(|l| l.as_array())
        .cloned()
        .unwrap_or_default();
    let script_nodes = sorted_script_nodes(&nodes);

    if script_nodes.is_empty() {
        return Err("流程中没有可执行的脚本节点".to_string());
    }
    let exec_order = build_execution_order(&script_nodes, &links)?;
    let script_ids = script_nodes
        .iter()
        .map(|node| node_id(node))
        .collect::<Vec<_>>();

    let selected = if let Some(target) = target_node_id {
        let Some(idx) = script_ids.iter().position(|&id| id == target) else {
            return Err("选择的节点不是可执行脚本节点".to_string());
        };
        vec![idx]
    } else {
        exec_order
    };

    for node_idx in selected {
        let node = script_nodes[node_idx];
        let title = node_title(node);
        let script_path = node
            .get("properties")
            .and_then(|p| p.get("script_path"))
            .and_then(|s| s.as_str())
            .unwrap_or("");
        if script_path.trim().is_empty() {
            return Err(format!("节点 '{title}' 未配置 R 脚本路径"));
        }
        if !file_exists(script_path).await {
            return Err(format!("节点 '{title}' 的 R 脚本不存在: {script_path}"));
        }

        if let Some(inputs) = node.get("inputs").and_then(|i| i.as_array()) {
            for input in inputs {
                let input_name = port_name(input);
                let Some(lid) = input.get("link").and_then(|l| l.as_i64()) else {
                    return Err(format!("节点 '{title}' 的输入 '{input_name}' 未连接"));
                };
                let Some(link) = links
                    .iter()
                    .find(|l| l.get(0).and_then(|v| v.as_i64()) == Some(lid))
                else {
                    return Err(format!("节点 '{title}' 的输入 '{input_name}' 连接无效"));
                };
                let origin_node_id = link.get(1).and_then(|v| v.as_i64()).unwrap_or(0);
                let origin_slot = link.get(2).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let Some(origin) = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|i| i.as_i64()) == Some(origin_node_id))
                else {
                    return Err(format!(
                        "节点 '{title}' 的输入 '{input_name}' 来源节点不存在"
                    ));
                };

                match origin.get("type").and_then(|t| t.as_str()).unwrap_or("") {
                    "constant/Value" => {
                        let value = origin
                            .get("properties")
                            .and_then(|p| p.get("value"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if value.trim().is_empty() {
                            return Err(format!(
                                "节点 '{title}' 的输入 '{input_name}' 连接了空的固定值"
                            ));
                        }
                    }
                    "datasource/File" => {
                        let file_path = origin
                            .get("properties")
                            .and_then(|p| p.get("file_path"))
                            .and_then(|f| f.as_str())
                            .unwrap_or("");
                        if file_path.trim().is_empty() {
                            return Err(format!(
                                "节点 '{title}' 的输入 '{input_name}' 未选择数据文件"
                            ));
                        }
                        let resolved = resolve_data_path(data_dir, file_path);
                        if !file_exists(&resolved).await {
                            return Err(format!(
                                "节点 '{title}' 的输入 '{input_name}' 文件不存在: {resolved}"
                            ));
                        }
                    }
                    _ => {
                        if target_node_id.is_some() {
                            let Some(src) = resolve_output_path(
                                origin,
                                origin_slot,
                                data_dir,
                                project_name,
                                &nodes,
                                &script_nodes,
                            ) else {
                                return Err(format!(
                                    "节点 '{title}' 的输入 '{input_name}' 无法解析上游输出"
                                ));
                            };
                            if !file_exists(&src).await {
                                return Err(format!(
                                    "节点 '{title}' 的输入 '{input_name}' 依赖的上游输出不存在，请先运行上游节点: {src}"
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn append_node_log(work_dir: &str, message: impl AsRef<str>) {
    use tokio::io::AsyncWriteExt;

    if let Ok(mut file) = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{work_dir}/stdout.log"))
        .await
    {
        let _ = file.write_all(message.as_ref().as_bytes()).await;
        let _ = file.write_all(b"\n").await;
    }
}

async fn append_node_log_kind(work_dir: &str, kind: LogKind, message: impl AsRef<str>) {
    append_node_log(work_dir, format_log_message(kind, message)).await;
}

async fn execute_flow(
    graph: serde_json::Value,
    project_name: String,
    data_dir: String,
    run_id: String,
    pool: sqlx::SqlitePool,
    runtime: RuntimeConfig,
    target_node_id: Option<i64>,
) {
    let nodes = graph
        .get("nodes")
        .and_then(|n| n.as_array())
        .cloned()
        .unwrap_or_default();
    let links = graph
        .get("links")
        .and_then(|l| l.as_array())
        .cloned()
        .unwrap_or_default();

    let script_nodes = sorted_script_nodes(&nodes);
    let mut exec_order = match build_execution_order(&script_nodes, &links) {
        Ok(order) => order,
        Err(error) => {
            tracing::error!("{error}");
            sqlx::query(
                "UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?",
            )
            .bind(&run_id)
            .execute(&pool)
            .await
            .ok();
            return;
        }
    };
    if let Some(target) = target_node_id {
        exec_order.retain(|&idx| node_id(script_nodes[idx]) == target);
        if exec_order.is_empty() {
            sqlx::query(
                "UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?",
            )
            .bind(&run_id)
            .execute(&pool)
            .await
            .ok();
            return;
        }
    }

    // Execute in topological order
    for (_step, &node_idx) in exec_order.iter().enumerate() {
        let node = script_nodes[node_idx];
        let node_title = node
            .get("title")
            .or(node.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("node");
        let work_dir = format!(
            "{}/projects/{}/{}_{}",
            data_dir, project_name, node_idx, node_title
        );
        tokio::fs::create_dir_all(&work_dir).await.ok();
        tokio::fs::remove_file(format!("{}/.exit_code", work_dir))
            .await
            .ok();
        tokio::fs::remove_file(format!("{}/.pid", work_dir))
            .await
            .ok();
        tokio::fs::write(format!("{}/stdout.log", work_dir), "")
            .await
            .ok();
        tokio::fs::write(format!("{}/stderr.log", work_dir), "")
            .await
            .ok();

        let step_id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO step_runs (id, flow_run_id, step_order, status) VALUES (?, ?, ?, 'running')")
            .bind(&step_id)
            .bind(&run_id)
            .bind(node_idx as i32)
            .execute(&pool)
            .await
            .ok();
        append_node_log_kind(
            &work_dir,
            LogKind::Section,
            format!("==> 开始执行节点: {node_title}"),
        )
        .await;

        // Resolve inputs
        let mut input_files: Vec<String> = Vec::new();
        let mut inputs_json = serde_json::Map::new();
        if let Some(inputs) = node.get("inputs").and_then(|i| i.as_array()) {
            for input in inputs {
                let input_name = port_name(input).to_string();
                let Some(lid) = input.get("link").and_then(|l| l.as_i64()) else {
                    append_node_log_kind(
                        &work_dir,
                        LogKind::Error,
                        format!("ERROR: 输入 '{input_name}' 未连接，已停止执行"),
                    )
                    .await;
                    sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&step_id).execute(&pool).await.ok();
                    sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&run_id).execute(&pool).await.ok();
                    return;
                };
                let Some(link) = links
                    .iter()
                    .find(|l| l.get(0).and_then(|v| v.as_i64()) == Some(lid))
                else {
                    append_node_log_kind(
                        &work_dir,
                        LogKind::Error,
                        format!("ERROR: 输入 '{input_name}' 连接无效，已停止执行"),
                    )
                    .await;
                    sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&step_id).execute(&pool).await.ok();
                    sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&run_id).execute(&pool).await.ok();
                    return;
                };
                let origin_node_id = link.get(1).and_then(|v| v.as_i64()).unwrap_or(0);
                let origin_slot = link.get(2).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let Some(origin) = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|i| i.as_i64()) == Some(origin_node_id))
                else {
                    append_node_log_kind(
                        &work_dir,
                        LogKind::Error,
                        format!("ERROR: 输入 '{input_name}' 来源节点不存在，已停止执行"),
                    )
                    .await;
                    sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&step_id).execute(&pool).await.ok();
                    sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&run_id).execute(&pool).await.ok();
                    return;
                };

                if origin.get("type").and_then(|t| t.as_str()) == Some("constant/Value") {
                    let value = origin
                        .get("properties")
                        .and_then(|p| p.get("value"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if value.is_empty() {
                        append_node_log_kind(
                            &work_dir,
                            LogKind::Error,
                            format!("ERROR: 输入 '{input_name}' 连接了空的固定值，已停止执行"),
                        )
                        .await;
                        sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                            .bind(&step_id).execute(&pool).await.ok();
                        sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                            .bind(&run_id).execute(&pool).await.ok();
                        return;
                    }
                    append_node_log_kind(
                        &work_dir,
                        LogKind::Info,
                        format!("输入 {input_name} = {value}"),
                    )
                    .await;
                    inputs_json.insert(input_name, serde_json::Value::String(value));
                    continue;
                }

                let source_path = resolve_output_path(
                    origin,
                    origin_slot,
                    &data_dir,
                    &project_name,
                    &nodes,
                    &script_nodes,
                );
                let Some(src) = source_path else {
                    append_node_log_kind(
                        &work_dir,
                        LogKind::Error,
                        format!("ERROR: 输入 '{input_name}' 无法解析来源路径，已停止执行"),
                    )
                    .await;
                    sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&step_id).execute(&pool).await.ok();
                    sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&run_id).execute(&pool).await.ok();
                    return;
                };
                if !file_exists(&src).await {
                    append_node_log_kind(
                        &work_dir,
                        LogKind::Error,
                        format!("ERROR: 输入 '{input_name}' 文件不存在: {src}"),
                    )
                    .await;
                    sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&step_id).execute(&pool).await.ok();
                    sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&run_id).execute(&pool).await.ok();
                    return;
                }
                let dest = format!("{}/{}", work_dir, input_name);
                tokio::fs::remove_file(&dest).await.ok();
                if let Err(err) = tokio::fs::symlink(&src, &dest).await {
                    append_node_log_kind(
                        &work_dir,
                        LogKind::Error,
                        format!("ERROR: 无法链接输入 '{input_name}': {err}"),
                    )
                    .await;
                    sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&step_id).execute(&pool).await.ok();
                    sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&run_id).execute(&pool).await.ok();
                    return;
                }
                append_node_log_kind(
                    &work_dir,
                    LogKind::Info,
                    format!("输入 {input_name} -> {src}"),
                )
                .await;
                input_files.push(src.clone());
                inputs_json.insert(input_name, serde_json::Value::String(src));
            }
        }

        let script_path = node
            .get("properties")
            .and_then(|p| p.get("script_path"))
            .and_then(|s| s.as_str())
            .unwrap_or("");
        if script_path.is_empty() {
            append_node_log_kind(&work_dir, LogKind::Error, "ERROR: 未配置 R 脚本路径").await;
            sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(&step_id).execute(&pool).await.ok();
            sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(&run_id).execute(&pool).await.ok();
            return;
        }

        let params = node
            .get("properties")
            .and_then(|p| p.get("params"))
            .cloned()
            .unwrap_or(serde_json::json!({}));
        tokio::fs::write(
            format!("{}/params.json", work_dir),
            serde_json::to_string_pretty(&params).unwrap_or_default(),
        )
        .await
        .ok();
        tokio::fs::write(
            format!("{}/inputs.json", work_dir),
            serde_json::to_string_pretty(&serde_json::Value::Object(inputs_json))
                .unwrap_or_default(),
        )
        .await
        .ok();

        // Per-node SIF override (optional)
        let node_sif = node
            .get("properties")
            .and_then(|p| p.get("sif"))
            .and_then(|s| s.as_str())
            .filter(|s| !s.trim().is_empty());

        // Validate: Singularity mode requires at least one SIF source
        if matches!(runtime.mode, crate::models::RuntimeMode::ClusterSingularity)
            && node_sif.is_none()
            && runtime.cluster.sif_path.trim().is_empty()
        {
            append_node_log_kind(
                &work_dir,
                LogKind::Error,
                format!("ERROR: 节点 '{node_title}' 未配置 SIF 镜像（全局和节点级别均为空）"),
            )
            .await;
            sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(&step_id).execute(&pool).await.ok();
            sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(&run_id).execute(&pool).await.ok();
            return;
        }

        // Log the full command to stdout.log before execution
        let cmd_line = format!(
            "$ Rscript {} {} {}\n\n",
            script_path,
            work_dir,
            input_files.join(" ")
        );
        append_node_log_kind(&work_dir, LogKind::Command, cmd_line).await;

        let result = crate::slurm::submit_job(
            std::path::Path::new(&work_dir),
            script_path,
            &format!("ripeline_{}_{}", run_id, node_idx),
            &input_files,
            &runtime,
            node_sif,
        )
        .await;

        match result {
            Ok(job_id) => {
                append_node_log_kind(&work_dir, LogKind::Success, format!("作业已提交: {job_id}"))
                    .await;
                sqlx::query("UPDATE step_runs SET slurm_job_id = ? WHERE id = ?")
                    .bind(&job_id)
                    .bind(&step_id)
                    .execute(&pool)
                    .await
                    .ok();

                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    let status = crate::slurm::job_status(&job_id).await.unwrap_or_default();
                    if status.contains("COMPLETED") {
                        append_node_log_kind(&work_dir, LogKind::Success, "节点执行完成").await;
                        sqlx::query("UPDATE step_runs SET status = 'completed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                            .bind(&step_id).execute(&pool).await.ok();
                        break;
                    } else if status.contains("FAILED") || status.contains("CANCELLED") {
                        append_node_log_kind(
                            &work_dir,
                            LogKind::Error,
                            format!("ERROR: 作业结束状态异常: {status}"),
                        )
                        .await;
                        sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                            .bind(&step_id).execute(&pool).await.ok();
                        sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                            .bind(&run_id).execute(&pool).await.ok();
                        return;
                    }
                }
            }
            Err(err) => {
                append_node_log_kind(
                    &work_dir,
                    LogKind::Error,
                    format!("ERROR: 作业提交失败: {err}"),
                )
                .await;
                sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                    .bind(&step_id).execute(&pool).await.ok();
                sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                    .bind(&run_id).execute(&pool).await.ok();
                return;
            }
        }
    }

    sqlx::query(
        "UPDATE flow_runs SET status = 'completed', finished_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(&run_id)
    .execute(&pool)
    .await
    .ok();
}

fn resolve_output_path(
    origin: &serde_json::Value,
    origin_slot: usize,
    data_dir: &str,
    project_name: &str,
    _all_nodes: &[serde_json::Value],
    script_nodes: &[&serde_json::Value],
) -> Option<String> {
    let origin_type = origin.get("type").and_then(|t| t.as_str()).unwrap_or("");
    if origin_type == "datasource/File" {
        let file_path = origin
            .get("properties")
            .and_then(|p| p.get("file_path"))
            .and_then(|f| f.as_str())?;
        if std::path::Path::new(file_path).is_absolute() {
            return Some(file_path.to_string());
        }
        return Some(format!("{}/{}", data_dir, file_path));
    }
    // Script node: output is in its work directory
    let origin_id = origin.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
    let idx = script_nodes
        .iter()
        .position(|n| n.get("id").and_then(|i| i.as_i64()) == Some(origin_id))?;
    let node_title = origin
        .get("title")
        .or(origin.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("node");
    let outputs = origin.get("outputs").and_then(|o| o.as_array())?;
    let raw_output_name = outputs
        .get(origin_slot)
        .and_then(|o| o.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("output");
    let output_name = raw_output_name
        .split(" :")
        .next()
        .unwrap_or(raw_output_name);
    Some(format!(
        "{}/projects/{}/{}_{}/{}",
        data_dir, project_name, idx, node_title, output_name
    ))
}

pub async fn run_status(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let flow = sqlx::query_as::<_, ProjectFlow>(
        "SELECT * FROM project_flows WHERE project_id = ? LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);
    let Some(flow) = flow else {
        return Ok(Json(serde_json::json!({"status": "no_flow"})));
    };

    let run = sqlx::query_as::<_, crate::models::FlowRun>(
        "SELECT * FROM flow_runs WHERE flow_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&flow.id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);
    let Some(run) = run else {
        return Ok(Json(serde_json::json!({"status": "idle"})));
    };

    let steps = sqlx::query_as::<_, crate::models::StepRun>(
        "SELECT * FROM step_runs WHERE flow_run_id = ? ORDER BY step_order",
    )
    .bind(&run.id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let steps_json: Vec<serde_json::Value> = steps.iter().map(|s| {
        serde_json::json!({"step_order": s.step_order, "status": s.status, "slurm_job_id": s.slurm_job_id})
    }).collect();

    Ok(Json(
        serde_json::json!({"status": run.status, "steps": steps_json}),
    ))
}

pub async fn run_logs(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);
    let Some(project) = project else {
        return Ok(Json(serde_json::json!({"logs": ""})));
    };

    let flow = sqlx::query_as::<_, ProjectFlow>(
        "SELECT * FROM project_flows WHERE project_id = ? LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);
    let Some(flow) = flow else {
        return Ok(Json(serde_json::json!({"logs": ""})));
    };

    let run = sqlx::query_as::<_, crate::models::FlowRun>(
        "SELECT * FROM flow_runs WHERE flow_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&flow.id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);
    let Some(_run) = run else {
        return Ok(Json(serde_json::json!({"logs": ""})));
    };

    let graph_data: serde_json::Value = serde_json::from_str(&flow.graph_data).unwrap_or_default();
    let nodes = graph_data
        .get("nodes")
        .and_then(|n| n.as_array())
        .cloned()
        .unwrap_or_default();
    let non_script = ["datasource/File", "constant/Value", "viewer/Preview"];
    let mut script_nodes: Vec<(usize, &str)> = Vec::new();
    for node in &nodes {
        let t = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if !non_script.contains(&t) {
            let title = node
                .get("title")
                .or(node.get("type"))
                .and_then(|t| t.as_str())
                .unwrap_or("node");
            script_nodes.push((script_nodes.len(), title));
        }
    }

    let mut logs = String::new();
    for (idx, title) in &script_nodes {
        let stderr_path = format!(
            "{}/projects/{}/{}_{}/stderr.log",
            state.data_dir, project.name, idx, title
        );
        let stdout_path = format!(
            "{}/projects/{}/{}_{}/stdout.log",
            state.data_dir, project.name, idx, title
        );
        if let Ok(content) = tokio::fs::read_to_string(&stdout_path).await {
            if !content.is_empty() {
                logs.push_str(&format!(
                    "{}\n{}\n",
                    format_log_message(LogKind::Section, format!("── {} [stdout] ──", title)),
                    content
                ));
            }
        }
        if let Ok(content) = tokio::fs::read_to_string(&stderr_path).await {
            if !content.is_empty() {
                let stderr_body = if contains_ansi(&content) {
                    content
                } else {
                    format_log_message(LogKind::Error, content)
                };
                logs.push_str(&format!(
                    "{}\n{}\n",
                    format_log_message(LogKind::Error, format!("── {} [stderr] ──", title)),
                    stderr_body
                ));
            }
        }
    }

    Ok(Json(serde_json::json!({"logs": logs})))
}

pub async fn cancel_flow(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let flow = sqlx::query_as::<_, ProjectFlow>(
        "SELECT * FROM project_flows WHERE project_id = ? LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);
    let Some(flow) = flow else {
        return Ok(Json(serde_json::json!({"ok": false})));
    };

    let run = sqlx::query_as::<_, crate::models::FlowRun>("SELECT * FROM flow_runs WHERE flow_id = ? AND status = 'running' ORDER BY created_at DESC LIMIT 1")
        .bind(&flow.id).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(run) = run else {
        return Ok(Json(serde_json::json!({"ok": false})));
    };

    let steps = sqlx::query_as::<_, crate::models::StepRun>(
        "SELECT * FROM step_runs WHERE flow_run_id = ? AND status = 'running'",
    )
    .bind(&run.id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    for step in &steps {
        if let Some(ref job_id) = step.slurm_job_id {
            let _ = crate::slurm::cancel_job(job_id).await;
        }
        sqlx::query("UPDATE step_runs SET status = 'cancelled', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(&step.id).execute(&state.pool).await.ok();
    }

    sqlx::query(
        "UPDATE flow_runs SET status = 'cancelled', finished_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(&run.id)
    .execute(&state.pool)
    .await
    .ok();

    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn delete(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    sqlx::query("DELETE FROM projects WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .ok();

    if is_htmx(&headers) {
        Ok(Html("").into_response())
    } else {
        Ok(Redirect::to("/projects").into_response())
    }
}

#[cfg(test)]
mod tests {
    use super::{contains_ansi, format_log_message, LogKind};

    #[test]
    fn formats_logs_with_ansi_sequences() {
        let formatted = format_log_message(LogKind::Error, "ERROR: failed");
        assert!(formatted.starts_with("\x1b[1;31m"));
        assert!(formatted.ends_with("\x1b[0m"));
    }

    #[test]
    fn detects_ansi_escape_sequences() {
        assert!(contains_ansi("\x1b[32mgreen\x1b[0m"));
        assert!(!contains_ansi("plain text"));
    }
}
