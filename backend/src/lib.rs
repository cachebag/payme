pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod openapi;
pub mod pdf;

use axum::{
    middleware::from_fn,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};

use handlers::{
    auth, budget, export, fixed_expenses, health, income, items, months, savings, stats,
};
use middleware::auth::auth_middleware;

/// Create the application router with all routes
pub fn create_app(pool: SqlitePool) -> Router {
    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login));

    let protected_routes = Router::new()
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/change-username", put(auth::change_username))
        .route("/api/auth/change-password", put(auth::change_password))
        .route("/api/auth/clear-data", delete(auth::clear_all_data))
        .route("/api/export", get(auth::export_db))
        .route("/api/months", get(months::list_months))
        .route(
            "/api/months/current",
            get(months::get_or_create_current_month),
        )
        .route("/api/months/{id}", get(months::get_month))
        .route("/api/months/{id}/close", post(months::close_month))
        .route("/api/months/{id}/pdf", get(months::get_month_pdf))
        .route(
            "/api/fixed-expenses",
            get(fixed_expenses::list_fixed_expenses),
        )
        .route(
            "/api/fixed-expenses",
            post(fixed_expenses::create_fixed_expense),
        )
        .route(
            "/api/fixed-expenses/{id}",
            put(fixed_expenses::update_fixed_expense),
        )
        .route(
            "/api/fixed-expenses/{id}",
            delete(fixed_expenses::delete_fixed_expense),
        )
        .route("/api/categories", get(budget::list_categories))
        .route("/api/categories", post(budget::create_category))
        .route("/api/categories/{id}", put(budget::update_category))
        .route("/api/categories/{id}", delete(budget::delete_category))
        .route(
            "/api/months/{id}/budgets",
            get(budget::list_monthly_budgets),
        )
        .route(
            "/api/months/{month_id}/budgets/{id}",
            put(budget::update_monthly_budget),
        )
        .route("/api/months/{id}/income", get(income::list_income))
        .route("/api/months/{id}/income", post(income::create_income))
        .route(
            "/api/months/{month_id}/income/{id}",
            put(income::update_income),
        )
        .route(
            "/api/months/{month_id}/income/{id}",
            delete(income::delete_income),
        )
        .route("/api/months/{id}/items", get(items::list_items))
        .route("/api/months/{id}/items", post(items::create_item))
        .route("/api/months/{month_id}/items/{id}", put(items::update_item))
        .route(
            "/api/months/{month_id}/items/{id}",
            delete(items::delete_item),
        )
        .route("/api/stats", get(stats::get_stats))
        .route("/api/savings", get(savings::get_savings))
        .route("/api/savings", put(savings::update_savings))
        .route("/api/savings/goal", put(savings::update_savings_goal))
        .route(
            "/api/retirement-savings",
            get(savings::get_retirement_savings),
        )
        .route(
            "/api/retirement-savings",
            put(savings::update_retirement_savings),
        )
        .route("/api/export/json", get(export::export_json))
        .route("/api/import/json", post(export::import_json))
        .layer(from_fn(auth_middleware));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false);

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
        .with_state(pool)
}
