use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use ai_core::{
    finance::{Money, Percentage},
    plan::Plan as CorePlan,
    planning::{
        CreditExpense,
        Expense as CoreExpense,
        ExpenseKind as CoreExpenseKind,
        ExpenseValue as CoreExpenseValue,
        IncomeKind as CoreIncomeKind,
        IncomeSource as CoreIncomeSource,
    },
};

use crate::presentation::formatting::FormattedPercentage;

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum IncomeKind {
    Salary,
    Other,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct IncomeSource {
    pub name: String,
    pub kind: IncomeKind,
    pub amount: String,
    pub tax_rate: String,
}

impl IncomeSource {
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            kind: IncomeKind::Other,
            amount: String::new(),
            tax_rate: "13".into(),
        }
    }
}

pub fn incomes_from_core_plan(plan: &CorePlan) -> Vec<IncomeSource> {
    plan.sources
        .iter()
        .map(|source| match &source.kind {
            CoreIncomeKind::Salary { gross, tax_rate } => IncomeSource {
                name: source.name.clone(),
                kind: IncomeKind::Salary,
                amount: gross.value.to_string(),
                tax_rate: FormattedPercentage::from(tax_rate.clone()).raw_value(),
            },
            CoreIncomeKind::Other { expected } => IncomeSource {
                name: source.name.clone(),
                kind: IncomeKind::Other,
                amount: expected.value.to_string(),
                tax_rate: "13".into(),
            },
        })
        .collect()
}

fn apply_incomes_to_core_plan(plan: &CorePlan, incomes: &[IncomeSource]) -> CorePlan {
    let mut updated = plan.clone();

    updated.sources = incomes
        .iter()
        .filter_map(|editable| {
            let amount = Decimal::from_str(&editable.amount).ok()?;
            let kind = match editable.kind {
                IncomeKind::Salary => {
                    let rate = Decimal::from_str(&editable.tax_rate).ok()?;
                    CoreIncomeKind::Salary {
                        gross: Money::new_rub(amount),
                        tax_rate: Percentage::from(rate),
                    }
                }
                IncomeKind::Other => CoreIncomeKind::Other {
                    expected: Money::new_rub(amount),
                },
            };
            Some(CoreIncomeSource::new(editable.name.clone(), kind))
        })
        .collect();

    updated
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum ValueKind {
    Money,
    Rate,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum ExpenseType {
    Envelope {
        value_kind: ValueKind,
        amount: String,
    },
    Credit {
        monthly_payment: String,
        total_amount: String,
        interest_rate: String,
        term_months: String,
        start_date: String,
    },
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum ActiveType {
    Envelope,
    Credit,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct EnvelopeData {
    pub value_kind: ValueKind,
    pub amount: String,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct CreditData {
    pub monthly_payment: String,
    pub total_amount: String,
    pub interest_rate: String,
    pub term_months: String,
    pub start_date: String,
}

impl CreditData {
    pub fn validation_errors(&self) -> Vec<&'static str> {
        let mut errors = Vec::new();
        Self::validate_field(
            &self.monthly_payment,
            |v| Decimal::from_str(v).is_ok(),
            "не указан ежемесячный платёж",
            "некорректный ежемесячный платёж",
            &mut errors,
        );
        Self::validate_field(
            &self.total_amount,
            |v| Decimal::from_str(v).is_ok(),
            "не указана сумма кредита",
            "некорректная сумма кредита",
            &mut errors,
        );
        Self::validate_field(
            &self.interest_rate,
            |v| Decimal::from_str(v).is_ok(),
            "не указана ставка",
            "некорректная ставка",
            &mut errors,
        );
        Self::validate_field(
            &self.term_months,
            |v| v.parse::<u32>().is_ok(),
            "не указан срок",
            "некорректный срок",
            &mut errors,
        );
        Self::validate_field(
            &self.start_date,
            |v| NaiveDate::parse_from_str(v, "%Y-%m-%d").is_ok(),
            "не указана дата оформления",
            "некорректная дата оформления",
            &mut errors,
        );
        errors
    }

    fn validate_field(
        value: &str,
        is_valid: impl Fn(&str) -> bool,
        empty_msg: &'static str,
        invalid_msg: &'static str,
        errors: &mut Vec<&'static str>,
    ) {
        if value.is_empty() {
            errors.push(empty_msg);
        } else if !is_valid(value) {
            errors.push(invalid_msg);
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct Expense {
    pub name: String,
    pub category: Option<String>,
    pub active_type: ActiveType,
    pub envelope: EnvelopeData,
    pub credit: CreditData,
}

impl Expense {
    pub fn primary_amount(&self) -> &str {
        match self.active_type {
            ActiveType::Envelope => &self.envelope.amount,
            ActiveType::Credit => &self.credit.monthly_payment,
        }
    }

    pub fn expense_type(&self) -> ExpenseType {
        match self.active_type {
            ActiveType::Envelope => ExpenseType::Envelope {
                value_kind: self.envelope.value_kind,
                amount: self.envelope.amount.clone(),
            },
            ActiveType::Credit => ExpenseType::Credit {
                monthly_payment: self.credit.monthly_payment.clone(),
                total_amount: self.credit.total_amount.clone(),
                interest_rate: self.credit.interest_rate.clone(),
                term_months: self.credit.term_months.clone(),
                start_date: self.credit.start_date.clone(),
            },
        }
    }

    pub fn empty() -> Self {
        Self {
            name: String::new(),
            category: None,
            active_type: ActiveType::Envelope,
            envelope: default_envelope(),
            credit: default_credit(),
        }
    }
}

fn default_envelope() -> EnvelopeData {
    EnvelopeData {
        value_kind: ValueKind::Rate,
        amount: String::new(),
    }
}

fn default_credit() -> CreditData {
    CreditData {
        monthly_payment: String::new(),
        total_amount: String::new(),
        interest_rate: String::new(),
        term_months: String::new(),
        start_date: String::new(),
    }
}

pub fn expenses_from_core_plan(plan: &CorePlan) -> Vec<Expense> {
    plan.expenses
        .iter()
        .map(|expense| match &expense.kind {
            CoreExpenseKind::Envelope { value } => {
                let (value_kind, amount) = match value {
                    CoreExpenseValue::MONEY { value } => {
                        (ValueKind::Money, value.value.to_string())
                    }
                    CoreExpenseValue::RATE { value } => {
                        let raw = value.to_string();
                        let trimmed = raw.trim_end_matches('%').trim().to_string();
                        (ValueKind::Rate, trimmed)
                    }
                };
                Expense {
                    name: expense.name.clone(),
                    category: expense.category.clone(),
                    active_type: ActiveType::Envelope,
                    envelope: EnvelopeData { value_kind, amount },
                    credit: default_credit(),
                }
            }
            CoreExpenseKind::Credit(credit) => Expense {
                name: expense.name.clone(),
                category: expense.category.clone(),
                active_type: ActiveType::Credit,
                envelope: default_envelope(),
                credit: CreditData {
                    monthly_payment: credit.monthly_payment.value.to_string(),
                    total_amount: credit.total_amount.value.to_string(),
                    interest_rate: FormattedPercentage::from_percentage(
                        credit.interest_rate.clone(),
                    )
                    .raw_value(),
                    term_months: credit.term_months.to_string(),
                    start_date: credit.start_date.to_string(),
                },
            },
        })
        .collect()
}

pub fn build_updated_plan(
    base: &CorePlan,
    incomes: &[IncomeSource],
    expenses: &[Expense],
) -> CorePlan {
    let with_incomes = apply_incomes_to_core_plan(base, incomes);
    apply_expenses_to_core_plan(&with_incomes, expenses)
}

fn apply_expenses_to_core_plan(plan: &CorePlan, expenses: &[Expense]) -> CorePlan {
    let mut updated = plan.clone();

    updated.expenses = expenses
        .iter()
        .filter_map(|editable| match &editable.expense_type() {
            ExpenseType::Envelope { value_kind, amount } => {
                let amount = Decimal::from_str(amount).ok()?;
                let value = match value_kind {
                    ValueKind::Money => CoreExpenseValue::MONEY {
                        value: Money::new_rub(amount),
                    },
                    ValueKind::Rate => CoreExpenseValue::RATE {
                        value: Percentage::from(amount),
                    },
                };
                Some(CoreExpense::envelope(
                    editable.name.clone(),
                    value,
                    editable.category.clone(),
                ))
            }
            ExpenseType::Credit {
                monthly_payment,
                total_amount,
                interest_rate,
                term_months,
                start_date,
            } => {
                let monthly = Decimal::from_str(monthly_payment).ok()?;
                let total = Decimal::from_str(total_amount).ok()?;
                let rate = Decimal::from_str(interest_rate).ok()?;
                let months = term_months.parse::<u32>().ok()?;
                let date = NaiveDate::parse_from_str(start_date, "%Y-%m-%d").ok()?;
                let credit = CreditExpense::new(
                    Money::new_rub(monthly),
                    Money::new_rub(total),
                    Percentage::from(rate),
                    months,
                    date,
                )
                .ok()?;
                Some(CoreExpense::credit(
                    editable.name.clone(),
                    credit,
                    editable.category.clone(),
                ))
            }
        })
        .collect();

    updated
}
