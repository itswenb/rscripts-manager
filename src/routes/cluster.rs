use axum::response::Html;
use askama::Template;
use crate::slurm;

#[derive(Template)]
#[template(path = "fragments/cluster_status.html")]
struct ClusterStatusFragment {
    status: slurm::ClusterStatus,
}

pub async fn status() -> Html<String> {
    let status = slurm::cluster_status().await;
    let tmpl = ClusterStatusFragment { status };
    Html(tmpl.render().unwrap_or_default())
}
