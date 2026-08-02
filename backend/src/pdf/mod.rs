use printpdf::*;

use crate::models::MonthSummary;

fn push_text_line(
    ops: &mut Vec<Op>,
    text: impl Into<String>,
    size: f32,
    x: f32,
    y: f32,
    font: BuiltinFont,
) {
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(x), Mm(y)),
    });
    ops.push(Op::SetFont {
        font: PdfFontHandle::Builtin(font),
        size: Pt(size),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(text.into())],
    });
    ops.push(Op::EndTextSection);
}

pub fn generate_pdf(summary: &MonthSummary) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let title = format!(
        "Financial Summary - {}/{}",
        summary.month.month, summary.month.year
    );
    let mut doc = PdfDocument::new(&title);
    let mut ops = Vec::new();

    let mut y = 270.0;
    let left_margin = 20.0;
    let line_height = 6.0;

    push_text_line(
        &mut ops,
        &title,
        16.0,
        left_margin,
        y,
        BuiltinFont::HelveticaBold,
    );
    y -= line_height * 2.0;

    push_text_line(
        &mut ops,
        "INCOME",
        12.0,
        left_margin,
        y,
        BuiltinFont::HelveticaBold,
    );
    y -= line_height;

    for entry in &summary.income_entries {
        let text = format!("  {} - ${:.2}", entry.label, entry.amount);
        push_text_line(&mut ops, text, 10.0, left_margin, y, BuiltinFont::Helvetica);
        y -= line_height;
    }

    let total_income_text = format!("Total Income: ${:.2}", summary.total_income);
    push_text_line(
        &mut ops,
        total_income_text,
        10.0,
        left_margin,
        y,
        BuiltinFont::HelveticaBold,
    );
    y -= line_height * 2.0;

    push_text_line(
        &mut ops,
        "FIXED EXPENSES",
        12.0,
        left_margin,
        y,
        BuiltinFont::HelveticaBold,
    );
    y -= line_height;

    for expense in &summary.fixed_expenses {
        let text = format!("  {} - ${:.2}", expense.label, expense.amount);
        push_text_line(&mut ops, text, 10.0, left_margin, y, BuiltinFont::Helvetica);
        y -= line_height;
    }

    let total_fixed_text = format!("Total Fixed: ${:.2}", summary.total_fixed);
    push_text_line(
        &mut ops,
        total_fixed_text,
        10.0,
        left_margin,
        y,
        BuiltinFont::HelveticaBold,
    );
    y -= line_height * 2.0;

    push_text_line(
        &mut ops,
        "BUDGET VS ACTUAL",
        12.0,
        left_margin,
        y,
        BuiltinFont::HelveticaBold,
    );
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
        push_text_line(&mut ops, text, 10.0, left_margin, y, BuiltinFont::Helvetica);
        y -= line_height;
    }

    y -= line_height;

    push_text_line(
        &mut ops,
        "SPENDING ITEMS",
        12.0,
        left_margin,
        y,
        BuiltinFont::HelveticaBold,
    );
    y -= line_height;

    for item in &summary.items {
        if y < 20.0 {
            break;
        }
        let text = format!(
            "  {} - {} - ${:.2} ({})",
            item.spent_on,
            item.description,
            item.amount,
            item.category_label.as_deref().unwrap_or("Uncategorized")
        );
        push_text_line(&mut ops, text, 9.0, left_margin, y, BuiltinFont::Helvetica);
        y -= line_height;
    }

    y -= line_height;

    push_text_line(
        &mut ops,
        "SUMMARY",
        12.0,
        left_margin,
        y,
        BuiltinFont::HelveticaBold,
    );
    y -= line_height;

    let total_spent_text = format!("Total Spent: ${:.2}", summary.total_spent);
    push_text_line(
        &mut ops,
        total_spent_text,
        10.0,
        left_margin,
        y,
        BuiltinFont::Helvetica,
    );
    y -= line_height;

    let remaining_text = if summary.remaining >= 0.0 {
        format!("Remaining: ${:.2}", summary.remaining)
    } else {
        format!("Deficit: -${:.2}", summary.remaining.abs())
    };

    push_text_line(
        &mut ops,
        remaining_text,
        10.0,
        left_margin,
        y,
        BuiltinFont::HelveticaBold,
    );

    let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);
    let mut warnings = Vec::new();
    let pdf_data = doc
        .with_pages(vec![page])
        .save(&PdfSaveOptions::default(), &mut warnings);

    Ok(pdf_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        IncomeEntry, ItemWithCategory, Month, MonthlyBudgetWithCategory, MonthlyFixedExpense,
        MonthlySavings,
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
                paid_on: None,
            }],
            fixed_expenses: vec![MonthlyFixedExpense {
                id: 1,
                month_id: 1,
                label: "Rent".to_string(),
                amount: 1500.0,
            }],
            budgets: vec![MonthlyBudgetWithCategory {
                id: 1,
                month_id: 1,
                category_id: 1,
                category_label: "Food".to_string(),
                category_color: "#71717a".to_string(),
                allocated_amount: 500.0,
                spent_amount: 300.0,
            }],
            items: vec![ItemWithCategory {
                id: 1,
                month_id: 1,
                category_id: Some(1),
                category_label: Some("Food".to_string()),
                category_color: Some("#71717a".to_string()),
                description: "Groceries".to_string(),
                amount: 150.0,
                spent_on: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                savings_destination: "none".to_string(),
            }],
            savings: Some(MonthlySavings {
                id: 1,
                month_id: 1,
                savings: 10000.0,
                retirement_savings: 50000.0,
                savings_goal: 20000.0,
            }),
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

        assert!(!pdf_data.is_empty());

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
            savings: None,
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
        summary.budgets[0].spent_amount = 600.0;

        let result = generate_pdf(&summary);
        assert!(result.is_ok());
    }
}
