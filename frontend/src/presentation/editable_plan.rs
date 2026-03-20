use ai_core::{
    finance::{Money, Percentage},
    plan::Plan as CorePlan,
    planning::{Expense as CoreExpense, ExpenseValue as CoreExpenseValue},
};
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Clone, PartialEq)]
pub struct EditableIncomeSource {
    pub index: usize,
    pub name: String,
    pub amount: String,
    pub is_valid: bool,
}

pub fn incomes_from_core_plan(plan: &CorePlan) -> Vec<EditableIncomeSource> {
    plan.sources
        .iter()
        .enumerate()
        .map(|(index, source)| EditableIncomeSource {
            index,
            name: source.name.clone(),
            amount: source.expected.value.to_string(),
            is_valid: true,
        })
        .collect()
}

pub fn apply_incomes_to_core_plan(
    plan: &CorePlan,
    incomes: &[EditableIncomeSource],
) -> CorePlan {
    let mut updated = plan.clone();

    for editable in incomes {
        if editable.index >= updated.sources.len() {
            continue;
        }

        if let Ok(amount) = Decimal::from_str(&editable.amount) {
            updated.sources[editable.index].expected = Money::new_rub(amount);
        }
    }

    updated
}

#[derive(Clone, Copy, PartialEq)]
pub enum EditableExpenseKind {
    Money,
    Rate,
}

#[derive(Clone, PartialEq)]
pub struct EditableExpense {
    pub index: usize,
    pub name: String,
    pub category: Option<String>,
    pub kind: EditableExpenseKind,
    pub amount: String,
    pub is_valid: bool,
}
pub fn expenses_from_core_plan(plan: &CorePlan) -> Vec<EditableExpense> {
    plan.expenses
        .iter()
        .enumerate()
        .map(|(index, expense)| {
            let (kind, amount) = match &expense.value {
                CoreExpenseValue::MONEY { value } => {
                    (EditableExpenseKind::Money, value.value.to_string())
                }
                CoreExpenseValue::RATE { value } => {
                    let raw = value.to_string();
                    let trimmed = raw.trim_end_matches('%').to_string();
                    (EditableExpenseKind::Rate, trimmed)
                }
            };
            EditableExpense {
                index,
                name: expense.name.clone(),
                category: expense.category.clone(),
                kind,
                amount,
                is_valid: true,
            }
        })
        .collect()
}
pub fn build_updated_plan(
    base: &CorePlan,
    incomes: &[EditableIncomeSource],
    expenses: &[EditableExpense],
) -> CorePlan {
    let with_incomes = apply_incomes_to_core_plan(base, incomes);
    apply_expenses_to_core_plan(&with_incomes, expenses)
}

pub fn apply_expenses_to_core_plan(
    plan: &CorePlan,
    expenses: &[EditableExpense],
) -> CorePlan {
    let mut updated = plan.clone();

    for editable in expenses {
        if editable.index >= updated.expenses.len() {
            continue;
        }

        if let Ok(amount) = Decimal::from_str(&editable.amount) {
            let core_expense: &mut CoreExpense = &mut updated.expenses[editable.index];

            core_expense.value = match editable.kind {
                EditableExpenseKind::Money => CoreExpenseValue::MONEY {
                    value: Money::new_rub(amount),
                },
                EditableExpenseKind::Rate => CoreExpenseValue::RATE {
                    value: Percentage::from(amount),
                },
            };
        }
    }

    updated
}
