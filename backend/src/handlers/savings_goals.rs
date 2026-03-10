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
use crate::models::CustomSavingsGoal;

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateSavingsGoal {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(range(min = 0.0))]
    pub current_amount: Option<f64>,
    #[validate(range(min = 0.01))]
    pub target_amount: f64,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateSavingsGoal {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(range(min = 0.0))]
    pub current_amount: Option<f64>,
    #[validate(range(min = 0.01))]
    pub target_amount: Option<f64>,
}

pub async fn list_savings_goals(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<CustomSavingsGoal>>, PaymeError> {
    let goals: Vec<CustomSavingsGoal> = sqlx::query_as(
        "SELECT id, user_id, name, current_amount, target_amount FROM custom_savings_goals WHERE user_id = ? ORDER BY id ASC",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?;

    Ok(Json(goals))
}

pub async fn create_savings_goal(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<CreateSavingsGoal>,
) -> Result<(StatusCode, Json<CustomSavingsGoal>), PaymeError> {
    payload.validate()?;
    let current_amount = payload.current_amount.unwrap_or(0.0);
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO custom_savings_goals (user_id, name, current_amount, target_amount) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(claims.sub)
    .bind(&payload.name)
    .bind(current_amount)
    .bind(payload.target_amount)
    .fetch_one(&pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CustomSavingsGoal {
            id,
            user_id: claims.sub,
            name: payload.name,
            current_amount,
            target_amount: payload.target_amount,
        }),
    ))
}

pub async fn update_savings_goal(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(goal_id): Path<i64>,
    Json(payload): Json<UpdateSavingsGoal>,
) -> Result<Json<CustomSavingsGoal>, PaymeError> {
    payload.validate()?;
    let existing: CustomSavingsGoal = sqlx::query_as(
        "SELECT id, user_id, name, current_amount, target_amount FROM custom_savings_goals WHERE id = ? AND user_id = ?",
    )
    .bind(goal_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    let name = payload.name.unwrap_or(existing.name);
    let current_amount = payload.current_amount.unwrap_or(existing.current_amount);
    let target_amount = payload.target_amount.unwrap_or(existing.target_amount);

    sqlx::query(
        "UPDATE custom_savings_goals SET name = ?, current_amount = ?, target_amount = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(current_amount)
    .bind(target_amount)
    .bind(goal_id)
    .execute(&pool)
    .await?;

    Ok(Json(CustomSavingsGoal {
        id: goal_id,
        user_id: claims.sub,
        name,
        current_amount,
        target_amount,
    }))
}

pub async fn delete_savings_goal(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(goal_id): Path<i64>,
) -> Result<StatusCode, PaymeError> {
    sqlx::query("DELETE FROM custom_savings_goals WHERE id = ? AND user_id = ?")
        .bind(goal_id)
        .bind(claims.sub)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
