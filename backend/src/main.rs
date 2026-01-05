mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod openapi;
mod pdf;

use axum::{
    middleware::from_fn,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::openapi::ApiDoc;
use config::Config;
use handlers::{
    auth, budget, export, fixed_expenses, health, income, items, months, savings, stats,
};
use middleware::auth::auth_middleware;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::from_env();
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login));

    let protected_routes = Router::new()
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
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
        .route("/api/roth-ira", get(savings::get_roth_ira))
        .route("/api/roth-ira", put(savings::update_roth_ira))
        .route("/api/export/json", get(export::export_json))
        .route("/api/import/json", post(export::import_json))
        .layer(from_fn(auth_middleware));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false);

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .fallback_service(ServeDir::new("/app/static"))
        .layer(cors)
        .with_state(pool);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Received Ctrl+C, shutting down"),
        _ = terminate => tracing::info!("Received SIGTERM, shutting down"),
    }
}
