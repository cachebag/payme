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

async fn month_summary(
    server: &axum_test::TestServer,
    token: &str,
    month_id: i64,
) -> serde_json::Value {
    let response = server
        .get(&format!("/api/months/{month_id}"))
        .add_header(auth_name(), auth_value(token))
        .await;
    response.assert_status_ok();
    response.json()
}

async fn create_expense(
    server: &axum_test::TestServer,
    token: &str,
    month_id: i64,
    label: &str,
    amount: f64,
) -> i64 {
    let response = server
        .post(&format!("/api/months/{month_id}/fixed-expenses"))
        .add_header(auth_name(), auth_value(token))
        .json(&json!({"label": label, "amount": amount}))
        .await;
    response.assert_status_ok();
    response.json::<serde_json::Value>()["id"].as_i64().unwrap()
}

fn labels_and_amounts(expenses: &serde_json::Value) -> Vec<(String, f64)> {
    expenses
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["label"].as_str().unwrap().to_string(),
                e["amount"].as_f64().unwrap(),
            )
        })
        .collect()
}

#[tokio::test]
async fn fixed_expenses_apply_forward_copies_set_amounts_and_order() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;
    let march_id = create_test_month(&pool, user_id, 2024, 3).await;

    // Creating from January already propagates; set up divergence afterwards.
    let rent_id = create_expense(&server, &token, january_id, "Rent", 1000.0).await;
    let wifi_id = create_expense(&server, &token, january_id, "Wifi", 50.0).await;
    create_expense(&server, &token, march_id, "Gym", 30.0).await; // only in March

    // Reorder and change an amount directly in January only (reorder never propagates).
    server
        .put(&format!("/api/months/{january_id}/fixed-expenses/reorder"))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({"ids": [wifi_id, rent_id]}))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
    sqlx::query("UPDATE monthly_fixed_expenses SET amount = 1100 WHERE id = ?")
        .bind(rent_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = server
        .post(&format!(
            "/api/months/{january_id}/fixed-expenses/apply-forward"
        ))
        .add_header(auth_name(), auth_value(&token))
        .await;
    response.assert_status_ok();
    assert_eq!(response.json::<serde_json::Value>()["months_updated"], 2);

    let expected = vec![("Wifi".to_string(), 50.0), ("Rent".to_string(), 1100.0)];
    for month_id in [february_id, march_id] {
        let summary = month_summary(&server, &token, month_id).await;
        assert_eq!(
            labels_and_amounts(&summary["fixed_expenses"]),
            expected,
            "month {month_id} should mirror January"
        );
    }

    // Copies stay linked to the source group so later edits still propagate.
    let groups: Vec<Option<i64>> = sqlx::query_scalar(
        "SELECT group_id FROM monthly_fixed_expenses WHERE month_id = ? ORDER BY sort_order",
    )
    .bind(march_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(groups, vec![Some(wifi_id), Some(rent_id)]);
}

#[tokio::test]
async fn fixed_expenses_apply_forward_skips_earlier_and_closed_months() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;
    let march_id = create_test_month(&pool, user_id, 2024, 3).await;
    let april_id = create_test_month(&pool, user_id, 2024, 4).await;
    create_expense(&server, &token, january_id, "Old", 1.0).await;
    close_test_month(&pool, march_id).await;

    sqlx::query("DELETE FROM monthly_fixed_expenses WHERE month_id = ?")
        .bind(february_id)
        .execute(&pool)
        .await
        .unwrap();
    create_expense(&server, &token, february_id, "New", 2.0).await;

    let response = server
        .post(&format!(
            "/api/months/{february_id}/fixed-expenses/apply-forward"
        ))
        .add_header(auth_name(), auth_value(&token))
        .await;
    response.assert_status_ok();
    assert_eq!(response.json::<serde_json::Value>()["months_updated"], 1);

    let january = month_summary(&server, &token, january_id).await;
    assert_eq!(
        labels_and_amounts(&january["fixed_expenses"]),
        vec![("Old".to_string(), 1.0)]
    );
    let march = month_summary(&server, &token, march_id).await;
    assert_eq!(
        labels_and_amounts(&march["fixed_expenses"]),
        vec![("Old".to_string(), 1.0)]
    );
    let april = month_summary(&server, &token, april_id).await;
    assert_eq!(
        labels_and_amounts(&april["fixed_expenses"]),
        vec![("New".to_string(), 2.0)]
    );
}

#[tokio::test]
async fn fixed_expenses_apply_forward_unknown_month_is_404() {
    let (server, _pool, _user_id, token) = setup_with_user().await;
    server
        .post("/api/months/999/fixed-expenses/apply-forward")
        .add_header(auth_name(), auth_value(&token))
        .await
        .assert_status_not_found();
}

#[tokio::test]
async fn budgets_apply_forward_copies_limits_not_spending() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let groceries = create_test_category(&pool, user_id, "Groceries", 500.0).await;
    let dining = create_test_category(&pool, user_id, "Dining", 250.0).await;
    let travel = create_test_category(&pool, user_id, "Travel", 200.0).await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;

    // January: groceries raised, dining kept, travel dropped.
    create_test_budget(&pool, january_id, groceries, 600.0).await;
    create_test_budget(&pool, january_id, dining, 250.0).await;
    create_test_item(&pool, january_id, groceries, "Costco", 120.0, "2024-01-05").await;

    // February: stale limits, still budgets travel, has its own spending.
    create_test_budget(&pool, february_id, groceries, 500.0).await;
    create_test_budget(&pool, february_id, dining, 250.0).await;
    create_test_budget(&pool, february_id, travel, 200.0).await;
    create_test_item(&pool, february_id, dining, "Pizza", 40.0, "2024-02-02").await;
    create_test_item(&pool, february_id, travel, "Train", 15.0, "2024-02-03").await;

    let response = server
        .post(&format!("/api/months/{january_id}/budgets/apply-forward"))
        .add_header(auth_name(), auth_value(&token))
        .await;
    response.assert_status_ok();
    assert_eq!(response.json::<serde_json::Value>()["months_updated"], 1);

    let february = month_summary(&server, &token, february_id).await;
    let budgets = february["budgets"].as_array().unwrap();
    let mut by_label: Vec<(String, f64, f64)> = budgets
        .iter()
        .map(|b| {
            (
                b["category_label"].as_str().unwrap().to_string(),
                b["allocated_amount"].as_f64().unwrap(),
                b["spent_amount"].as_f64().unwrap(),
            )
        })
        .collect();
    by_label.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        by_label,
        vec![
            ("Dining".to_string(), 250.0, 40.0),
            ("Groceries".to_string(), 600.0, 0.0),
        ]
    );

    // February's transactions are untouched, including the one in the dropped category.
    let items = february["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    let train_category: Option<i64> =
        sqlx::query_scalar("SELECT category_id FROM items WHERE description = 'Train'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(train_category, Some(travel));

    // Category defaults follow, so the next month created starts from these limits.
    let defaults: Vec<(String, f64)> = sqlx::query_as(
        "SELECT label, default_amount FROM budget_categories WHERE user_id = ? ORDER BY label",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        defaults,
        vec![
            ("Dining".to_string(), 250.0),
            ("Groceries".to_string(), 600.0),
            ("Travel".to_string(), 200.0),
        ]
    );
}

#[tokio::test]
async fn budgets_apply_forward_skips_earlier_and_closed_months() {
    let (server, pool, user_id, token) = setup_with_user().await;
    let groceries = create_test_category(&pool, user_id, "Groceries", 500.0).await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;
    let march_id = create_test_month(&pool, user_id, 2024, 3).await;
    create_test_budget(&pool, january_id, groceries, 100.0).await;
    create_test_budget(&pool, february_id, groceries, 700.0).await;
    create_test_budget(&pool, march_id, groceries, 100.0).await;
    close_test_month(&pool, march_id).await;

    let response = server
        .post(&format!("/api/months/{february_id}/budgets/apply-forward"))
        .add_header(auth_name(), auth_value(&token))
        .await;
    response.assert_status_ok();
    assert_eq!(response.json::<serde_json::Value>()["months_updated"], 0);

    for month_id in [january_id, march_id] {
        let summary = month_summary(&server, &token, month_id).await;
        assert_eq!(summary["budgets"][0]["allocated_amount"], 100.0);
    }
}
