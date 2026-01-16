mod common;

use common::{
    auth_name, auth_value, close_test_month, create_test_budget, create_test_category,
    create_test_month, create_test_pool, create_test_server, create_test_user, generate_token,
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
async fn test_list_categories() {
    let (server, pool, user_id, token) = setup_with_user().await;

    create_test_category(&pool, user_id, "Food", 500.0).await;
    create_test_category(&pool, user_id, "Transport", 200.0).await;

    let response = server
        .get("/api/categories")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 2);
}

#[tokio::test]
async fn test_create_category() {
    let (server, pool, user_id, token) = setup_with_user().await;

    create_test_month(&pool, user_id, 2024, 6).await;

    let response = server
        .post("/api/categories")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "label": "Entertainment",
            "default_amount": 300.0
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["label"], "Entertainment");
    assert_eq!(body["default_amount"], 300.0);
    assert!(body["id"].as_i64().is_some());
}

#[tokio::test]
async fn test_create_category_validation() {
    let (server, _pool, _user_id, token) = setup_with_user().await;

    let response = server
        .post("/api/categories")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "label": "",
            "default_amount": 300.0
        }))
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_update_category() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;

    let response = server
        .put(&format!("/api/categories/{}", cat_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "label": "Groceries",
            "default_amount": 600.0
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["label"], "Groceries");
    assert_eq!(body["default_amount"], 600.0);
}

#[tokio::test]
async fn test_update_category_not_found() {
    let (server, _pool, _user_id, token) = setup_with_user().await;

    let response = server
        .put("/api/categories/99999")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "label": "Groceries"
        }))
        .await;

    response.assert_status_not_found();
}

#[tokio::test]
async fn test_delete_category() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;

    let response = server
        .delete(&format!("/api/categories/{}", cat_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status(axum::http::StatusCode::NO_CONTENT);

    let list_response = server
        .get("/api/categories")
        .add_header(auth_name(), auth_value(&token))
        .await;

    let body: Vec<serde_json::Value> = list_response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn test_list_monthly_budgets() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    create_test_budget(&pool, month_id, cat_id, 500.0).await;

    let response = server
        .get(&format!("/api/months/{}/budgets", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["allocated_amount"], 500.0);
}

#[tokio::test]
async fn test_update_monthly_budget() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    let budget_id = create_test_budget(&pool, month_id, cat_id, 500.0).await;

    let response = server
        .put(&format!("/api/months/{}/budgets/{}", month_id, budget_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "allocated_amount": 750.0
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["allocated_amount"], 750.0);
}

#[tokio::test]
async fn test_update_monthly_budget_closed_month() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    let budget_id = create_test_budget(&pool, month_id, cat_id, 500.0).await;
    close_test_month(&pool, month_id).await;

    let response = server
        .put(&format!("/api/months/{}/budgets/{}", month_id, budget_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "allocated_amount": 750.0
        }))
        .await;

    response.assert_status_bad_request();
}
