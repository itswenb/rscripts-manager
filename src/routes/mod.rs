use crate::AppState;
use axum::{
    response::Html,
    routing::{delete, get, post},
    Router,
};

pub mod audit;
pub mod auth;
pub mod cluster;
pub mod files;
pub mod flows;
pub mod pipelines;
pub mod projects;
pub mod settings;
pub mod users;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/login", get(auth::login_page).post(auth::login))
        .route("/logout", get(auth::logout))
        .route("/projects", get(projects::list).post(projects::create))
        .route(
            "/projects/{id}",
            get(projects::detail).delete(projects::delete),
        )
        .route(
            "/projects/{id}/flow",
            get(projects::get_flow).post(projects::save_flow),
        )
        .route("/projects/{id}/run", post(projects::run_flow))
        .route(
            "/projects/{id}/run-node/{node_id}",
            post(projects::run_node),
        )
        .route(
            "/projects/{id}/run-through-node/{node_id}",
            post(projects::run_through_node),
        )
        .route("/projects/{id}/run-group", post(projects::run_group))
        .route(
            "/projects/{id}/run-through-group",
            post(projects::run_through_group),
        )
        .route("/projects/{id}/cancel", post(projects::cancel_flow))
        .route("/projects/{id}/run-status", get(projects::run_status))
        .route("/projects/{id}/run-logs", get(projects::run_logs))
        .route(
            "/projects/{id}/output/{node_index}",
            get(projects::node_output),
        )
        .route(
            "/projects/{id}/output/{node_index}/{filename}",
            get(projects::node_output_file),
        )
        .route("/pipelines", get(pipelines::list).post(pipelines::create))
        .route("/pipelines/scripts", get(pipelines::available_scripts))
        .route("/pipelines/registered", get(pipelines::registered_scripts))
        .route(
            "/pipelines/{id}",
            delete(pipelines::delete).put(pipelines::update),
        )
        .route("/files", get(files::list))
        .route("/files/picker", get(files::data_picker))
        .route("/files/list-json", get(files::list_json))
        .route("/files/upload", post(files::upload))
        .route("/files/upload-chunk", post(files::upload_chunk))
        .route("/files/mkdir", post(files::mkdir))
        .route("/files/delete", post(files::delete))
        .route("/files/rename", post(files::rename))
        .route("/files/read", get(files::read_file))
        .route("/files/save", post(files::save_file))
        .route("/files/download", get(files::download))
        .route("/cluster/status", get(cluster::status))
        .route(
            "/settings",
            get(settings::get_settings).post(settings::save_settings),
        )
        .route("/runtime/detect", get(settings::detect))
        .route("/runtime/sif-list", get(settings::list_sif))
        .route("/runtime/modules", get(settings::list_modules))
        .route("/flows/{flow_id}/run", post(flows::start_run))
        .route("/flows/{flow_id}/pause", post(flows::pause))
        .route("/flows/{flow_id}/resume", post(flows::resume))
        .route("/flows/{flow_id}/reset/{step}", post(flows::reset_to_step))
        .route("/flows/{flow_id}/status", get(flows::status))
        .route("/users", get(users::list))
        .route("/users/summary", get(users::summary))
        .route("/users/profile", post(users::update_profile))
        .route("/users/avatar", post(users::update_avatar))
        .route("/audit", get(audit::list))
        .with_state(state)
}

async fn index() -> Html<&'static str> {
    Html("<meta http-equiv='refresh' content='0;url=/projects'>")
}
