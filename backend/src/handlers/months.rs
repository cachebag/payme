use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{Datelike, Utc};
use sqlx::SqlitePool;

use crate::error::PaymeError;
use crate::middleware::auth::Claims;
use crate::models::{
    FixedExpense, IncomeEntry, ItemWithCategory, Month, MonthSummary, MonthlyBudgetWithCategory,
};
use crate::pdf;

#[utoipa::path(
    get,
    path = "/api/months",
    responses(
        (status = 200, description = "List all months for the user", body = [Month]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Months",
    summary = "List all budget months",
    description = "Retrieves a history of all months created by the user, ordered by date."
)]
pub async fn list_months(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<Month>>, PaymeError> {
    let months: Vec<Month> = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE user_id = ? ORDER BY year DESC, month DESC",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await?;

    Ok(Json(months))
}

#[utoipa::path(
    get,
    path = "/api/months/current",
    responses(
        (status = 200, description = "Get current month or create it if it doesn't exist", body = MonthSummary),
        (status = 500, description = "Internal server error")
    ),
    tag = "Months",
    summary = "Get current month summary",
    description = "Checks for the current calendar month. If it doesn't exist, it creates it and copies over your default categories."
)]
pub async fn get_or_create_current_month(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<MonthSummary>, PaymeError> {
    let now = Utc::now();
    let year = now.year();
    let month = now.month() as i32;

    let existing: Option<Month> = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE user_id = ? AND year = ? AND month = ?",
    )
    .bind(claims.sub)
    .bind(year)
    .bind(month)
    .fetch_optional(&pool)
    .await?;

    let month_record = match existing {
        Some(m) => m,
        None => {
            let id: i64 = sqlx::query_scalar(
                "INSERT INTO months (user_id, year, month) VALUES (?, ?, ?) RETURNING id",
            )
            .bind(claims.sub)
            .bind(year)
            .bind(month)
            .fetch_one(&pool)
            .await?;

            let categories: Vec<(i64, f64)> = sqlx::query_as(
                "SELECT id, default_amount FROM budget_categories WHERE user_id = ?",
            )
            .bind(claims.sub)
            .fetch_all(&pool)
            .await?;

            for (cat_id, default_amount) in categories {
                sqlx::query(
                    "INSERT INTO monthly_budgets (month_id, category_id, allocated_amount) VALUES (?, ?, ?)",
                )
                .bind(id)
                .bind(cat_id)
                .bind(default_amount)
                .execute(&pool)
                .await
                .ok();
            }

            Month {
                id,
                user_id: claims.sub,
                year,
                month,
                is_closed: false,
                closed_at: None,
            }
        }
    };

    get_month_summary(&pool, claims.sub, month_record.id).await
}

#[utoipa::path(
    get,
    path = "/api/months/{id}",
    params(
        ("id" = i64, Path, description = "Month ID")
    ),
    responses(
        (status = 200, description = "Get full summary for a specific month", body = MonthSummary),
        (status = 500, description = "Internal server error")
    ),
    tag = "Months",
    summary = "Get specific month details",
    description = "Returns a complete financial summary for a given month ID, including income, fixed expenses, and itemized spending."
)]
pub async fn get_month(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<Json<MonthSummary>, PaymeError> {
    let month: Month = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE id = ? AND user_id = ?",
    )
    .bind(month_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    get_month_summary(&pool, claims.sub, month.id).await
}

async fn get_month_summary(
    pool: &SqlitePool,
    user_id: i64,
    month_id: i64,
) -> Result<Json<MonthSummary>, PaymeError> {
    let month: Month = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE id = ?",
    )
    .bind(month_id)
    .fetch_one(pool)
    .await?;

    let income_entries: Vec<IncomeEntry> =
        sqlx::query_as("SELECT id, month_id, label, amount FROM income_entries WHERE month_id = ?")
            .bind(month_id)
            .fetch_all(pool)
            .await?;

    let fixed_expenses: Vec<FixedExpense> =
        sqlx::query_as("SELECT id, user_id, label, amount FROM fixed_expenses WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(pool)
            .await?;

    let budgets: Vec<MonthlyBudgetWithCategory> =
        sqlx::query_as::<_, (i64, i64, i64, String, f64)>(
            r#"
        SELECT mb.id, mb.month_id, mb.category_id, bc.label, mb.allocated_amount
        FROM monthly_budgets mb
        JOIN budget_categories bc ON mb.category_id = bc.id
        WHERE mb.month_id = ?
        "#,
        )
        .bind(month_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(
            |(id, month_id, category_id, category_label, allocated_amount)| {
                MonthlyBudgetWithCategory {
                    id,
                    month_id,
                    category_id,
                    category_label,
                    allocated_amount,
                    spent_amount: 0.0,
                }
            },
        )
        .collect();

    let items: Vec<ItemWithCategory> = sqlx::query_as(
        r#"
        SELECT i.id, i.month_id, i.category_id, bc.label as category_label, i.description, i.amount, i.spent_on
        FROM items i
        JOIN budget_categories bc ON i.category_id = bc.id
        WHERE i.month_id = ?
        ORDER BY i.spent_on DESC
        "#,
    )
    .bind(month_id)
    .fetch_all(pool)
    .await?;

    let budgets: Vec<MonthlyBudgetWithCategory> = budgets
        .into_iter()
        .map(|mut b| {
            b.spent_amount = items
                .iter()
                .filter(|i| i.category_id == b.category_id)
                .map(|i| i.amount)
                .sum();
            b
        })
        .collect();

    let total_income: f64 = income_entries.iter().map(|i| i.amount).sum();
    let total_fixed: f64 = fixed_expenses.iter().map(|e| e.amount).sum();
    let total_budgeted: f64 = budgets.iter().map(|b| b.allocated_amount).sum();
    let total_spent: f64 = items.iter().map(|i| i.amount).sum();
    let remaining = total_income - total_fixed - total_spent;

    Ok(Json(MonthSummary {
        month,
        income_entries,
        fixed_expenses,
        budgets,
        items,
        total_income,
        total_fixed,
        total_budgeted,
        total_spent,
        remaining,
    }))
}

#[utoipa::path(
    post,
    path = "/api/months/{id}/close",
    params(
        ("id" = i64, Path, description = "Month ID")
    ),
    responses(
        (status = 200, description = "Month closed and PDF snapshot generated", body = Month),
        (status = 400, description = "Month is already closed"),
        (status = 404, description = "Month not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Months",
    summary = "Close month and generate report",
    description = "Finalizes the month, prevents further edits, and generates a PDF snapshot for long-term storage."
)]
pub async fn close_month(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<Json<Month>, PaymeError> {
    let month: Month = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE id = ? AND user_id = ?",
    )
    .bind(month_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    if month.is_closed {
        return Err(PaymeError::BadRequest(
            "Month is already closed".to_string(),
        ));
    }

    let summary = get_month_summary(&pool, claims.sub, month_id).await?.0;
    let pdf_data = pdf::generate_pdf(&summary).map_err(|e| PaymeError::Internal(e.to_string()))?;

    sqlx::query("INSERT INTO monthly_snapshots (month_id, pdf_data) VALUES (?, ?)")
        .bind(month_id)
        .bind(&pdf_data)
        .execute(&pool)
        .await?;

    let now = Utc::now();
    sqlx::query("UPDATE months SET is_closed = 1, closed_at = ? WHERE id = ?")
        .bind(now)
        .bind(month_id)
        .execute(&pool)
        .await?;

    let updated: Month = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE id = ?",
    )
    .bind(month_id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(updated))
}

#[utoipa::path(
    get,
    path = "/api/months/{id}/pdf",
    params(
        ("id" = i64, Path, description = "Month ID")
    ),
    responses(
        (status = 200, description = "Download the PDF snapshot", content_type = "application/pdf"),
        (status = 404, description = "PDF snapshot not found for this month")
    ),
    tag = "Months",
    summary = "Download month PDF",
    description = "Retrieves the binary PDF data for a closed month's financial report."
)]
pub async fn get_month_pdf(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<impl axum::response::IntoResponse, PaymeError> {
    let _month: Month = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE id = ? AND user_id = ?",
    )
    .bind(month_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await?
    .ok_or(PaymeError::NotFound)?;

    let snapshot: (Vec<u8>,) =
        sqlx::query_as("SELECT pdf_data FROM monthly_snapshots WHERE month_id = ?")
            .bind(month_id)
            .fetch_optional(&pool)
            .await?
            .ok_or(PaymeError::NotFound)?;

    Ok((
        [
            ("Content-Type", "application/pdf"),
            ("Content-Disposition", "attachment; filename=\"month.pdf\""),
        ],
        snapshot.0,
    ))
}
