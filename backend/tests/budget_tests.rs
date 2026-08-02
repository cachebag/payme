mod common;

use common::{
    auth_name, auth_value, close_test_month, create_test_budget, create_test_category,
    create_test_item, create_test_month, create_test_pool, create_test_server, create_test_user,
    generate_token,
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

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    create_test_budget(&pool, month_id, cat_id, 500.0).await;

    let response = server
        .delete(&format!("/api/months/{}/categories/{}", month_id, cat_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status(axum::http::StatusCode::NO_CONTENT);

    let list_response = server
        .get("/api/categories")
        .add_header(auth_name(), auth_value(&token))
        .await;

    let body: Vec<serde_json::Value> = list_response.json();
    assert!(body.is_empty());

    let budgets: Vec<serde_json::Value> = server
        .get(&format!("/api/months/{}/budgets", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();
    assert!(budgets.is_empty(), "the month it was deleted from loses it");
}

#[tokio::test]
async fn test_delete_category_keeps_earlier_months_intact() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let august = create_test_month(&pool, user_id, 2024, 8).await;
    let september = create_test_month(&pool, user_id, 2024, 9).await;
    let cat_id = create_test_category(&pool, user_id, "Foo", 500.0).await;
    create_test_budget(&pool, august, cat_id, 500.0).await;
    create_test_budget(&pool, september, cat_id, 500.0).await;
    create_test_item(&pool, august, cat_id, "Coffee", 12.0, "2024-08-04").await;

    let response = server
        .delete(&format!("/api/months/{}/categories/{}", september, cat_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status(axum::http::StatusCode::NO_CONTENT);

    let august_summary: serde_json::Value = server
        .get(&format!("/api/months/{}", august))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();

    assert_eq!(
        august_summary["budgets"].as_array().unwrap().len(),
        1,
        "August keeps its budget line"
    );
    assert_eq!(
        august_summary["items"].as_array().unwrap().len(),
        1,
        "August keeps its transactions"
    );
    assert_eq!(august_summary["total_spent"], 12.0);

    let september_summary: serde_json::Value = server
        .get(&format!("/api/months/{}", september))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();

    assert!(
        september_summary["budgets"].as_array().unwrap().is_empty(),
        "September drops the category it was deleted from"
    );
}

#[tokio::test]
async fn test_delete_category_propagates_to_later_open_months_only() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let september = create_test_month(&pool, user_id, 2024, 9).await;
    let october = create_test_month(&pool, user_id, 2024, 10).await;
    let november = create_test_month(&pool, user_id, 2024, 11).await;
    let cat_id = create_test_category(&pool, user_id, "Foo", 500.0).await;
    for month_id in [september, october, november] {
        create_test_budget(&pool, month_id, cat_id, 500.0).await;
    }
    close_test_month(&pool, november).await;

    server
        .delete(&format!("/api/months/{}/categories/{}", september, cat_id))
        .add_header(auth_name(), auth_value(&token))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    for (month_id, label) in [(september, "September"), (october, "October")] {
        let budgets: Vec<serde_json::Value> = server
            .get(&format!("/api/months/{}/budgets", month_id))
            .add_header(auth_name(), auth_value(&token))
            .await
            .json();
        assert!(budgets.is_empty(), "{} should drop the category", label);
    }

    let closed_budgets: Vec<serde_json::Value> = server
        .get(&format!("/api/months/{}/budgets", november))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();
    assert_eq!(
        closed_budgets.len(),
        1,
        "a closed month is settled history and keeps the category"
    );
}

#[tokio::test]
async fn test_delete_category_unlabels_its_transactions() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 9).await;
    let cat_id = create_test_category(&pool, user_id, "Foo", 500.0).await;
    create_test_budget(&pool, month_id, cat_id, 500.0).await;
    create_test_item(&pool, month_id, cat_id, "Dinner", 40.0, "2024-09-02").await;

    server
        .delete(&format!("/api/months/{}/categories/{}", month_id, cat_id))
        .add_header(auth_name(), auth_value(&token))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    let summary: serde_json::Value = server
        .get(&format!("/api/months/{}", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();

    assert_eq!(
        summary["budgets"].as_array().unwrap().len(),
        0,
        "the category's budget line goes with it"
    );
    let items = summary["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "the transaction itself is never removed");
    assert!(
        items[0]["category_id"].is_null(),
        "the transaction becomes uncategorized instead of pointing at a ghost"
    );
    assert!(items[0]["category_label"].is_null());
    assert_eq!(summary["total_spent"], 40.0, "totals are unaffected");
}

#[tokio::test]
async fn test_delete_category_unlabels_later_open_months_but_not_earlier_ones() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let august_id = create_test_month(&pool, user_id, 2024, 8).await;
    let september_id = create_test_month(&pool, user_id, 2024, 9).await;
    let october_id = create_test_month(&pool, user_id, 2024, 10).await;
    let cat_id = create_test_category(&pool, user_id, "Transport", 150.0).await;
    create_test_item(&pool, august_id, cat_id, "Bus pass", 40.0, "2024-08-03").await;
    create_test_item(&pool, september_id, cat_id, "Bus pass", 40.0, "2024-09-03").await;
    create_test_item(&pool, october_id, cat_id, "Bus pass", 40.0, "2024-10-03").await;

    server
        .delete(&format!(
            "/api/months/{}/categories/{}",
            september_id, cat_id
        ))
        .add_header(auth_name(), auth_value(&token))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    let august: serde_json::Value = server
        .get(&format!("/api/months/{}", august_id))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();
    assert_eq!(
        august["items"][0]["category_label"], "Transport",
        "earlier months keep resolving the archived label"
    );

    for month_id in [september_id, october_id] {
        let summary: serde_json::Value = server
            .get(&format!("/api/months/{}", month_id))
            .add_header(auth_name(), auth_value(&token))
            .await
            .json();
        assert!(
            summary["items"][0]["category_id"].is_null(),
            "month {month_id} unlabels the transaction"
        );
    }
}

#[tokio::test]
async fn test_recreated_category_starts_genuinely_unused() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 9).await;
    let cat_id = create_test_category(&pool, user_id, "Travel", 0.0).await;
    create_test_item(&pool, month_id, cat_id, "Flights", 500.0, "2024-09-02").await;

    server
        .delete(&format!("/api/months/{}/categories/{}", month_id, cat_id))
        .add_header(auth_name(), auth_value(&token))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    let recreated: serde_json::Value = server
        .post("/api/categories")
        .add_header(auth_name(), auth_value(&token))
        .json(&serde_json::json!({"label": "Travel", "default_amount": 0.0, "month_id": month_id}))
        .await
        .json();
    let new_id = recreated["id"].as_i64().unwrap();
    assert_ne!(new_id, cat_id, "recreation is a fresh category");

    let summary: serde_json::Value = server
        .get(&format!("/api/months/{}", month_id))
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();
    let travel_budget = summary["budgets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["category_id"] == new_id)
        .expect("recreated category gets a budget line");
    assert_eq!(
        travel_budget["spent_amount"], 0.0,
        "no orphaned spending is attributed to the new category"
    );
    assert!(
        summary["items"][0]["category_id"].is_null(),
        "the old transaction stays uncategorized"
    );
}

#[tokio::test]
async fn test_delete_category_rejects_closed_month() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    create_test_budget(&pool, month_id, cat_id, 500.0).await;
    close_test_month(&pool, month_id).await;

    let response = server
        .delete(&format!("/api/months/{}/categories/{}", month_id, cat_id))
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_delete_category_is_scoped_to_the_owner() {
    let (server, pool, user_id, _token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    create_test_budget(&pool, month_id, cat_id, 500.0).await;

    let mallory_id = create_test_user(&pool, "mallory", "password123").await;
    let mallory_token = generate_token(mallory_id, "mallory");

    let response = server
        .delete(&format!("/api/months/{}/categories/{}", month_id, cat_id))
        .add_header(auth_name(), auth_value(&mallory_token))
        .await;

    response.assert_status_not_found();
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
