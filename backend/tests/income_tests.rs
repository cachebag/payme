mod common;

use common::{
    auth_name, auth_value, close_test_month, create_test_income, create_test_month,
    create_test_pool, create_test_server, create_test_user, generate_token,
};
use payme::create_app;
use serde_json::json;

async fn setup_with_user() -> (axum_test::TestServer, sqlx::SqlitePool, i64, String) {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool, "testuser", "password123").await;
    let token = generate_token(user_id, "testuser");
    let app = create_app(pool.clone());
    let server = create_test_server(app);
    (server, pool, user_id, token)
}

#[tokio::test]
async fn test_list_income() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    create_test_income(&pool, month_id, "Salary", 5000.0).await;
    create_test_income(&pool, month_id, "Bonus", 1000.0).await;

    let response = server
        .get(&format!("/api/months/{}/income", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 2);
}

#[tokio::test]
async fn test_create_income() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;

    let response = server
        .post(&format!("/api/months/{}/income", month_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "label": "Freelance",
            "amount": 2000.0
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["label"], "Freelance");
    assert_eq!(body["amount"], 2000.0);
    assert!(body["id"].as_i64().is_some());
}

#[tokio::test]
async fn test_create_income_closed_month() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    close_test_month(&pool, month_id).await;

    let response = server
        .post(&format!("/api/months/{}/income", month_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "label": "Freelance",
            "amount": 2000.0
        }))
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_update_income() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let income_id = create_test_income(&pool, month_id, "Salary", 5000.0).await;

    let response = server
        .put(&format!("/api/months/{}/income/{}", month_id, income_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "label": "Main Salary",
            "amount": 5500.0
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["label"], "Main Salary");
    assert_eq!(body["amount"], 5500.0);
}

#[tokio::test]
async fn test_update_income_not_found() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;

    let response = server
        .put(&format!("/api/months/{}/income/99999", month_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "label": "Updated"
        }))
        .await;

    response.assert_status_not_found();
}

#[tokio::test]
async fn test_delete_income() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let income_id = create_test_income(&pool, month_id, "Salary", 5000.0).await;

    let response = server
        .delete(&format!("/api/months/{}/income/{}", month_id, income_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status(axum::http::StatusCode::NO_CONTENT);

    let list_response = server
        .get(&format!("/api/months/{}/income", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    let body: Vec<serde_json::Value> = list_response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn test_income_wrong_user_month() {
    let pool = create_test_pool().await;

    let user1_id = create_test_user(&pool, "user1", "password123").await;
    let user2_id = create_test_user(&pool, "user2", "password123").await;

    let month_id = create_test_month(&pool, user1_id, 2024, 6).await;

    let token2 = generate_token(user2_id, "user2");
    let app = create_app(pool);
    let server = create_test_server(app);

    let response = server
        .get(&format!("/api/months/{}/income", month_id))
        .add_header(auth_name(), auth_value(&token2))
        .await;

    response.assert_status_not_found();
}
