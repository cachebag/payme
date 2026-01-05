use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::error::PaymeError;
use crate::middleware::auth::Claims;
use crate::models::{BudgetCategory, FixedExpense, IncomeEntry, Item, Month};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserExport {
    pub version: u32,
    pub savings: Option<f64>,
    pub roth_ira: Option<f64>,
    pub fixed_expenses: Vec<FixedExpenseExport>,
    pub categories: Vec<CategoryExport>,
    pub months: Vec<MonthExport>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FixedExpenseExport {
    pub label: String,
    pub amount: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CategoryExport {
    pub label: String,
    pub default_amount: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MonthExport {
    pub year: i32,
    pub month: i32,
    pub is_closed: bool,
    pub income_entries: Vec<IncomeExport>,
    pub budgets: Vec<BudgetExport>,
    pub items: Vec<ItemExport>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct IncomeExport {
    pub label: String,
    pub amount: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct BudgetExport {
    pub category_label: String,
    pub allocated_amount: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ItemExport {
    pub category_label: String,
    pub description: String,
    pub amount: f64,
    pub spent_on: String,
}

#[utoipa::path(
    get,
    path = "/api/export/json",
    responses(
        (status = 200, description = "A complete JSON export of all user data", body = UserExport),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error during database aggregation")
    ),
    tag = "Data Management",
    summary = "Export all data to JSON",
    description = "Gathers all user profile info, fixed expenses, categories, and monthly history into a single portable JSON object."
)]
pub async fn export_json(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<UserExport>, PaymeError> {
    let savings: f64 = sqlx::query_scalar("SELECT savings FROM users WHERE id = ?")
        .bind(claims.sub)
        .fetch_one(&pool)
        .await
        .unwrap_or(0.0);

    let roth_ira: f64 = sqlx::query_scalar("SELECT roth_ira FROM users WHERE id = ?")
        .bind(claims.sub)
        .fetch_one(&pool)
        .await
        .unwrap_or(0.0);

    let fixed_expenses: Vec<FixedExpense> =
        sqlx::query_as("SELECT id, user_id, label, amount FROM fixed_expenses WHERE user_id = ?")
            .bind(claims.sub)
            .fetch_all(&pool)
            .await?;

    let categories: Vec<BudgetCategory> = sqlx::query_as(
        "SELECT id, user_id, label, default_amount FROM budget_categories WHERE user_id = ?",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?;

    let months: Vec<Month> = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE user_id = ? ORDER BY year, month",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?;

    let mut month_exports = Vec::new();

    for m in &months {
        let income_entries: Vec<IncomeEntry> = sqlx::query_as(
            "SELECT id, month_id, label, amount FROM income_entries WHERE month_id = ?",
        )
        .bind(m.id)
        .fetch_all(&pool)
        .await?;

        let budgets: Vec<(String, f64)> = sqlx::query_as(
            r#"
            SELECT bc.label, mb.allocated_amount
            FROM monthly_budgets mb
            JOIN budget_categories bc ON mb.category_id = bc.id
            WHERE mb.month_id = ?
            "#,
        )
        .bind(m.id)
        .fetch_all(&pool)
        .await?;

        let items: Vec<Item> = sqlx::query_as(
            "SELECT id, month_id, category_id, description, amount, spent_on FROM items WHERE month_id = ?",
        )
        .bind(m.id)
        .fetch_all(&pool)
        .await?;

        let mut item_exports = Vec::new();
        for item in items {
            let cat = categories.iter().find(|c| c.id == item.category_id);
            if let Some(cat) = cat {
                item_exports.push(ItemExport {
                    category_label: cat.label.clone(),
                    description: item.description,
                    amount: item.amount,
                    spent_on: item.spent_on.to_string(),
                });
            }
        }

        month_exports.push(MonthExport {
            year: m.year,
            month: m.month,
            is_closed: m.is_closed,
            income_entries: income_entries
                .into_iter()
                .map(|i| IncomeExport {
                    label: i.label,
                    amount: i.amount,
                })
                .collect(),
            budgets: budgets
                .into_iter()
                .map(|(label, amount)| BudgetExport {
                    category_label: label,
                    allocated_amount: amount,
                })
                .collect(),
            items: item_exports,
        });
    }

    Ok(Json(UserExport {
        version: 1,
        savings: Some(savings),
        roth_ira: Some(roth_ira),
        fixed_expenses: fixed_expenses
            .into_iter()
            .map(|e| FixedExpenseExport {
                label: e.label,
                amount: e.amount,
            })
            .collect(),
        categories: categories
            .into_iter()
            .map(|c| CategoryExport {
                label: c.label,
                default_amount: c.default_amount,
            })
            .collect(),
        months: month_exports,
    }))
}

#[utoipa::path(
    post,
    path = "/api/import/json",
    request_body = UserExport,
    responses(
        (status = 200, description = "Data imported successfully. Note: This overwrites existing user data."),
        (status = 500, description = "Internal server error during database restoration")
    ),
    tag = "Data Management",
    summary = "Import data from JSON",
    description = "Overwrites the current user's database records with the provided JSON export. This action is destructive and irreversible."
)]
pub async fn import_json(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(data): Json<UserExport>,
) -> Result<StatusCode, PaymeError> {
    let mut tx = pool.begin().await?;

    let months: Vec<(i64,)> = sqlx::query_as("SELECT id FROM months WHERE user_id = ?")
        .bind(claims.sub)
        .fetch_all(&mut *tx)
        .await?;

    for (month_id,) in &months {
        sqlx::query("DELETE FROM items WHERE month_id = ?")
            .bind(month_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM monthly_budgets WHERE month_id = ?")
            .bind(month_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM income_entries WHERE month_id = ?")
            .bind(month_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM monthly_snapshots WHERE month_id = ?")
            .bind(month_id)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM months WHERE user_id = ?")
        .bind(claims.sub)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM budget_categories WHERE user_id = ?")
        .bind(claims.sub)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM fixed_expenses WHERE user_id = ?")
        .bind(claims.sub)
        .execute(&mut *tx)
        .await?;

    if let Some(savings) = data.savings {
        sqlx::query("UPDATE users SET savings = ? WHERE id = ?")
            .bind(savings)
            .bind(claims.sub)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(roth_ira) = data.roth_ira {
        sqlx::query("UPDATE users SET roth_ira = ? WHERE id = ?")
            .bind(roth_ira)
            .bind(claims.sub)
            .execute(&mut *tx)
            .await?;
    }

    for expense in &data.fixed_expenses {
        sqlx::query("INSERT INTO fixed_expenses (user_id, label, amount) VALUES (?, ?, ?)")
            .bind(claims.sub)
            .bind(&expense.label)
            .bind(expense.amount)
            .execute(&mut *tx)
            .await?;
    }

    let mut category_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for cat in &data.categories {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO budget_categories (user_id, label, default_amount) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(claims.sub)
        .bind(&cat.label)
        .bind(cat.default_amount)
        .fetch_one(&mut *tx)
        .await?;
        category_map.insert(cat.label.clone(), id);
    }

    for month_data in &data.months {
        let month_id: i64 = sqlx::query_scalar(
            "INSERT INTO months (user_id, year, month, is_closed) VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(claims.sub)
        .bind(month_data.year)
        .bind(month_data.month)
        .bind(month_data.is_closed)
        .fetch_one(&mut *tx)
        .await?;

        for income in &month_data.income_entries {
            sqlx::query("INSERT INTO income_entries (month_id, label, amount) VALUES (?, ?, ?)")
                .bind(month_id)
                .bind(&income.label)
                .bind(income.amount)
                .execute(&mut *tx)
                .await?;
        }

        for budget in &month_data.budgets {
            if let Some(&cat_id) = category_map.get(&budget.category_label) {
                sqlx::query(
                    "INSERT INTO monthly_budgets (month_id, category_id, allocated_amount) VALUES (?, ?, ?)",
                )
                .bind(month_id)
                .bind(cat_id)
                .bind(budget.allocated_amount)
                .execute(&mut *tx)
                .await?;
            }
        }

        for item in &month_data.items {
            if let Some(&cat_id) = category_map.get(&item.category_label) {
                sqlx::query(
                    "INSERT INTO items (month_id, category_id, description, amount, spent_on) VALUES (?, ?, ?, ?, ?)",
                )
                .bind(month_id)
                .bind(cat_id)
                .bind(&item.description)
                .bind(item.amount)
                .bind(&item.spent_on)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    tx.commit().await?;
    Ok(StatusCode::OK)
}
