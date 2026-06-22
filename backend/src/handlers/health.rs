use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use sqlx::SqlitePool;

/// Represents the response for the health check endpoint.
#[derive(Serialize)]
pub struct HealthResponse {
    /// The overall health status of the service, typically "healthy" when operational.
    pub status: &'static str,
    /// The status of the database connection, either "connected" or "disconnected".
    pub database: &'static str,
}

pub async fn health_check(
    State(pool): State<SqlitePool>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let db_status = sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .map(|_| "connected")
        .unwrap_or("disconnected");

    if db_status == "connected" {
        Ok(Json(HealthResponse {
            status: "healthy",
            database: db_status,
        }))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
