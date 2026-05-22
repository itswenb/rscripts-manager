use super::auth::is_authenticated;
use crate::models::PipelineNode;
use crate::AppState;
use askama::Template;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

pub struct NodeView {
    pub id: String,
    pub name: String,
    pub script_path: String,
    pub default_sif: String,
    pub input_count: usize,
    pub output_count: usize,
}

#[derive(Template)]
#[template(path = "pipelines.html")]
struct PipelinesTemplate {
    active_nav: &'static str,
    nodes: Vec<NodeView>,
}

#[derive(Template)]
#[template(path = "fragments/node_list.html")]
struct NodeListFragment {
    nodes: Vec<NodeView>,
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers.contains_key("hx-request")
}

fn is_nav_request(headers: &HeaderMap) -> bool {
    headers.get("hx-target").map(|v| v.as_bytes()) == Some(b"main-content")
}

fn to_node_views(rows: &[PipelineNode]) -> Vec<NodeView> {
    rows.iter()
        .map(|n| {
            let inputs: Vec<serde_json::Value> =
                serde_json::from_str(&n.inputs_schema).unwrap_or_default();
            let outputs: Vec<serde_json::Value> =
                serde_json::from_str(&n.outputs_schema).unwrap_or_default();
            NodeView {
                id: n.id.clone(),
                name: n.name.clone(),
                script_path: n.script_path.clone(),
                default_sif: n.default_sif.clone(),
                input_count: inputs.len(),
                output_count: outputs.len(),
            }
        })
        .collect()
}

pub async fn list(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }
    let rows =
        sqlx::query_as::<_, PipelineNode>("SELECT * FROM pipeline_nodes ORDER BY created_at DESC")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();
    let nodes = to_node_views(&rows);

    if is_htmx(&headers) && !is_nav_request(&headers) {
        let tmpl = NodeListFragment { nodes };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    } else {
        let tmpl = PipelinesTemplate {
            active_nav: "pipelines",
            nodes,
        };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    }
}

#[derive(Deserialize)]
pub struct CreateNode {
    pub name: String,
    pub script_path: String,
    #[serde(default)]
    pub default_sif: String,
}

pub async fn create(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Form(form): Form<CreateNode>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }

    let script_content = tokio::fs::read_to_string(&form.script_path)
        .await
        .unwrap_or_default();
    let meta = crate::rparser::parse_script(&script_content);

    let id = uuid::Uuid::new_v4().to_string();
    let inputs_json = serde_json::to_string(&meta.inputs.iter().map(|i| {
        serde_json::json!({"name": i.name, "type": i.r#type, "default": i.default, "description": i.description})
    }).collect::<Vec<_>>()).unwrap_or_default();
    let outputs_json = serde_json::to_string(&meta.outputs.iter().map(|o| {
        serde_json::json!({"name": o.name, "type": o.r#type, "default": o.default, "description": o.description})
    }).collect::<Vec<_>>()).unwrap_or_default();

    sqlx::query("INSERT INTO pipeline_nodes (id, name, script_path, params_schema, inputs_schema, outputs_schema, default_sif) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(&form.name).bind(&form.script_path)
        .bind("[]").bind(&inputs_json).bind(&outputs_json)
        .bind(&form.default_sif)
        .execute(&state.pool).await.ok();

    if is_htmx(&headers) {
        let rows = sqlx::query_as::<_, PipelineNode>(
            "SELECT * FROM pipeline_nodes ORDER BY created_at DESC",
        )
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
        let tmpl = NodeListFragment {
            nodes: to_node_views(&rows),
        };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    } else {
        Ok(Redirect::to("/pipelines").into_response())
    }
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
    sqlx::query("DELETE FROM pipeline_nodes WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .ok();

    if is_htmx(&headers) {
        Ok(Html("").into_response())
    } else {
        Ok(Redirect::to("/pipelines").into_response())
    }
}

pub async fn update(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<CreateNode>,
) -> Result<Response, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }

    let script_content = tokio::fs::read_to_string(&form.script_path)
        .await
        .unwrap_or_default();
    let meta = crate::rparser::parse_script(&script_content);

    let inputs_json = serde_json::to_string(&meta.inputs.iter().map(|i| {
        serde_json::json!({"name": i.name, "type": i.r#type, "default": i.default, "description": i.description})
    }).collect::<Vec<_>>()).unwrap_or_default();
    let outputs_json = serde_json::to_string(&meta.outputs.iter().map(|o| {
        serde_json::json!({"name": o.name, "type": o.r#type, "default": o.default, "description": o.description})
    }).collect::<Vec<_>>()).unwrap_or_default();

    sqlx::query("UPDATE pipeline_nodes SET name = ?, script_path = ?, inputs_schema = ?, outputs_schema = ?, default_sif = ? WHERE id = ?")
        .bind(&form.name).bind(&form.script_path)
        .bind(&inputs_json).bind(&outputs_json)
        .bind(&form.default_sif)
        .bind(&id).execute(&state.pool).await.ok();

    if is_htmx(&headers) {
        let rows = sqlx::query_as::<_, PipelineNode>(
            "SELECT * FROM pipeline_nodes ORDER BY created_at DESC",
        )
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
        let tmpl = NodeListFragment {
            nodes: to_node_views(&rows),
        };
        Ok(Html(tmpl.render().unwrap_or_default()).into_response())
    } else {
        Ok(Redirect::to("/pipelines").into_response())
    }
}

#[derive(Serialize)]
pub struct ScriptInfo {
    pub path: String,
    pub filename: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub default_sif: String,
    pub inputs: Vec<serde_json::Value>,
    pub outputs: Vec<serde_json::Value>,
}

pub async fn available_scripts(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<ScriptInfo>>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }

    let scripts_dir = format!("{}/scripts", state.data_dir);
    let mut scripts = Vec::new();

    let mut entries = match tokio::fs::read_dir(&scripts_dir).await {
        Ok(e) => e,
        Err(_) => return Ok(Json(scripts)),
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !matches!(path.extension().and_then(|e| e.to_str()), Some("R" | "r")) {
            continue;
        }
        let content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
        let meta = crate::rparser::parse_script(&content);
        scripts.push(ScriptInfo {
            path: path.to_string_lossy().to_string(),
            filename: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            title: meta.title,
            description: meta.description,
            default_sif: String::new(),
            inputs: meta.inputs.iter().map(|i| serde_json::json!({"name": i.name, "type": i.r#type, "default": i.default, "description": i.description})).collect(),
            outputs: meta.outputs.iter().map(|o| serde_json::json!({"name": o.name, "type": o.r#type, "default": o.default, "description": o.description})).collect(),
        });
    }

    Ok(Json(scripts))
}

pub async fn registered_scripts(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<ScriptInfo>>, Redirect> {
    if !is_authenticated(&jar, &state.secret) {
        return Err(Redirect::to("/login"));
    }

    let nodes =
        sqlx::query_as::<_, PipelineNode>("SELECT * FROM pipeline_nodes ORDER BY created_at DESC")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    let mut scripts = Vec::new();
    for node in &nodes {
        let content = tokio::fs::read_to_string(&node.script_path)
            .await
            .unwrap_or_default();
        let meta = crate::rparser::parse_script(&content);
        scripts.push(ScriptInfo {
            path: node.script_path.clone(),
            filename: std::path::Path::new(&node.script_path).file_name().unwrap_or_default().to_string_lossy().to_string(),
            title: meta.title.or_else(|| Some(node.name.clone())),
            description: meta.description,
            default_sif: node.default_sif.clone(),
            inputs: meta.inputs.iter().map(|i| serde_json::json!({"name": i.name, "type": i.r#type, "default": i.default, "description": i.description})).collect(),
            outputs: meta.outputs.iter().map(|o| serde_json::json!({"name": o.name, "type": o.r#type, "default": o.default, "description": o.description})).collect(),
        });
    }

    Ok(Json(scripts))
}
