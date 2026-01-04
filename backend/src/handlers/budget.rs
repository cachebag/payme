use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::middleware::auth::Claims;
use crate::models::{BudgetCategory, MonthlyBudget};

#[derive(Deserialize)]
pub struct CreateCategory {
    pub label: String,
    pub default_amount: f64,
}

#[derive(Deserialize)]
pub struct UpdateCategory {
    pub label: Option<String>,
    pub default_amount: Option<f64>,
}

#[derive(Deserialize)]
pub struct UpdateMonthlyBudget {
    pub allocated_amount: f64,
}

pub async fn list_categories(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<BudgetCategory>>, StatusCode> {
    let categories: Vec<BudgetCategory> = sqlx::query_as(
        "SELECT id, user_id, label, default_amount FROM budget_categories WHERE user_id = ?",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(categories))
}

pub async fn create_category(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateCategory>,
) -> Result<Json<BudgetCategory>, StatusCode> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO budget_categories (user_id, label, default_amount) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(claims.sub)
    .bind(&payload.label)
    .bind(payload.default_amount)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let open_months: Vec<(i64,)> =
        sqlx::query_as("SELECT id FROM months WHERE user_id = ? AND is_closed = 0")
            .bind(claims.sub)
            .fetch_all(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

pub async fn update_category(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(category_id): Path<i64>,
    Json(payload): Json<UpdateCategory>,
) -> Result<Json<BudgetCategory>, StatusCode> {
    let existing: BudgetCategory = sqlx::query_as(
        "SELECT id, user_id, label, default_amount FROM budget_categories WHERE id = ? AND user_id = ?",
    )
    .bind(category_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let label = payload.label.unwrap_or(existing.label);
    let default_amount = payload.default_amount.unwrap_or(existing.default_amount);

    sqlx::query("UPDATE budget_categories SET label = ?, default_amount = ? WHERE id = ?")
        .bind(&label)
        .bind(default_amount)
        .bind(category_id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(BudgetCategory {
        id: category_id,
        user_id: claims.sub,
        label,
        default_amount,
    }))
}

pub async fn delete_category(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(category_id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM budget_categories WHERE id = ? AND user_id = ?")
        .bind(category_id)
        .bind(claims.sub)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_monthly_budgets(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<Json<Vec<MonthlyBudget>>, StatusCode> {
    let _month: (i64,) = sqlx::query_as("SELECT id FROM months WHERE id = ? AND user_id = ?")
        .bind(month_id)
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let budgets: Vec<MonthlyBudget> = sqlx::query_as(
        "SELECT id, month_id, category_id, allocated_amount FROM monthly_budgets WHERE month_id = ?",
    )
    .bind(month_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(budgets))
}

pub async fn update_monthly_budget(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, budget_id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateMonthlyBudget>,
) -> Result<Json<MonthlyBudget>, StatusCode> {
    let month: (bool,) =
        sqlx::query_as("SELECT is_closed FROM months WHERE id = ? AND user_id = ?")
            .bind(month_id)
            .bind(claims.sub)
            .fetch_optional(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

    if month.0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let existing: MonthlyBudget = sqlx::query_as(
        "SELECT id, month_id, category_id, allocated_amount FROM monthly_budgets WHERE id = ? AND month_id = ?",
    )
    .bind(budget_id)
    .bind(month_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    sqlx::query("UPDATE monthly_budgets SET allocated_amount = ? WHERE id = ?")
        .bind(payload.allocated_amount)
        .bind(budget_id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MonthlyBudget {
        id: budget_id,
        month_id,
        category_id: existing.category_id,
        allocated_amount: payload.allocated_amount,
    }))
}
