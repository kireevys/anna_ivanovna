use std::collections::BTreeMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use ai_core::{
    finance::{Money, Percentage},
    plan::Plan as CorePlan,
    planning::{
        Expense as ExpenseCore,
        ExpenseKind as CoreExpenseKind,
        ExpenseValue as ExpenseValueCore,
        IncomeKind,
        IncomeSource as IncomeSourceCore,
    },
};

const NO_CATEGORY: &str = "Без категории";

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum CategoryKey {
    NoCategory,
    Named(String),
}

impl CategoryKey {
    pub fn display_name(&self) -> &str {
        match self {
            CategoryKey::NoCategory => NO_CATEGORY,
            CategoryKey::Named(name) => name,
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct IncomeSource {
    pub id: String,
    pub name: String,
    pub amount: Money,
    pub source_kind: IncomeKind,
}

impl From<&IncomeSourceCore> for IncomeSource {
    fn from(source: &IncomeSourceCore) -> Self {
        Self {
            id: source.name.clone(), // FIXME: source_id == name
            name: source.name.clone(),
            amount: source.net(),
            source_kind: source.kind.clone(),
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum AccountingUnit {
    Money,
    Rate,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct ExpenseValue {
    pub money: Money,
    pub rate: Percentage,
    pub unit: AccountingUnit,
}

impl ExpenseValue {
    fn from_core(value: &ExpenseValueCore, total_income: Money) -> Self {
        match value {
            ExpenseValueCore::MONEY { value } => {
                let rate = if total_income.value == Decimal::ZERO {
                    Percentage::ZERO
                } else {
                    Percentage::of(value.value, total_income.value)
                };
                Self {
                    rate,
                    money: *value,
                    unit: AccountingUnit::Money,
                }
            }
            ExpenseValueCore::RATE { value } => Self {
                money: Money::new_rub(value.apply_to(total_income.value)),
                rate: value.clone(),
                unit: AccountingUnit::Rate,
            },
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum ExpenseKindView {
    Envelope,
    Credit {
        total_amount: Money,
        interest_rate: Percentage,
        term_months: u32,
        monthly_payment: Money,
        start_date: NaiveDate,
    },
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct Expense {
    pub name: String,
    pub value: ExpenseValue,
    pub kind: ExpenseKindView,
}

impl Expense {
    fn from_core(expense: &ExpenseCore, total_income: Money) -> Self {
        let kind = match &expense.kind {
            CoreExpenseKind::Envelope { .. } => ExpenseKindView::Envelope,
            CoreExpenseKind::Credit(credit) => ExpenseKindView::Credit {
                total_amount: credit.total_amount,
                interest_rate: credit.interest_rate.clone(),
                term_months: credit.term_months,
                monthly_payment: credit.monthly_payment,
                start_date: credit.start_date,
            },
        };
        Self {
            name: expense.name.clone(),
            value: ExpenseValue::from_core(&expense.value(), total_income),
            kind,
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct Plan {
    pub sources: Vec<IncomeSource>,
    pub total_income: Money,
    pub total_expenses: Money,
    pub balance: Money,
    pub categories: BTreeMap<CategoryKey, Vec<Expense>>,
}

impl From<&CorePlan> for Plan {
    fn from(plan: &CorePlan) -> Self {
        let sources: Vec<IncomeSource> =
            plan.sources.iter().map(IncomeSource::from).collect();

        let total_income = plan.total_incomes();
        let total_expenses = plan.total_expenses();
        let balance = plan.balance();

        let income = plan.total_incomes();

        let mut categories: BTreeMap<CategoryKey, Vec<Expense>> = plan
            .expenses
            .iter()
            .fold(BTreeMap::new(), |mut acc, expense| {
                let key = match &expense.category {
                    None => CategoryKey::NoCategory,
                    Some(name) => CategoryKey::Named(name.clone()),
                };
                acc.entry(key)
                    .or_default()
                    .push(Expense::from_core(expense, income));
                acc
            });

        for expenses in categories.values_mut() {
            expenses.sort_by(|a, b| a.name.cmp(&b.name));
        }

        Self {
            sources,
            total_income,
            total_expenses,
            balance,
            categories,
        }
    }
}
