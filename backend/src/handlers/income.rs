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
use crate::models::IncomeEntry;

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateIncome {
    #[validate(length(min = 1, max = 100))]
    pub label: String,
    #[validate(range(min = 0.0))]
    pub amount: f64,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateIncome {
    #[validate(length(min = 1, max = 100))]
    pub label: Option<String>,
    #[validate(range(min = 0.0))]
    pub amount: Option<f64>,
}

#[utoipa::path(
    get, path = "/api/months/{id}/income",
    params(("id" = i64, Path)),
    responses(
        (status = 200, body = [IncomeEntry]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Income",
    summary = "List monthly income",
    description = "Retrieves all sources of income (paychecks, gifts, etc.) recorded for a specific month."
)]
pub async fn list_income(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<Json<Vec<IncomeEntry>>, PaymeError> {
    verify_month_access(&pool, claims.sub, month_id).await?;

    let entries: Vec<IncomeEntry> =
        sqlx::query_as("SELECT id, month_id, label, amount FROM income_entries WHERE month_id = ?")
            .bind(month_id)
            .fetch_all(&pool)
            .await?;

    Ok(Json(entries))
}

#[utoipa::path(
    post, path = "/api/months/{id}/income",
    params(("id" = i64, Path)),
    request_body = CreateIncome,
    responses(
        (status = 200, body = IncomeEntry),
        (status = 500, description = "Internal server error")   
    ),
    tag = "Income",
    summary = "Add income entry",
    description = "Records a new income source for the month. Only available if the month is open."
)]
pub async fn create_income(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
    Json(payload): Json<CreateIncome>,
) -> Result<Json<IncomeEntry>, PaymeError> {
    payload.validate()?;
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO income_entries (month_id, label, amount) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(month_id)
    .bind(&payload.label)
    .bind(payload.amount)
    .fetch_one(&pool)
    .await?;

    Ok(Json(IncomeEntry {
        id,
        month_id,
        label: payload.label,
        amount: payload.amount,
    }))
}

#[utoipa::path(
    put,
    path = "/api/months/{month_id}/income/{id}",
    params(
        ("month_id" = i64, Path, description = "Month ID"),
        ("id" = i64, Path, description = "Income Entry ID")
    ),
    request_body = UpdateIncome,
    responses(
        (status = 200, description = "Income updated successfully", body = IncomeEntry),
        (status = 500, description = "Internal server error")
    ),
    tag = "Income",
    summary = "Update income entry",
    description = "Modifies an existing income record's label or amount."
)]
pub async fn update_income(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, income_id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateIncome>,
) -> Result<Json<IncomeEntry>, PaymeError> {
    payload.validate()?;
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    let existing: IncomeEntry = sqlx::query_as(
        "SELECT id, month_id, label, amount FROM income_entries WHERE id = ? AND month_id = ?",
    )
    .bind(income_id)
    .bind(month_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    let label = payload.label.unwrap_or(existing.label);
    let amount = payload.amount.unwrap_or(existing.amount);

    sqlx::query("UPDATE income_entries SET label = ?, amount = ? WHERE id = ?")
        .bind(&label)
        .bind(amount)
        .bind(income_id)
        .execute(&pool)
        .await?;

    Ok(Json(IncomeEntry {
        id: income_id,
        month_id,
        label,
        amount,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/months/{month_id}/income/{id}",
    params(
        ("month_id" = i64, Path, description = "Month ID"),
        ("id" = i64, Path, description = "Income Entry ID")
    ),
    responses(
        (status = 204, description = "Income deleted successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Income",
    summary = "Delete income entry",
    description = "Removes a specific income source from the month's records."
)]
pub async fn delete_income(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, income_id)): Path<(i64, i64)>,
) -> Result<StatusCode, PaymeError> {
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    sqlx::query("DELETE FROM income_entries WHERE id = ? AND month_id = ?")
        .bind(income_id)
        .bind(month_id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn verify_month_access(
    pool: &SqlitePool,
    user_id: i64,
    month_id: i64,
) -> Result<(), PaymeError> {
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM months WHERE id = ? AND user_id = ?")
            .bind(month_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    exists.map(|_| ()).ok_or(PaymeError::NotFound)
}

async fn verify_month_not_closed(
    pool: &SqlitePool,
    user_id: i64,
    month_id: i64,
) -> Result<(), PaymeError> {
    let month: Option<(bool,)> =
        sqlx::query_as("SELECT is_closed FROM months WHERE id = ? AND user_id = ?")
            .bind(month_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    match month {
        Some((true,)) => Err(PaymeError::BadRequest("Month is closed".to_string())),
        Some((false,)) => Ok(()),
        None => Err(PaymeError::NotFound),
    }
}
