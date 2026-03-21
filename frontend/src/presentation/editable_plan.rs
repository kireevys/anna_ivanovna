use std::str::FromStr;

use rust_decimal::Decimal;

use ai_core::{
    finance::{Money, Percentage},
    plan::Plan as CorePlan,
    planning::{
        Expense as CoreExpense,
        ExpenseValue as CoreExpenseValue,
        IncomeKind as CoreIncomeKind,
        IncomeSource as CoreIncomeSource,
    },
};

use crate::presentation::formatting::FormattedPercentage;

#[derive(Clone, Copy, PartialEq)]
pub enum EditableIncomeKind {
    Salary,
    Other,
}

#[derive(Clone, PartialEq)]
pub struct EditableIncomeSource {
    pub name: String,
    pub kind: EditableIncomeKind,
    pub amount: String,
    pub tax_rate: String,
    pub is_valid: bool,
}

impl EditableIncomeSource {
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            kind: EditableIncomeKind::Other,
            amount: String::new(),
            tax_rate: "13".into(),
            is_valid: true,
        }
    }
}

pub fn incomes_from_core_plan(plan: &CorePlan) -> Vec<EditableIncomeSource> {
    plan.sources
        .iter()
        .map(|source| match &source.kind {
            CoreIncomeKind::Salary { gross, tax_rate } => EditableIncomeSource {
                name: source.name.clone(),
                kind: EditableIncomeKind::Salary,
                amount: gross.value.to_string(),
                tax_rate: FormattedPercentage::from(tax_rate.clone()).raw_value(),
                is_valid: true,
            },
            CoreIncomeKind::Other { expected } => EditableIncomeSource {
                name: source.name.clone(),
                kind: EditableIncomeKind::Other,
                amount: expected.value.to_string(),
                tax_rate: "13".into(),
                is_valid: true,
            },
        })
        .collect()
}

pub fn apply_incomes_to_core_plan(
    plan: &CorePlan,
    incomes: &[EditableIncomeSource],
) -> CorePlan {
    let mut updated = plan.clone();

    updated.sources = incomes
        .iter()
        .filter_map(|editable| {
            let amount = Decimal::from_str(&editable.amount).ok()?;
            let kind = match editable.kind {
                EditableIncomeKind::Salary => {
                    let rate = Decimal::from_str(&editable.tax_rate).ok()?;
                    CoreIncomeKind::Salary {
                        gross: Money::new_rub(amount),
                        tax_rate: Percentage::from(rate),
                    }
                }
                EditableIncomeKind::Other => CoreIncomeKind::Other {
                    expected: Money::new_rub(amount),
                },
            };
            Some(CoreIncomeSource::new(editable.name.clone(), kind))
        })
        .collect();

    updated
}

#[derive(Clone, Copy, PartialEq)]
pub enum EditableExpenseKind {
    Money,
    Rate,
}

#[derive(Clone, PartialEq)]
pub struct EditableExpense {
    pub name: String,
    pub category: Option<String>,
    pub kind: EditableExpenseKind,
    pub amount: String,
    pub is_valid: bool,
}

impl EditableExpense {
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            category: None,
            kind: EditableExpenseKind::Rate,
            amount: String::new(),
            is_valid: true,
        }
    }
}
pub fn expenses_from_core_plan(plan: &CorePlan) -> Vec<EditableExpense> {
    plan.expenses
        .iter()
        .map(|expense| {
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

    updated.expenses = expenses
        .iter()
        .filter_map(|editable| {
            let amount = Decimal::from_str(&editable.amount).ok()?;
            let value = match editable.kind {
                EditableExpenseKind::Money => CoreExpenseValue::MONEY {
                    value: Money::new_rub(amount),
                },
                EditableExpenseKind::Rate => CoreExpenseValue::RATE {
                    value: Percentage::from(amount),
                },
            };
            Some(CoreExpense::new(
                editable.name.clone(),
                value,
                editable.category.clone(),
            ))
        })
        .collect();

    updated
}
