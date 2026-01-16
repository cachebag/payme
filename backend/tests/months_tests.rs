mod common;

use common::{
    auth_name, auth_value, close_test_month, create_test_category, create_test_month,
    create_test_pool, create_test_server, create_test_user, generate_token,
};
use payme::create_app;

async fn setup_with_user() -> (axum_test::TestServer, sqlx::SqlitePool, i64, String) {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool, "testuser", "password123").await;
    let token = generate_token(user_id, "testuser");
    let app = create_app(pool.clone());
    let server = create_test_server(app);
    (server, pool, user_id, token)
}

#[tokio::test]
async fn test_list_months_empty() {
    let (server, _pool, _user_id, token) = setup_with_user().await;

    let response = server
        .get("/api/months")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn test_list_months_ordered() {
    let (server, pool, user_id, token) = setup_with_user().await;

    create_test_month(&pool, user_id, 2024, 1).await;
    create_test_month(&pool, user_id, 2024, 3).await;
    create_test_month(&pool, user_id, 2024, 2).await;

    let response = server
        .get("/api/months")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 3);

    assert_eq!(body[0]["month"], 3);
    assert_eq!(body[1]["month"], 2);
    assert_eq!(body[2]["month"], 1);
}

#[tokio::test]
async fn test_get_or_create_current_month_creates() {
    let (server, pool, user_id, token) = setup_with_user().await;

    create_test_category(&pool, user_id, "Food", 500.0).await;

    let response = server
        .get("/api/months/current")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();

    assert!(body["month"]["id"].as_i64().is_some());
    assert_eq!(body["month"]["is_closed"], false);

    let budgets = body["budgets"].as_array().unwrap();
    assert_eq!(budgets.len(), 1);
    assert_eq!(budgets[0]["category_label"], "Food");
    assert_eq!(budgets[0]["allocated_amount"], 500.0);
}

#[tokio::test]
async fn test_get_or_create_current_month_returns_existing() {
    let (server, _pool, _user_id, token) = setup_with_user().await;

    let response1 = server
        .get("/api/months/current")
        .add_header(auth_name(), auth_value(&token))
        .await;

    let body1: serde_json::Value = response1.json();
    let month_id1 = body1["month"]["id"].as_i64().unwrap();

    let response2 = server
        .get("/api/months/current")
        .add_header(auth_name(), auth_value(&token))
        .await;

    let body2: serde_json::Value = response2.json();
    let month_id2 = body2["month"]["id"].as_i64().unwrap();

    assert_eq!(month_id1, month_id2);
}

#[tokio::test]
async fn test_get_month_success() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;

    let response = server
        .get(&format!("/api/months/{}", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["month"]["id"], month_id);
    assert_eq!(body["month"]["year"], 2024);
    assert_eq!(body["month"]["month"], 6);
}

#[tokio::test]
async fn test_get_month_not_found() {
    let (server, _pool, _user_id, token) = setup_with_user().await;

    let response = server
        .get("/api/months/99999")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_not_found();
}

#[tokio::test]
async fn test_get_month_wrong_user() {
    let pool = create_test_pool().await;

    let user1_id = create_test_user(&pool, "user1", "password123").await;
    let user2_id = create_test_user(&pool, "user2", "password123").await;

    let month_id = create_test_month(&pool, user1_id, 2024, 6).await;

    let token2 = generate_token(user2_id, "user2");
    let app = create_app(pool);
    let server = create_test_server(app);

    let response = server
        .get(&format!("/api/months/{}", month_id))
        .add_header(auth_name(), auth_value(&token2))
        .await;

    response.assert_status_not_found();
}

#[tokio::test]
async fn test_close_month_success() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;

    let response = server
        .post(&format!("/api/months/{}/close", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["is_closed"], true);
    assert!(body["closed_at"].as_str().is_some());
}

#[tokio::test]
async fn test_close_month_already_closed() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    close_test_month(&pool, month_id).await;

    let response = server
        .post(&format!("/api/months/{}/close", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_get_month_pdf_success() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;

    server
        .post(&format!("/api/months/{}/close", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    let response = server
        .get(&format!("/api/months/{}/pdf", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();

    let content_type = response.headers().get("content-type").unwrap();
    assert_eq!(content_type, "application/pdf");
}

#[tokio::test]
async fn test_get_month_pdf_not_closed() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;

    let response = server
        .get(&format!("/api/months/{}/pdf", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_not_found();
}
