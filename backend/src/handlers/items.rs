use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use utoipa::ToSchema;
use validator::Validate;

use crate::error::PaymeError;
use crate::middleware::auth::Claims;
use crate::models::{Item, ItemWithCategory, TagSummary};

fn default_savings_destination() -> String {
    "none".to_string()
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateItem {
    pub category_id: Option<i64>,
    #[validate(length(min = 1, max = 200))]
    pub description: String,
    #[validate(range(min = 0.0))]
    pub amount: f64,
    pub spent_on: NaiveDate,
    #[serde(default = "default_savings_destination")]
    pub savings_destination: String,
    #[serde(default)]
    pub tag_ids: Vec<i64>,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateItem {
    pub category_id: Option<i64>,
    #[validate(length(min = 1, max = 200))]
    pub description: Option<String>,
    #[validate(range(min = 0.0))]
    pub amount: Option<f64>,
    pub spent_on: Option<NaiveDate>,
    pub savings_destination: Option<String>,
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Deserialize, ToSchema)]
pub struct ReorderItems {
    pub ids: Vec<i64>,
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

    let mut items: Vec<ItemWithCategory> = sqlx::query_as(
        r#"
        SELECT i.id, i.month_id, i.category_id, bc.label as category_label, bc.color as category_color, i.description, i.amount, i.spent_on, i.savings_destination
        FROM items i
        LEFT JOIN budget_categories bc ON i.category_id = bc.id
        WHERE i.month_id = ?
        ORDER BY i.sort_order, i.id
        "#,
    )
    .bind(month_id)
    .fetch_all(&pool)
    .await?;

    load_item_tags(&pool, month_id, &mut items).await?;

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
) -> Result<Json<ItemWithCategory>, PaymeError> {
    payload.validate()?;
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    // Uncategorized items are allowed; a concrete category must be live and owned.
    if let Some(category_id) = payload.category_id {
        let _category: (i64,) = sqlx::query_as(
            "SELECT id FROM budget_categories WHERE id = ? AND user_id = ? AND archived_at IS NULL",
        )
        .bind(category_id)
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or(PaymeError::BadRequest("Invalid category".to_string()))?;
    }

    validate_tag_ids(
        &pool,
        claims.sub,
        &payload.tag_ids,
        &HashSet::new(),
        &payload.savings_destination,
    )
    .await?;

    let sort_order: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM items WHERE month_id = ?",
    )
    .bind(month_id)
    .fetch_one(&pool)
    .await?;

    let mut tx = pool.begin().await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO items (month_id, category_id, description, amount, spent_on, savings_destination, sort_order) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(month_id)
    .bind(payload.category_id)
    .bind(&payload.description)
    .bind(payload.amount)
    .bind(payload.spent_on)
    .bind(&payload.savings_destination)
    .bind(sort_order)
    .fetch_one(&mut *tx)
    .await?;

    for tag_id in &payload.tag_ids {
        sqlx::query("INSERT INTO tag_assignments (tag_id, item_id) VALUES (?, ?)")
            .bind(tag_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    match payload.savings_destination.as_str() {
        "savings" => {
            sqlx::query("UPDATE users SET savings = savings + ? WHERE id = ?")
                .bind(payload.amount)
                .bind(claims.sub)
                .execute(&mut *tx)
                .await?;
        }
        "retirement_savings" => {
            sqlx::query(
                "UPDATE users SET retirement_savings = retirement_savings + ? WHERE id = ?",
            )
            .bind(payload.amount)
            .bind(claims.sub)
            .execute(&mut *tx)
            .await?;
        }
        _ => {}
    }

    tx.commit().await?;
    Ok(Json(get_item_with_tags(&pool, month_id, id).await?))
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
) -> Result<Json<ItemWithCategory>, PaymeError> {
    payload.validate()?;
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    let existing: Item = sqlx::query_as(
        "SELECT id, month_id, category_id, description, amount, spent_on, savings_destination FROM items WHERE id = ? AND month_id = ?",
    )
    .bind(item_id)
    .bind(month_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    // `Some` re-categorizes; `None` keeps whatever the item had (possibly uncategorized).
    let category_id = payload.category_id.or(existing.category_id);
    let description = payload.description.unwrap_or(existing.description);
    let amount = payload.amount.unwrap_or(existing.amount);
    let spent_on = payload.spent_on.unwrap_or(existing.spent_on);
    let savings_destination = payload
        .savings_destination
        .unwrap_or(existing.savings_destination.clone());

    let existing_tag_ids: HashSet<i64> =
        sqlx::query_scalar("SELECT tag_id FROM tag_assignments WHERE item_id = ?")
            .bind(item_id)
            .fetch_all(&pool)
            .await?
            .into_iter()
            .collect();

    if let Some(tag_ids) = &payload.tag_ids {
        validate_tag_ids(
            &pool,
            claims.sub,
            tag_ids,
            &existing_tag_ids,
            &savings_destination,
        )
        .await?;
    } else if savings_destination != "none" && !existing_tag_ids.is_empty() {
        return Err(PaymeError::BadRequest(
            "Transfers cannot have Tags".to_string(),
        ));
    }

    if payload.category_id.is_some() {
        let _category: (i64,) = sqlx::query_as(
            "SELECT id FROM budget_categories WHERE id = ? AND user_id = ? AND archived_at IS NULL",
        )
        .bind(category_id)
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or(PaymeError::BadRequest("Invalid category".to_string()))?;
    }

    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE items SET category_id = ?, description = ?, amount = ?, spent_on = ?, savings_destination = ? WHERE id = ?",
    )
    .bind(category_id)
    .bind(&description)
    .bind(amount)
    .bind(spent_on)
    .bind(&savings_destination)
    .bind(item_id)
    .execute(&mut *tx)
    .await?;

    if let Some(tag_ids) = &payload.tag_ids {
        sqlx::query("DELETE FROM tag_assignments WHERE item_id = ?")
            .bind(item_id)
            .execute(&mut *tx)
            .await?;
        for tag_id in tag_ids {
            sqlx::query("INSERT INTO tag_assignments (tag_id, item_id) VALUES (?, ?)")
                .bind(tag_id)
                .bind(item_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    let old_dest = existing.savings_destination.as_str();
    let new_dest = savings_destination.as_str();

    if old_dest != new_dest || (old_dest != "none" && existing.amount != amount) {
        match old_dest {
            "savings" => {
                sqlx::query("UPDATE users SET savings = savings - ? WHERE id = ?")
                    .bind(existing.amount)
                    .bind(claims.sub)
                    .execute(&mut *tx)
                    .await?;
            }
            "retirement_savings" => {
                sqlx::query(
                    "UPDATE users SET retirement_savings = retirement_savings - ? WHERE id = ?",
                )
                .bind(existing.amount)
                .bind(claims.sub)
                .execute(&mut *tx)
                .await?;
            }
            _ => {}
        }

        match new_dest {
            "savings" => {
                sqlx::query("UPDATE users SET savings = savings + ? WHERE id = ?")
                    .bind(amount)
                    .bind(claims.sub)
                    .execute(&mut *tx)
                    .await?;
            }
            "retirement_savings" => {
                sqlx::query(
                    "UPDATE users SET retirement_savings = retirement_savings + ? WHERE id = ?",
                )
                .bind(amount)
                .bind(claims.sub)
                .execute(&mut *tx)
                .await?;
            }
            _ => {}
        }
    }

    tx.commit().await?;
    Ok(Json(get_item_with_tags(&pool, month_id, item_id).await?))
}

pub async fn reorder_items(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
    Json(payload): Json<ReorderItems>,
) -> Result<StatusCode, PaymeError> {
    verify_month_not_closed(&pool, claims.sub, month_id).await?;

    for (index, id) in payload.ids.iter().enumerate() {
        sqlx::query("UPDATE items SET sort_order = ? WHERE id = ? AND month_id = ?")
            .bind(index as i64)
            .bind(id)
            .bind(month_id)
            .execute(&pool)
            .await?;
    }

    Ok(StatusCode::NO_CONTENT)
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

    let item: Item = sqlx::query_as(
        "SELECT id, month_id, category_id, description, amount, spent_on, savings_destination FROM items WHERE id = ? AND month_id = ?",
    )
    .bind(item_id)
    .bind(month_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    match item.savings_destination.as_str() {
        "savings" => {
            sqlx::query("UPDATE users SET savings = savings - ? WHERE id = ?")
                .bind(item.amount)
                .bind(claims.sub)
                .execute(&pool)
                .await?;
        }
        "retirement_savings" => {
            sqlx::query(
                "UPDATE users SET retirement_savings = retirement_savings - ? WHERE id = ?",
            )
            .bind(item.amount)
            .bind(claims.sub)
            .execute(&pool)
            .await?;
        }
        _ => {}
    }

    sqlx::query("DELETE FROM items WHERE id = ? AND month_id = ?")
        .bind(item_id)
        .bind(month_id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn validate_tag_ids(
    pool: &SqlitePool,
    user_id: i64,
    tag_ids: &[i64],
    existing_tag_ids: &HashSet<i64>,
    savings_destination: &str,
) -> Result<(), PaymeError> {
    if tag_ids.len() > 5 {
        return Err(PaymeError::BadRequest(
            "A Spending Item can have at most five Tags".to_string(),
        ));
    }
    if tag_ids.iter().collect::<HashSet<_>>().len() != tag_ids.len() {
        return Err(PaymeError::BadRequest(
            "Tag identifiers must be distinct".to_string(),
        ));
    }
    if savings_destination != "none" && !tag_ids.is_empty() {
        return Err(PaymeError::BadRequest(
            "Transfers cannot have Tags".to_string(),
        ));
    }

    // ponytail: at most five bounded lookups; replace with a dynamic IN query only if this limit grows.
    for tag_id in tag_ids {
        let stopped: bool =
            sqlx::query_scalar("SELECT stopped FROM tags WHERE id = ? AND user_id = ?")
                .bind(tag_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| PaymeError::BadRequest("Invalid Tag".to_string()))?;

        if stopped && !existing_tag_ids.contains(tag_id) {
            return Err(PaymeError::BadRequest("Tag is stopped".to_string()));
        }
    }
    Ok(())
}

async fn get_item_with_tags(
    pool: &SqlitePool,
    month_id: i64,
    item_id: i64,
) -> Result<ItemWithCategory, PaymeError> {
    let item: ItemWithCategory = sqlx::query_as(
        r#"SELECT i.id, i.month_id, i.category_id, bc.label AS category_label,
                  bc.color AS category_color, i.description, i.amount, i.spent_on,
                  i.savings_destination
           FROM items i
           LEFT JOIN budget_categories bc ON i.category_id = bc.id
           WHERE i.id = ? AND i.month_id = ?"#,
    )
    .bind(item_id)
    .bind(month_id)
    .fetch_optional(pool)
    .await?
    .ok_or(PaymeError::NotFound)?;
    let mut items = vec![item];
    load_item_tags(pool, month_id, &mut items).await?;
    Ok(items.remove(0))
}

pub(crate) async fn load_item_tags(
    pool: &SqlitePool,
    month_id: i64,
    items: &mut [ItemWithCategory],
) -> Result<(), PaymeError> {
    let rows: Vec<(i64, i64, String, String, bool)> = sqlx::query_as(
        r#"SELECT ta.item_id, t.id, t.label, t.color, t.stopped
           FROM tag_assignments ta
           JOIN tags t ON t.id = ta.tag_id
           JOIN items i ON i.id = ta.item_id
           WHERE i.month_id = ?
           ORDER BY lower(t.label), t.id"#,
    )
    .bind(month_id)
    .fetch_all(pool)
    .await?;

    let mut tags_by_item: HashMap<i64, Vec<TagSummary>> = HashMap::new();
    for (item_id, id, label, color, stopped) in rows {
        tags_by_item.entry(item_id).or_default().push(TagSummary {
            id,
            label,
            color,
            stopped,
        });
    }
    for item in items {
        item.tags = tags_by_item.remove(&item.id).unwrap_or_default();
    }
    Ok(())
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
