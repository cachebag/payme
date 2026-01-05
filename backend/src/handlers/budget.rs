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
use crate::middleware::auth::Claims;
use crate::models::{BudgetCategory, MonthlyBudget};

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateCategory {
    #[validate(length(min = 1, max = 100))]
    pub label: String,
    #[validate(range(min = 0.0))]
    pub default_amount: f64,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateCategory {
    #[validate(length(min = 1, max = 100))]
    pub label: Option<String>,
    #[validate(range(min = 0.0))]
    pub default_amount: Option<f64>,
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
        "SELECT id, user_id, label, default_amount FROM budget_categories WHERE user_id = ?",
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
        (status = 201, description = "Category created and added to open months", body = BudgetCategory),
        (status = 500, description = "Internal server error")
    ),
    tag = "Configuration",
    summary = "Create a category",
    description = "Creates a new category template."
)]
pub async fn create_category(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateCategory>,
) -> Result<Json<BudgetCategory>, PaymeError> {
    payload.validate()?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO budget_categories (user_id, label, default_amount) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(claims.sub)
    .bind(&payload.label)
    .bind(payload.default_amount)
    .fetch_one(&pool)
    .await?;

    let open_months: Vec<(i64,)> =
        sqlx::query_as("SELECT id FROM months WHERE user_id = ? AND is_closed = 0")
            .bind(claims.sub)
            .fetch_all(&pool)
            .await?;

    for (month_id,) in open_months {
        sqlx::query(
            "INSERT OR IGNORE INTO monthly_budgets (month_id, category_id, allocated_amount) VALUES (?, ?, ?)",
        )
        .bind(month_id)
        .bind(id)
        .bind(payload.default_amount)
        .execute(&pool)
        .await
        .ok();
    }

    Ok(Json(BudgetCategory {
        id,
        user_id: claims.sub,
        label: payload.label,
        default_amount: payload.default_amount,
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
        "SELECT id, user_id, label, default_amount FROM budget_categories WHERE id = ? AND user_id = ?",
    )
    .bind(category_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    let label = payload.label.unwrap_or(existing.label);
    let default_amount = payload.default_amount.unwrap_or(existing.default_amount);

    sqlx::query("UPDATE budget_categories SET label = ?, default_amount = ? WHERE id = ?")
        .bind(&label)
        .bind(default_amount)
        .bind(category_id)
        .execute(&pool)
        .await?;

    Ok(Json(BudgetCategory {
        id: category_id,
        user_id: claims.sub,
        label,
        default_amount,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/categories/{id}",
    params(("id" = i64, Path, description = "Category ID")),
    responses((status = 204, description = "Deleted")),
    tag = "Configuration",
    summary = "Delete global category",
)]
pub async fn delete_category(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(category_id): Path<i64>,
) -> Result<StatusCode, PaymeError> {
    sqlx::query("DELETE FROM budget_categories WHERE id = ? AND user_id = ?")
        .bind(category_id)
        .bind(claims.sub)
        .execute(&pool)
        .await?;

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
