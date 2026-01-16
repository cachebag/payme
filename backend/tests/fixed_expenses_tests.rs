mod common;

use common::{
    auth_name, auth_value, create_test_fixed_expense, create_test_pool, create_test_server,
    create_test_user, generate_token,
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
async fn test_list_fixed_expenses() {
    let (server, pool, user_id, token) = setup_with_user().await;

    create_test_fixed_expense(&pool, user_id, "Rent", 1500.0).await;
    create_test_fixed_expense(&pool, user_id, "Internet", 80.0).await;

    let response = server
        .get("/api/fixed-expenses")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 2);
}

#[tokio::test]
async fn test_create_fixed_expense() {
    let (server, _pool, _user_id, token) = setup_with_user().await;

    let response = server
        .post("/api/fixed-expenses")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "label": "Electricity",
            "amount": 120.0
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["label"], "Electricity");
    assert_eq!(body["amount"], 120.0);
    assert!(body["id"].as_i64().is_some());
}

#[tokio::test]
async fn test_update_fixed_expense() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let expense_id = create_test_fixed_expense(&pool, user_id, "Rent", 1500.0).await;

    let response = server
        .put(&format!("/api/fixed-expenses/{}", expense_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "label": "Rent Updated",
            "amount": 1600.0
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["label"], "Rent Updated");
    assert_eq!(body["amount"], 1600.0);
}

#[tokio::test]
async fn test_update_fixed_expense_not_found() {
    let (server, _pool, _user_id, token) = setup_with_user().await;

    let response = server
        .put("/api/fixed-expenses/99999")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "label": "Updated"
        }))
        .await;

    response.assert_status_not_found();
}

#[tokio::test]
async fn test_delete_fixed_expense() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let expense_id = create_test_fixed_expense(&pool, user_id, "Rent", 1500.0).await;

    let response = server
        .delete(&format!("/api/fixed-expenses/{}", expense_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status(axum::http::StatusCode::NO_CONTENT);

    let list_response = server
        .get("/api/fixed-expenses")
        .add_header(auth_name(), auth_value(&token))
        .await;

    let body: Vec<serde_json::Value> = list_response.json();
    assert!(body.is_empty());
}
