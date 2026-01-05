use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::error::PaymeError;
use crate::middleware::auth::Claims;
use crate::models::FixedExpense;

#[derive(Deserialize, ToSchema)]
pub struct CreateFixedExpense {
    pub label: String,
    pub amount: f64,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateFixedExpense {
    pub label: Option<String>,
    pub amount: Option<f64>,
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
        sqlx::query_as("SELECT id, user_id, label, amount FROM fixed_expenses WHERE user_id = ?")
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
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO fixed_expenses (user_id, label, amount) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(claims.sub)
    .bind(&payload.label)
    .bind(payload.amount)
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
