use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum PaymeError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for PaymeError {
    fn into_response(self) -> Response {
        let status = match &self {
            PaymeError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PaymeError::Validation(_) => StatusCode::BAD_REQUEST,
            PaymeError::NotFound => StatusCode::NOT_FOUND,
            PaymeError::Unauthorized => StatusCode::UNAUTHORIZED,
            PaymeError::BadRequest(_) => StatusCode::BAD_REQUEST,
            PaymeError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        tracing::error!("{self}");
        status.into_response()
    }
}
