use crate::{
    models::{ClusterRuntimeMode, RuntimeConfig, RuntimeMode},
    runtime, AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct SettingsResponse {
    pub mode: String,
    pub singularity_image_dir: String,
    pub singularity_image: String,
    pub module_name: String,
    pub detection: runtime::RuntimeDetection,
}

pub async fn get_settings(State(state): State<AppState>) -> Json<SettingsResponse> {
    let map = load_settings_map(&state.pool).await;
    let detection = runtime::detect().await;
    Json(SettingsResponse {
        mode: map
            .get("runtime.mode")
            .cloned()
            .unwrap_or_else(|| "host".into()),
        singularity_image_dir: map
            .get("runtime.singularity_image_dir")
            .cloned()
            .unwrap_or_default(),
        singularity_image: map
            .get("runtime.singularity_image")
            .cloned()
            .unwrap_or_default(),
        module_name: map.get("runtime.module_name").cloned().unwrap_or_default(),
        detection,
    })
}

#[derive(Deserialize)]
pub struct SettingsForm {
    pub mode: String,
    #[serde(default)]
    pub singularity_image_dir: String,
    #[serde(default)]
    pub singularity_image: String,
    #[serde(default)]
    pub module_name: String,
}

pub async fn save_settings(
    State(state): State<AppState>,
    Json(body): Json<SettingsForm>,
) -> Json<serde_json::Value> {
    let entries = [
        ("runtime.mode", body.mode.as_str()),
        (
            "runtime.singularity_image_dir",
            body.singularity_image_dir.as_str(),
        ),
        ("runtime.singularity_image", body.singularity_image.as_str()),
        ("runtime.module_name", body.module_name.as_str()),
    ];
    for (k, v) in entries {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
            .bind(k).bind(v)
            .execute(&state.pool).await.ok();
    }
    Json(json!({"ok": true}))
}

#[derive(Deserialize)]
pub struct SifQuery {
    pub dir: String,
}

pub async fn list_sif(Query(q): Query<SifQuery>) -> Json<serde_json::Value> {
    let files = runtime::list_sif(&q.dir).await;
    Json(json!({"files": files}))
}

pub async fn list_modules() -> Json<serde_json::Value> {
    let modules = runtime::list_r_modules().await;
    Json(json!({"modules": modules}))
}

pub async fn detect() -> Json<runtime::RuntimeDetection> {
    Json(runtime::detect().await)
}

async fn load_settings_map(pool: &sqlx::SqlitePool) -> HashMap<String, String> {
    sqlx::query_as::<_, (String, String)>("SELECT key, value FROM settings")
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .collect()
}

pub async fn load_runtime_config(pool: &sqlx::SqlitePool) -> RuntimeConfig {
    let map = load_settings_map(pool).await;
    let mode = match map
        .get("runtime.mode")
        .map(|s| s.as_str())
        .unwrap_or("host")
    {
        "cluster_singularity" => RuntimeMode::ClusterSingularity,
        "cluster_module" => RuntimeMode::ClusterModule,
        "cluster_bundled" => RuntimeMode::ClusterBundled,
        "auto" => RuntimeMode::Auto,
        _ => RuntimeMode::Host,
    };
    let cluster_mode = match mode {
        RuntimeMode::ClusterSingularity => ClusterRuntimeMode::Singularity,
        RuntimeMode::ClusterModule => ClusterRuntimeMode::Module,
        _ => ClusterRuntimeMode::Bundled,
    };
    let mut cfg = RuntimeConfig::default();
    cfg.mode = mode;
    cfg.cluster.mode = cluster_mode;
    cfg.cluster.module_name = map.get("runtime.module_name").cloned().unwrap_or_default();
    cfg.cluster.sif_dir = map
        .get("runtime.singularity_image_dir")
        .cloned()
        .unwrap_or_default();
    cfg.cluster.sif_path = map
        .get("runtime.singularity_image")
        .cloned()
        .unwrap_or_default();
    cfg
}
