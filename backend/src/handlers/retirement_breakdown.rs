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
use crate::models::RetirementBreakdownItem;

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateRetirementBreakdownItem {
    #[validate(length(min = 1, max = 100))]
    pub label: String,
    #[validate(range(min = 0.0))]
    pub amount: f64,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateRetirementBreakdownItem {
    #[validate(length(min = 1, max = 100))]
    pub label: Option<String>,
    #[validate(range(min = 0.0))]
    pub amount: Option<f64>,
}

#[derive(Deserialize, ToSchema)]
pub struct ReorderRetirementBreakdown {
    pub ids: Vec<i64>,
}

pub async fn list_retirement_breakdown(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<RetirementBreakdownItem>>, PaymeError> {
    let items: Vec<RetirementBreakdownItem> = sqlx::query_as(
        "SELECT id, user_id, label, amount FROM retirement_breakdown_items WHERE user_id = ? ORDER BY sort_order, id",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?;

    Ok(Json(items))
}

pub async fn create_retirement_breakdown_item(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateRetirementBreakdownItem>,
) -> Result<(StatusCode, Json<RetirementBreakdownItem>), PaymeError> {
    payload.validate()?;
    let sort_order: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM retirement_breakdown_items WHERE user_id = ?",
    )
    .bind(claims.sub)
    .fetch_one(&pool)
    .await?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO retirement_breakdown_items (user_id, label, amount, sort_order) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(claims.sub)
    .bind(&payload.label)
    .bind(payload.amount)
    .bind(sort_order)
    .fetch_one(&pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(RetirementBreakdownItem {
            id,
            user_id: claims.sub,
            label: payload.label,
            amount: payload.amount,
        }),
    ))
}

pub async fn update_retirement_breakdown_item(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(item_id): Path<i64>,
    Json(payload): Json<UpdateRetirementBreakdownItem>,
) -> Result<Json<RetirementBreakdownItem>, PaymeError> {
    payload.validate()?;
    let existing: RetirementBreakdownItem = sqlx::query_as(
        "SELECT id, user_id, label, amount FROM retirement_breakdown_items WHERE id = ? AND user_id = ?",
    )
    .bind(item_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    let label = payload.label.unwrap_or(existing.label);
    let amount = payload.amount.unwrap_or(existing.amount);

    sqlx::query("UPDATE retirement_breakdown_items SET label = ?, amount = ? WHERE id = ?")
        .bind(&label)
        .bind(amount)
        .bind(item_id)
        .execute(&pool)
        .await?;

    Ok(Json(RetirementBreakdownItem {
        id: item_id,
        user_id: claims.sub,
        label,
        amount,
    }))
}

pub async fn reorder_retirement_breakdown(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<ReorderRetirementBreakdown>,
) -> Result<StatusCode, PaymeError> {
    for (index, id) in payload.ids.iter().enumerate() {
        sqlx::query(
            "UPDATE retirement_breakdown_items SET sort_order = ? WHERE id = ? AND user_id = ?",
        )
        .bind(index as i64)
        .bind(id)
        .bind(claims.sub)
        .execute(&pool)
        .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_retirement_breakdown_item(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(item_id): Path<i64>,
) -> Result<StatusCode, PaymeError> {
    sqlx::query("DELETE FROM retirement_breakdown_items WHERE id = ? AND user_id = ?")
        .bind(item_id)
        .bind(claims.sub)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
