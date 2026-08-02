#![allow(unused_must_use)]

use axum::extract::{Path, State};
use axum::Json;
use payme::db::run_migrations;
use payme::handlers::budget::{
    create_category, delete_month_category, list_categories, update_category,
    update_monthly_budget, CreateCategory, UpdateCategory, UpdateMonthlyBudget,
};
use payme::handlers::income::{create_income, list_income, CreateIncome};
use payme::handlers::months::{create_month, list_months, reopen_month};
use payme::handlers::retirement_breakdown::{
    create_retirement_breakdown_item, delete_retirement_breakdown_item, list_retirement_breakdown,
    update_retirement_breakdown_item, CreateRetirementBreakdownItem, UpdateRetirementBreakdownItem,
};
use payme::handlers::savings_goals::{
    create_savings_goal, delete_savings_goal, list_savings_goals, update_savings_goal,
    CreateSavingsGoal, UpdateSavingsGoal,
};
use payme::middleware::auth::Claims;
use sqlx::SqlitePool;

async fn setup() -> (SqlitePool, Claims) {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    run_migrations(&pool).await.unwrap();
    let claims = add_user(&pool, "alice").await;
    (pool, claims)
}

async fn add_user(pool: &SqlitePool, username: &str) -> Claims {
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash) VALUES (?, ?) RETURNING id",
    )
    .bind(username)
    .bind("$argon2id$placeholder")
    .fetch_one(pool)
    .await
    .unwrap();
    Claims {
        sub: user_id,
        username: username.to_string(),
        exp: 9_999_999_999,
    }
}

fn st(pool: SqlitePool) -> State<SqlitePool> {
    State(pool)
}

fn ext(claims: Claims) -> axum::Extension<Claims> {
    axum::Extension(claims)
}

#[tokio::test]
async fn migrations_create_all_expected_tables() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    run_migrations(&pool).await.unwrap();

    let expected = [
        "users",
        "fixed_expenses",
        "budget_categories",
        "months",
        "income_entries",
        "monthly_budgets",
        "items",
        "monthly_snapshots",
        "monthly_fixed_expenses",
        "monthly_savings",
        "custom_savings_goals",
        "retirement_breakdown_items",
    ];

    let tables: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM sqlite_master WHERE type = 'table'")
            .fetch_all(&pool)
            .await
            .unwrap();

    let names: Vec<&str> = tables.iter().map(|(n,)| n.as_str()).collect();
    for table in expected {
        assert!(names.contains(&table), "expected table '{table}' to exist");
    }
}

#[tokio::test]
async fn migrations_are_idempotent() {
    // Running migrations twice must not error (IF NOT EXISTS guards).
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    run_migrations(&pool).await.unwrap();
    run_migrations(&pool).await.unwrap();
}

#[tokio::test]
async fn category_create_and_list() {
    let (pool, claims) = setup().await;

    let Json(created) = create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Groceries".to_string(),
            default_amount: 400.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    assert_eq!(created.label, "Groceries");
    assert_eq!(created.default_amount, 400.0);
    assert_eq!(created.color, "#71717a"); // default color

    let Json(list) = list_categories(st(pool), ext(claims)).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, created.id);
}

#[tokio::test]
async fn category_update() {
    let (pool, claims) = setup().await;

    let Json(cat) = create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Old".to_string(),
            default_amount: 100.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    let Json(updated) = update_category(
        st(pool.clone()),
        ext(claims.clone()),
        Path(cat.id),
        Json(UpdateCategory {
            label: Some("New".to_string()),
            default_amount: Some(250.0),
            color: Some("#ff0000".to_string()),
        }),
    )
    .await
    .unwrap();

    assert_eq!(updated.label, "New");
    assert_eq!(updated.default_amount, 250.0);
    assert_eq!(updated.color, "#ff0000");
}

#[tokio::test]
async fn category_delete_removes_it_from_list() {
    let (pool, claims) = setup().await;

    let Json(cat) = create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Dining".to_string(),
            default_amount: 200.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 1,
        }),
    )
    .await
    .unwrap();

    delete_month_category(
        st(pool.clone()),
        ext(claims.clone()),
        Path((summary.month.id, cat.id)),
    )
    .await
    .unwrap();

    let Json(list) = list_categories(st(pool), ext(claims)).await.unwrap();
    assert!(list.is_empty());
}

#[tokio::test]
async fn category_delete_clears_the_month_but_not_its_history() {
    let (pool, claims) = setup().await;

    let Json(cat) = create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Transport".to_string(),
            default_amount: 150.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    // Two months, each seeded with the category.
    let Json(august) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 8,
        }),
    )
    .await
    .unwrap();
    let Json(september) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 9,
        }),
    )
    .await
    .unwrap();

    sqlx::query("INSERT INTO items (month_id, category_id, description, amount, spent_on, savings_destination) VALUES (?, ?, 'Bus pass', 40.0, '2025-08-03', 'none')")
        .bind(august.month.id)
        .bind(cat.id)
        .execute(&pool)
        .await
        .unwrap();

    delete_month_category(
        st(pool.clone()),
        ext(claims.clone()),
        Path((september.month.id, cat.id)),
    )
    .await
    .unwrap();

    let august_budgets: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM monthly_budgets WHERE month_id = ?")
            .bind(august.month.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(august_budgets, 1, "August keeps its allocation");

    let august_items: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM items WHERE month_id = ?")
        .bind(august.month.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(august_items, 1, "August keeps its transactions");

    let september_budgets: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM monthly_budgets WHERE month_id = ?")
            .bind(september.month.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(september_budgets, 0, "September drops the allocation");
}

#[tokio::test]
async fn deleted_category_does_not_come_back_in_new_months() {
    let (pool, claims) = setup().await;

    let Json(cat) = create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Hobbies".to_string(),
            default_amount: 75.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    let Json(september) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 9,
        }),
    )
    .await
    .unwrap();

    delete_month_category(
        st(pool.clone()),
        ext(claims.clone()),
        Path((september.month.id, cat.id)),
    )
    .await
    .unwrap();

    let Json(october) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 10,
        }),
    )
    .await
    .unwrap();

    assert!(
        october.budgets.is_empty(),
        "a retired category should not seed later months"
    );
}

#[tokio::test]
async fn month_creation_seeds_existing_categories() {
    let (pool, claims) = setup().await;

    create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Rent".to_string(),
            default_amount: 1500.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Food".to_string(),
            default_amount: 300.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 3,
        }),
    )
    .await
    .unwrap();

    assert_eq!(summary.budgets.len(), 2);
    let rent = summary
        .budgets
        .iter()
        .find(|b| b.category_label == "Rent")
        .unwrap();
    assert_eq!(rent.allocated_amount, 1500.0);
}

#[tokio::test]
async fn month_creation_is_idempotent() {
    let (pool, claims) = setup().await;

    let Json(first) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 6,
        }),
    )
    .await
    .unwrap();

    let Json(second) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 6,
        }),
    )
    .await
    .unwrap();

    assert_eq!(
        first.month.id, second.month.id,
        "creating the same month twice should return the same record"
    );

    let Json(months) = list_months(st(pool), ext(claims)).await.unwrap();
    assert_eq!(months.len(), 1, "only one month row should exist");
}

#[tokio::test]
async fn month_creation_rejects_invalid_month_number() {
    let (pool, claims) = setup().await;

    let result = create_month(
        st(pool),
        ext(claims),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 13,
        }),
    )
    .await;

    assert!(result.is_err(), "month 13 should be rejected");
}

#[tokio::test]
async fn category_created_mid_month_seeds_that_month_not_earlier_ones() {
    // The key invariant: adding a category while a month is open gives that month a
    // monthly_budget row for it, while months before it keep the shape they had.
    let (pool, claims) = setup().await;

    let Json(earlier) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 3,
        }),
    )
    .await
    .unwrap();

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 4,
        }),
    )
    .await
    .unwrap();

    let month_id = summary.month.id;

    // Month starts with zero budgets (no categories existed yet).
    let count_before: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM monthly_budgets WHERE month_id = ?")
            .bind(month_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count_before, 0);

    // Now create a category from that month — it lands there, but not in March.
    create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Entertainment".to_string(),
            default_amount: 100.0,
            color: None,
            month_id: Some(month_id),
        }),
    )
    .await
    .unwrap();

    let count_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM monthly_budgets WHERE month_id = ?")
            .bind(month_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        count_after, 1,
        "new category should be seeded into the month it was added from"
    );

    let earlier_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM monthly_budgets WHERE month_id = ?")
            .bind(earlier.month.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        earlier_count, 0,
        "an earlier month should not gain a category added later"
    );
}

#[tokio::test]
async fn monthly_budget_allocation_can_be_updated() {
    let (pool, claims) = setup().await;

    create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Misc".to_string(),
            default_amount: 50.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 7,
        }),
    )
    .await
    .unwrap();

    let budget = &summary.budgets[0];
    assert_eq!(budget.allocated_amount, 50.0);

    let Json(updated) = update_monthly_budget(
        st(pool.clone()),
        ext(claims.clone()),
        Path((summary.month.id, budget.id)),
        Json(UpdateMonthlyBudget {
            allocated_amount: 200.0,
        }),
    )
    .await
    .unwrap();

    assert_eq!(updated.allocated_amount, 200.0);
}

#[tokio::test]
async fn closed_month_rejects_budget_update() {
    use payme::handlers::months::close_month;
    let (pool, claims) = setup().await;

    create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Bills".to_string(),
            default_amount: 300.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 8,
        }),
    )
    .await
    .unwrap();

    let month_id = summary.month.id;
    let budget_id = summary.budgets[0].id;

    close_month(st(pool.clone()), ext(claims.clone()), Path(month_id))
        .await
        .unwrap();

    let result = update_monthly_budget(
        st(pool),
        ext(claims),
        Path((month_id, budget_id)),
        Json(UpdateMonthlyBudget {
            allocated_amount: 999.0,
        }),
    )
    .await;

    assert!(
        result.is_err(),
        "updating a budget on a closed month should fail"
    );
}

#[tokio::test]
async fn close_then_reopen_month() {
    use payme::handlers::months::close_month;
    let (pool, claims) = setup().await;

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 9,
        }),
    )
    .await
    .unwrap();
    let month_id = summary.month.id;

    let Json(closed) = close_month(st(pool.clone()), ext(claims.clone()), Path(month_id))
        .await
        .unwrap();
    assert!(closed.is_closed);

    let Json(reopened) = reopen_month(st(pool.clone()), ext(claims.clone()), Path(month_id))
        .await
        .unwrap();
    assert!(!reopened.is_closed);
    assert!(reopened.closed_at.is_none());
}

#[tokio::test]
async fn income_create_and_list() {
    let (pool, claims) = setup().await;

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 1,
        }),
    )
    .await
    .unwrap();
    let month_id = summary.month.id;

    create_income(
        st(pool.clone()),
        ext(claims.clone()),
        Path(month_id),
        Json(CreateIncome {
            label: "Salary".to_string(),
            amount: 5000.0,
            paid_on: None,
        }),
    )
    .await
    .unwrap();

    create_income(
        st(pool.clone()),
        ext(claims.clone()),
        Path(month_id),
        Json(CreateIncome {
            label: "Freelance".to_string(),
            amount: 800.0,
            paid_on: None,
        }),
    )
    .await
    .unwrap();

    let Json(entries) = list_income(st(pool), ext(claims), Path(month_id))
        .await
        .unwrap();
    assert_eq!(entries.len(), 2);
    let total: f64 = entries.iter().map(|e| e.amount).sum();
    assert_eq!(total, 5800.0);
}

#[tokio::test]
async fn savings_goal_full_crud() {
    let (pool, claims) = setup().await;

    // Create
    let (_, Json(goal)) = create_savings_goal(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateSavingsGoal {
            name: "Emergency Fund".to_string(),
            current_amount: Some(500.0),
            target_amount: 10000.0,
        }),
    )
    .await
    .unwrap();

    assert_eq!(goal.name, "Emergency Fund");
    assert_eq!(goal.current_amount, 500.0);
    assert_eq!(goal.target_amount, 10000.0);

    // List
    let Json(list) = list_savings_goals(st(pool.clone()), ext(claims.clone()))
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, goal.id);

    // Update
    let Json(updated) = update_savings_goal(
        st(pool.clone()),
        ext(claims.clone()),
        Path(goal.id),
        Json(UpdateSavingsGoal {
            name: None,
            current_amount: Some(2500.0),
            target_amount: None,
        }),
    )
    .await
    .unwrap();
    assert_eq!(updated.current_amount, 2500.0);
    assert_eq!(updated.name, "Emergency Fund"); // unchanged

    // Delete
    delete_savings_goal(st(pool.clone()), ext(claims.clone()), Path(goal.id))
        .await
        .unwrap();

    let Json(after_delete) = list_savings_goals(st(pool), ext(claims)).await.unwrap();
    assert!(after_delete.is_empty());
}

#[tokio::test]
async fn savings_goal_rejects_zero_target() {
    let (pool, claims) = setup().await;

    let result = create_savings_goal(
        st(pool),
        ext(claims),
        Json(CreateSavingsGoal {
            name: "Bad Goal".to_string(),
            current_amount: None,
            target_amount: 0.0, // invalid per validate(range(min = 0.01))
        }),
    )
    .await;

    assert!(result.is_err(), "zero target_amount should fail validation");
}

#[tokio::test]
async fn retirement_breakdown_full_crud() {
    let (pool, claims) = setup().await;

    // Create two items
    let (_, Json(item1)) = create_retirement_breakdown_item(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateRetirementBreakdownItem {
            label: "401k".to_string(),
            amount: 45000.0,
        }),
    )
    .await
    .unwrap();

    let (_, Json(item2)) = create_retirement_breakdown_item(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateRetirementBreakdownItem {
            label: "Roth IRA".to_string(),
            amount: 12000.0,
        }),
    )
    .await
    .unwrap();

    // List
    let Json(list) = list_retirement_breakdown(st(pool.clone()), ext(claims.clone()))
        .await
        .unwrap();
    assert_eq!(list.len(), 2);
    let total: f64 = list.iter().map(|i| i.amount).sum();
    assert_eq!(total, 57000.0);

    // Update
    let Json(updated) = update_retirement_breakdown_item(
        st(pool.clone()),
        ext(claims.clone()),
        Path(item1.id),
        Json(UpdateRetirementBreakdownItem {
            label: None,
            amount: Some(50000.0),
        }),
    )
    .await
    .unwrap();
    assert_eq!(updated.amount, 50000.0);
    assert_eq!(updated.label, "401k");

    // Delete one
    delete_retirement_breakdown_item(st(pool.clone()), ext(claims.clone()), Path(item2.id))
        .await
        .unwrap();

    let Json(after) = list_retirement_breakdown(st(pool), ext(claims))
        .await
        .unwrap();
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].label, "401k");
}

// ─── user isolation ──────────────────────────────────────────────────────────

#[tokio::test]
async fn categories_are_scoped_to_user() {
    let (pool, alice) = setup().await;
    let bob = add_user(&pool, "bob").await;

    create_category(
        st(pool.clone()),
        ext(alice.clone()),
        Json(CreateCategory {
            label: "Alice's Category".to_string(),
            default_amount: 100.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    let Json(bob_list) = list_categories(st(pool), ext(bob)).await.unwrap();
    assert!(bob_list.is_empty(), "bob should not see alice's categories");
}

#[tokio::test]
async fn savings_goals_are_scoped_to_user() {
    let (pool, alice) = setup().await;
    let bob = add_user(&pool, "bob").await;

    create_savings_goal(
        st(pool.clone()),
        ext(alice),
        Json(CreateSavingsGoal {
            name: "Alice's Goal".to_string(),
            current_amount: None,
            target_amount: 5000.0,
        }),
    )
    .await
    .unwrap();

    let Json(bob_goals) = list_savings_goals(st(pool), ext(bob)).await.unwrap();
    assert!(
        bob_goals.is_empty(),
        "bob should not see alice's savings goals"
    );
}

#[tokio::test]
async fn retirement_breakdown_is_scoped_to_user() {
    let (pool, alice) = setup().await;
    let bob = add_user(&pool, "bob").await;

    create_retirement_breakdown_item(
        st(pool.clone()),
        ext(alice),
        Json(CreateRetirementBreakdownItem {
            label: "Alice's 401k".to_string(),
            amount: 30000.0,
        }),
    )
    .await
    .unwrap();

    let Json(bob_items) = list_retirement_breakdown(st(pool), ext(bob)).await.unwrap();
    assert!(
        bob_items.is_empty(),
        "bob should not see alice's retirement breakdown"
    );
}

#[tokio::test]
async fn months_are_scoped_to_user() {
    let (pool, alice) = setup().await;
    let bob = add_user(&pool, "bob").await;

    create_month(
        st(pool.clone()),
        ext(alice),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2025,
            month: 1,
        }),
    )
    .await
    .unwrap();

    let Json(bob_months) = list_months(st(pool), ext(bob)).await.unwrap();
    assert!(bob_months.is_empty(), "bob should not see alice's months");
}

#[tokio::test]
async fn cannot_update_another_users_savings_goal() {
    let (pool, alice) = setup().await;
    let bob = add_user(&pool, "bob").await;

    let (_, Json(alice_goal)) = create_savings_goal(
        st(pool.clone()),
        ext(alice),
        Json(CreateSavingsGoal {
            name: "Alice's goal".to_string(),
            current_amount: None,
            target_amount: 1000.0,
        }),
    )
    .await
    .unwrap();

    // Bob tries to update Alice's goal.
    let result = update_savings_goal(
        st(pool),
        ext(bob),
        Path(alice_goal.id),
        Json(UpdateSavingsGoal {
            name: Some("Hijacked".to_string()),
            current_amount: None,
            target_amount: None,
        }),
    )
    .await;

    assert!(
        result.is_err(),
        "bob should not be able to update alice's goal"
    );
}

#[tokio::test]
async fn migrations_rebuild_legacy_not_null_items_table() {
    // Simulate a database from before items.category_id became nullable.
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    for sql in [
        "CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT NOT NULL UNIQUE, password_hash TEXT NOT NULL, savings REAL NOT NULL DEFAULT 0, savings_goal REAL NOT NULL DEFAULT 0, retirement_savings REAL NOT NULL DEFAULT 0, created_at TEXT NOT NULL DEFAULT (datetime('now')))",
        "CREATE TABLE budget_categories (id INTEGER PRIMARY KEY AUTOINCREMENT, user_id INTEGER NOT NULL, label TEXT NOT NULL, default_amount REAL NOT NULL, color TEXT NOT NULL DEFAULT '#71717a', sort_order INTEGER NOT NULL DEFAULT 0)",
        "CREATE TABLE months (id INTEGER PRIMARY KEY AUTOINCREMENT, user_id INTEGER NOT NULL, year INTEGER NOT NULL, month INTEGER NOT NULL, is_closed INTEGER NOT NULL DEFAULT 0, closed_at TEXT, UNIQUE(user_id, year, month))",
        "CREATE TABLE items (id INTEGER PRIMARY KEY AUTOINCREMENT, month_id INTEGER NOT NULL, category_id INTEGER NOT NULL, description TEXT NOT NULL, amount REAL NOT NULL, spent_on TEXT NOT NULL, savings_destination TEXT NOT NULL DEFAULT 'none', sort_order INTEGER NOT NULL DEFAULT 0, FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE, FOREIGN KEY (category_id) REFERENCES budget_categories(id) ON DELETE CASCADE)",
        "INSERT INTO users (id, username, password_hash) VALUES (1, 'alice', 'x')",
        "INSERT INTO budget_categories (id, user_id, label, default_amount) VALUES (7, 1, 'Food', 100.0)",
        "INSERT INTO months (id, user_id, year, month) VALUES (1, 1, 2026, 7)",
        "INSERT INTO items (id, month_id, category_id, description, amount, spent_on) VALUES (42, 1, 7, 'Lunch', 12.5, '2026-07-01')",
    ] {
        sqlx::query(sql).execute(&pool).await.unwrap();
    }

    run_migrations(&pool).await.unwrap();

    let items_sql: String =
        sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'items'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        !items_sql.contains("category_id INTEGER NOT NULL"),
        "rebuild must drop the NOT NULL constraint"
    );

    let (id, category_id, description, amount): (i64, Option<i64>, String, f64) =
        sqlx::query_as("SELECT id, category_id, description, amount FROM items WHERE id = 42")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        (id, category_id, description.as_str(), amount),
        (42, Some(7), "Lunch", 12.5)
    );

    // Idempotent: a second run must not attempt another rebuild.
    run_migrations(&pool).await.unwrap();
}

#[tokio::test]
async fn migrations_null_out_items_orphaned_by_old_hard_deletes() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    run_migrations(&pool).await.unwrap();

    // The pragma is per-connection, so pin one connection for the orphan insert.
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query("INSERT INTO users (id, username, password_hash) VALUES (1, 'alice', 'x')")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query("INSERT INTO months (id, user_id, year, month) VALUES (1, 1, 2026, 7)")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO items (id, month_id, category_id, description, amount, spent_on) VALUES (1, 1, 999, 'Ghost', 5.0, '2026-07-01')",
    )
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *conn)
        .await
        .unwrap();
    drop(conn);

    run_migrations(&pool).await.unwrap();

    let category_id: Option<i64> = sqlx::query_scalar("SELECT category_id FROM items WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(category_id, None, "orphaned reference is cleared");
}

#[tokio::test]
async fn migrations_give_open_months_a_line_for_every_live_category() {
    let (pool, claims) = setup().await;

    // A month created before the category existed, plus a closed one.
    let Json(open) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2026,
            month: 8,
        }),
    )
    .await
    .unwrap();
    let Json(closed) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2026,
            month: 9,
        }),
    )
    .await
    .unwrap();
    sqlx::query("UPDATE months SET is_closed = 1 WHERE id = ?")
        .bind(closed.month.id)
        .execute(&pool)
        .await
        .unwrap();

    // Created without naming a month, so it seeds nothing on its own.
    let Json(cat) = create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Travel".to_string(),
            default_amount: 250.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    let before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM monthly_budgets WHERE month_id = ? AND category_id = ?",
    )
    .bind(open.month.id)
    .bind(cat.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(before, 0, "the open month starts out missing the category");

    run_migrations(&pool).await.unwrap();

    let allocated: Option<f64> = sqlx::query_scalar(
        "SELECT allocated_amount FROM monthly_budgets WHERE month_id = ? AND category_id = ?",
    )
    .bind(open.month.id)
    .bind(cat.id)
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert_eq!(
        allocated,
        Some(250.0),
        "the open month gains a line at the category's default"
    );

    let closed_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM monthly_budgets WHERE month_id = ? AND category_id = ?",
    )
    .bind(closed.month.id)
    .bind(cat.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(closed_count, 0, "closed months stay as they were closed");
}

#[tokio::test]
async fn migrations_do_not_resurrect_a_category_removed_from_a_month() {
    let (pool, claims) = setup().await;

    let Json(cat) = create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Dining".to_string(),
            default_amount: 100.0,
            color: None,
            month_id: None,
        }),
    )
    .await
    .unwrap();

    let Json(month) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest {
            year: 2026,
            month: 8,
        }),
    )
    .await
    .unwrap();

    delete_month_category(
        st(pool.clone()),
        ext(claims.clone()),
        Path((month.month.id, cat.id)),
    )
    .await
    .unwrap();

    run_migrations(&pool).await.unwrap();

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM monthly_budgets WHERE month_id = ? AND category_id = ?",
    )
    .bind(month.month.id)
    .bind(cat.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        count, 0,
        "an archived category must not come back on restart"
    );
}
