mod common;

use common::{
    auth_name, auth_value, create_test_pool, create_test_server, create_test_user, generate_token,
};
use payme::create_app;
use serde_json::json;

async fn setup_with_user() -> (axum_test::TestServer, i64, String) {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool, "testuser", "password123").await;
    let token = generate_token(user_id, "testuser");
    let app = create_app(pool);
    let server = create_test_server(app);
    (server, user_id, token)
}

#[tokio::test]
async fn test_get_savings() {
    let (server, _user_id, token) = setup_with_user().await;

    let response = server
        .get("/api/savings")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["savings"], 0.0);
    assert_eq!(body["savings_goal"], 0.0);
}

#[tokio::test]
async fn test_update_savings() {
    let (server, _user_id, token) = setup_with_user().await;

    let response = server
        .put("/api/savings")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "savings": 10000.0
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["savings"], 10000.0);
}

#[tokio::test]
async fn test_update_savings_goal() {
    let (server, _user_id, token) = setup_with_user().await;

    let response = server
        .put("/api/savings/goal")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "savings_goal": 50000.0
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["savings_goal"], 50000.0);
}

#[tokio::test]
async fn test_get_retirement_savings() {
    let (server, _user_id, token) = setup_with_user().await;

    let response = server
        .get("/api/retirement-savings")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["retirement_savings"], 0.0);
}

#[tokio::test]
async fn test_update_retirement_savings() {
    let (server, _user_id, token) = setup_with_user().await;

    let response = server
        .put("/api/retirement-savings")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "retirement_savings": 25000.0
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["retirement_savings"], 25000.0);
}

#[tokio::test]
async fn test_savings_validation() {
    let (server, _user_id, token) = setup_with_user().await;

    let response = server
        .put("/api/savings")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "savings": -100.0
        }))
        .await;

    response.assert_status_bad_request();
}
