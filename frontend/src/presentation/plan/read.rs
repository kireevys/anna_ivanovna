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
        IncomeSource as IncomeSourceCore,
    },
};

use crate::presentation::{
    formatting::{FormattedMoney, FormattedPercentage},
    income::SourceKind,
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
    pub amount: FormattedMoney,
    pub source_kind: SourceKind,
}

impl From<&IncomeSourceCore> for IncomeSource {
    fn from(source: &IncomeSourceCore) -> Self {
        Self {
            id: source.name.clone(), // FIXME: source_id == name
            name: source.name.clone(),
            amount: FormattedMoney::from_money(source.net()),
            source_kind: SourceKind::from(&source.kind),
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
    pub money: FormattedMoney,
    pub rate: FormattedPercentage,
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
                    rate: FormattedPercentage::from_percentage(rate),
                    money: FormattedMoney::from_money(*value),
                    unit: AccountingUnit::Money,
                }
            }
            ExpenseValueCore::RATE { value } => Self {
                money: FormattedMoney::from_money(Money::new_rub(
                    value.apply_to(total_income.value),
                )),
                rate: FormattedPercentage::from_percentage(value.clone()),
                unit: AccountingUnit::Rate,
            },
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum ExpenseKindView {
    Envelope,
    Credit {
        total_amount: FormattedMoney,
        interest_rate: FormattedPercentage,
        term_months: u32,
        monthly_payment: FormattedMoney,
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
                total_amount: FormattedMoney::from_money(credit.total_amount),
                interest_rate: FormattedPercentage::from_percentage(
                    credit.interest_rate.clone(),
                ),
                term_months: credit.term_months,
                monthly_payment: FormattedMoney::from_money(credit.monthly_payment),
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
    pub total_income: FormattedMoney,
    pub total_expenses: FormattedMoney,
    pub balance: FormattedMoney,
    pub categories: BTreeMap<CategoryKey, Vec<Expense>>,
}

impl From<&CorePlan> for Plan {
    fn from(plan: &CorePlan) -> Self {
        // Источники дохода
        let sources: Vec<IncomeSource> =
            plan.sources.iter().map(IncomeSource::from).collect();

        let total_income = FormattedMoney::from_money(plan.total_incomes());
        let total_expenses = FormattedMoney::from_money(plan.total_expenses());
        let balance = FormattedMoney::from_money(plan.balance());

        let income = plan.total_incomes();

        // Группируем расходы по категориям и преобразуем в ViewModel за один проход
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

        // Сортируем расходы внутри каждой категории
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
