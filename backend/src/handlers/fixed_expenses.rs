use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::middleware::auth::Claims;
use crate::models::FixedExpense;

#[derive(Deserialize)]
pub struct CreateFixedExpense {
    pub label: String,
    pub amount: f64,
}

#[derive(Deserialize)]
pub struct UpdateFixedExpense {
    pub label: Option<String>,
    pub amount: Option<f64>,
}

pub async fn list_fixed_expenses(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<FixedExpense>>, StatusCode> {
    let expenses: Vec<FixedExpense> =
        sqlx::query_as("SELECT id, user_id, label, amount FROM fixed_expenses WHERE user_id = ?")
            .bind(claims.sub)
            .fetch_all(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(expenses))
}

pub async fn create_fixed_expense(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateFixedExpense>,
) -> Result<Json<FixedExpense>, StatusCode> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO fixed_expenses (user_id, label, amount) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(claims.sub)
    .bind(&payload.label)
    .bind(payload.amount)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(FixedExpense {
        id,
        user_id: claims.sub,
        label: payload.label,
        amount: payload.amount,
    }))
}

pub async fn update_fixed_expense(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(expense_id): Path<i64>,
    Json(payload): Json<UpdateFixedExpense>,
) -> Result<Json<FixedExpense>, StatusCode> {
    let existing: FixedExpense = sqlx::query_as(
        "SELECT id, user_id, label, amount FROM fixed_expenses WHERE id = ? AND user_id = ?",
    )
    .bind(expense_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let label = payload.label.unwrap_or(existing.label);
    let amount = payload.amount.unwrap_or(existing.amount);

    sqlx::query("UPDATE fixed_expenses SET label = ?, amount = ? WHERE id = ?")
        .bind(&label)
        .bind(amount)
        .bind(expense_id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(FixedExpense {
        id: expense_id,
        user_id: claims.sub,
        label,
        amount,
    }))
}

pub async fn delete_fixed_expense(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(expense_id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM fixed_expenses WHERE id = ? AND user_id = ?")
        .bind(expense_id)
        .bind(claims.sub)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
