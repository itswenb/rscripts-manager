use axum::extract::{State, Path};
use axum::response::{Html, Redirect, IntoResponse, Response};
use axum::{Form, Json};
use axum_extra::extract::cookie::CookieJar;
use axum::http::HeaderMap;
use askama::Template;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::models::{Project, ProjectFlow};
use super::auth::is_authenticated;

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

pub async fn list(State(state): State<AppState>, jar: CookieJar, headers: HeaderMap) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let projects = sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY created_at DESC")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    if is_htmx(&headers) && !is_nav_request(&headers) {
        let tmpl = ProjectListFragment { projects };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    } else {
        let tmpl = ProjectsTemplate { active_nav: "projects", projects };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    }
}

#[derive(Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create(State(state): State<AppState>, jar: CookieJar, headers: HeaderMap, Form(form): Form<CreateProject>) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
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
        let projects = sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY created_at DESC")
            .fetch_all(&state.pool).await.unwrap_or_default();
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

pub async fn detail(State(state): State<AppState>, jar: CookieJar, Path(id): Path<String>) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);

    match project {
        Some(p) => {
            let tmpl = ProjectDetailTemplate { active_nav: "projects", project: p };
            Ok(Html(tmpl.render().unwrap_or_default()).into_response())
        }
        None => Ok(Redirect::to("/projects").into_response()),
    }
}

pub async fn get_flow(State(state): State<AppState>, jar: CookieJar, Path(id): Path<String>) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let flow = sqlx::query_as::<_, ProjectFlow>("SELECT * FROM project_flows WHERE project_id = ? LIMIT 1")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);

    let graph_data = flow.map(|f| f.graph_data).unwrap_or_else(|| "{}".to_string());
    let json: serde_json::Value = serde_json::from_str(&graph_data).unwrap_or(serde_json::json!({}));
    Ok(Json(json))
}

#[derive(Deserialize)]
pub struct SaveFlow {
    pub graph_data: serde_json::Value,
}

pub async fn save_flow(State(state): State<AppState>, jar: CookieJar, Path(id): Path<String>, Json(body): Json<SaveFlow>) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let graph_str = serde_json::to_string(&body.graph_data).unwrap_or_default();

    let existing = sqlx::query_scalar::<_, String>("SELECT id FROM project_flows WHERE project_id = ? LIMIT 1")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);

    if let Some(flow_id) = existing {
        sqlx::query("UPDATE project_flows SET graph_data = ? WHERE id = ?")
            .bind(&graph_str).bind(&flow_id)
            .execute(&state.pool).await.ok();
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

pub async fn node_output(State(state): State<AppState>, jar: CookieJar, Path((id, node_index)): Path<(String, String)>) -> Result<Json<NodeOutput>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id).fetch_optional(&state.pool).await.unwrap_or(None);

    let Some(project) = project else { return Err(Redirect::to("/projects")); };
    let output_dir = format!("{}/projects/{}/{}", state.data_dir, project.name, node_index);
    let mut files = Vec::new();
    if let Ok(mut rd) = tokio::fs::read_dir(&output_dir).await {
        while let Ok(Some(entry)) = rd.next_entry().await {
            files.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(Json(NodeOutput { files, path: output_dir }))
}

pub async fn node_output_file(State(state): State<AppState>, jar: CookieJar, Path((id, node_index, filename)): Path<(String, String, String)>) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(project) = project else { return Err(Redirect::to("/projects")); };
    let file_path = format!("{}/projects/{}/{}/{}", state.data_dir, project.name, node_index, filename);
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

pub async fn run_flow(State(state): State<AppState>, jar: CookieJar, Path(id): Path<String>) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let flow = sqlx::query_as::<_, ProjectFlow>("SELECT * FROM project_flows WHERE project_id = ? LIMIT 1")
        .bind(&id).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(flow) = flow else { return Ok(Json(serde_json::json!({"error": "no flow"}))); };
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(project) = project else { return Err(Redirect::to("/projects")); };

    let graph: serde_json::Value = serde_json::from_str(&flow.graph_data).unwrap_or_default();
    let run_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO flow_runs (id, flow_id, status) VALUES (?, ?, 'running')")
        .bind(&run_id).bind(&flow.id).execute(&state.pool).await.ok();

    let data_dir = state.data_dir.clone();
    let pool = state.pool.clone();
    tokio::spawn(async move {
        execute_flow(graph, project.name, data_dir, run_id, pool).await;
    });

    Ok(Json(serde_json::json!({"ok": true})))
}

async fn execute_flow(graph: serde_json::Value, project_name: String, data_dir: String, run_id: String, pool: sqlx::SqlitePool) {
    let nodes = graph.get("nodes").and_then(|n| n.as_array()).cloned().unwrap_or_default();
    let links = graph.get("links").and_then(|l| l.as_array()).cloned().unwrap_or_default();

    let non_script_types = ["datasource/File", "constant/Value", "viewer/Preview"];
    let mut script_nodes: Vec<&serde_json::Value> = nodes.iter()
        .filter(|n| {
            let t = n.get("type").and_then(|t| t.as_str()).unwrap_or("");
            !non_script_types.contains(&t)
        })
        .collect();
    script_nodes.sort_by_key(|n| n.get("id").and_then(|i| i.as_i64()).unwrap_or(0));

    // Build topological order based on link dependencies
    let script_ids: Vec<i64> = script_nodes.iter()
        .map(|n| n.get("id").and_then(|i| i.as_i64()).unwrap_or(0))
        .collect();

    // For each script node, find which other script nodes it depends on (via links)
    let mut deps: Vec<Vec<usize>> = vec![Vec::new(); script_nodes.len()];
    for (i, node) in script_nodes.iter().enumerate() {
        if let Some(inputs) = node.get("inputs").and_then(|x| x.as_array()) {
            for input in inputs {
                let link_id = input.get("link").and_then(|l| l.as_i64());
                if let Some(lid) = link_id {
                    if let Some(link) = links.iter().find(|l| l.get(0).and_then(|v| v.as_i64()) == Some(lid)) {
                        let origin_id = link.get(1).and_then(|v| v.as_i64()).unwrap_or(0);
                        if let Some(dep_idx) = script_ids.iter().position(|&id| id == origin_id) {
                            deps[i].push(dep_idx);
                        }
                    }
                }
            }
        }
    }

    // Kahn's algorithm for topological sort
    let mut in_degree: Vec<usize> = deps.iter().map(|d| d.len()).collect();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); script_nodes.len()];
    for (i, d) in deps.iter().enumerate() {
        for &dep in d {
            adj[dep].push(i);
        }
    }

    let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 { queue.push_back(i); }
    }
    let mut exec_order: Vec<usize> = Vec::new();
    while let Some(idx) = queue.pop_front() {
        exec_order.push(idx);
        for &next in &adj[idx] {
            in_degree[next] -= 1;
            if in_degree[next] == 0 { queue.push_back(next); }
        }
    }

    // Execute in topological order
    for (_step, &node_idx) in exec_order.iter().enumerate() {
        let node = script_nodes[node_idx];
        let node_title = node.get("title").or(node.get("type")).and_then(|t| t.as_str()).unwrap_or("node");
        let work_dir = format!("{}/projects/{}/{}_{}", data_dir, project_name, node_idx, node_title);
        tokio::fs::create_dir_all(&work_dir).await.ok();
        tokio::fs::remove_file(format!("{}/.exit_code", work_dir)).await.ok();
        tokio::fs::remove_file(format!("{}/.pid", work_dir)).await.ok();

        // Resolve inputs
        let mut input_files: Vec<String> = Vec::new();
        if let Some(inputs) = node.get("inputs").and_then(|i| i.as_array()) {
            for input in inputs {
                if let Some(lid) = input.get("link").and_then(|l| l.as_i64()) {
                    if let Some(link) = links.iter().find(|l| l.get(0).and_then(|v| v.as_i64()) == Some(lid)) {
                        let origin_node_id = link.get(1).and_then(|v| v.as_i64()).unwrap_or(0);
                        let origin_slot = link.get(2).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        if let Some(origin) = nodes.iter().find(|n| n.get("id").and_then(|i| i.as_i64()) == Some(origin_node_id)) {
                            let source_path = resolve_output_path(origin, origin_slot, &data_dir, &project_name, &nodes, &script_nodes);
                            if let Some(src) = source_path {
                                let raw_name = input.get("name").and_then(|n| n.as_str()).unwrap_or("input");
                                let input_name = raw_name.split(" :").next().unwrap_or(raw_name);
                                let dest = format!("{}/{}", work_dir, input_name);
                                let _ = tokio::fs::symlink(&src, &dest).await;
                                input_files.push(src);
                            }
                        }
                    }
                }
            }
        }

        let script_path = node.get("properties")
            .and_then(|p| p.get("script_path"))
            .and_then(|s| s.as_str())
            .unwrap_or("");
        if script_path.is_empty() { continue; }

        let params = node.get("properties").and_then(|p| p.get("params")).cloned().unwrap_or(serde_json::json!({}));
        tokio::fs::write(format!("{}/params.json", work_dir), serde_json::to_string_pretty(&params).unwrap_or_default()).await.ok();

        // Log the full command to stdout.log before execution
        let cmd_line = format!("$ Rscript {} {} {}\n\n", script_path, work_dir, input_files.join(" "));
        let stdout_path = format!("{}/stdout.log", work_dir);
        tokio::fs::write(&stdout_path, &cmd_line).await.ok();

        let result = crate::slurm::submit_job(std::path::Path::new(&work_dir), script_path, &format!("rflow_{}_{}", run_id, node_idx), &input_files).await;

        match result {
            Ok(job_id) => {
                let step_id = uuid::Uuid::new_v4().to_string();
                sqlx::query("INSERT INTO step_runs (id, flow_run_id, step_order, status, slurm_job_id) VALUES (?, ?, ?, 'running', ?)")
                    .bind(&step_id).bind(&run_id).bind(node_idx as i32).bind(&job_id)
                    .execute(&pool).await.ok();

                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    let status = crate::slurm::job_status(&job_id).await.unwrap_or_default();
                    if status.contains("COMPLETED") {
                        sqlx::query("UPDATE step_runs SET status = 'completed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                            .bind(&step_id).execute(&pool).await.ok();
                        break;
                    } else if status.contains("FAILED") || status.contains("CANCELLED") {
                        sqlx::query("UPDATE step_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                            .bind(&step_id).execute(&pool).await.ok();
                        sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                            .bind(&run_id).execute(&pool).await.ok();
                        return;
                    }
                }
            }
            Err(_) => {
                sqlx::query("UPDATE flow_runs SET status = 'failed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
                    .bind(&run_id).execute(&pool).await.ok();
                return;
            }
        }
    }

    sqlx::query("UPDATE flow_runs SET status = 'completed', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&run_id).execute(&pool).await.ok();
}

fn resolve_output_path(origin: &serde_json::Value, origin_slot: usize, data_dir: &str, project_name: &str, _all_nodes: &[serde_json::Value], script_nodes: &[&serde_json::Value]) -> Option<String> {
    let origin_type = origin.get("type").and_then(|t| t.as_str()).unwrap_or("");
    if origin_type == "datasource/File" {
        let file_path = origin.get("properties").and_then(|p| p.get("file_path")).and_then(|f| f.as_str())?;
        if std::path::Path::new(file_path).is_absolute() {
            return Some(file_path.to_string());
        }
        return Some(format!("{}/{}", data_dir, file_path));
    }
    // Script node: output is in its work directory
    let origin_id = origin.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
    let idx = script_nodes.iter().position(|n| n.get("id").and_then(|i| i.as_i64()) == Some(origin_id))?;
    let node_title = origin.get("title").or(origin.get("type")).and_then(|t| t.as_str()).unwrap_or("node");
    let outputs = origin.get("outputs").and_then(|o| o.as_array())?;
    let raw_output_name = outputs.get(origin_slot).and_then(|o| o.get("name")).and_then(|n| n.as_str()).unwrap_or("output");
    let output_name = raw_output_name.split(" :").next().unwrap_or(raw_output_name);
    Some(format!("{}/projects/{}/{}_{}/{}", data_dir, project_name, idx, node_title, output_name))
}

pub async fn run_status(State(state): State<AppState>, jar: CookieJar, Path(id): Path<String>) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let flow = sqlx::query_as::<_, ProjectFlow>("SELECT * FROM project_flows WHERE project_id = ? LIMIT 1")
        .bind(&id).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(flow) = flow else { return Ok(Json(serde_json::json!({"status": "no_flow"}))); };

    let run = sqlx::query_as::<_, crate::models::FlowRun>("SELECT * FROM flow_runs WHERE flow_id = ? ORDER BY created_at DESC LIMIT 1")
        .bind(&flow.id).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(run) = run else { return Ok(Json(serde_json::json!({"status": "idle"}))); };

    let steps = sqlx::query_as::<_, crate::models::StepRun>("SELECT * FROM step_runs WHERE flow_run_id = ? ORDER BY step_order")
        .bind(&run.id).fetch_all(&state.pool).await.unwrap_or_default();

    let steps_json: Vec<serde_json::Value> = steps.iter().map(|s| {
        serde_json::json!({"step_order": s.step_order, "status": s.status, "slurm_job_id": s.slurm_job_id})
    }).collect();

    Ok(Json(serde_json::json!({"status": run.status, "steps": steps_json})))
}

pub async fn run_logs(State(state): State<AppState>, jar: CookieJar, Path(id): Path<String>) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(&id).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(project) = project else { return Ok(Json(serde_json::json!({"logs": ""}))); };

    let flow = sqlx::query_as::<_, ProjectFlow>("SELECT * FROM project_flows WHERE project_id = ? LIMIT 1")
        .bind(&id).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(flow) = flow else { return Ok(Json(serde_json::json!({"logs": ""}))); };

    let run = sqlx::query_as::<_, crate::models::FlowRun>("SELECT * FROM flow_runs WHERE flow_id = ? ORDER BY created_at DESC LIMIT 1")
        .bind(&flow.id).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(_run) = run else { return Ok(Json(serde_json::json!({"logs": ""}))); };

    let graph_data: serde_json::Value = serde_json::from_str(&flow.graph_data).unwrap_or_default();
    let nodes = graph_data.get("nodes").and_then(|n| n.as_array()).cloned().unwrap_or_default();
    let non_script = ["datasource/File", "constant/Value", "viewer/Preview"];
    let mut script_nodes: Vec<(usize, &str)> = Vec::new();
    for node in &nodes {
        let t = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if !non_script.contains(&t) {
            let title = node.get("title").or(node.get("type")).and_then(|t| t.as_str()).unwrap_or("node");
            script_nodes.push((script_nodes.len(), title));
        }
    }

    let mut logs = String::new();
    for (idx, title) in &script_nodes {
        let stderr_path = format!("{}/projects/{}/{}_{}/stderr.log", state.data_dir, project.name, idx, title);
        let stdout_path = format!("{}/projects/{}/{}_{}/stdout.log", state.data_dir, project.name, idx, title);
        if let Ok(content) = tokio::fs::read_to_string(&stdout_path).await {
            if !content.is_empty() {
                logs.push_str(&format!("── {} [stdout] ──\n{}\n", title, content));
            }
        }
        if let Ok(content) = tokio::fs::read_to_string(&stderr_path).await {
            if !content.is_empty() {
                logs.push_str(&format!("── {} [stderr] ──\n{}\n", title, content));
            }
        }
    }

    Ok(Json(serde_json::json!({"logs": logs})))
}

pub async fn cancel_flow(State(state): State<AppState>, jar: CookieJar, Path(id): Path<String>) -> Result<Json<serde_json::Value>, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    let flow = sqlx::query_as::<_, ProjectFlow>("SELECT * FROM project_flows WHERE project_id = ? LIMIT 1")
        .bind(&id).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(flow) = flow else { return Ok(Json(serde_json::json!({"ok": false}))); };

    let run = sqlx::query_as::<_, crate::models::FlowRun>("SELECT * FROM flow_runs WHERE flow_id = ? AND status = 'running' ORDER BY created_at DESC LIMIT 1")
        .bind(&flow.id).fetch_optional(&state.pool).await.unwrap_or(None);
    let Some(run) = run else { return Ok(Json(serde_json::json!({"ok": false}))); };

    let steps = sqlx::query_as::<_, crate::models::StepRun>("SELECT * FROM step_runs WHERE flow_run_id = ? AND status = 'running'")
        .bind(&run.id).fetch_all(&state.pool).await.unwrap_or_default();

    for step in &steps {
        if let Some(ref job_id) = step.slurm_job_id {
            let _ = crate::slurm::cancel_job(job_id).await;
        }
        sqlx::query("UPDATE step_runs SET status = 'cancelled', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(&step.id).execute(&state.pool).await.ok();
    }

    sqlx::query("UPDATE flow_runs SET status = 'cancelled', finished_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&run.id).execute(&state.pool).await.ok();

    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn delete(State(state): State<AppState>, jar: CookieJar, headers: HeaderMap, Path(id): Path<String>) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) { return Err(Redirect::to("/login")); }
    sqlx::query("DELETE FROM projects WHERE id = ?").bind(&id).execute(&state.pool).await.ok();

    if is_htmx(&headers) {
        Ok(Html("").into_response())
    } else {
        Ok(Redirect::to("/projects").into_response())
    }
}
