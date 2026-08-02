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
use crate::models::{MonthlyFixedExpense, MonthlySavings};

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateMonthlyFixedExpense {
    #[validate(length(min = 1, max = 100))]
    pub label: String,
    #[validate(range(min = 0.0))]
    pub amount: f64,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateMonthlyFixedExpense {
    #[validate(length(min = 1, max = 100))]
    pub label: Option<String>,
    #[validate(range(min = 0.0))]
    pub amount: Option<f64>,
}

#[derive(Deserialize, ToSchema)]
pub struct ReorderMonthlyFixedExpenses {
    pub ids: Vec<i64>,
}

#[utoipa::path(
    post,
    path = "/api/months/{month_id}/fixed-expenses",
    params(("month_id" = i64, Path, description = "Month ID")),
    request_body = CreateMonthlyFixedExpense,
    responses(
        (status = 201, body = MonthlyFixedExpense),
        (status = 404, description = "Month not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Months",
    summary = "Add fixed expense to specific month",
    description = "Adds a fixed expense to a specific month's snapshot."
)]
pub async fn create_monthly_fixed_expense(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
    Json(payload): Json<CreateMonthlyFixedExpense>,
) -> Result<Json<MonthlyFixedExpense>, PaymeError> {
    payload.validate()?;

    let (year, month): (i64, i64) =
        sqlx::query_as("SELECT year, month FROM months WHERE id = ? AND user_id = ?")
            .bind(month_id)
            .bind(claims.sub)
            .fetch_optional(&pool)
            .await?
            .ok_or(PaymeError::NotFound)?;

    let sort_order: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM monthly_fixed_expenses WHERE month_id = ?",
    )
    .bind(month_id)
    .fetch_one(&pool)
    .await?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO monthly_fixed_expenses (month_id, label, amount, sort_order) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(month_id)
    .bind(&payload.label)
    .bind(payload.amount)
    .bind(sort_order)
    .fetch_one(&pool)
    .await?;

    // The new row starts its own group; the group links copies of this expense across
    // months so later edits can propagate forward.
    sqlx::query("UPDATE monthly_fixed_expenses SET group_id = id WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    // A fixed expense recurs until stopped: add it to every later open month too.
    // Closed months are settled history and are left alone.
    let later_open_months = later_open_month_ids(&pool, claims.sub, year, month).await?;
    for later_month_id in later_open_months {
        let sort_order: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM monthly_fixed_expenses WHERE month_id = ?",
        )
        .bind(later_month_id)
        .fetch_one(&pool)
        .await?;

        sqlx::query(
            "INSERT INTO monthly_fixed_expenses (month_id, label, amount, sort_order, group_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(later_month_id)
        .bind(&payload.label)
        .bind(payload.amount)
        .bind(sort_order)
        .bind(id)
        .execute(&pool)
        .await?;
    }

    Ok(Json(MonthlyFixedExpense {
        id,
        month_id,
        label: payload.label,
        amount: payload.amount,
    }))
}

/// IDs of all of the user's open (not closed) months strictly after `year`/`month`,
/// i.e. the months a recurring change should propagate forward into.
pub(crate) async fn later_open_month_ids(
    pool: &SqlitePool,
    user_id: i64,
    year: i64,
    month: i64,
) -> Result<Vec<i64>, PaymeError> {
    let ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM months WHERE user_id = ? AND is_closed = 0 AND (year > ? OR (year = ? AND month > ?)) ORDER BY year, month",
    )
    .bind(user_id)
    .bind(year)
    .bind(year)
    .bind(month)
    .fetch_all(pool)
    .await?;

    Ok(ids)
}

#[utoipa::path(
    put,
    path = "/api/months/{month_id}/fixed-expenses/{id}",
    params(
        ("month_id" = i64, Path, description = "Month ID"),
        ("id" = i64, Path, description = "Fixed expense ID")
    ),
    request_body = UpdateMonthlyFixedExpense,
    responses(
        (status = 200, body = MonthlyFixedExpense),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Months",
    summary = "Update monthly fixed expense",
    description = "Updates a fixed expense for a specific month."
)]
pub async fn update_monthly_fixed_expense(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, expense_id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateMonthlyFixedExpense>,
) -> Result<Json<MonthlyFixedExpense>, PaymeError> {
    payload.validate()?;

    let (year, month): (i64, i64) =
        sqlx::query_as("SELECT year, month FROM months WHERE id = ? AND user_id = ?")
            .bind(month_id)
            .bind(claims.sub)
            .fetch_optional(&pool)
            .await?
            .ok_or(PaymeError::NotFound)?;

    let (existing_label, existing_amount, group_id): (String, f64, Option<i64>) = sqlx::query_as(
        "SELECT label, amount, group_id FROM monthly_fixed_expenses WHERE id = ? AND month_id = ?",
    )
    .bind(expense_id)
    .bind(month_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    let label = payload.label.unwrap_or(existing_label);
    let amount = payload.amount.unwrap_or(existing_amount);

    sqlx::query("UPDATE monthly_fixed_expenses SET label = ?, amount = ? WHERE id = ?")
        .bind(&label)
        .bind(amount)
        .bind(expense_id)
        .execute(&pool)
        .await?;

    // Propagate the change to this expense's copies in later open months. Earlier
    // months and closed months keep their values: history stays frozen.
    if let Some(group_id) = group_id {
        let later_open_months = later_open_month_ids(&pool, claims.sub, year, month).await?;
        for later_month_id in later_open_months {
            sqlx::query(
                "UPDATE monthly_fixed_expenses SET label = ?, amount = ? WHERE group_id = ? AND month_id = ?",
            )
            .bind(&label)
            .bind(amount)
            .bind(group_id)
            .bind(later_month_id)
            .execute(&pool)
            .await?;
        }
    }

    Ok(Json(MonthlyFixedExpense {
        id: expense_id,
        month_id,
        label,
        amount,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/months/{month_id}/fixed-expenses/{id}",
    params(
        ("month_id" = i64, Path, description = "Month ID"),
        ("id" = i64, Path, description = "Fixed expense ID")
    ),
    responses((status = 204, description = "Deleted")),
    tag = "Months",
    summary = "Delete monthly fixed expense",
    description = "Removes a fixed expense from a specific month."
)]
pub async fn delete_monthly_fixed_expense(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, expense_id)): Path<(i64, i64)>,
) -> Result<StatusCode, PaymeError> {
    let (year, month): (i64, i64) =
        sqlx::query_as("SELECT year, month FROM months WHERE id = ? AND user_id = ?")
            .bind(month_id)
            .bind(claims.sub)
            .fetch_optional(&pool)
            .await?
            .ok_or(PaymeError::NotFound)?;

    let group_id: Option<Option<i64>> = sqlx::query_scalar(
        "SELECT group_id FROM monthly_fixed_expenses WHERE id = ? AND month_id = ?",
    )
    .bind(expense_id)
    .bind(month_id)
    .fetch_optional(&pool)
    .await?;

    sqlx::query("DELETE FROM monthly_fixed_expenses WHERE id = ? AND month_id = ?")
        .bind(expense_id)
        .bind(month_id)
        .execute(&pool)
        .await?;

    // Deleting means "this expense stops here": remove its copies from later open
    // months too. Earlier and closed months keep it, so history is preserved.
    if let Some(Some(group_id)) = group_id {
        let later_open_months = later_open_month_ids(&pool, claims.sub, year, month).await?;
        for later_month_id in later_open_months {
            sqlx::query("DELETE FROM monthly_fixed_expenses WHERE group_id = ? AND month_id = ?")
                .bind(group_id)
                .bind(later_month_id)
                .execute(&pool)
                .await?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn reorder_monthly_fixed_expenses(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
    Json(payload): Json<ReorderMonthlyFixedExpenses>,
) -> Result<StatusCode, PaymeError> {
    let _: (i64,) = sqlx::query_as("SELECT id FROM months WHERE id = ? AND user_id = ?")
        .bind(month_id)
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or(PaymeError::NotFound)?;

    for (index, id) in payload.ids.iter().enumerate() {
        sqlx::query(
            "UPDATE monthly_fixed_expenses SET sort_order = ? WHERE id = ? AND month_id = ?",
        )
        .bind(index as i64)
        .bind(id)
        .bind(month_id)
        .execute(&pool)
        .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateMonthlySavings {
    #[validate(range(min = 0.0))]
    pub savings: Option<f64>,
    #[validate(range(min = 0.0))]
    pub retirement_savings: Option<f64>,
    #[validate(range(min = 0.0))]
    pub savings_goal: Option<f64>,
}

#[utoipa::path(
    get,
    path = "/api/months/{month_id}/savings",
    params(("month_id" = i64, Path, description = "Month ID")),
    responses(
        (status = 200, body = MonthlySavings),
        (status = 404, description = "Month not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Months",
    summary = "Get monthly savings snapshot",
    description = "Retrieves the savings values for a specific month."
)]
pub async fn get_monthly_savings(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<Json<MonthlySavings>, PaymeError> {
    let _: (i64,) = sqlx::query_as("SELECT id FROM months WHERE id = ? AND user_id = ?")
        .bind(month_id)
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or(PaymeError::NotFound)?;

    let existing: Option<MonthlySavings> = sqlx::query_as(
        "SELECT id, month_id, savings, retirement_savings, savings_goal FROM monthly_savings WHERE month_id = ?",
    )
    .bind(month_id)
    .fetch_optional(&pool)
    .await?;

    match existing {
        Some(savings) => Ok(Json(savings)),
        None => {
            let (savings, retirement_savings, savings_goal): (f64, f64, f64) = sqlx::query_as(
                "SELECT savings, retirement_savings, savings_goal FROM users WHERE id = ?",
            )
            .bind(claims.sub)
            .fetch_one(&pool)
            .await?;

            let id: i64 = sqlx::query_scalar(
                "INSERT INTO monthly_savings (month_id, savings, retirement_savings, savings_goal) VALUES (?, ?, ?, ?) RETURNING id",
            )
            .bind(month_id)
            .bind(savings)
            .bind(retirement_savings)
            .bind(savings_goal)
            .fetch_one(&pool)
            .await?;

            Ok(Json(MonthlySavings {
                id,
                month_id,
                savings,
                retirement_savings,
                savings_goal,
            }))
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/months/{month_id}/savings",
    params(("month_id" = i64, Path, description = "Month ID")),
    request_body = UpdateMonthlySavings,
    responses(
        (status = 200, body = MonthlySavings),
        (status = 404, description = "Month not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Months",
    summary = "Update monthly savings snapshot",
    description = "Updates the savings values for a specific month."
)]
pub async fn update_monthly_savings(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
    Json(payload): Json<UpdateMonthlySavings>,
) -> Result<Json<MonthlySavings>, PaymeError> {
    payload.validate()?;

    let _: (i64,) = sqlx::query_as("SELECT id FROM months WHERE id = ? AND user_id = ?")
        .bind(month_id)
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or(PaymeError::NotFound)?;

    let existing: Option<MonthlySavings> = sqlx::query_as(
        "SELECT id, month_id, savings, retirement_savings, savings_goal FROM monthly_savings WHERE month_id = ?",
    )
    .bind(month_id)
    .fetch_optional(&pool)
    .await?;

    let (savings, retirement_savings, savings_goal) = match existing {
        Some(ref e) => (
            payload.savings.unwrap_or(e.savings),
            payload.retirement_savings.unwrap_or(e.retirement_savings),
            payload.savings_goal.unwrap_or(e.savings_goal),
        ),
        None => (
            payload.savings.unwrap_or(0.0),
            payload.retirement_savings.unwrap_or(0.0),
            payload.savings_goal.unwrap_or(0.0),
        ),
    };

    if existing.is_some() {
        sqlx::query(
            "UPDATE monthly_savings SET savings = ?, retirement_savings = ?, savings_goal = ? WHERE month_id = ?",
        )
        .bind(savings)
        .bind(retirement_savings)
        .bind(savings_goal)
        .bind(month_id)
        .execute(&pool)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO monthly_savings (month_id, savings, retirement_savings, savings_goal) VALUES (?, ?, ?, ?)",
        )
        .bind(month_id)
        .bind(savings)
        .bind(retirement_savings)
        .bind(savings_goal)
        .execute(&pool)
        .await?;
    }

    let updated: MonthlySavings = sqlx::query_as(
        "SELECT id, month_id, savings, retirement_savings, savings_goal FROM monthly_savings WHERE month_id = ?",
    )
    .bind(month_id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(updated))
}
