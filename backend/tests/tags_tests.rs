mod common;

use axum::http::StatusCode;
use common::{
    auth_name, auth_value, create_test_category, create_test_item, create_test_month,
    create_test_pool, create_test_server, create_test_user, generate_token,
};
use payme::create_app;
use serde_json::{json, Value};

async fn setup() -> (axum_test::TestServer, sqlx::SqlitePool, i64, String) {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool, "owner", "password123").await;
    let token = generate_token(user_id, "owner");
    let server = create_test_server(create_app(pool.clone()));
    (server, pool, user_id, token)
}

async fn create_tag(server: &axum_test::TestServer, token: &str, label: &str) -> Value {
    let response = server
        .post("/api/tags")
        .add_header(auth_name(), auth_value(token))
        .json(&json!({ "label": label, "color": "#3b82f6" }))
        .await;
    response.assert_status(StatusCode::CREATED);
    response.json()
}

#[tokio::test]
async fn test_tag_validation_and_case_insensitive_collisions() {
    let (server, _pool, _user_id, token) = setup().await;

    let tag = create_tag(&server, &token, "  Breakfast  ").await;
    assert_eq!(tag["label"], "Breakfast");

    for label in ["", &"x".repeat(51)] {
        server
            .post("/api/tags")
            .add_header(auth_name(), auth_value(&token))
            .json(&json!({ "label": label, "color": "#3b82f6" }))
            .await
            .assert_status_bad_request();
    }

    server
        .post("/api/tags")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({ "label": "breakFAST", "color": "#f97316" }))
        .await
        .assert_status(StatusCode::CONFLICT);

    let lunch = create_tag(&server, &token, "Lunch").await;
    let lunch_id = lunch["id"].as_i64().unwrap();
    server
        .put(&format!("/api/tags/{lunch_id}"))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({ "label": "BREAKFAST" }))
        .await
        .assert_status(StatusCode::CONFLICT);
    let tags: Vec<Value> = server
        .get("/api/tags")
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();
    assert!(tags.iter().any(|tag| tag["label"] == "Lunch"));
}

#[tokio::test]
async fn test_stopped_collision_identifies_restorable_tag() {
    let (server, _pool, _user_id, token) = setup().await;
    let tag = create_tag(&server, &token, "Breakfast").await;
    let id = tag["id"].as_i64().unwrap();

    server
        .post(&format!("/api/tags/{id}/stop"))
        .add_header(auth_name(), auth_value(&token))
        .await
        .assert_status_ok();

    let response = server
        .post("/api/tags")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({ "label": " breakfast ", "color": "#f97316" }))
        .await;
    response.assert_status(StatusCode::CONFLICT);
    let body: Value = response.json();
    assert_eq!(body["restorable_tag_id"], id);
}

#[tokio::test]
async fn test_tag_operations_are_owner_scoped() {
    let (server, pool, _user_id, token) = setup().await;
    let other_id = create_test_user(&pool, "other", "password123").await;
    let other_token = generate_token(other_id, "other");
    let tag = create_tag(&server, &token, "Owner only").await;
    let id = tag["id"].as_i64().unwrap();

    server
        .put(&format!("/api/tags/{id}"))
        .add_header(auth_name(), auth_value(&other_token))
        .json(&json!({ "label": "Stolen" }))
        .await
        .assert_status_not_found();
    server
        .post(&format!("/api/tags/{id}/stop"))
        .add_header(auth_name(), auth_value(&other_token))
        .await
        .assert_status_not_found();

    let list: Vec<Value> = server
        .get("/api/tags")
        .add_header(auth_name(), auth_value(&other_token))
        .await
        .json();
    assert!(list.is_empty());
}

#[tokio::test]
async fn test_list_includes_usage_counts_and_stop_restore_lifecycle() {
    let (server, pool, user_id, token) = setup().await;
    let tag = create_tag(&server, &token, "Cafe").await;
    let id = tag["id"].as_i64().unwrap();
    let category_id = create_test_category(&pool, user_id, "Food", 100.0).await;
    let month_id = create_test_month(&pool, user_id, 2026, 10).await;
    let item_id = create_test_item(&pool, month_id, category_id, "Coffee", 4.0, "2026-10-01").await;
    sqlx::query("INSERT INTO tag_assignments (tag_id, item_id) VALUES (?, ?)")
        .bind(id)
        .bind(item_id)
        .execute(&pool)
        .await
        .unwrap();

    let updated: Value = server
        .put(&format!("/api/tags/{id}"))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({ "label": "Coffee", "color": "#f97316" }))
        .await
        .json();
    assert_eq!(updated["label"], "Coffee");
    assert_eq!(updated["color"], "#f97316");

    let stopped: Value = server
        .post(&format!("/api/tags/{id}/stop"))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();
    assert_eq!(stopped["stopped"], true);

    let tags: Vec<Value> = server
        .get("/api/tags")
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();
    assert_eq!(tags[0]["usage_count"], 1);
    assert_eq!(tags[0]["stopped"], true);

    let restored: Value = server
        .post(&format!("/api/tags/{id}/restore"))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();
    assert_eq!(restored["stopped"], false);
}
