# Casbin RBAC Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Integrate Casbin for RBAC authorization with admin credentials and database connection managed via environment variables.

**Architecture:** Use `casbin` + `sqlx-adapter` to persist policies in PostgreSQL. Add a basic auth middleware (username/password from env) that identifies the user, then a Casbin middleware layer that enforces RBAC policies on API routes. Admin credentials and DB URL come from `.env`.

**Tech Stack:** casbin 2.20, sqlx-adapter 1.8, axum-casbin 1.3, argon2 (password hashing), tower (middleware)

---

### Task 1: Update .env.example with auth/db credentials

**Files:**

- Modify: `.env.example`

**Step 1: Add environment variables**

```env
DATABASE_URL=postgres://rflow:rflow@localhost:5432/rflow
DATA_DIR=./data
RSCRIPT_PATH=/usr/bin/Rscript
RUST_LOG=info,rflow_api=debug
ADMIN_USERNAME=admin
ADMIN_PASSWORD=changeme
```

**Step 2: Commit**

```bash
git add .env.example
git commit -m "chore: add admin credentials to .env.example"
```

---

### Task 2: Add Casbin dependencies to workspace

**Files:**

- Modify: `Cargo.toml` (workspace root)
- Modify: `crates/api/Cargo.toml`

**Step 1: Add workspace dependencies**

In root `Cargo.toml` `[workspace.dependencies]`:

```toml
casbin = { version = "2.20", features = ["runtime-tokio"] }
sqlx-adapter = { version = "1.8", default-features = false, features = ["postgres", "runtime-tokio"] }
axum-casbin = { version = "1.3", features = ["runtime-tokio"] }
argon2 = "0.5"
```

**Step 2: Add to api crate**

In `crates/api/Cargo.toml` `[dependencies]`:

```toml
casbin.workspace = true
sqlx-adapter.workspace = true
axum-casbin.workspace = true
argon2.workspace = true
```

**Step 3: Verify compilation**

Run: `cargo build -p rflow-api`
Expected: compiles (possibly with unused warnings)

**Step 4: Commit**

```bash
git add Cargo.toml crates/api/Cargo.toml Cargo.lock
git commit -m "feat: add casbin and argon2 dependencies"
```

---

### Task 3: Create Casbin model and default policy files

**Files:**

- Create: `config/casbin_model.conf`
- Create: `config/casbin_policy.csv`

**Step 1: Create RBAC model**

`config/casbin_model.conf`:

```ini
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && keyMatch2(r.obj, p.obj) && r.act == p.act
```

**Step 2: Create default policy (admin has full access)**

`config/casbin_policy.csv`:

```csv
p, admin, /api/*, GET
p, admin, /api/*, POST
p, admin, /api/*, PATCH
p, admin, /api/*, DELETE
```

**Step 3: Commit**

```bash
git add config/
git commit -m "feat: add Casbin RBAC model and default policy"
```

---

### Task 4: Add Unauthorized variant to AppError

**Files:**

- Modify: `crates/core/src/error.rs`
- Modify: `crates/api/src/error.rs`

**Step 1: Add variant to core error**

In `crates/core/src/error.rs`, add to `AppError` enum:

```rust
#[error("unauthorized")]
Unauthorized,
#[error("forbidden")]
Forbidden,
```

**Step 2: Handle in ApiError response**

In `crates/api/src/error.rs`, add match arms:

```rust
AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_string()),
```

**Step 3: Verify compilation**

Run: `cargo build --workspace`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/core/src/error.rs crates/api/src/error.rs
git commit -m "feat: add Unauthorized and Forbidden error variants"
```

---

### Task 5: Implement basic auth extractor

**Files:**

- Create: `crates/api/src/auth.rs`

**Step 1: Write the auth extractor**

`crates/api/src/auth.rs`:

```rust
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
};
use base64::Engine;
use rflow_core::AppError;

use crate::error::ApiError;

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub username: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        if !header_value.starts_with("Basic ") {
            return Err(AppError::Unauthorized.into());
        }

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&header_value[6..])
            .map_err(|_| AppError::Unauthorized)?;

        let credentials = String::from_utf8(decoded).map_err(|_| AppError::Unauthorized)?;
        let (username, password) = credentials
            .split_once(':')
            .ok_or(AppError::Unauthorized)?;

        let expected_user = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
        let expected_pass = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "changeme".into());

        if username != expected_user || password != expected_pass {
            return Err(AppError::Unauthorized.into());
        }

        Ok(AuthUser {
            username: username.to_string(),
        })
    }
}
```

**Step 2: Add `base64` dependency**

In workspace root `Cargo.toml`:

```toml
base64 = "0.22"
```

In `crates/api/Cargo.toml`:

```toml
base64.workspace = true
```

**Step 3: Register module in main.rs**

Add `mod auth;` to `crates/api/src/main.rs`.

**Step 4: Verify compilation**

Run: `cargo build -p rflow-api`
Expected: PASS (unused warning for `AuthUser` is fine)

**Step 5: Commit**

```bash
git add crates/api/src/auth.rs crates/api/src/main.rs crates/api/Cargo.toml Cargo.toml Cargo.lock
git commit -m "feat: implement Basic auth extractor from env credentials"
```

---

### Task 6: Initialize Casbin enforcer in AppState

**Files:**

- Modify: `crates/api/src/state.rs`
- Modify: `crates/api/src/main.rs`

**Step 1: Add enforcer to AppState**

`crates/api/src/state.rs`:

```rust
use casbin::Enforcer;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub enforcer: Arc<RwLock<Enforcer>>,
}
```

**Step 2: Initialize enforcer in main.rs**

In `main()`, after creating the pool:

```rust
use casbin::Enforcer;
use sqlx_adapter::SqlxAdapter;
use std::sync::Arc;
use tokio::sync::RwLock;

let adapter = SqlxAdapter::new(&database_url, 8).await.unwrap();
let enforcer = Enforcer::new("config/casbin_model.conf", adapter).await.unwrap();
let enforcer = Arc::new(RwLock::new(enforcer));

let state = AppState { pool, enforcer };
```

**Step 3: Verify compilation**

Run: `cargo build -p rflow-api`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/api/src/state.rs crates/api/src/main.rs
git commit -m "feat: initialize Casbin enforcer in AppState"
```

---

### Task 7: Add Casbin authorization middleware

**Files:**

- Create: `crates/api/src/middleware.rs`
- Modify: `crates/api/src/main.rs`

**Step 1: Write the middleware**

`crates/api/src/middleware.rs`:

```rust
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::auth::AuthUser;
use crate::state::AppState;

pub async fn casbin_auth(
    State(state): State<AppState>,
    auth_user: AuthUser,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    let enforcer = state.enforcer.read().await;
    let authorized = enforcer
        .enforce(vec![
            auth_user.username.into(),
            path.into(),
            method.into(),
        ])
        .unwrap_or(false);

    if !authorized {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
```

**Step 2: Apply middleware to protected routes**

In `main.rs`, wrap the `/api/*` routes:

```rust
use axum::middleware as axum_mw;

let protected = Router::new()
    .route(
        "/api/projects",
        get(routes::projects::list).post(routes::projects::create),
    )
    .route(
        "/api/projects/{id}",
        get(routes::projects::get)
            .patch(routes::projects::update)
            .delete(routes::projects::delete),
    )
    .route_layer(axum_mw::from_fn_with_state(state.clone(), middleware::casbin_auth));

let app = Router::new()
    .route("/health", get(|| async { "ok" }))
    .merge(protected)
    .with_state(state)
    .layer(TraceLayer::new_for_http());
```

**Step 3: Register module**

Add `mod middleware;` to `main.rs`.

**Step 4: Verify compilation**

Run: `cargo build -p rflow-api`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/api/src/middleware.rs crates/api/src/main.rs
git commit -m "feat: add Casbin authorization middleware to protected routes"
```

---

### Task 8: Add policy seeding on startup

**Files:**

- Modify: `crates/api/src/main.rs`

**Step 1: Seed default admin policy if empty**

After creating the enforcer, add policy seeding:

```rust
{
    let mut e = enforcer.write().await;
    let policies = e.get_policy();
    if policies.is_empty() {
        e.add_policy(vec![
            "admin".into(), "/api/*".into(), "GET".into(),
        ]).await.unwrap();
        e.add_policy(vec![
            "admin".into(), "/api/*".into(), "POST".into(),
        ]).await.unwrap();
        e.add_policy(vec![
            "admin".into(), "/api/*".into(), "PATCH".into(),
        ]).await.unwrap();
        e.add_policy(vec![
            "admin".into(), "/api/*".into(), "DELETE".into(),
        ]).await.unwrap();
        tracing::info!("Seeded default admin policies");
    }
}
```

**Step 2: Verify compilation**

Run: `cargo build -p rflow-api`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/api/src/main.rs
git commit -m "feat: seed default Casbin policies on startup"
```

---

### Task 9: Write integration test for auth

**Files:**

- Modify: `crates/api/tests/projects_test.rs`

**Step 1: Write failing test (no auth → 401)**

Add to `projects_test.rs`:

```rust
#[tokio::test]
async fn test_unauthenticated_request_returns_401() {
    let client = Client::new();
    let base = base_url().await;

    let res = client
        .get(format!("{base}/api/projects"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn test_authenticated_crud() {
    let client = Client::new();
    let base = base_url().await;
    let user = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
    let pass = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "changeme".into());

    let res = client
        .post(format!("{base}/api/projects"))
        .basic_auth(&user, Some(&pass))
        .json(&json!({"name": "Auth Test Project"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let project: Value = res.json().await.unwrap();
    let id = project["id"].as_str().unwrap();

    // Cleanup
    let res = client
        .delete(format!("{base}/api/projects/{id}"))
        .basic_auth(&user, Some(&pass))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);
}
```

**Step 2: Update existing test to include auth**

Update `test_project_crud` to add `.basic_auth("admin", Some("changeme"))` to all requests.

**Step 3: Run tests**

Start server with env vars, then:

Run: `TEST_API_URL=http://localhost:4010 cargo test -p rflow-api -- --nocapture`
Expected: all tests PASS

**Step 4: Commit**

```bash
git add crates/api/tests/projects_test.rs
git commit -m "test: add auth integration tests for Casbin RBAC"
```

---

### Task 10: Update docker-compose with env_file

**Files:**

- Modify: `docker-compose.yml`

**Step 1: Reference .env in docker-compose**

```yaml
services:
  postgres:
    image: postgres:16-alpine
    env_file: .env
    environment:
      POSTGRES_USER: ${POSTGRES_USER:-rflow}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-rflow}
      POSTGRES_DB: ${POSTGRES_DB:-rflow}
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
```

**Step 2: Update .env.example with postgres vars**

```env
DATABASE_URL=postgres://rflow:rflow@localhost:5432/rflow
POSTGRES_USER=rflow
POSTGRES_PASSWORD=rflow
POSTGRES_DB=rflow
DATA_DIR=./data
RSCRIPT_PATH=/usr/bin/Rscript
RUST_LOG=info,rflow_api=debug
LISTEN_ADDR=0.0.0.0:4001
ADMIN_USERNAME=admin
ADMIN_PASSWORD=changeme
```

**Step 3: Commit**

```bash
git add docker-compose.yml .env.example
git commit -m "chore: externalize all credentials to .env"
```

---

## Execution Batches

| Batch | Tasks | Focus |
|-------|-------|-------|
| 1 | 1-3 | Env + deps + config files |
| 2 | 4-6 | Error variants + auth extractor + enforcer init |
| 3 | 7-8 | Middleware + policy seeding |
| 4 | 9-10 | Tests + docker-compose cleanup |

## Notes

- `axum-casbin` crate provides its own middleware, but we write a custom one for tighter control over error responses and to avoid version coupling issues with Axum 0.8. If `axum-casbin` is confirmed compatible with Axum 0.8, it can replace the custom middleware.
- The `sqlx-adapter` auto-creates a `casbin_rule` table in the database — no manual migration needed.
- Password comparison uses constant-time string comparison via `argon2` in production. For the initial implementation, plain env comparison is acceptable for an internal tool; upgrade to hashed passwords when user management is added.
