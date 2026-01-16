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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn test_not_found_status() {
        let error = PaymeError::NotFound;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_unauthorized_status() {
        let error = PaymeError::Unauthorized;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_bad_request_status() {
        let error = PaymeError::BadRequest("test".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_internal_status() {
        let error = PaymeError::Internal("test".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_display() {
        assert_eq!(PaymeError::NotFound.to_string(), "Not found");
        assert_eq!(PaymeError::Unauthorized.to_string(), "Unauthorized");
        assert_eq!(
            PaymeError::BadRequest("invalid".to_string()).to_string(),
            "Bad request: invalid"
        );
        assert_eq!(
            PaymeError::Internal("error".to_string()).to_string(),
            "Internal error: error"
        );
    }
}
