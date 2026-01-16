#![allow(dead_code)]

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::Router;
use axum_test::TestServer;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use axum::http::{HeaderName, HeaderValue};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub exp: usize,
}

/// Create an in-memory SQLite pool and run migrations
pub async fn create_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:")
        .await
        .expect("Failed to create in-memory database");

    run_migrations(&pool).await;
    pool
}

/// Run database migrations (copied from db/mod.rs to avoid circular deps)
async fn run_migrations(pool: &SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            savings REAL NOT NULL DEFAULT 0,
            savings_goal REAL NOT NULL DEFAULT 0,
            retirement_savings REAL NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create users table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS fixed_expenses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            label TEXT NOT NULL,
            amount REAL NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create fixed_expenses table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS budget_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            label TEXT NOT NULL,
            default_amount REAL NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create budget_categories table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS months (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            year INTEGER NOT NULL,
            month INTEGER NOT NULL,
            is_closed INTEGER NOT NULL DEFAULT 0,
            closed_at TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE(user_id, year, month)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create months table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS income_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL,
            label TEXT NOT NULL,
            amount REAL NOT NULL,
            FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create income_entries table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS monthly_budgets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            allocated_amount REAL NOT NULL,
            FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES budget_categories(id) ON DELETE CASCADE,
            UNIQUE(month_id, category_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create monthly_budgets table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            description TEXT NOT NULL,
            amount REAL NOT NULL,
            spent_on TEXT NOT NULL,
            FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES budget_categories(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create items table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS monthly_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL UNIQUE,
            pdf_data BLOB NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create monthly_snapshots table");
}

/// Create a test user and return their ID
pub async fn create_test_user(pool: &SqlitePool, username: &str, password: &str) -> i64 {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (username, password_hash) VALUES (?, ?) RETURNING id",
    )
    .bind(username)
    .bind(&password_hash)
    .fetch_one(pool)
    .await
    .expect("Failed to create test user")
}

/// Generate a JWT token for a user
pub fn generate_token(user_id: i64, username: &str) -> String {
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "payme-secret-key-change-in-production".to_string());

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: (Utc::now() + Duration::days(30)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Failed to generate token")
}

/// Generate an expired JWT token for testing
pub fn generate_expired_token(user_id: i64, username: &str) -> String {
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "payme-secret-key-change-in-production".to_string());

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: (Utc::now() - Duration::days(1)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Failed to generate token")
}

/// Create a test category and return its ID
pub async fn create_test_category(
    pool: &SqlitePool,
    user_id: i64,
    label: &str,
    default_amount: f64,
) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO budget_categories (user_id, label, default_amount) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(user_id)
    .bind(label)
    .bind(default_amount)
    .fetch_one(pool)
    .await
    .expect("Failed to create test category")
}

/// Create a test month and return its ID
pub async fn create_test_month(pool: &SqlitePool, user_id: i64, year: i32, month: i32) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO months (user_id, year, month) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(user_id)
    .bind(year)
    .bind(month)
    .fetch_one(pool)
    .await
    .expect("Failed to create test month")
}

/// Create a test fixed expense and return its ID
pub async fn create_test_fixed_expense(
    pool: &SqlitePool,
    user_id: i64,
    label: &str,
    amount: f64,
) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO fixed_expenses (user_id, label, amount) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(user_id)
    .bind(label)
    .bind(amount)
    .fetch_one(pool)
    .await
    .expect("Failed to create test fixed expense")
}

/// Create a test income entry and return its ID
pub async fn create_test_income(pool: &SqlitePool, month_id: i64, label: &str, amount: f64) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO income_entries (month_id, label, amount) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(month_id)
    .bind(label)
    .bind(amount)
    .fetch_one(pool)
    .await
    .expect("Failed to create test income")
}

/// Create a test item and return its ID
pub async fn create_test_item(
    pool: &SqlitePool,
    month_id: i64,
    category_id: i64,
    description: &str,
    amount: f64,
    spent_on: &str,
) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO items (month_id, category_id, description, amount, spent_on) VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(month_id)
    .bind(category_id)
    .bind(description)
    .bind(amount)
    .bind(spent_on)
    .fetch_one(pool)
    .await
    .expect("Failed to create test item")
}

/// Create a test monthly budget and return its ID
pub async fn create_test_budget(
    pool: &SqlitePool,
    month_id: i64,
    category_id: i64,
    allocated_amount: f64,
) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO monthly_budgets (month_id, category_id, allocated_amount) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(month_id)
    .bind(category_id)
    .bind(allocated_amount)
    .fetch_one(pool)
    .await
    .expect("Failed to create test budget")
}

/// Close a month
pub async fn close_test_month(pool: &SqlitePool, month_id: i64) {
    sqlx::query("UPDATE months SET is_closed = 1, closed_at = datetime('now') WHERE id = ?")
        .bind(month_id)
        .execute(pool)
        .await
        .expect("Failed to close test month");
}

/// Authorization header name
pub fn auth_name() -> HeaderName {
    HeaderName::from_static("authorization")
}

/// Authorization header value
pub fn auth_value(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!("Bearer {}", token)).unwrap()
}

/// Create a test server from a router
pub fn create_test_server(app: Router) -> TestServer {
    TestServer::new(app).unwrap()
}
