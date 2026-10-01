mod common;

use axum::http::StatusCode;
use common::{
    auth_name, auth_value, close_test_month, create_test_category, create_test_item,
    create_test_month, create_test_pool, create_test_server, create_test_user, generate_token,
};
use payme::create_app;
use serde_json::{json, Value};

async fn setup() -> (
    axum_test::TestServer,
    sqlx::SqlitePool,
    i64,
    String,
    i64,
    i64,
) {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool, "owner", "password123").await;
    let token = generate_token(user_id, "owner");
    let month_id = create_test_month(&pool, user_id, 2026, 10).await;
    let category_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    let server = create_test_server(create_app(pool.clone()));
    (server, pool, user_id, token, month_id, category_id)
}

async fn create_tag(server: &axum_test::TestServer, token: &str, label: &str) -> i64 {
    let response = server
        .post("/api/tags")
        .add_header(auth_name(), auth_value(token))
        .json(&json!({ "label": label, "color": "#3b82f6" }))
        .await;
    response.assert_status(StatusCode::CREATED);
    response.json::<Value>()["id"].as_i64().unwrap()
}

async fn create_spending_item(
    server: &axum_test::TestServer,
    token: &str,
    month_id: i64,
    category_id: i64,
    tag_ids: Vec<i64>,
) -> Value {
    server
        .post(&format!("/api/months/{month_id}/items"))
        .add_header(auth_name(), auth_value(token))
        .json(&json!({
            "category_id": category_id,
            "description": "Coffee",
            "amount": 5.0,
            "spent_on": "2026-10-01",
            "tag_ids": tag_ids
        }))
        .await
        .json()
}

#[tokio::test]
async fn test_create_list_and_month_summary_include_tags() {
    let (server, _pool, _user_id, token, month_id, category_id) = setup().await;
    let breakfast = create_tag(&server, &token, "Breakfast").await;
    let cafe = create_tag(&server, &token, "Cafe").await;

    let created = create_spending_item(
        &server,
        &token,
        month_id,
        category_id,
        vec![breakfast, cafe],
    )
    .await;
    assert_eq!(created["tags"].as_array().unwrap().len(), 2);

    let listed: Vec<Value> = server
        .get(&format!("/api/months/{month_id}/items"))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();
    assert_eq!(listed[0]["tags"][0]["label"], "Breakfast");
    assert_eq!(listed[0]["tags"][1]["label"], "Cafe");

    let summary: Value = server
        .get(&format!("/api/months/{month_id}"))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();
    assert_eq!(summary["items"][0]["tags"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_update_omitted_tags_preserves_and_empty_clears() {
    let (server, _pool, _user_id, token, month_id, category_id) = setup().await;
    let tag_id = create_tag(&server, &token, "Cafe").await;
    let created = create_spending_item(&server, &token, month_id, category_id, vec![tag_id]).await;
    let item_id = created["id"].as_i64().unwrap();

    let preserved: Value = server
        .put(&format!("/api/months/{month_id}/items/{item_id}"))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({ "description": "Iced coffee" }))
        .await
        .json();
    assert_eq!(preserved["tags"].as_array().unwrap().len(), 1);

    let cleared: Value = server
        .put(&format!("/api/months/{month_id}/items/{item_id}"))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({ "tag_ids": [] }))
        .await
        .json();
    assert!(cleared["tags"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_assignment_validation_and_atomic_update() {
    let (server, pool, _user_id, token, month_id, category_id) = setup().await;
    let tag_id = create_tag(&server, &token, "Cafe").await;
    let created = create_spending_item(&server, &token, month_id, category_id, vec![tag_id]).await;
    let item_id = created["id"].as_i64().unwrap();

    let other_id = create_test_user(&pool, "other", "password123").await;
    let foreign_tag: i64 = sqlx::query_scalar(
        "INSERT INTO tags (user_id, label, normalized_label, color) VALUES (?, 'Other', 'other', '#71717a') RETURNING id",
    )
    .bind(other_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    for tag_ids in [vec![tag_id, tag_id], vec![999_999], vec![foreign_tag]] {
        server
            .put(&format!("/api/months/{month_id}/items/{item_id}"))
            .add_header(auth_name(), auth_value(&token))
            .json(&json!({ "description": "Must not change", "tag_ids": tag_ids }))
            .await
            .assert_status_bad_request();
    }

    let mut six = Vec::new();
    for index in 0..6 {
        six.push(create_tag(&server, &token, &format!("Tag {index}")).await);
    }
    server
        .put(&format!("/api/months/{month_id}/items/{item_id}"))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({ "description": "Must not change", "tag_ids": six }))
        .await
        .assert_status_bad_request();

    let item: (String,) = sqlx::query_as("SELECT description FROM items WHERE id = ?")
        .bind(item_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(item.0, "Coffee");
    let assignment_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tag_assignments WHERE item_id = ?")
            .bind(item_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(assignment_count, 1);
}

#[tokio::test]
async fn test_stopped_tag_stays_visible_can_be_removed_but_not_newly_assigned() {
    let (server, _pool, _user_id, token, month_id, category_id) = setup().await;
    let tag_id = create_tag(&server, &token, "Cafe").await;
    let first = create_spending_item(&server, &token, month_id, category_id, vec![tag_id]).await;
    let first_id = first["id"].as_i64().unwrap();

    server
        .post(&format!("/api/tags/{tag_id}/stop"))
        .add_header(auth_name(), auth_value(&token))
        .await
        .assert_status_ok();

    let listed: Vec<Value> = server
        .get(&format!("/api/months/{month_id}/items"))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();
    assert_eq!(listed[0]["tags"][0]["stopped"], true);

    server
        .post(&format!("/api/months/{month_id}/items"))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "category_id": category_id,
            "description": "Tea",
            "amount": 4.0,
            "spent_on": "2026-10-02",
            "tag_ids": [tag_id]
        }))
        .await
        .assert_status_bad_request();

    server
        .put(&format!("/api/months/{month_id}/items/{first_id}"))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({ "tag_ids": [] }))
        .await
        .assert_status_ok();
}

#[tokio::test]
async fn test_transfers_and_closed_month_reject_assignments() {
    let (server, pool, _user_id, token, month_id, category_id) = setup().await;
    let tag_id = create_tag(&server, &token, "Cafe").await;

    server
        .post(&format!("/api/months/{month_id}/items"))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({
            "description": "Move money",
            "amount": 10.0,
            "spent_on": "2026-10-01",
            "savings_destination": "savings",
            "tag_ids": [tag_id]
        }))
        .await
        .assert_status_bad_request();

    let item_id = create_test_item(&pool, month_id, category_id, "Coffee", 5.0, "2026-10-01").await;
    close_test_month(&pool, month_id).await;
    server
        .put(&format!("/api/months/{month_id}/items/{item_id}"))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({ "tag_ids": [tag_id] }))
        .await
        .assert_status_bad_request();
}

#[tokio::test]
async fn test_deleting_item_cascades_assignments_not_tags() {
    let (server, pool, _user_id, token, month_id, category_id) = setup().await;
    let tag_id = create_tag(&server, &token, "Cafe").await;
    let created = create_spending_item(&server, &token, month_id, category_id, vec![tag_id]).await;
    let item_id = created["id"].as_i64().unwrap();

    server
        .delete(&format!("/api/months/{month_id}/items/{item_id}"))
        .add_header(auth_name(), auth_value(&token))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let assignments: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tag_assignments")
        .fetch_one(&pool)
        .await
        .unwrap();
    let tags: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE id = ?")
        .bind(tag_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(assignments, 0);
    assert_eq!(tags, 1);
}
