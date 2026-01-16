mod common;

use common::{
    auth_name, auth_value, create_test_budget, create_test_category, create_test_fixed_expense,
    create_test_income, create_test_item, create_test_month, create_test_pool, create_test_server,
    create_test_user, generate_token,
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
async fn test_export_json() {
    let (server, pool, user_id, token) = setup_with_user().await;

    create_test_fixed_expense(&pool, user_id, "Rent", 1500.0).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    create_test_income(&pool, month_id, "Salary", 5000.0).await;
    create_test_budget(&pool, month_id, cat_id, 500.0).await;
    create_test_item(&pool, month_id, cat_id, "Groceries", 150.0, "2024-06-15").await;

    let response = server
        .get("/api/export/json")
        .add_header(auth_name(), auth_value(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();

    assert_eq!(body["version"], 1);
    assert_eq!(body["fixed_expenses"].as_array().unwrap().len(), 1);
    assert_eq!(body["categories"].as_array().unwrap().len(), 1);
    assert_eq!(body["months"].as_array().unwrap().len(), 1);

    let month = &body["months"][0];
    assert_eq!(month["income_entries"].as_array().unwrap().len(), 1);
    assert_eq!(month["budgets"].as_array().unwrap().len(), 1);
    assert_eq!(month["items"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_import_json() {
    let (server, _pool, _user_id, token) = setup_with_user().await;

    let import_data = json!({
        "version": 1,
        "savings": 10000.0,
        "retirement_savings": 25000.0,
        "fixed_expenses": [
            {"label": "Rent", "amount": 1500.0},
            {"label": "Internet", "amount": 80.0}
        ],
        "categories": [
            {"label": "Food", "default_amount": 500.0},
            {"label": "Transport", "default_amount": 200.0}
        ],
        "months": [
            {
                "year": 2024,
                "month": 6,
                "is_closed": false,
                "income_entries": [
                    {"label": "Salary", "amount": 5000.0}
                ],
                "budgets": [
                    {"category_label": "Food", "allocated_amount": 600.0}
                ],
                "items": [
                    {"category_label": "Food", "description": "Groceries", "amount": 150.0, "spent_on": "2024-06-15"}
                ]
            }
        ]
    });

    let response = server
        .post("/api/import/json")
        .add_header(auth_name(), auth_value(&token))
        .json(&import_data)
        .await;

    response.assert_status_ok();

    let export_response = server
        .get("/api/export/json")
        .add_header(auth_name(), auth_value(&token))
        .await;

    let exported: serde_json::Value = export_response.json();
    assert_eq!(exported["fixed_expenses"].as_array().unwrap().len(), 2);
    assert_eq!(exported["categories"].as_array().unwrap().len(), 2);
    assert_eq!(exported["months"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_export_import_round_trip() {
    let (server, pool, user_id, token) = setup_with_user().await;

    create_test_fixed_expense(&pool, user_id, "Rent", 1500.0).await;
    let cat_id = create_test_category(&pool, user_id, "Food", 500.0).await;
    let month_id = create_test_month(&pool, user_id, 2024, 6).await;
    create_test_income(&pool, month_id, "Salary", 5000.0).await;
    create_test_budget(&pool, month_id, cat_id, 500.0).await;
    create_test_item(&pool, month_id, cat_id, "Groceries", 150.0, "2024-06-15").await;

    let export_response = server
        .get("/api/export/json")
        .add_header(auth_name(), auth_value(&token))
        .await;

    let exported: serde_json::Value = export_response.json();

    let import_response = server
        .post("/api/import/json")
        .add_header(auth_name(), auth_value(&token))
        .json(&exported)
        .await;

    import_response.assert_status_ok();

    let export_response2 = server
        .get("/api/export/json")
        .add_header(auth_name(), auth_value(&token))
        .await;

    let exported2: serde_json::Value = export_response2.json();

    assert_eq!(
        exported["fixed_expenses"].as_array().unwrap().len(),
        exported2["fixed_expenses"].as_array().unwrap().len()
    );
    assert_eq!(
        exported["categories"].as_array().unwrap().len(),
        exported2["categories"].as_array().unwrap().len()
    );
    assert_eq!(
        exported["months"].as_array().unwrap().len(),
        exported2["months"].as_array().unwrap().len()
    );
}

#[tokio::test]
async fn test_import_json_replaces_existing() {
    let (server, pool, user_id, token) = setup_with_user().await;

    create_test_fixed_expense(&pool, user_id, "Old Expense", 100.0).await;
    create_test_category(&pool, user_id, "Old Category", 100.0).await;

    let import_data = json!({
        "version": 1,
        "savings": 0.0,
        "retirement_savings": 0.0,
        "fixed_expenses": [
            {"label": "New Expense", "amount": 200.0}
        ],
        "categories": [
            {"label": "New Category", "default_amount": 300.0}
        ],
        "months": []
    });

    let response = server
        .post("/api/import/json")
        .add_header(auth_name(), auth_value(&token))
        .json(&import_data)
        .await;

    response.assert_status_ok();

    let export_response = server
        .get("/api/export/json")
        .add_header(auth_name(), auth_value(&token))
        .await;

    let exported: serde_json::Value = export_response.json();

    let expenses = exported["fixed_expenses"].as_array().unwrap();
    assert_eq!(expenses.len(), 1);
    assert_eq!(expenses[0]["label"], "New Expense");

    let categories = exported["categories"].as_array().unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0]["label"], "New Category");
}
