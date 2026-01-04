use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::middleware::auth::Claims;

#[derive(Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub id: i64,
    pub username: String,
}

pub async fn register(
    State(pool): State<SqlitePool>,
    Json(payload): Json<AuthRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let result = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (username, password_hash) VALUES (?, ?) RETURNING id",
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::CONFLICT)?;

    Ok(Json(AuthResponse {
        id: result,
        username: payload.username,
    }))
}

pub async fn login(
    State(pool): State<SqlitePool>,
    jar: CookieJar,
    Json(payload): Json<AuthRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let user: (i64, String, String) =
        sqlx::query_as("SELECT id, username, password_hash FROM users WHERE username = ?")
            .bind(&payload.username)
            .fetch_optional(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

    let parsed_hash = PasswordHash::new(&user.2).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

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
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::seconds(0))
        .build();

    jar.add(cookie)
}

pub async fn me(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let user: (i64, String) = sqlx::query_as("SELECT id, username FROM users WHERE id = ?")
        .bind(claims.sub)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(AuthResponse {
        id: user.0,
        username: user.1,
    }))
}

pub async fn export_db(
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<impl IntoResponse, StatusCode> {
    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:payme.db".to_string())
        .replace("sqlite:", "");

    let data = tokio::fs::read(&db_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
