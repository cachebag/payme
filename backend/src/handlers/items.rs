use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;
use validator::Validate;

use crate::error::PaymeError;
use crate::middleware::auth::Claims;
use crate::models::{Item, ItemWithCategory};

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateItem {
    pub category_id: i64,
    #[validate(length(min = 1, max = 200))]
    pub description: String,
    #[validate(range(min = 0.0))]
    pub amount: f64,
    pub spent_on: NaiveDate,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateItem {
    pub category_id: Option<i64>,
    #[validate(length(min = 1, max = 200))]
    pub description: Option<String>,
    #[validate(range(min = 0.0))]
    pub amount: Option<f64>,
    pub spent_on: Option<NaiveDate>,
}

#[utoipa::path(
    get, path = "/api/months/{id}/items",
    params(("id" = i64, Path)),
    responses(
        (status = 200, body = [ItemWithCategory]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Items",
    summary = "List transactions",
    description = "Retrieves all itemized spending for the month, including category labels."
)]
pub async fn list_items(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<Json<Vec<ItemWithCategory>>, PaymeError> {
    verify_month_access(&pool, claims.sub, month_id).await?;

    let items: Vec<ItemWithCategory> = sqlx::query_as(
        r#"
        SELECT i.id, i.month_id, i.category_id, bc.label as category_label, i.description, i.amount, i.spent_on
        FROM items i
        JOIN budget_categories bc ON i.category_id = bc.id
        WHERE i.month_id = ?
        ORDER BY i.spent_on DESC
        "#,
    )
    .bind(month_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(items))
}

#[utoipa::path(
    post, path = "/api/months/{id}/items",
    params(("id" = i64, Path)),
    request_body = CreateItem,
    responses(
        (status = 200, body = Item),
        (status = 500, description = "Internal server error")
    ),
    tag = "Items",
    summary = "Record transaction",
    description = "Logs a new expense against a specific budget category."
)]
pub async fn create_item(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
    Json(payload): Json<CreateItem>,
) -> Result<Json<Item>, PaymeError> {
    payload.validate()?;
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    let _category: (i64,) =
        sqlx::query_as("SELECT id FROM budget_categories WHERE id = ? AND user_id = ?")
            .bind(payload.category_id)
            .bind(claims.sub)
            .fetch_optional(&pool)
            .await?
            .ok_or(PaymeError::BadRequest("Invalid category".to_string()))?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO items (month_id, category_id, description, amount, spent_on) VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(month_id)
    .bind(payload.category_id)
    .bind(&payload.description)
    .bind(payload.amount)
    .bind(payload.spent_on)
    .fetch_one(&pool)
    .await?;

    Ok(Json(Item {
        id,
        month_id,
        category_id: payload.category_id,
        description: payload.description,
        amount: payload.amount,
        spent_on: payload.spent_on,
    }))
}

#[utoipa::path(
    put,
    path = "/api/months/{month_id}/items/{id}",
    params(
        ("month_id" = i64, Path, description = "Month ID"),
        ("id" = i64, Path, description = "Item (Transaction) ID")
    ),
    request_body = UpdateItem,
    responses(
        (status = 200, description = "Item updated successfully", body = Item),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Items",
    summary = "Update transaction details",
    description = "Updates an existing transaction. Supports partial updates for category, description, amount, or date."
)]
pub async fn update_item(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, item_id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateItem>,
) -> Result<Json<Item>, PaymeError> {
    payload.validate()?;
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    let existing: Item = sqlx::query_as(
        "SELECT id, month_id, category_id, description, amount, spent_on FROM items WHERE id = ? AND month_id = ?",
    )
    .bind(item_id)
    .bind(month_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    let category_id = payload.category_id.unwrap_or(existing.category_id);
    let description = payload.description.unwrap_or(existing.description);
    let amount = payload.amount.unwrap_or(existing.amount);
    let spent_on = payload.spent_on.unwrap_or(existing.spent_on);

    if payload.category_id.is_some() {
        let _category: (i64,) =
            sqlx::query_as("SELECT id FROM budget_categories WHERE id = ? AND user_id = ?")
                .bind(category_id)
                .bind(claims.sub)
                .fetch_optional(&pool)
                .await?
                .ok_or(PaymeError::BadRequest("Invalid category".to_string()))?;
    }

    sqlx::query(
        "UPDATE items SET category_id = ?, description = ?, amount = ?, spent_on = ? WHERE id = ?",
    )
    .bind(category_id)
    .bind(&description)
    .bind(amount)
    .bind(spent_on)
    .bind(item_id)
    .execute(&pool)
    .await?;

    Ok(Json(Item {
        id: item_id,
        month_id,
        category_id,
        description,
        amount,
        spent_on,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/months/{month_id}/items/{id}",
    params(
        ("month_id" = i64, Path, description = "Month ID"),
        ("id" = i64, Path, description = "Item (Transaction) ID")
    ),
    responses(
        (status = 204, description = "Item deleted successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Items",
    summary = "Delete transaction",
    description = "Permanently removes a transaction from the month's spending list."
)]
pub async fn delete_item(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, item_id)): Path<(i64, i64)>,
) -> Result<StatusCode, PaymeError> {
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    sqlx::query("DELETE FROM items WHERE id = ? AND month_id = ?")
        .bind(item_id)
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
