use crate::presentation::formatting::{FormattedMoney, FormattedPercentage};
use ai_core::{
    plan::Plan as CorePlan,
    planning::{
        Expense as ExpenseCore,
        ExpenseValue as ExpenseValueCore,
        IncomeSource as IncomeSourceCore,
    },
};
use std::{collections::BTreeMap, fmt};

const NO_CATEGORY: &str = "Без категории";

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, PartialEq)]
pub struct IncomeSource {
    pub id: String,
    pub name: String,
    pub amount: FormattedMoney,
}

impl From<&IncomeSourceCore> for IncomeSource {
    fn from(source: &IncomeSourceCore) -> Self {
        Self {
            id: source.name.clone(), // FIXME: source_id == name
            name: source.name.clone(),
            amount: FormattedMoney::from_money(source.expected),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ExpenseValue {
    Money(FormattedMoney),
    Percentage(FormattedPercentage),
}

impl fmt::Display for ExpenseValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpenseValue::Money(money) => write!(f, "{}", money),
            ExpenseValue::Percentage(percentage) => write!(f, "{}", percentage),
        }
    }
}

impl From<&ExpenseValueCore> for ExpenseValue {
    fn from(value: &ExpenseValueCore) -> Self {
        match value {
            ExpenseValueCore::MONEY { value } => {
                ExpenseValue::Money(FormattedMoney::from_money(*value))
            }
            ExpenseValueCore::RATE { value } => ExpenseValue::Percentage(
                FormattedPercentage::from_percentage(value.clone()),
            ),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Expense {
    pub name: String,
    pub value: ExpenseValue,
}

impl From<&ExpenseCore> for Expense {
    fn from(expense: &ExpenseCore) -> Self {
        Self {
            name: expense.name.clone(),
            value: ExpenseValue::from(&expense.value),
        }
    }
}

#[derive(Clone, PartialEq)]
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

        // Группируем расходы по категориям и преобразуем в ViewModel за один проход
        let mut categories: BTreeMap<CategoryKey, Vec<Expense>> = plan
            .expenses
            .iter()
            .fold(BTreeMap::new(), |mut acc, expense| {
                let key = match &expense.category {
                    None => CategoryKey::NoCategory,
                    Some(name) => CategoryKey::Named(name.clone()),
                };
                acc.entry(key).or_default().push(Expense::from(expense));
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
