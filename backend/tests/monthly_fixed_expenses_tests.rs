mod common;

use common::{
    auth_name, auth_value, close_test_month, create_test_month, create_test_pool,
    create_test_server, create_test_user, generate_token,
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

async fn get_fixed_expenses(
    server: &axum_test::TestServer,
    token: &str,
    month_id: i64,
) -> Vec<serde_json::Value> {
    let response = server
        .get(&format!("/api/months/{month_id}"))
        .add_header(auth_name(), auth_value(token))
        .await;
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    body["fixed_expenses"].as_array().unwrap().clone()
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
    let body: serde_json::Value = response.json();
    body["id"].as_i64().unwrap()
}

#[tokio::test]
async fn test_create_propagates_to_later_open_months() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;
    let march_id = create_test_month(&pool, user_id, 2024, 3).await;

    create_expense(&server, &token, january_id, "Gym", 30.0).await;

    for month_id in [january_id, february_id, march_id] {
        let expenses = get_fixed_expenses(&server, &token, month_id).await;
        assert_eq!(
            expenses.len(),
            1,
            "month {month_id} should have the expense"
        );
        assert_eq!(expenses[0]["label"], "Gym");
        assert_eq!(expenses[0]["amount"], 30.0);
    }
}

#[tokio::test]
async fn test_create_does_not_touch_earlier_months() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;

    create_expense(&server, &token, february_id, "Gym", 30.0).await;

    let january = get_fixed_expenses(&server, &token, january_id).await;
    assert!(january.is_empty());
    let february = get_fixed_expenses(&server, &token, february_id).await;
    assert_eq!(february.len(), 1);
}

#[tokio::test]
async fn test_create_skips_closed_later_months() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;
    let march_id = create_test_month(&pool, user_id, 2024, 3).await;
    close_test_month(&pool, february_id).await;

    create_expense(&server, &token, january_id, "Gym", 30.0).await;

    let february = get_fixed_expenses(&server, &token, february_id).await;
    assert!(february.is_empty(), "closed month must stay untouched");
    let march = get_fixed_expenses(&server, &token, march_id).await;
    assert_eq!(march.len(), 1);
}

#[tokio::test]
async fn test_update_propagates_forward_not_backward() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;
    let march_id = create_test_month(&pool, user_id, 2024, 3).await;

    create_expense(&server, &token, january_id, "Gym", 30.0).await;

    let february = get_fixed_expenses(&server, &token, february_id).await;
    let february_expense_id = february[0]["id"].as_i64().unwrap();

    let response = server
        .put(&format!(
            "/api/months/{february_id}/fixed-expenses/{february_expense_id}"
        ))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({"amount": 45.0}))
        .await;
    response.assert_status_ok();

    let january = get_fixed_expenses(&server, &token, january_id).await;
    assert_eq!(january[0]["amount"], 30.0, "earlier month stays frozen");
    let february = get_fixed_expenses(&server, &token, february_id).await;
    assert_eq!(february[0]["amount"], 45.0);
    let march = get_fixed_expenses(&server, &token, march_id).await;
    assert_eq!(march[0]["amount"], 45.0, "later month follows the change");
}

#[tokio::test]
async fn test_update_skips_closed_later_months() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;
    let march_id = create_test_month(&pool, user_id, 2024, 3).await;

    let expense_id = create_expense(&server, &token, january_id, "Gym", 30.0).await;
    close_test_month(&pool, february_id).await;

    let response = server
        .put(&format!(
            "/api/months/{january_id}/fixed-expenses/{expense_id}"
        ))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({"amount": 45.0}))
        .await;
    response.assert_status_ok();

    let february = get_fixed_expenses(&server, &token, february_id).await;
    assert_eq!(february[0]["amount"], 30.0, "closed month stays frozen");
    let march = get_fixed_expenses(&server, &token, march_id).await;
    assert_eq!(march[0]["amount"], 45.0);
}

#[tokio::test]
async fn test_rename_keeps_expenses_linked() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;

    let expense_id = create_expense(&server, &token, january_id, "Gym", 30.0).await;

    // Rename in January; February should follow.
    let response = server
        .put(&format!(
            "/api/months/{january_id}/fixed-expenses/{expense_id}"
        ))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({"label": "Fitness"}))
        .await;
    response.assert_status_ok();

    let february = get_fixed_expenses(&server, &token, february_id).await;
    assert_eq!(february[0]["label"], "Fitness");

    // A later amount change must still propagate even though the label changed,
    // proving the link is the hidden group id rather than the label.
    let response = server
        .put(&format!(
            "/api/months/{january_id}/fixed-expenses/{expense_id}"
        ))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({"amount": 50.0}))
        .await;
    response.assert_status_ok();

    let february = get_fixed_expenses(&server, &token, february_id).await;
    assert_eq!(february[0]["label"], "Fitness");
    assert_eq!(february[0]["amount"], 50.0);
}

#[tokio::test]
async fn test_delete_propagates_forward_not_backward() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;
    let march_id = create_test_month(&pool, user_id, 2024, 3).await;

    create_expense(&server, &token, january_id, "Gym", 30.0).await;

    let february = get_fixed_expenses(&server, &token, february_id).await;
    let february_expense_id = february[0]["id"].as_i64().unwrap();

    // Stop paying for the gym as of February.
    let response = server
        .delete(&format!(
            "/api/months/{february_id}/fixed-expenses/{february_expense_id}"
        ))
        .add_header(auth_name(), auth_value(&token))
        .await;
    response.assert_status(axum::http::StatusCode::NO_CONTENT);

    let january = get_fixed_expenses(&server, &token, january_id).await;
    assert_eq!(january.len(), 1, "history keeps the expense");
    let february = get_fixed_expenses(&server, &token, february_id).await;
    assert!(february.is_empty());
    let march = get_fixed_expenses(&server, &token, march_id).await;
    assert!(march.is_empty(), "later months drop the expense");
}

#[tokio::test]
async fn test_delete_skips_closed_later_months() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    let february_id = create_test_month(&pool, user_id, 2024, 2).await;
    let march_id = create_test_month(&pool, user_id, 2024, 3).await;

    let expense_id = create_expense(&server, &token, january_id, "Gym", 30.0).await;
    close_test_month(&pool, february_id).await;

    let response = server
        .delete(&format!(
            "/api/months/{january_id}/fixed-expenses/{expense_id}"
        ))
        .add_header(auth_name(), auth_value(&token))
        .await;
    response.assert_status(axum::http::StatusCode::NO_CONTENT);

    let february = get_fixed_expenses(&server, &token, february_id).await;
    assert_eq!(february.len(), 1, "closed month keeps its snapshot");
    let march = get_fixed_expenses(&server, &token, march_id).await;
    assert!(march.is_empty());
}

#[tokio::test]
async fn test_new_month_joins_existing_group() {
    let (server, pool, user_id, token) = setup_with_user().await;

    let january_id = create_test_month(&pool, user_id, 2024, 1).await;
    create_expense(&server, &token, january_id, "Gym", 30.0).await;

    // February is created after the fact and seeds from January.
    let response = server
        .post("/api/months")
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({"year": 2024, "month": 2}))
        .await;
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    let february_id = body["month"]["id"].as_i64().unwrap();

    let february = get_fixed_expenses(&server, &token, february_id).await;
    assert_eq!(february.len(), 1);

    // Editing January must reach the freshly seeded February copy.
    let january = get_fixed_expenses(&server, &token, january_id).await;
    let january_expense_id = january[0]["id"].as_i64().unwrap();
    let response = server
        .put(&format!(
            "/api/months/{january_id}/fixed-expenses/{january_expense_id}"
        ))
        .add_header(auth_name(), auth_value(&token))
        .json(&json!({"amount": 45.0}))
        .await;
    response.assert_status_ok();

    let february = get_fixed_expenses(&server, &token, february_id).await;
    assert_eq!(february[0]["amount"], 45.0);
}
