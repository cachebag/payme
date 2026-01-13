use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use utoipa::{IntoParams, ToSchema};

use crate::audit::{AuditLog, AuditLogger};
use crate::error::PaymeError;
use crate::middleware::auth::Claims;

#[derive(Deserialize, IntoParams)]
pub struct AuditListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(serde::Serialize, ToSchema)]
pub struct AuditListResponse {
    pub logs: Vec<AuditLog>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(serde::Serialize, ToSchema)]
pub struct ActivitySummary {
    pub action: String,
    pub count: i64,
}

#[derive(serde::Serialize, ToSchema)]
pub struct ActivitySummaryResponse {
    pub summary: Vec<ActivitySummary>,
    pub days: i64,
}

#[utoipa::path(
    get,
    path = "/api/audit/logs",
    params(AuditListQuery),
    responses(
        (status = 200, body = AuditListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Audit",
    summary = "List audit logs",
    description = "Retrieves audit logs for the authenticated user with pagination."
)]
pub async fn list_audit_logs(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Query(query): Query<AuditListQuery>,
) -> Result<Json<AuditListResponse>, PaymeError> {
    let logger = AuditLogger::new(pool);

    let logs = logger
        .list_by_user(claims.sub, query.limit, query.offset)
        .await?;

    let total = logger.count_by_user(claims.sub).await?;

    Ok(Json(AuditListResponse {
        logs,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

#[derive(Deserialize, IntoParams)]
pub struct ActivityQuery {
    #[serde(default = "default_days")]
    pub days: i64,
}

fn default_days() -> i64 {
    30
}

#[utoipa::path(
    get,
    path = "/api/audit/activity",
    params(ActivityQuery),
    responses(
        (status = 200, body = ActivitySummaryResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Audit",
    summary = "Get activity summary",
    description = "Retrieves a summary of user activities grouped by action type."
)]
pub async fn get_activity_summary(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Query(query): Query<ActivityQuery>,
) -> Result<Json<ActivitySummaryResponse>, PaymeError> {
    let logger = AuditLogger::new(pool);

    let summary_data = logger
        .get_user_activity_summary(claims.sub, query.days)
        .await?;

    let summary = summary_data
        .into_iter()
        .map(|(action, count)| ActivitySummary { action, count })
        .collect();

    Ok(Json(ActivitySummaryResponse {
        summary,
        days: query.days,
    }))
}
