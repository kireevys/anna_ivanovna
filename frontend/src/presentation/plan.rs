use crate::presentation::formatting::{format_decimal, format_money};
use ai_core::plan::Plan as CorePlan;
use ai_core::planning::{Expense as ExpenseCore, ExpenseValue as ExpenseValueCore};
use std::collections::HashMap;
const NO_CATEGORY: &str = "Без категории";

#[derive(Clone, PartialEq)]
pub struct IncomeSource {
    pub id: String,
    pub name: String,
    pub amount: String,
}

#[derive(Clone, PartialEq)]
pub struct Expense {
    pub name: String,
    pub value: String,
}

#[derive(Clone, PartialEq)]
pub struct Category {
    pub name: String,
    pub expenses: Vec<Expense>,
}

#[derive(Clone, PartialEq)]
pub struct Plan {
    pub sources: Vec<IncomeSource>,
    pub total_income: String,
    pub total_expenses: String,
    pub balance: String,
    pub categories: Vec<Category>,
}

impl From<&CorePlan> for Plan {
    fn from(plan: &CorePlan) -> Self {
        // Источники дохода
        let sources: Vec<IncomeSource> = plan
            .sources
            .iter()
            .map(|source| IncomeSource {
                id: source.name.clone(),
                name: source.name.clone(),
                amount: format_money(&source.expected),
            })
            .collect();

        // Общий доход
        let total_income_decimal: rust_decimal::Decimal =
            plan.sources.iter().map(|s| s.expected.value).sum();
        let total_income = format!("₽{}", format_decimal(&total_income_decimal));

        // Общие расходы
        let total_expenses_decimal: rust_decimal::Decimal = plan
            .expenses
            .iter()
            .map(|expense| match &expense.value {
                ExpenseValueCore::MONEY { value } => value.value,
                ExpenseValueCore::RATE { value } => value.apply_to(total_income_decimal),
            })
            .sum();
        let total_expenses = format!("₽{}", format_decimal(&total_expenses_decimal));

        // Остаток
        let balance_decimal = total_income_decimal - total_expenses_decimal;
        let balance = format!("₽{}", format_decimal(&balance_decimal));

        // Группируем расходы по категориям
        let expenses_by_category: HashMap<Option<String>, Vec<&ExpenseCore>> =
            plan.expenses.iter().fold(HashMap::new(), |mut s, e| {
                s.entry(e.category.clone()).or_default().push(e);
                s
            });

        // Преобразуем в ViewModel
        let mut categories: Vec<Category> = expenses_by_category
            .iter()
            .map(|(category, expenses)| {
                let name = category
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or(NO_CATEGORY)
                    .to_string();

                let mut expenses: Vec<Expense> = expenses
                    .iter()
                    .map(|expense| Expense {
                        name: expense.name.clone(),
                        value: match &expense.value {
                            ExpenseValueCore::MONEY { value } => format_money(value),
                            ExpenseValueCore::RATE { value } => value.to_string(),
                        },
                    })
                    .collect();

                expenses.sort_by(|a, b| a.name.cmp(&b.name));
                Category { name, expenses }
            })
            .collect();

        // Сортируем категории (сначала "Без категории", потом по алфавиту)
        categories.sort_by(|a, b| match (a.name.as_str(), b.name.as_str()) {
            (NO_CATEGORY, _) => std::cmp::Ordering::Less,
            (_, NO_CATEGORY) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Self {
            sources,
            total_income,
            total_expenses,
            balance,
            categories,
        }
    }
}
