use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FixedExpense {
    pub id: i64,
    pub user_id: i64,
    pub label: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BudgetCategory {
    pub id: i64,
    pub user_id: i64,
    pub label: String,
    pub default_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Month {
    pub id: i64,
    pub user_id: i64,
    pub year: i32,
    pub month: i32,
    pub is_closed: bool,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IncomeEntry {
    pub id: i64,
    pub month_id: i64,
    pub label: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MonthlyBudget {
    pub id: i64,
    pub month_id: i64,
    pub category_id: i64,
    pub allocated_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: i64,
    pub month_id: i64,
    pub category_id: i64,
    pub description: String,
    pub amount: f64,
    pub spent_on: NaiveDate,
}

#[derive(Debug, Serialize)]
pub struct MonthlyBudgetWithCategory {
    pub id: i64,
    pub month_id: i64,
    pub category_id: i64,
    pub category_label: String,
    pub allocated_amount: f64,
    pub spent_amount: f64,
}

#[derive(Debug, Serialize)]
pub struct MonthSummary {
    pub month: Month,
    pub income_entries: Vec<IncomeEntry>,
    pub fixed_expenses: Vec<FixedExpense>,
    pub budgets: Vec<MonthlyBudgetWithCategory>,
    pub items: Vec<ItemWithCategory>,
    pub total_income: f64,
    pub total_fixed: f64,
    pub total_budgeted: f64,
    pub total_spent: f64,
    pub remaining: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemWithCategory {
    pub id: i64,
    pub month_id: i64,
    pub category_id: i64,
    pub category_label: String,
    pub description: String,
    pub amount: f64,
    pub spent_on: NaiveDate,
}

#[derive(Debug, Serialize)]
pub struct CategoryStats {
    pub category_id: i64,
    pub category_label: String,
    pub current_month_spent: f64,
    pub previous_month_spent: f64,
    pub change_amount: f64,
    pub change_percent: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct MonthlyStats {
    pub year: i32,
    pub month: i32,
    pub total_income: f64,
    pub total_spent: f64,
    pub total_fixed: f64,
    pub net: f64,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub category_comparisons: Vec<CategoryStats>,
    pub monthly_trends: Vec<MonthlyStats>,
    pub average_monthly_spending: f64,
    pub average_monthly_income: f64,
}
