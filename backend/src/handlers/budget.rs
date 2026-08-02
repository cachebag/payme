use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;
use validator::Validate;

use crate::error::PaymeError;
use crate::handlers::monthly_data::later_open_month_ids;
use crate::middleware::auth::Claims;
use crate::models::{BudgetCategory, MonthlyBudget};

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateCategory {
    #[validate(length(min = 1, max = 100))]
    pub label: String,
    #[validate(range(min = 0.0))]
    pub default_amount: f64,
    pub color: Option<String>,
    /// The month the category is being added from. It is given an allocation there and in
    /// every later open month; omit it to create the template without touching any month.
    #[serde(default)]
    pub month_id: Option<i64>,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateCategory {
    #[validate(length(min = 1, max = 100))]
    pub label: Option<String>,
    #[validate(range(min = 0.0))]
    pub default_amount: Option<f64>,
    pub color: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct ReorderCategories {
    pub ids: Vec<i64>,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateMonthlyBudget {
    #[validate(range(min = 0.0))]
    pub allocated_amount: f64,
}

#[utoipa::path(
    get,
    path = "/api/categories",
    responses(
        (status = 200, body = [BudgetCategory]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Configuration",
    summary = "List all categories",
    description = "Retrieves all budget categories used as templates for new months."
)]
pub async fn list_categories(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<BudgetCategory>>, PaymeError> {
    let categories: Vec<BudgetCategory> = sqlx::query_as(
        "SELECT id, user_id, label, default_amount, color FROM budget_categories WHERE user_id = ? AND archived_at IS NULL ORDER BY sort_order, id",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?;

    Ok(Json(categories))
}

#[utoipa::path(
    post,
    path = "/api/categories",
    request_body = CreateCategory,
    responses(
        (status = 201, description = "Category created and added from month_id onward", body = BudgetCategory),
        (status = 400, description = "Month is closed"),
        (status = 404, description = "Month not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Configuration",
    summary = "Create a category",
    description = "Creates a new category template and gives it an allocation in the month it was added from and every later open month. Earlier months are left as they were."
)]
pub async fn create_category(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateCategory>,
) -> Result<Json<BudgetCategory>, PaymeError> {
    payload.validate()?;
    let color = payload.color.unwrap_or_else(|| "#71717a".to_string());
    let sort_order: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM budget_categories WHERE user_id = ?",
    )
    .bind(claims.sub)
    .fetch_one(&pool)
    .await?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO budget_categories (user_id, label, default_amount, color, sort_order) VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(claims.sub)
    .bind(&payload.label)
    .bind(payload.default_amount)
    .bind(&color)
    .bind(sort_order)
    .fetch_one(&pool)
    .await?;

    // A category starts in the month you added it from and carries forward into later open
    // months. Earlier months are settled: adding a category now would change what they say
    // was budgeted back then.
    if let Some(month_id) = payload.month_id {
        let (year, month, is_closed): (i64, i64, bool) = sqlx::query_as(
            "SELECT year, month, is_closed FROM months WHERE id = ? AND user_id = ?",
        )
        .bind(month_id)
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or(PaymeError::NotFound)?;

        if is_closed {
            return Err(PaymeError::BadRequest("Month is closed".to_string()));
        }

        let mut target_months = vec![month_id];
        target_months.extend(later_open_month_ids(&pool, claims.sub, year, month).await?);

        for target_month_id in target_months {
            sqlx::query(
                "INSERT OR IGNORE INTO monthly_budgets (month_id, category_id, allocated_amount) VALUES (?, ?, ?)",
            )
            .bind(target_month_id)
            .bind(id)
            .bind(payload.default_amount)
            .execute(&pool)
            .await
            .ok();
        }
    }

    Ok(Json(BudgetCategory {
        id,
        user_id: claims.sub,
        label: payload.label,
        default_amount: payload.default_amount,
        color,
    }))
}

#[utoipa::path(
    put,
    path = "/api/categories/{id}",
    params(("id" = i64, Path, description = "Category ID")),
    request_body = UpdateCategory,
    responses(
        (status = 200, body = BudgetCategory),
        (status = 500, description = "Internal server error")
    ),
    tag = "Configuration",
    summary = "Update a category",
    description = "Updates the label or default amount for a category template."
)]
pub async fn update_category(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(category_id): Path<i64>,
    Json(payload): Json<UpdateCategory>,
) -> Result<Json<BudgetCategory>, PaymeError> {
    payload.validate()?;
    let existing: BudgetCategory = sqlx::query_as(
        "SELECT id, user_id, label, default_amount, color FROM budget_categories WHERE id = ? AND user_id = ? AND archived_at IS NULL",
    )
    .bind(category_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    let label = payload.label.unwrap_or(existing.label);
    let default_amount = payload.default_amount.unwrap_or(existing.default_amount);
    let color = payload.color.unwrap_or(existing.color);

    sqlx::query(
        "UPDATE budget_categories SET label = ?, default_amount = ?, color = ? WHERE id = ?",
    )
    .bind(&label)
    .bind(default_amount)
    .bind(&color)
    .bind(category_id)
    .execute(&pool)
    .await?;

    Ok(Json(BudgetCategory {
        id: category_id,
        user_id: claims.sub,
        label,
        default_amount,
        color,
    }))
}

pub async fn reorder_categories(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<ReorderCategories>,
) -> Result<StatusCode, PaymeError> {
    for (index, id) in payload.ids.iter().enumerate() {
        sqlx::query("UPDATE budget_categories SET sort_order = ? WHERE id = ? AND user_id = ?")
            .bind(index as i64)
            .bind(id)
            .bind(claims.sub)
            .execute(&pool)
            .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/months/{month_id}/categories/{id}",
    params(
        ("month_id" = i64, Path, description = "Month the category is being removed from"),
        ("id" = i64, Path, description = "Category ID")
    ),
    responses(
        (status = 204, description = "Removed from this month onward"),
        (status = 400, description = "Month is closed"),
        (status = 404, description = "Month or category not found")
    ),
    tag = "Budgets",
    summary = "Stop using a category",
    description = "Removes a category from this month and every later open month; its transactions in those months become uncategorized. Earlier months and closed months are left untouched."
)]
pub async fn delete_month_category(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, category_id)): Path<(i64, i64)>,
) -> Result<StatusCode, PaymeError> {
    let (year, month, is_closed): (i64, i64, bool) =
        sqlx::query_as("SELECT year, month, is_closed FROM months WHERE id = ? AND user_id = ?")
            .bind(month_id)
            .bind(claims.sub)
            .fetch_optional(&pool)
            .await?
            .ok_or(PaymeError::NotFound)?;

    if is_closed {
        return Err(PaymeError::BadRequest("Month is closed".to_string()));
    }

    let _category: (i64,) =
        sqlx::query_as("SELECT id FROM budget_categories WHERE id = ? AND user_id = ?")
            .bind(category_id)
            .bind(claims.sub)
            .fetch_optional(&pool)
            .await?
            .ok_or(PaymeError::NotFound)?;

    // Retire the template rather than deleting the row. The category stops being offered for
    // new transactions and stops seeding new months, but earlier months keep resolving their
    // allocations and transactions through it for the label and color.
    sqlx::query("UPDATE budget_categories SET archived_at = datetime('now') WHERE id = ? AND user_id = ? AND archived_at IS NULL")
        .bind(category_id)
        .bind(claims.sub)
        .execute(&pool)
        .await?;

    // Deleting means "this category stops here": drop its allocation from this month and every
    // later open month. Earlier months and closed months are settled history and are left alone.
    let mut target_months = vec![month_id];
    target_months.extend(later_open_month_ids(&pool, claims.sub, year, month).await?);

    for target_month_id in target_months {
        sqlx::query("DELETE FROM monthly_budgets WHERE month_id = ? AND category_id = ?")
            .bind(target_month_id)
            .bind(category_id)
            .execute(&pool)
            .await?;

        // The transactions themselves are never removed, but the label goes with the
        // category: they become uncategorized instead of pointing at a ghost. Earlier and
        // closed months keep resolving the old label through the archived row.
        sqlx::query("UPDATE items SET category_id = NULL WHERE month_id = ? AND category_id = ?")
            .bind(target_month_id)
            .bind(category_id)
            .execute(&pool)
            .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/months/{id}/budgets",
    params(("id" = i64, Path, description = "Month ID")),
    responses(
        (status = 200, body = [MonthlyBudget]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Budgets",
    summary = "List monthly allocations",
    description = "Retrieves the specific budget allocations for a specific month."
)]
pub async fn list_monthly_budgets(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<Json<Vec<MonthlyBudget>>, PaymeError> {
    let _month: (i64,) = sqlx::query_as("SELECT id FROM months WHERE id = ? AND user_id = ?")
        .bind(month_id)
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or(PaymeError::NotFound)?;

    let budgets: Vec<MonthlyBudget> = sqlx::query_as(
        "SELECT id, month_id, category_id, allocated_amount FROM monthly_budgets WHERE month_id = ?",
    )
    .bind(month_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(budgets))
}

#[utoipa::path(
    put,
    path = "/api/months/{month_id}/budgets/{id}",
    params(
        ("month_id" = i64, Path, description = "Month ID"),
        ("id" = i64, Path, description = "Budget ID")
    ),
    request_body = UpdateMonthlyBudget,
    responses(
        (status = 200, body = MonthlyBudget),
        (status = 500, description = "Internal server error")
    ),
    tag = "Budgets",
    summary = "Update monthly allocation",
    description = "Adjust the amount of money allocated to a specific category for a specific month."
)]
pub async fn update_monthly_budget(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, budget_id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateMonthlyBudget>,
) -> Result<Json<MonthlyBudget>, PaymeError> {
    payload.validate()?;
    let month: (bool,) =
        sqlx::query_as("SELECT is_closed FROM months WHERE id = ? AND user_id = ?")
            .bind(month_id)
            .bind(claims.sub)
            .fetch_optional(&pool)
            .await?
            .ok_or(PaymeError::NotFound)?;

    if month.0 {
        return Err(PaymeError::BadRequest("Month is closed".to_string()));
    }

    let existing: MonthlyBudget = sqlx::query_as(
        "SELECT id, month_id, category_id, allocated_amount FROM monthly_budgets WHERE id = ? AND month_id = ?",
    )
    .bind(budget_id)
    .bind(month_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    sqlx::query("UPDATE monthly_budgets SET allocated_amount = ? WHERE id = ?")
        .bind(payload.allocated_amount)
        .bind(budget_id)
        .execute(&pool)
        .await?;

    Ok(Json(MonthlyBudget {
        id: budget_id,
        month_id,
        category_id: existing.category_id,
        allocated_amount: payload.allocated_amount,
    }))
}
