use printpdf::*;
use std::io::BufWriter;

use crate::models::MonthSummary;

pub fn generate_pdf(summary: &MonthSummary) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let title = format!(
        "Financial Summary - {}/{}",
        summary.month.month, summary.month.year
    );
    let (doc, page1, layer1) = PdfDocument::new(&title, Mm(210.0), Mm(297.0), "Layer 1");

    let layer = doc.get_page(page1).get_layer(layer1);
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

    let mut y = 270.0;
    let left_margin = 20.0;
    let line_height = 6.0;

    layer.use_text(&title, 16.0, Mm(left_margin), Mm(y), &font_bold);
    y -= line_height * 2.0;

    layer.use_text("INCOME", 12.0, Mm(left_margin), Mm(y), &font_bold);
    y -= line_height;

    for entry in &summary.income_entries {
        let text = format!("  {} - ${:.2}", entry.label, entry.amount);
        layer.use_text(&text, 10.0, Mm(left_margin), Mm(y), &font);
        y -= line_height;
    }

    let total_income_text = format!("Total Income: ${:.2}", summary.total_income);
    layer.use_text(&total_income_text, 10.0, Mm(left_margin), Mm(y), &font_bold);
    y -= line_height * 2.0;

    layer.use_text("FIXED EXPENSES", 12.0, Mm(left_margin), Mm(y), &font_bold);
    y -= line_height;

    for expense in &summary.fixed_expenses {
        let text = format!("  {} - ${:.2}", expense.label, expense.amount);
        layer.use_text(&text, 10.0, Mm(left_margin), Mm(y), &font);
        y -= line_height;
    }

    let total_fixed_text = format!("Total Fixed: ${:.2}", summary.total_fixed);
    layer.use_text(&total_fixed_text, 10.0, Mm(left_margin), Mm(y), &font_bold);
    y -= line_height * 2.0;

    layer.use_text("BUDGET VS ACTUAL", 12.0, Mm(left_margin), Mm(y), &font_bold);
    y -= line_height;

    for budget in &summary.budgets {
        let status = if budget.spent_amount > budget.allocated_amount {
            format!(
                "OVER by ${:.2}",
                budget.spent_amount - budget.allocated_amount
            )
        } else {
            format!(
                "${:.2} remaining",
                budget.allocated_amount - budget.spent_amount
            )
        };

        let text = format!(
            "  {}: ${:.2} / ${:.2} ({})",
            budget.category_label, budget.spent_amount, budget.allocated_amount, status
        );
        layer.use_text(&text, 10.0, Mm(left_margin), Mm(y), &font);
        y -= line_height;
    }

    y -= line_height;

    layer.use_text("SPENDING ITEMS", 12.0, Mm(left_margin), Mm(y), &font_bold);
    y -= line_height;

    for item in &summary.items {
        if y < 20.0 {
            break;
        }
        let text = format!(
            "  {} - {} - ${:.2} ({})",
            item.spent_on, item.description, item.amount, item.category_label
        );
        layer.use_text(&text, 9.0, Mm(left_margin), Mm(y), &font);
        y -= line_height;
    }

    y -= line_height;

    layer.use_text("SUMMARY", 12.0, Mm(left_margin), Mm(y), &font_bold);
    y -= line_height;

    let total_spent_text = format!("Total Spent: ${:.2}", summary.total_spent);
    layer.use_text(&total_spent_text, 10.0, Mm(left_margin), Mm(y), &font);
    y -= line_height;

    let remaining_text = if summary.remaining >= 0.0 {
        format!("Remaining: ${:.2}", summary.remaining)
    } else {
        format!("Deficit: -${:.2}", summary.remaining.abs())
    };

    layer.use_text(&remaining_text, 10.0, Mm(left_margin), Mm(y), &font_bold);

    let mut buffer = BufWriter::new(Vec::new());
    doc.save(&mut buffer)?;
    Ok(buffer.into_inner()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        FixedExpense, IncomeEntry, ItemWithCategory, Month, MonthlyBudgetWithCategory,
    };
    use chrono::NaiveDate;

    fn create_test_summary() -> MonthSummary {
        MonthSummary {
            month: Month {
                id: 1,
                user_id: 1,
                year: 2024,
                month: 6,
                is_closed: false,
                closed_at: None,
            },
            income_entries: vec![IncomeEntry {
                id: 1,
                month_id: 1,
                label: "Salary".to_string(),
                amount: 5000.0,
            }],
            fixed_expenses: vec![FixedExpense {
                id: 1,
                user_id: 1,
                label: "Rent".to_string(),
                amount: 1500.0,
            }],
            budgets: vec![MonthlyBudgetWithCategory {
                id: 1,
                month_id: 1,
                category_id: 1,
                category_label: "Food".to_string(),
                allocated_amount: 500.0,
                spent_amount: 300.0,
            }],
            items: vec![ItemWithCategory {
                id: 1,
                month_id: 1,
                category_id: 1,
                category_label: "Food".to_string(),
                description: "Groceries".to_string(),
                amount: 150.0,
                spent_on: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            }],
            total_income: 5000.0,
            total_fixed: 1500.0,
            total_budgeted: 500.0,
            total_spent: 300.0,
            remaining: 3200.0,
        }
    }

    #[test]
    fn test_generate_pdf_basic() {
        let summary = create_test_summary();
        let result = generate_pdf(&summary);

        assert!(result.is_ok());
        let pdf_data = result.unwrap();

        // PDF should have content
        assert!(!pdf_data.is_empty());

        // PDF should start with %PDF header
        assert!(pdf_data.starts_with(b"%PDF"));
    }

    #[test]
    fn test_generate_pdf_empty_summary() {
        let summary = MonthSummary {
            month: Month {
                id: 1,
                user_id: 1,
                year: 2024,
                month: 6,
                is_closed: false,
                closed_at: None,
            },
            income_entries: vec![],
            fixed_expenses: vec![],
            budgets: vec![],
            items: vec![],
            total_income: 0.0,
            total_fixed: 0.0,
            total_budgeted: 0.0,
            total_spent: 0.0,
            remaining: 0.0,
        };

        let result = generate_pdf(&summary);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_pdf_with_deficit() {
        let mut summary = create_test_summary();
        summary.remaining = -500.0;

        let result = generate_pdf(&summary);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_pdf_over_budget() {
        let mut summary = create_test_summary();
        summary.budgets[0].spent_amount = 600.0; // Over the 500 allocated

        let result = generate_pdf(&summary);
        assert!(result.is_ok());
    }
}
