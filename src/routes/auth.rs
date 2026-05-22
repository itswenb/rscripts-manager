use crate::AppState;
use axum::extract::State;
use axum::response::{Html, Redirect};
use axum::Form;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;

const COOKIE_NAME: &str = "ripeline_session";

pub async fn login_page() -> Html<&'static str> {
    Html(include_str!("../../templates/login.html"))
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> (CookieJar, Redirect) {
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT username, password_hash FROM admin WHERE username = ?",
    )
    .bind(&form.username)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    if let Some((_username, hash)) = row {
        if crate::verify_password(&form.password, &hash) {
            let token = make_token(&state.secret);
            let cookie = Cookie::build((COOKIE_NAME, token))
                .path("/")
                .http_only(true)
                .build();
            return (jar.add(cookie), Redirect::to("/projects"));
        }
    }
    (jar, Redirect::to("/login?error=1"))
}

pub async fn logout(jar: CookieJar) -> (CookieJar, Redirect) {
    (
        jar.remove(Cookie::from(COOKIE_NAME)),
        Redirect::to("/login"),
    )
}

pub fn is_authenticated(jar: &CookieJar, secret: &str) -> bool {
    jar.get(COOKIE_NAME)
        .map(|c| verify_token(c.value(), secret))
        .unwrap_or(false)
}

fn make_token(secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let msg = "authenticated";
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(msg.as_bytes());
    format!("{}.{}", msg, hex::encode(mac.finalize().into_bytes()))
}

fn verify_token(token: &str, secret: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let Some((msg, sig)) = token.split_once('.') else {
        return false;
    };
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(msg.as_bytes());
    hex::encode(mac.finalize().into_bytes()) == sig
}
