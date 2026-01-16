mod common;

use common::{
    auth_name, auth_value, create_test_pool, create_test_server, create_test_user,
    generate_expired_token, generate_token,
};
use payme::create_app;

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
async fn test_auth_from_cookie() {
    use axum_test::TestServerConfig;

    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool, "testuser", "password123").await;
    let app = create_app(pool);

    let mut config = TestServerConfig::new();
    config.save_cookies = true;
    let server = config.build(app).unwrap();

    let login_response = server
        .post("/api/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "password123"
        }))
        .await;

    login_response.assert_status_ok();

    let me_response = server.get("/api/auth/me").await;

    me_response.assert_status_ok();
    let body: serde_json::Value = me_response.json();
    assert_eq!(body["id"], user_id);
}

#[tokio::test]
async fn test_auth_from_bearer() {
    let (server, user_id, token) = setup_with_user().await;

    let response = server
        .get("/api/auth/me")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["id"], user_id);
}

#[tokio::test]
async fn test_auth_expired_token() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool, "testuser", "password123").await;
    let expired_token = generate_expired_token(user_id, "testuser");
    let app = create_app(pool);
    let server = create_test_server(app);

    let response = server
        .get("/api/auth/me")
        .add_header(auth_name(), auth_value(&expired_token))
        .await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_auth_invalid_token() {
    let server = setup().await;

    let response = server
        .get("/api/auth/me")
        .add_header(auth_name(), auth_value("invalid.token.here"))
        .await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_auth_missing_token() {
    let server = setup().await;

    let response = server.get("/api/auth/me").await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_health_check_no_auth() {
    let server = setup().await;

    let response = server.get("/health").await;

    response.assert_status_ok();
}

#[tokio::test]
async fn test_register_no_auth() {
    let server = setup().await;

    let response = server
        .post("/api/auth/register")
        .json(&serde_json::json!({
            "username": "newuser",
            "password": "password123"
        }))
        .await;

    response.assert_status_ok();
}

#[tokio::test]
async fn test_login_no_auth() {
    let pool = create_test_pool().await;
    create_test_user(&pool, "testuser", "password123").await;
    let app = create_app(pool);
    let server = create_test_server(app);

    let response = server
        .post("/api/auth/login")
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "password123"
        }))
        .await;

    response.assert_status_ok();
}
