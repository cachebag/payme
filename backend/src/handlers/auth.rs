use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, response::IntoResponse, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use url::Url;
use utoipa::ToSchema;
use validator::Validate;

use crate::error::PaymeError;
use crate::middleware::auth::Claims;

#[derive(Deserialize, ToSchema, Validate)]
pub struct AuthRequest {
    #[validate(length(min = 3, max = 32))]
    pub username: String,
    #[validate(length(min = 6, max = 128))]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub id: i64,
    pub username: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = AuthRequest,
    responses(
        (status = 200, description = "User registered successfully", body = AuthResponse),
        (status = 409, description = "Username already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth",
    summary = "Register a new account",
    description = "Creates a new user record. Returns the newly created user's ID and username."
)]
pub async fn register(
    State(pool): State<SqlitePool>,
    Json(payload): Json<AuthRequest>,
) -> Result<impl IntoResponse, PaymeError> {
    payload.validate()?;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| PaymeError::Internal(e.to_string()))?
        .to_string();

    let result = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (username, password_hash) VALUES (?, ?) RETURNING id",
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await?;

    Ok(Json(AuthResponse {
        id: result,
        username: payload.username,
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = AuthRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth",
    summary = "Authenticate user",
    description = "Verifies credentials and issues a JWT token."
)]
pub async fn login(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
    Json(payload): Json<AuthRequest>,
) -> Result<impl IntoResponse, PaymeError> {
    payload.validate()?;
    let user: (i64, String, String) =
        sqlx::query_as("SELECT id, username, password_hash FROM users WHERE username = ?")
            .bind(&payload.username)
            .fetch_optional(&pool)
            .await?
            .ok_or(PaymeError::Unauthorized)?;

    let parsed_hash =
        PasswordHash::new(&user.2).map_err(|e| PaymeError::Internal(e.to_string()))?;
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| PaymeError::Unauthorized)?;

    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "payme-secret-key-change-in-production".to_string());

    let claims = Claims {
        sub: user.0,
        username: user.1.clone(),
        exp: (Utc::now() + Duration::days(30)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| PaymeError::Internal(e.to_string()))?;

    let cookie = Cookie::build(("token", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(30))
        .build();

    Ok((
        jar.add(cookie),
        Json(AuthResponse {
            id: user.0,
            username: user.1,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "Logout successful."),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth",
    summary = "Log out user",
    description = "Clears the authentication token by setting the session cookie to expire immediately."
)]
pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::seconds(0))
        .build();

    jar.add(cookie)
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "Current user retrieved", body = AuthResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth",
    summary = "Get current user profile",
    description = "Retrives authenticated user's information."
)]
pub async fn me(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<AuthResponse>, PaymeError> {
    let user: (i64, String) = sqlx::query_as("SELECT id, username FROM users WHERE id = ?")
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await?
        .ok_or(PaymeError::NotFound)?;

    Ok(Json(AuthResponse {
        id: user.0,
        username: user.1,
    }))
}

pub async fn export_db(
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<impl IntoResponse, PaymeError> {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:payme.db".to_string());

    let db_path = Url::parse(&db_url)
        .map_err(|e| PaymeError::Internal(e.to_string()))?
        .path()
        .to_string();

    let data = tokio::fs::read(&db_path)
        .await
        .map_err(|e| PaymeError::Internal(e.to_string()))?;

    let filename = format!("attachment; filename=\"payme-{}.db\"", claims.username);
    Ok((
        [
            (
                "Content-Type".to_string(),
                "application/octet-stream".to_string(),
            ),
            ("Content-Disposition".to_string(), filename),
        ],
        data,
    ))
}
