use super::auth::is_authenticated;
use crate::models::AuditLog;
use crate::AppState;
use askama::Template;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::CookieJar;

#[derive(Template)]
#[template(path = "audit.html")]
struct AuditTemplate {
    active_nav: &'static str,
    logs: Vec<AuditLog>,
}

#[derive(Template)]
#[template(path = "fragments/audit_list.html")]
struct AuditListFragment {
    logs: Vec<AuditLog>,
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
    let logs = sqlx::query_as::<_, AuditLog>(
        "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT 200",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    if is_htmx(&headers) && !is_nav_request(&headers) {
        Ok(Html(AuditListFragment { logs }.render().unwrap_or_default()).into_response())
    } else {
        Ok(Html(
            AuditTemplate {
                active_nav: "audit",
                logs,
            }
            .render()
            .unwrap_or_default(),
        )
        .into_response())
    }
}
