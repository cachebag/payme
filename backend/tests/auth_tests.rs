mod common;

use common::{
    auth_name, auth_value, create_test_pool, create_test_server, create_test_user, generate_token,
};
use payme::create_app;
use serde_json::json;

async fn setup() -> axum_test::TestServer {
    let pool = create_test_pool().await;
    let app = create_app(pool);
    create_test_server(app)
}

async fn setup_with_user() -> (axum_test::TestServer, i64, String) {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool, "testuser", "password123").await;
    let token = generate_token(user_id, "testuser");
    let app = create_app(pool);
    let server = create_test_server(app);
    (server, user_id, token)
}

#[tokio::test]
async fn test_register_success() {
    let server = setup().await;

    let response = server
        .post("/api/auth/register")
        .json(&json!({
            "username": "newuser",
            "password": "password123"
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["username"], "newuser");
    assert!(body["id"].as_i64().is_some());
}

#[tokio::test]
async fn test_register_duplicate_username() {
    let pool = create_test_pool().await;
    create_test_user(&pool, "existinguser", "password123").await;
    let app = create_app(pool);
    let server = create_test_server(app);

    let response = server
        .post("/api/auth/register")
        .json(&json!({
            "username": "existinguser",
            "password": "password456"
        }))
        .await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
async fn test_register_validation_short_username() {
    let server = setup().await;

    let response = server
        .post("/api/auth/register")
        .json(&json!({
            "username": "ab",
            "password": "password123"
        }))
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_register_validation_short_password() {
    let server = setup().await;

    let response = server
        .post("/api/auth/register")
        .json(&json!({
            "username": "validuser",
            "password": "12345"
        }))
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_login_success() {
    let pool = create_test_pool().await;
    create_test_user(&pool, "loginuser", "password123").await;
    let app = create_app(pool);
    let server = create_test_server(app);

    let response = server
        .post("/api/auth/login")
        .json(&json!({
            "username": "loginuser",
            "password": "password123"
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["username"], "loginuser");

    let cookies = response.cookies();
    assert!(cookies.iter().any(|c| c.name() == "token"));
}

#[tokio::test]
async fn test_login_invalid_username() {
    let server = setup().await;

    let response = server
        .post("/api/auth/login")
        .json(&json!({
            "username": "nonexistent",
            "password": "password123"
        }))
        .await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_login_invalid_password() {
    let pool = create_test_pool().await;
    create_test_user(&pool, "loginuser", "password123").await;
    let app = create_app(pool);
    let server = create_test_server(app);

    let response = server
        .post("/api/auth/login")
        .json(&json!({
            "username": "loginuser",
            "password": "wrongpassword"
        }))
        .await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_logout() {
    let (server, _user_id, token) = setup_with_user().await;

    let response = server
        .post("/api/auth/logout")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();

    let cookies = response.cookies();
    let token_cookie = cookies.iter().find(|c| c.name() == "token");
    if let Some(cookie) = token_cookie {
        assert!(cookie
            .max_age()
            .map(|d| d.whole_seconds() <= 0)
            .unwrap_or(true));
    }
}

#[tokio::test]
async fn test_me_authenticated() {
    let (server, user_id, token) = setup_with_user().await;

    let response = server
        .get("/api/auth/me")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["id"], user_id);
    assert_eq!(body["username"], "testuser");
}

#[tokio::test]
async fn test_me_unauthenticated() {
    let server = setup().await;

    let response = server.get("/api/auth/me").await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_change_username_success() {
    let (server, user_id, token) = setup_with_user().await;

    let response = server
        .put("/api/auth/change-username")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "new_username": "newusername"
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["id"], user_id);
    assert_eq!(body["username"], "newusername");
}

#[tokio::test]
async fn test_change_username_duplicate() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool, "user1", "password123").await;
    create_test_user(&pool, "user2", "password123").await;
    let token = generate_token(user_id, "user1");
    let app = create_app(pool);
    let server = create_test_server(app);

    let response = server
        .put("/api/auth/change-username")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "new_username": "user2"
        }))
        .await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
async fn test_change_password_success() {
    let (server, _user_id, token) = setup_with_user().await;

    let response = server
        .put("/api/auth/change-password")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "current_password": "password123",
            "new_password": "newpassword456"
        }))
        .await;

    response.assert_status_ok();
}

#[tokio::test]
async fn test_change_password_wrong_current() {
    let (server, _user_id, token) = setup_with_user().await;

    let response = server
        .put("/api/auth/change-password")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "current_password": "wrongpassword",
            "new_password": "newpassword456"
        }))
        .await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_clear_all_data() {
    let (server, _user_id, token) = setup_with_user().await;

    let response = server
        .delete("/api/auth/clear-data")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "password": "password123"
        }))
        .await;

    response.assert_status_ok();

    let me_response = server
        .get("/api/auth/me")
        .add_header(auth_name(), auth_value(&token))
        .await;

    me_response.assert_status_not_found();
}

#[tokio::test]
async fn test_clear_all_data_wrong_password() {
    let (server, _user_id, token) = setup_with_user().await;

    let response = server
        .delete("/api/auth/clear-data")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "password": "wrongpassword"
        }))
        .await;

    response.assert_status_unauthorized();
}
