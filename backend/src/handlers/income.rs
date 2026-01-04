use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::middleware::auth::Claims;
use crate::models::IncomeEntry;

#[derive(Deserialize)]
pub struct CreateIncome {
    pub label: String,
    pub amount: f64,
}

#[derive(Deserialize)]
pub struct UpdateIncome {
    pub label: Option<String>,
    pub amount: Option<f64>,
}

pub async fn list_income(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<Json<Vec<IncomeEntry>>, StatusCode> {
    verify_month_access(&pool, claims.sub, month_id).await?;

    let entries: Vec<IncomeEntry> =
        sqlx::query_as("SELECT id, month_id, label, amount FROM income_entries WHERE month_id = ?")
            .bind(month_id)
            .fetch_all(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(entries))
}

pub async fn create_income(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
    Json(payload): Json<CreateIncome>,
) -> Result<Json<IncomeEntry>, StatusCode> {
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO income_entries (month_id, label, amount) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(month_id)
    .bind(&payload.label)
    .bind(payload.amount)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(IncomeEntry {
        id,
        month_id,
        label: payload.label,
        amount: payload.amount,
    }))
}

pub async fn update_income(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, income_id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateIncome>,
) -> Result<Json<IncomeEntry>, StatusCode> {
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    let existing: IncomeEntry = sqlx::query_as(
        "SELECT id, month_id, label, amount FROM income_entries WHERE id = ? AND month_id = ?",
    )
    .bind(income_id)
    .bind(month_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let label = payload.label.unwrap_or(existing.label);
    let amount = payload.amount.unwrap_or(existing.amount);

    sqlx::query("UPDATE income_entries SET label = ?, amount = ? WHERE id = ?")
        .bind(&label)
        .bind(amount)
        .bind(income_id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(IncomeEntry {
        id: income_id,
        month_id,
        label,
        amount,
    }))
}

pub async fn delete_income(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, income_id)): Path<(i64, i64)>,
) -> Result<StatusCode, StatusCode> {
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    sqlx::query("DELETE FROM income_entries WHERE id = ? AND month_id = ?")
        .bind(income_id)
        .bind(month_id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn verify_month_access(
    pool: &SqlitePool,
    user_id: i64,
    month_id: i64,
) -> Result<(), StatusCode> {
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM months WHERE id = ? AND user_id = ?")
            .bind(month_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    exists.map(|_| ()).ok_or(StatusCode::NOT_FOUND)
}

async fn verify_month_not_closed(
    pool: &SqlitePool,
    user_id: i64,
    month_id: i64,
) -> Result<(), StatusCode> {
    let month: Option<(bool,)> =
        sqlx::query_as("SELECT is_closed FROM months WHERE id = ? AND user_id = ?")
            .bind(month_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match month {
        Some((true,)) => Err(StatusCode::BAD_REQUEST),
        Some((false,)) => Ok(()),
        None => Err(StatusCode::NOT_FOUND),
    }
}
