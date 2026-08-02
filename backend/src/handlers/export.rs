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
    pub retirement_savings: Option<f64>,
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
    pub color: String,
    /// Retired categories are kept so historical months still resolve their labels.
    #[serde(default)]
    pub archived: bool,
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
    #[serde(default)]
    pub paid_on: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct BudgetExport {
    pub category_label: String,
    pub allocated_amount: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ItemExport {
    /// `None` means the transaction is uncategorized.
    #[serde(default)]
    pub category_label: Option<String>,
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

    let retirement_savings: f64 =
        sqlx::query_scalar("SELECT retirement_savings FROM users WHERE id = ?")
            .bind(claims.sub)
            .fetch_one(&pool)
            .await
            .unwrap_or(0.0);

    let fixed_expenses: Vec<FixedExpense> =
        sqlx::query_as("SELECT id, user_id, label, amount FROM fixed_expenses WHERE user_id = ? ORDER BY sort_order, id")
            .bind(claims.sub)
            .fetch_all(&pool)
            .await?;

    let categories: Vec<BudgetCategory> = sqlx::query_as(
        "SELECT id, user_id, label, default_amount, color FROM budget_categories WHERE user_id = ? ORDER BY sort_order, id",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?;

    let archived_category_ids: std::collections::HashSet<i64> = sqlx::query_scalar(
        "SELECT id FROM budget_categories WHERE user_id = ? AND archived_at IS NOT NULL",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?
    .into_iter()
    .collect();

    let months: Vec<Month> = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE user_id = ? ORDER BY year, month",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?;

    let mut month_exports = Vec::new();

    for m in &months {
        let income_entries: Vec<IncomeEntry> = sqlx::query_as(
            "SELECT id, month_id, label, amount, paid_on FROM income_entries WHERE month_id = ? ORDER BY sort_order, id",
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
            ORDER BY bc.sort_order, bc.id
            "#,
        )
        .bind(m.id)
        .fetch_all(&pool)
        .await?;

        let items: Vec<Item> = sqlx::query_as(
            "SELECT id, month_id, category_id, description, amount, spent_on, savings_destination FROM items WHERE month_id = ? ORDER BY sort_order, id",
        )
        .bind(m.id)
        .fetch_all(&pool)
        .await?;

        let mut item_exports = Vec::new();
        for item in items {
            // Uncategorized transactions export with no label rather than being dropped.
            let category_label = item
                .category_id
                .and_then(|id| categories.iter().find(|c| c.id == id))
                .map(|c| c.label.clone());
            item_exports.push(ItemExport {
                category_label,
                description: item.description,
                amount: item.amount,
                spent_on: item.spent_on.to_string(),
            });
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
                    paid_on: i.paid_on.map(|date| date.to_string()),
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
        retirement_savings: Some(retirement_savings),
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
                archived: archived_category_ids.contains(&c.id),
                label: c.label,
                default_amount: c.default_amount,
                color: c.color,
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

    if let Some(retirement_savings) = data.retirement_savings {
        sqlx::query("UPDATE users SET retirement_savings = ? WHERE id = ?")
            .bind(retirement_savings)
            .bind(claims.sub)
            .execute(&mut *tx)
            .await?;
    }

    for (index, expense) in data.fixed_expenses.iter().enumerate() {
        sqlx::query(
            "INSERT INTO fixed_expenses (user_id, label, amount, sort_order) VALUES (?, ?, ?, ?)",
        )
        .bind(claims.sub)
        .bind(&expense.label)
        .bind(expense.amount)
        .bind(index as i64)
        .execute(&mut *tx)
        .await?;
    }

    let mut category_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for (index, cat) in data.categories.iter().enumerate() {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO budget_categories (user_id, label, default_amount, color, sort_order, archived_at) VALUES (?, ?, ?, ?, ?, CASE WHEN ? THEN datetime('now') END) RETURNING id",
        )
        .bind(claims.sub)
        .bind(&cat.label)
        .bind(cat.default_amount)
        .bind(&cat.color)
        .bind(index as i64)
        .bind(cat.archived)
        .fetch_one(&mut *tx)
        .await?;

        // The export identifies categories by label, so a retired category and a live one
        // sharing a name collapse into a single mapping. Point it at the live one.
        if !cat.archived || !category_map.contains_key(&cat.label) {
            category_map.insert(cat.label.clone(), id);
        }
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

        for (index, income) in month_data.income_entries.iter().enumerate() {
            sqlx::query("INSERT INTO income_entries (month_id, label, amount, paid_on, sort_order) VALUES (?, ?, ?, ?, ?)")
                .bind(month_id)
                .bind(&income.label)
                .bind(income.amount)
                .bind(income.paid_on.as_deref())
                .bind(index as i64)
                .execute(&mut *tx)
                .await?;
        }

        for budget in &month_data.budgets {
            if let Some(&cat_id) = category_map.get(&budget.category_label) {
                // Two same-named categories can both hold a line in one month; they collapse
                // onto one mapping here, so keep the first rather than failing the import.
                sqlx::query(
                    "INSERT OR IGNORE INTO monthly_budgets (month_id, category_id, allocated_amount) VALUES (?, ?, ?)",
                )
                .bind(month_id)
                .bind(cat_id)
                .bind(budget.allocated_amount)
                .execute(&mut *tx)
                .await?;
            }
        }

        for (index, item) in month_data.items.iter().enumerate() {
            // A missing or unmapped label imports as uncategorized instead of losing the row.
            let cat_id: Option<i64> = item
                .category_label
                .as_ref()
                .and_then(|label| category_map.get(label).copied());
            sqlx::query(
                "INSERT INTO items (month_id, category_id, description, amount, spent_on, sort_order) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(month_id)
            .bind(cat_id)
            .bind(&item.description)
            .bind(item.amount)
            .bind(&item.spent_on)
            .bind(index as i64)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(StatusCode::OK)
}
