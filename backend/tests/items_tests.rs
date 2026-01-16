mod common;

use common::{
    auth_name, auth_value, close_test_month, create_test_category, create_test_item,
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
async fn test_list_items() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    create_test_item(&pool, month_id, cat_id, "Groceries", 150.0, "2024-06-15").await;
    create_test_item(&pool, month_id, cat_id, "Restaurant", 50.0, "2024-06-16").await;

    let response = server
        .get(&format!("/api/months/{}/items", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 2);
    assert!(body[0]["category_label"].as_str().is_some());
}

#[tokio::test]
async fn test_create_item() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;

    let response = server
        .post(&format!("/api/months/{}/items", month_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "category_id": cat_id,
            "description": "Coffee",
            "amount": 5.0,
            "spent_on": "2024-06-15"
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["description"], "Coffee");
    assert_eq!(body["amount"], 5.0);
    assert_eq!(body["category_id"], cat_id);
}

#[tokio::test]
async fn test_create_item_invalid_category() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;

    let response = server
        .post(&format!("/api/months/{}/items", month_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "category_id": 99999,
            "description": "Coffee",
            "amount": 5.0,
            "spent_on": "2024-06-15"
        }))
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_create_item_closed_month() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    close_test_month(&pool, month_id).await;

    let response = server
        .post(&format!("/api/months/{}/items", month_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "category_id": cat_id,
            "description": "Coffee",
            "amount": 5.0,
            "spent_on": "2024-06-15"
        }))
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_update_item() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    let item_id = create_test_item(&pool, month_id, cat_id, "Groceries", 150.0, "2024-06-15").await;

    let response = server
        .put(&format!("/api/months/{}/items/{}", month_id, item_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "description": "Weekly Groceries",
            "amount": 175.0
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["description"], "Weekly Groceries");
    assert_eq!(body["amount"], 175.0);
}

#[tokio::test]
async fn test_update_item_change_category() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id1 = create_test_category(&pool, user_id, "Food", 500.0).await;
    let cat_id2 = create_test_category(&pool, user_id, "Entertainment", 300.0).await;
    let item_id = create_test_item(&pool, month_id, cat_id1, "Dinner", 50.0, "2024-06-15").await;

    let response = server
        .put(&format!("/api/months/{}/items/{}", month_id, item_id))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "category_id": cat_id2
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["category_id"], cat_id2);
}

#[tokio::test]
async fn test_delete_item() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    let item_id = create_test_item(&pool, month_id, cat_id, "Groceries", 150.0, "2024-06-15").await;

    let response = server
        .delete(&format!("/api/months/{}/items/{}", month_id, item_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status(axum::http::StatusCode::NO_CONTENT);

    let list_response = server
        .get(&format!("/api/months/{}/items", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    let body: Vec<serde_json::Value> = list_response.json();
    assert!(body.is_empty());
}
