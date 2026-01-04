use axum::{extract::State, http::StatusCode, Json};
use sqlx::SqlitePool;

use crate::middleware::auth::Claims;
use crate::models::{CategoryStats, MonthlyStats, StatsResponse};

pub async fn get_stats(
    State(pool): State<SqlitePool>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<StatsResponse>, StatusCode> {
    let months: Vec<(i64, i32, i32)> = sqlx::query_as(
        "SELECT id, year, month FROM months WHERE user_id = ? ORDER BY year DESC, month DESC",
    )
    .bind(claims.sub)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if months.is_empty() {
        return Ok(Json(StatsResponse {
            category_comparisons: vec![],
            monthly_trends: vec![],
            average_monthly_spending: 0.0,
            average_monthly_income: 0.0,
        }));
    }

    let mut monthly_trends: Vec<MonthlyStats> = vec![];
    let mut total_spending = 0.0;
    let mut total_income_all = 0.0;

    for (month_id, year, month) in &months {
        let income: (f64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(amount), 0) FROM income_entries WHERE month_id = ?",
        )
        .bind(month_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let spent: (f64,) =
            sqlx::query_as("SELECT COALESCE(SUM(amount), 0) FROM items WHERE month_id = ?")
                .bind(month_id)
                .fetch_one(&pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let fixed: (f64,) =
            sqlx::query_as("SELECT COALESCE(SUM(amount), 0) FROM fixed_expenses WHERE user_id = ?")
                .bind(claims.sub)
                .fetch_one(&pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        total_spending += spent.0;
        total_income_all += income.0;

        monthly_trends.push(MonthlyStats {
            year: *year,
            month: *month,
            total_income: income.0,
            total_spent: spent.0,
            total_fixed: fixed.0,
            net: income.0 - fixed.0 - spent.0,
        });
    }

    let month_count = months.len() as f64;
    let average_monthly_spending = if month_count > 0.0 {
        total_spending / month_count
    } else {
        0.0
    };
    let average_monthly_income = if month_count > 0.0 {
        total_income_all / month_count
    } else {
        0.0
    };

    let mut category_comparisons: Vec<CategoryStats> = vec![];

    if months.len() >= 1 {
        let current_month_id = months[0].0;
        let previous_month_id = months.get(1).map(|m| m.0);

        let categories: Vec<(i64, String)> =
            sqlx::query_as("SELECT id, label FROM budget_categories WHERE user_id = ?")
                .bind(claims.sub)
                .fetch_all(&pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        for (cat_id, cat_label) in categories {
            let current_spent: (f64,) = sqlx::query_as(
                "SELECT COALESCE(SUM(amount), 0) FROM items WHERE month_id = ? AND category_id = ?",
            )
            .bind(current_month_id)
            .bind(cat_id)
            .fetch_one(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let previous_spent: f64 = if let Some(prev_id) = previous_month_id {
                let result: (f64,) = sqlx::query_as(
                    "SELECT COALESCE(SUM(amount), 0) FROM items WHERE month_id = ? AND category_id = ?",
                )
                .bind(prev_id)
                .bind(cat_id)
                .fetch_one(&pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                result.0
            } else {
                0.0
            };

            let change_amount = current_spent.0 - previous_spent;
            let change_percent = if previous_spent > 0.0 {
                Some((change_amount / previous_spent) * 100.0)
            } else {
                None
            };

            category_comparisons.push(CategoryStats {
                category_id: cat_id,
                category_label: cat_label,
                current_month_spent: current_spent.0,
                previous_month_spent: previous_spent,
                change_amount,
                change_percent,
            });
        }
    }

    Ok(Json(StatsResponse {
        category_comparisons,
        monthly_trends,
        average_monthly_spending,
        average_monthly_income,
    }))
}
