mod common;

use common::{
    auth_name, auth_value, create_test_category, create_test_fixed_expense, create_test_income,
    create_test_item, create_test_month, create_test_monthly_fixed_expense, create_test_pool,
    create_test_server, create_test_user, generate_token,
};
use payme::create_app;

async fn setup_with_user() -> (axum_test::TestServer, sqlx::SqlitePool, i64, String) {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool, "testuser", "password123").await;
    let token = generate_token(user_id, "testuser");
    let app = create_app(pool.clone());
    let server = create_test_server(app);
    (server, pool, user_id, token)
}

#[tokio::test]
async fn test_stats_empty() {
    let (server, _pool, _user_id, token) = setup_with_user().await;

    let response = server
        .get("/api/stats")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["average_monthly_spending"], 0.0);
    assert_eq!(body["average_monthly_income"], 0.0);
    assert!(body["monthly_trends"].as_array().unwrap().is_empty());
    assert!(body["category_comparisons"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_stats_with_data() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month1_id = create_test_month(&pool, user_id, 2024, 5).await;
    let month2_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    create_test_fixed_expense(&pool, user_id, "Rent", 1000.0).await;

    create_test_income(&pool, month1_id, "Salary", 5000.0).await;
    create_test_income(&pool, month2_id, "Salary", 5000.0).await;

    create_test_item(&pool, month1_id, cat_id, "Groceries", 300.0, "2024-05-15").await;
    create_test_item(&pool, month2_id, cat_id, "Groceries", 400.0, "2024-06-15").await;

    let response = server
        .get("/api/stats")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();

    assert_eq!(body["average_monthly_spending"], 350.0);
    assert_eq!(body["average_monthly_income"], 5000.0);
    assert_eq!(body["monthly_trends"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_stats_category_comparison() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month1_id = create_test_month(&pool, user_id, 2024, 5).await;
    let month2_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    create_test_fixed_expense(&pool, user_id, "Rent", 1000.0).await;

    create_test_income(&pool, month1_id, "Salary", 3000.0).await;
    create_test_income(&pool, month2_id, "Salary", 3000.0).await;
    create_test_item(&pool, month1_id, cat_id, "Groceries", 300.0, "2024-05-15").await;
    create_test_item(&pool, month2_id, cat_id, "Groceries", 450.0, "2024-06-15").await;

    let response = server
        .get("/api/stats")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();

    let comparisons = body["category_comparisons"].as_array().unwrap();
    assert_eq!(comparisons.len(), 1);

    let food_comparison = &comparisons[0];
    assert_eq!(food_comparison["category_label"], "Food");
    assert_eq!(food_comparison["current_month_spent"], 450.0);
    assert_eq!(food_comparison["previous_month_spent"], 300.0);
    assert_eq!(food_comparison["change_amount"], 150.0);
}

#[tokio::test]
async fn test_stats_change_percent() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month1_id = create_test_month(&pool, user_id, 2024, 5).await;
    let month2_id = create_test_month(&pool, user_id, 2024, 6).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    create_test_fixed_expense(&pool, user_id, "Rent", 1000.0).await;

    create_test_income(&pool, month1_id, "Salary", 3000.0).await;
    create_test_income(&pool, month2_id, "Salary", 3000.0).await;
    create_test_item(&pool, month1_id, cat_id, "Groceries", 200.0, "2024-05-15").await;
    create_test_item(&pool, month2_id, cat_id, "Groceries", 300.0, "2024-06-15").await;

    let response = server
        .get("/api/stats")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();

    let comparisons = body["category_comparisons"].as_array().unwrap();
    let food_comparison = &comparisons[0];

    assert_eq!(food_comparison["change_percent"], 50.0);
}

#[tokio::test]
async fn test_stats_fixed_totals_come_from_the_month_snapshot() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 5).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    create_test_income(&pool, month_id, "Salary", 5000.0).await;
    create_test_item(&pool, month_id, cat_id, "Groceries", 300.0, "2024-05-15").await;
    // Recorded per-month, which is what the UI edits. The user-level template stays empty.
    create_test_monthly_fixed_expense(&pool, month_id, "Rent", 1500.0).await;
    create_test_monthly_fixed_expense(&pool, month_id, "Internet", 80.0).await;

    let body: serde_json::Value = server
        .get("/api/stats")
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();

    let trend = &body["monthly_trends"][0];
    assert_eq!(
        trend["total_fixed"], 1580.0,
        "fixed comes from the snapshot"
    );
    assert_eq!(
        trend["net"], 3120.0,
        "net subtracts fixed: 5000 - 1580 - 300"
    );
}

#[tokio::test]
async fn test_stats_averages_ignore_months_with_nothing_in_them() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let may_id = create_test_month(&pool, user_id, 2024, 5).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    create_test_income(&pool, may_id, "Salary", 5000.0).await;
    create_test_item(&pool, may_id, cat_id, "Groceries", 300.0, "2024-05-15").await;

    // Clicking through the navigator creates empty months; they must not drag the averages.
    create_test_month(&pool, user_id, 2024, 6).await;
    create_test_month(&pool, user_id, 2024, 7).await;

    let body: serde_json::Value = server
        .get("/api/stats")
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();

    assert_eq!(body["average_monthly_spending"], 300.0);
    assert_eq!(body["average_monthly_income"], 5000.0);
    assert_eq!(
        body["monthly_trends"].as_array().unwrap().len(),
        3,
        "empty months still appear in the trend line"
    );
}

#[tokio::test]
async fn test_stats_counts_a_month_with_only_fixed_expenses() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let month_id = create_test_month(&pool, user_id, 2024, 5).await;
    create_test_monthly_fixed_expense(&pool, month_id, "Rent", 1500.0).await;

    let body: serde_json::Value = server
        .get("/api/stats")
        .add_header(auth_name(), auth_value(&token))
        .await
        .json();

    // No income and no spending, but the month is not empty, so it counts as active.
    assert_eq!(body["average_monthly_spending"], 0.0);
    assert_eq!(body["monthly_trends"][0]["net"], -1500.0);
}
