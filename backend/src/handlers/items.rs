use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::middleware::auth::Claims;
use crate::models::{Item, ItemWithCategory};

#[derive(Deserialize)]
pub struct CreateItem {
    pub category_id: i64,
    pub description: String,
    pub amount: f64,
    pub spent_on: NaiveDate,
}

#[derive(Deserialize)]
pub struct UpdateItem {
    pub category_id: Option<i64>,
    pub description: Option<String>,
    pub amount: Option<f64>,
    pub spent_on: Option<NaiveDate>,
}

pub async fn list_items(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<Json<Vec<ItemWithCategory>>, StatusCode> {
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
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(items))
}

pub async fn create_item(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
    Json(payload): Json<CreateItem>,
) -> Result<Json<Item>, StatusCode> {
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    let _category: (i64,) =
        sqlx::query_as("SELECT id FROM budget_categories WHERE id = ? AND user_id = ?")
            .bind(payload.category_id)
            .bind(claims.sub)
            .fetch_optional(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::BAD_REQUEST)?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO items (month_id, category_id, description, amount, spent_on) VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(month_id)
    .bind(payload.category_id)
    .bind(&payload.description)
    .bind(payload.amount)
    .bind(payload.spent_on)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Item {
        id,
        month_id,
        category_id: payload.category_id,
        description: payload.description,
        amount: payload.amount,
        spent_on: payload.spent_on,
    }))
}

pub async fn update_item(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, item_id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateItem>,
) -> Result<Json<Item>, StatusCode> {
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    let existing: Item = sqlx::query_as(
        "SELECT id, month_id, category_id, description, amount, spent_on FROM items WHERE id = ? AND month_id = ?",
    )
    .bind(item_id)
    .bind(month_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

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
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::BAD_REQUEST)?;
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
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Item {
        id: item_id,
        month_id,
        category_id,
        description,
        amount,
        spent_on,
    }))
}

pub async fn delete_item(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path((month_id, item_id)): Path<(i64, i64)>,
) -> Result<StatusCode, StatusCode> {
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    sqlx::query("DELETE FROM items WHERE id = ? AND month_id = ?")
        .bind(item_id)
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
