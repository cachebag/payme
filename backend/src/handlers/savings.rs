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
pub struct RetirementSavingsResponse {
    pub retirement_savings: f64,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateRetirementSavings {
    #[validate(range(min = 0.0))]
    pub retirement_savings: f64,
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
    path = "/api/retirement-savings",
    responses(
        (status = 200, body = RetirementSavingsResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wealth",
    summary = "Get retirement savings balance",
    description = "Retrieves the user's total retirement savings balance."
)]
pub async fn get_retirement_savings(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<RetirementSavingsResponse>, PaymeError> {
    let retirement_savings: f64 =
        sqlx::query_scalar("SELECT retirement_savings FROM users WHERE id = ?")
            .bind(claims.sub)
            .fetch_one(&pool)
            .await
            .unwrap_or(0.0);

    Ok(Json(RetirementSavingsResponse { retirement_savings }))
}

#[utoipa::path(
    put,
    path = "/api/retirement-savings",
    request_body = UpdateRetirementSavings,
    responses(
        (status = 200, body = RetirementSavingsResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wealth",
    summary = "Update retirement savings balance",
    description = "Sets a new value for the user's total retirement savings balance."
)]
pub async fn update_retirement_savings(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(payload): Json<UpdateRetirementSavings>,
) -> Result<Json<RetirementSavingsResponse>, PaymeError> {
    payload.validate()?;
    sqlx::query("UPDATE users SET retirement_savings = ? WHERE id = ?")
        .bind(payload.retirement_savings)
        .bind(claims.sub)
        .execute(&pool)
        .await?;

    Ok(Json(RetirementSavingsResponse {
        retirement_savings: payload.retirement_savings,
    }))
}
