use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;
use validator::Validate;

use crate::error::PaymeError;
use crate::middleware::auth::Claims;

#[derive(Serialize, ToSchema)]
pub struct SavingsResponse {
    pub savings: f64,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateSavings {
    #[validate(range(min = 0.0))]
    pub savings: f64,
}

#[derive(Serialize, ToSchema)]
pub struct RothIraResponse {
    pub roth_ira: f64,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateRothIra {
    #[validate(range(min = 0.0))]
    pub roth_ira: f64,
}

#[utoipa::path(
    get,
    path = "/api/savings",
    responses(
        (status = 200, body = SavingsResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wealth",
    summary = "Get savings balance",
    description = "Retrieves the user's total liquid savings amount stored in their profile."
)]
pub async fn get_savings(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<SavingsResponse>, PaymeError> {
    let savings: f64 = sqlx::query_scalar("SELECT savings FROM users WHERE id = ?")
        .bind(claims.sub)
        .fetch_one(&pool)
        .await?;

    Ok(Json(SavingsResponse { savings }))
}

#[utoipa::path(
    put,
    path = "/api/savings",
    request_body = UpdateSavings,
    responses(
        (status = 200, body = SavingsResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wealth",
    summary = "Update savings balance",
    description = "Sets a new value for the user's total liquid savings."
)]
pub async fn update_savings(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<UpdateSavings>,
) -> Result<Json<SavingsResponse>, PaymeError> {
    payload.validate()?;
    sqlx::query("UPDATE users SET savings = ? WHERE id = ?")
        .bind(payload.savings)
        .bind(claims.sub)
        .execute(&pool)
        .await?;

    Ok(Json(SavingsResponse {
        savings: payload.savings,
    }))
}

#[utoipa::path(
    get,
    path = "/api/roth-ira",
    responses(
        (status = 200, body = RothIraResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wealth",
    summary = "Get Roth IRA balance",
    description = "Retrieves the user's total Roth IRA investment balance."
)]
pub async fn get_roth_ira(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<RothIraResponse>, PaymeError> {
    let roth_ira: f64 = sqlx::query_scalar("SELECT roth_ira FROM users WHERE id = ?")
        .bind(claims.sub)
        .fetch_one(&pool)
        .await
        .unwrap_or(0.0);

    Ok(Json(RothIraResponse { roth_ira }))
}

#[utoipa::path(
    put,
    path = "/api/roth-ira",
    request_body = UpdateRothIra,
    responses(
        (status = 200, body = RothIraResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wealth",
    summary = "Update Roth IRA balance",
    description = "Sets a new value for the user's total Roth IRA balance."
)]
pub async fn update_roth_ira(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<UpdateRothIra>,
) -> Result<Json<RothIraResponse>, PaymeError> {
    payload.validate()?;
    sqlx::query("UPDATE users SET roth_ira = ? WHERE id = ?")
        .bind(payload.roth_ira)
        .bind(claims.sub)
        .execute(&pool)
        .await?;

    Ok(Json(RothIraResponse {
        roth_ira: payload.roth_ira,
    }))
}
