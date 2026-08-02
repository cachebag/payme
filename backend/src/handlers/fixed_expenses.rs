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
use crate::models::FixedExpense;

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateFixedExpense {
    #[validate(length(min = 1, max = 100))]
    pub label: String,
    #[validate(range(min = 0.0))]
    pub amount: f64,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateFixedExpense {
    #[validate(length(min = 1, max = 100))]
    pub label: Option<String>,
    #[validate(range(min = 0.0))]
    pub amount: Option<f64>,
}

#[derive(Deserialize, ToSchema)]
pub struct ReorderFixedExpenses {
    pub ids: Vec<i64>,
}

#[utoipa::path(
    get,
    path = "/api/fixed-expenses",
    responses(
        (status = 200, body = [FixedExpense]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Configuration",
    summary = "List fixed expenses",
    description = "Retrieves all fixed expenses associated with the authenticated user."
)]
pub async fn list_fixed_expenses(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<FixedExpense>>, PaymeError> {
    let expenses: Vec<FixedExpense> =
        sqlx::query_as("SELECT id, user_id, label, amount FROM fixed_expenses WHERE user_id = ? ORDER BY sort_order, id")
            .bind(claims.sub)
            .fetch_all(&pool)
            .await?;

    Ok(Json(expenses))
}

#[utoipa::path(
    post,
    path = "/api/fixed-expenses",
    request_body = CreateFixedExpense,
    responses(
        (status = 201, body = FixedExpense),
        (status = 500, description = "Internal server error")
    ),
    tag = "Configuration",
    summary = "Create fixed expense",
    description = "Adds a new recurring expense (e.g., Rent, Internet) to the user's profile."
)]
pub async fn create_fixed_expense(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateFixedExpense>,
) -> Result<Json<FixedExpense>, PaymeError> {
    payload.validate()?;
    let sort_order: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM fixed_expenses WHERE user_id = ?",
    )
    .bind(claims.sub)
    .fetch_one(&pool)
    .await?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO fixed_expenses (user_id, label, amount, sort_order) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(claims.sub)
    .bind(&payload.label)
    .bind(payload.amount)
    .bind(sort_order)
    .fetch_one(&pool)
    .await?;

    Ok(Json(FixedExpense {
        id,
        user_id: claims.sub,
        label: payload.label,
        amount: payload.amount,
    }))
}

#[utoipa::path(
    put,
    path = "/api/fixed-expenses/{id}",
    params(("id" = i64, Path, description = "Expense ID")),
    request_body = UpdateFixedExpense,
    responses(
        (status = 200, body = FixedExpense),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Configuration",
    summary = "Update fixed expense",
    description = "Updates the label or amount of an existing fixed expense by ID."
)]
pub async fn update_fixed_expense(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(expense_id): Path<i64>,
    Json(payload): Json<UpdateFixedExpense>,
) -> Result<Json<FixedExpense>, PaymeError> {
    payload.validate()?;
    let existing: FixedExpense = sqlx::query_as(
        "SELECT id, user_id, label, amount FROM fixed_expenses WHERE id = ? AND user_id = ?",
    )
    .bind(expense_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    let label = payload.label.unwrap_or(existing.label);
    let amount = payload.amount.unwrap_or(existing.amount);

    sqlx::query("UPDATE fixed_expenses SET label = ?, amount = ? WHERE id = ?")
        .bind(&label)
        .bind(amount)
        .bind(expense_id)
        .execute(&pool)
        .await?;

    Ok(Json(FixedExpense {
        id: expense_id,
        user_id: claims.sub,
        label,
        amount,
    }))
}

pub async fn reorder_fixed_expenses(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<ReorderFixedExpenses>,
) -> Result<StatusCode, PaymeError> {
    for (index, id) in payload.ids.iter().enumerate() {
        sqlx::query("UPDATE fixed_expenses SET sort_order = ? WHERE id = ? AND user_id = ?")
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
    path = "/api/fixed-expenses/{id}",
    params(("id" = i64, Path, description = "Expense ID")),
    responses((status = 204, description = "Deleted")),
    tag = "Configuration",
    summary = "Delete fixed expense",
    description = "Permanently removes a recurring expense template."
)]
pub async fn delete_fixed_expense(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(expense_id): Path<i64>,
) -> Result<StatusCode, PaymeError> {
    sqlx::query("DELETE FROM fixed_expenses WHERE id = ? AND user_id = ?")
        .bind(expense_id)
        .bind(claims.sub)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
