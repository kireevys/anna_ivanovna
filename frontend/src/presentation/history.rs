use std::collections::HashMap;

use ai_core::{finance::Money, planning::IncomeKind};

use crate::{
    api::BudgetEntry,
    presentation::formatting::{FormattedMoney, FormattedPercentage},
};

const NO_CATEGORY: &str = "Без категории";

#[derive(Clone, PartialEq)]
pub enum TaxInfo {
    Salary {
        gross: FormattedMoney,
        tax_rate: String,
        tax_amount: FormattedMoney,
    },
    NoTax,
}

#[derive(Clone, PartialEq)]
pub struct HistoryEntry {
    pub id: String,
    pub date: String,
    pub source_name: String,
    pub kind_label: String,
    pub income_amount: FormattedMoney,
    pub tax_info: TaxInfo,
    pub rest: FormattedMoney,
    pub categories: Vec<Category>,
}

#[derive(Clone, PartialEq)]
pub struct Category {
    pub name: String,
    pub entries: Vec<ExpenseEntry>,
}

#[derive(Clone, PartialEq)]
pub struct ExpenseEntry {
    pub name: String,
    pub amount: FormattedMoney,
}

impl From<&BudgetEntry> for HistoryEntry {
    fn from(storage_budget: &BudgetEntry) -> Self {
        let budget = &storage_budget.budget;

        // Дата и источник дохода
        let date = budget.income.date.format("%Y-%m-%d").to_string();
        let source_name = budget.income.source.name.clone();
        let income_amount = FormattedMoney::from_money(budget.income.amount);
        let rest = FormattedMoney::from_money(budget.rest);

        let (kind_label, tax_info) = match &budget.income.source.kind {
            IncomeKind::Salary { gross, tax_rate } => {
                let tax = gross.value - budget.income.source.net().value;
                (
                    "Зарплата".to_string(),
                    TaxInfo::Salary {
                        gross: FormattedMoney::from_money(*gross),
                        tax_rate: FormattedPercentage::from(tax_rate.clone())
                            .raw_value(),
                        tax_amount: FormattedMoney::from_money(Money::new(
                            tax,
                            gross.currency,
                        )),
                    },
                )
            }
            IncomeKind::Other { .. } => ("Другое".to_string(), TaxInfo::NoTax),
        };

        // Группируем расходы по категориям
        let mut categories_map: HashMap<String, Vec<ExpenseEntry>> = HashMap::new();

        // Расходы без категории
        if !budget.no_category.is_empty() {
            let entries: Vec<ExpenseEntry> = budget
                .no_category
                .iter()
                .map(|entry| ExpenseEntry {
                    name: entry.expense.name.clone(),
                    amount: FormattedMoney::from_money(entry.amount),
                })
                .collect();
            categories_map.insert(NO_CATEGORY.to_string(), entries);
        }

        // Расходы по категориям
        for (category_name, entries) in &budget.categories {
            let expense_entries: Vec<ExpenseEntry> = entries
                .iter()
                .map(|entry| ExpenseEntry {
                    name: entry.expense.name.clone(),
                    amount: FormattedMoney::from_money(entry.amount),
                })
                .collect();
            categories_map.insert(category_name.clone(), expense_entries);
        }

        // Преобразуем в Vec<Category> и сортируем
        let mut categories: Vec<Category> = categories_map
            .iter()
            .map(|(name, entries)| Category {
                name: name.clone(),
                entries: {
                    let mut sorted = entries.clone();
                    sorted.sort_by(|a, b| a.name.cmp(&b.name));
                    sorted
                },
            })
            .collect();

        // Сортируем категории (сначала "Без категории", потом по алфавиту)
        categories.sort_by(|a, b| match (a.name.as_str(), b.name.as_str()) {
            (NO_CATEGORY, _) => std::cmp::Ordering::Less,
            (_, NO_CATEGORY) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Self {
            id: storage_budget.id.clone(),
            date,
            source_name,
            kind_label,
            income_amount,
            tax_info,
            rest,
            categories,
        }
    }
}
