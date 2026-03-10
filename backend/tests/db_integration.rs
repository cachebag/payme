#![allow(unused_must_use)]

use axum::extract::{Path, State};
use axum::Json;
use payme::db::run_migrations;
use payme::handlers::budget::{
    create_category, delete_category, list_categories, update_category, update_monthly_budget,
    CreateCategory, UpdateCategory, UpdateMonthlyBudget,
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
    let user_id: i64 =
        sqlx::query_scalar("INSERT INTO users (username, password_hash) VALUES (?, ?) RETURNING id")
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
        }),
    )
    .await
    .unwrap();

    delete_category(st(pool.clone()), ext(claims.clone()), Path(cat.id))
        .await
        .unwrap();

    let Json(list) = list_categories(st(pool), ext(claims)).await.unwrap();
    assert!(list.is_empty());
}

#[tokio::test]
async fn category_delete_cascades_to_monthly_budgets() {
    let (pool, claims) = setup().await;

    // Create a category then a month (which seeds monthly_budgets).
    let Json(cat) = create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Transport".to_string(),
            default_amount: 150.0,
            color: None,
        }),
    )
    .await
    .unwrap();

    create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest { year: 2025, month: 1 }),
    )
    .await
    .unwrap();

    let budget_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM monthly_budgets WHERE category_id = ?")
            .bind(cat.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(budget_count, 1, "monthly_budget row should exist before delete");

    // Deleting the category must cascade.
    delete_category(st(pool.clone()), ext(claims.clone()), Path(cat.id))
        .await
        .unwrap();

    let budget_count_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM monthly_budgets WHERE category_id = ?")
            .bind(cat.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(budget_count_after, 0, "monthly_budget rows should be deleted by cascade");
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
        }),
    )
    .await
    .unwrap();

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest { year: 2025, month: 3 }),
    )
    .await
    .unwrap();

    assert_eq!(summary.budgets.len(), 2);
    let rent = summary.budgets.iter().find(|b| b.category_label == "Rent").unwrap();
    assert_eq!(rent.allocated_amount, 1500.0);
}

#[tokio::test]
async fn month_creation_is_idempotent() {
    let (pool, claims) = setup().await;

    let Json(first) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest { year: 2025, month: 6 }),
    )
    .await
    .unwrap();

    let Json(second) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest { year: 2025, month: 6 }),
    )
    .await
    .unwrap();

    assert_eq!(first.month.id, second.month.id, "creating the same month twice should return the same record");

    let Json(months) = list_months(st(pool), ext(claims)).await.unwrap();
    assert_eq!(months.len(), 1, "only one month row should exist");
}

#[tokio::test]
async fn month_creation_rejects_invalid_month_number() {
    let (pool, claims) = setup().await;

    let result = create_month(
        st(pool),
        ext(claims),
        Json(payme::handlers::months::CreateMonthRequest { year: 2025, month: 13 }),
    )
    .await;

    assert!(result.is_err(), "month 13 should be rejected");
}

#[tokio::test]
async fn category_created_mid_month_seeds_open_months() {
    // The key invariant: if you add a new category while a month is already
    // open, that month should immediately get a monthly_budget row for it.
    let (pool, claims) = setup().await;

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest { year: 2025, month: 4 }),
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

    // Now create a category — it should be retroactively seeded into the open month.
    create_category(
        st(pool.clone()),
        ext(claims.clone()),
        Json(CreateCategory {
            label: "Entertainment".to_string(),
            default_amount: 100.0,
            color: None,
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
    assert_eq!(count_after, 1, "new category should be seeded into the existing open month");
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
        }),
    )
    .await
    .unwrap();

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest { year: 2025, month: 7 }),
    )
    .await
    .unwrap();

    let budget = &summary.budgets[0];
    assert_eq!(budget.allocated_amount, 50.0);

    let Json(updated) = update_monthly_budget(
        st(pool.clone()),
        ext(claims.clone()),
        Path((summary.month.id, budget.id)),
        Json(UpdateMonthlyBudget { allocated_amount: 200.0 }),
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
        }),
    )
    .await
    .unwrap();

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest { year: 2025, month: 8 }),
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
        Json(UpdateMonthlyBudget { allocated_amount: 999.0 }),
    )
    .await;

    assert!(result.is_err(), "updating a budget on a closed month should fail");
}

#[tokio::test]
async fn close_then_reopen_month() {
    use payme::handlers::months::close_month;
    let (pool, claims) = setup().await;

    let Json(summary) = create_month(
        st(pool.clone()),
        ext(claims.clone()),
        Json(payme::handlers::months::CreateMonthRequest { year: 2025, month: 9 }),
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
        Json(payme::handlers::months::CreateMonthRequest { year: 2025, month: 1 }),
    )
    .await
    .unwrap();
    let month_id = summary.month.id;

    create_income(
        st(pool.clone()),
        ext(claims.clone()),
        Path(month_id),
        Json(CreateIncome { label: "Salary".to_string(), amount: 5000.0 }),
    )
    .await
    .unwrap();

    create_income(
        st(pool.clone()),
        ext(claims.clone()),
        Path(month_id),
        Json(CreateIncome { label: "Freelance".to_string(), amount: 800.0 }),
    )
    .await
    .unwrap();

    let Json(entries) = list_income(st(pool), ext(claims), Path(month_id)).await.unwrap();
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
    let Json(list) = list_savings_goals(st(pool.clone()), ext(claims.clone())).await.unwrap();
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

    let Json(after) = list_retirement_breakdown(st(pool), ext(claims)).await.unwrap();
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
    assert!(bob_goals.is_empty(), "bob should not see alice's savings goals");
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
    assert!(bob_items.is_empty(), "bob should not see alice's retirement breakdown");
}

#[tokio::test]
async fn months_are_scoped_to_user() {
    let (pool, alice) = setup().await;
    let bob = add_user(&pool, "bob").await;

    create_month(
        st(pool.clone()),
        ext(alice),
        Json(payme::handlers::months::CreateMonthRequest { year: 2025, month: 1 }),
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

    assert!(result.is_err(), "bob should not be able to update alice's goal");
}
