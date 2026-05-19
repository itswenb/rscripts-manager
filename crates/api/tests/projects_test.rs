use reqwest::Client;
use serde_json::{json, Value};

async fn base_url() -> String {
    std::env::var("TEST_API_URL").unwrap_or_else(|_| "http://localhost:4010".into())
}

#[tokio::test]
async fn test_project_crud() {
    let client = Client::new();
    let base = base_url().await;

    // Create
    let res = client
        .post(format!("{base}/api/projects"))
        .json(&json!({"name": "Integration Test Project", "description": "testing"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let project: Value = res.json().await.unwrap();
    let id = project["id"].as_str().unwrap();

    // Get
    let res = client
        .get(format!("{base}/api/projects/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let fetched: Value = res.json().await.unwrap();
    assert_eq!(fetched["name"], "Integration Test Project");

    // Update
    let res = client
        .patch(format!("{base}/api/projects/{id}"))
        .json(&json!({"name": "Updated Name"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let updated: Value = res.json().await.unwrap();
    assert_eq!(updated["name"], "Updated Name");

    // List
    let res = client
        .get(format!("{base}/api/projects"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let list: Vec<Value> = res.json().await.unwrap();
    assert!(!list.is_empty());

    // Delete
    let res = client
        .delete(format!("{base}/api/projects/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Verify deleted
    let res = client
        .get(format!("{base}/api/projects/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}
