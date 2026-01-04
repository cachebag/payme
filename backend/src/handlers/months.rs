use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sqlx::SqlitePool;

use crate::middleware::auth::Claims;
use crate::models::{
    FixedExpense, IncomeEntry, ItemWithCategory, Month, MonthSummary, MonthlyBudgetWithCategory,
};
use crate::pdf;

pub async fn list_months(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<Month>>, StatusCode> {
    let months: Vec<Month> = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE user_id = ? ORDER BY year DESC, month DESC",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(months))
}

pub async fn get_or_create_current_month(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<MonthSummary>, StatusCode> {
    let now = Utc::now();
    let year = now.format("%Y").to_string().parse::<i32>().unwrap();
    let month = now.format("%m").to_string().parse::<i32>().unwrap();

    let existing: Option<Month> = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE user_id = ? AND year = ? AND month = ?",
    )
    .bind(claims.sub)
    .bind(year)
    .bind(month)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let categories: Vec<(i64, f64)> = sqlx::query_as(
                "SELECT id, default_amount FROM budget_categories WHERE user_id = ?",
            )
            .bind(claims.sub)
            .fetch_all(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

pub async fn get_month(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<Json<MonthSummary>, StatusCode> {
    let month: Month = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE id = ? AND user_id = ?",
    )
    .bind(month_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    get_month_summary(&pool, claims.sub, month.id).await
}

async fn get_month_summary(
    pool: &SqlitePool,
    user_id: i64,
    month_id: i64,
) -> Result<Json<MonthSummary>, StatusCode> {
    let month: Month = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE id = ?",
    )
    .bind(month_id)
    .fetch_one(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let income_entries: Vec<IncomeEntry> =
        sqlx::query_as("SELECT id, month_id, label, amount FROM income_entries WHERE month_id = ?")
            .bind(month_id)
            .fetch_all(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let fixed_expenses: Vec<FixedExpense> =
        sqlx::query_as("SELECT id, user_id, label, amount FROM fixed_expenses WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
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
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

pub async fn close_month(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<Json<Month>, StatusCode> {
    let month: Month = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE id = ? AND user_id = ?",
    )
    .bind(month_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if month.is_closed {
        return Err(StatusCode::BAD_REQUEST);
    }

    let summary = get_month_summary(&pool, claims.sub, month_id).await?.0;
    let pdf_data = pdf::generate_pdf(&summary).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("INSERT INTO monthly_snapshots (month_id, pdf_data) VALUES (?, ?)")
        .bind(month_id)
        .bind(&pdf_data)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let now = Utc::now();
    sqlx::query("UPDATE months SET is_closed = 1, closed_at = ? WHERE id = ?")
        .bind(now)
        .bind(month_id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let updated: Month = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE id = ?",
    )
    .bind(month_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated))
}

pub async fn get_month_pdf(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(month_id): Path<i64>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let _month: Month = sqlx::query_as(
        "SELECT id, user_id, year, month, is_closed, closed_at FROM months WHERE id = ? AND user_id = ?",
    )
    .bind(month_id)
    .bind(claims.sub)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let snapshot: (Vec<u8>,) =
        sqlx::query_as("SELECT pdf_data FROM monthly_snapshots WHERE month_id = ?")
            .bind(month_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

    Ok((
        [
            ("Content-Type", "application/pdf"),
            ("Content-Disposition", "attachment; filename=\"month.pdf\""),
        ],
        snapshot.0,
    ))
}
