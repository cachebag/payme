use utoipa::OpenApi;

use crate::handlers::{
    auth::{AuthRequest, AuthResponse},
    budget::{CreateCategory, UpdateCategory, UpdateMonthlyBudget},
    fixed_expenses::{CreateFixedExpense, UpdateFixedExpense},
    income::{CreateIncome, UpdateIncome},
    items::{CreateItem, UpdateItem},
    savings::{RothIraResponse, SavingsResponse, UpdateSavings, UpdateRothIra},
    export::{UserExport, CategoryExport, MonthExport, FixedExpenseExport, IncomeExport, BudgetExport, ItemExport}
};
use crate::models::{
    BudgetCategory, FixedExpense, IncomeEntry, Item, 
    ItemWithCategory, Month, MonthSummary, MonthlyBudget, 
    StatsResponse, CategoryStats, MonthlyStats
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::auth::register, 
        crate::handlers::auth::login,
        crate::handlers::auth::logout,
        crate::handlers::auth::me,
        crate::handlers::export::export_json,
        crate::handlers::export::import_json,
        crate::handlers::budget::list_monthly_budgets,
        crate::handlers::budget::update_monthly_budget,
        crate::handlers::income::list_income,
        crate::handlers::income::create_income,
        crate::handlers::income::update_income,
        crate::handlers::income::delete_income,
        crate::handlers::items::list_items,
        crate::handlers::items::create_item,
        crate::handlers::items::update_item,
        crate::handlers::items::delete_item,
        crate::handlers::fixed_expenses::list_fixed_expenses,
        crate::handlers::fixed_expenses::create_fixed_expense,
        crate::handlers::fixed_expenses::update_fixed_expense,
        crate::handlers::fixed_expenses::delete_fixed_expense,
        crate::handlers::budget::list_categories,
        crate::handlers::budget::create_category,
        crate::handlers::budget::update_category,
        crate::handlers::budget::delete_category,
        crate::handlers::months::list_months,
        crate::handlers::months::get_or_create_current_month,
        crate::handlers::months::get_month,
        crate::handlers::months::close_month,
        crate::handlers::months::get_month_pdf,
        crate::handlers::savings::get_savings,
        crate::handlers::savings::update_savings,
        crate::handlers::savings::get_roth_ira,
        crate::handlers::savings::update_roth_ira,
        crate::handlers::stats::get_stats
    ),
    components(
        schemas(
            AuthRequest, AuthResponse, MonthlyBudget, UpdateMonthlyBudget, 
            IncomeEntry, CreateIncome, UpdateIncome,
            Item, ItemWithCategory, CreateItem, UpdateItem,
            FixedExpense, CreateFixedExpense, UpdateFixedExpense,
            BudgetCategory, CreateCategory, UpdateCategory,
            Month, MonthSummary, 
            StatsResponse, CategoryStats, MonthlyStats,
            RothIraResponse, SavingsResponse, UpdateSavings, UpdateRothIra,
            UserExport, CategoryExport, MonthExport, FixedExpenseExport, 
            IncomeExport, BudgetExport, ItemExport
        )
    )
)]
pub struct ApiDoc;