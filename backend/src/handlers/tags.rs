use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::{error::PaymeError, middleware::auth::Claims, models::Tag};

const PRESET_COLORS: [&str; 10] = [
    "#71717a", "#ef4444", "#f97316", "#f59e0b", "#10b981", "#06b6d4", "#3b82f6", "#6366f1",
    "#8b5cf6", "#d946ef",
];

#[derive(Deserialize, ToSchema)]
pub struct CreateTag {
    pub label: String,
    pub color: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTag {
    pub label: Option<String>,
    pub color: Option<String>,
}

fn validate_label(label: String) -> Result<String, PaymeError> {
    let label = label.trim().to_string();
    if !(1..=50).contains(&label.chars().count()) {
        return Err(PaymeError::BadRequest(
            "Tag label must contain 1-50 characters".to_string(),
        ));
    }
    Ok(label)
}

fn validate_color(color: &str) -> Result<(), PaymeError> {
    if PRESET_COLORS.contains(&color) {
        Ok(())
    } else {
        Err(PaymeError::BadRequest("Invalid Tag color".to_string()))
    }
}

async fn ensure_unique_label(
    pool: &SqlitePool,
    user_id: i64,
    label: &str,
    excluding_id: Option<i64>,
) -> Result<(), PaymeError> {
    let existing: Option<(i64, bool)> = sqlx::query_as(
        "SELECT id, stopped FROM tags WHERE user_id = ? AND normalized_label = ? AND id <> COALESCE(?, -1)",
    )
    .bind(user_id)
    .bind(label.to_lowercase())
    .bind(excluding_id)
    .fetch_optional(pool)
    .await?;

    if let Some((id, stopped)) = existing {
        return Err(PaymeError::TagLabelConflict {
            restorable_tag_id: stopped.then_some(id),
        });
    }
    Ok(())
}

async fn get_tag(pool: &SqlitePool, user_id: i64, id: i64) -> Result<Tag, PaymeError> {
    sqlx::query_as(
        r#"SELECT t.id, t.user_id, t.label, t.color, t.stopped,
                  COUNT(ta.item_id) AS usage_count
           FROM tags t
           LEFT JOIN tag_assignments ta ON ta.tag_id = t.id
           WHERE t.id = ? AND t.user_id = ?
           GROUP BY t.id"#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(PaymeError::NotFound)
}

pub async fn list_tags(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<Tag>>, PaymeError> {
    let tags = sqlx::query_as(
        r#"SELECT t.id, t.user_id, t.label, t.color, t.stopped,
                  COUNT(ta.item_id) AS usage_count
           FROM tags t
           LEFT JOIN tag_assignments ta ON ta.tag_id = t.id
           WHERE t.user_id = ?
           GROUP BY t.id"#,
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?;
    Ok(Json(tags))
}

pub async fn create_tag(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateTag>,
) -> Result<(StatusCode, Json<Tag>), PaymeError> {
    let label = validate_label(payload.label)?;
    validate_color(&payload.color)?;
    ensure_unique_label(&pool, claims.sub, &label, None).await?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO tags (user_id, label, normalized_label, color) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(claims.sub)
    .bind(&label)
    .bind(label.to_lowercase())
    .bind(&payload.color)
    .fetch_one(&pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(get_tag(&pool, claims.sub, id).await?),
    ))
}

pub async fn update_tag(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTag>,
) -> Result<Json<Tag>, PaymeError> {
    let existing = get_tag(&pool, claims.sub, id).await?;
    let label = match payload.label {
        Some(label) => validate_label(label)?,
        None => existing.label,
    };
    let color = payload.color.unwrap_or(existing.color);
    validate_color(&color)?;
    ensure_unique_label(&pool, claims.sub, &label, Some(id)).await?;

    sqlx::query(
        "UPDATE tags SET label = ?, normalized_label = ?, color = ? WHERE id = ? AND user_id = ?",
    )
    .bind(&label)
    .bind(label.to_lowercase())
    .bind(color)
    .bind(id)
    .bind(claims.sub)
    .execute(&pool)
    .await?;
    Ok(Json(get_tag(&pool, claims.sub, id).await?))
}

async fn set_stopped(
    pool: SqlitePool,
    user_id: i64,
    id: i64,
    stopped: bool,
) -> Result<Json<Tag>, PaymeError> {
    get_tag(&pool, user_id, id).await?;
    sqlx::query("UPDATE tags SET stopped = ? WHERE id = ? AND user_id = ?")
        .bind(stopped)
        .bind(id)
        .bind(user_id)
        .execute(&pool)
        .await?;
    Ok(Json(get_tag(&pool, user_id, id).await?))
}

pub async fn stop_tag(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<Tag>, PaymeError> {
    set_stopped(pool, claims.sub, id, true).await
}

pub async fn restore_tag(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<Json<Tag>, PaymeError> {
    set_stopped(pool, claims.sub, id, false).await
}
