use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use ai_core::{finance::Money, planning::IncomeKind};

use crate::{
    api::{BudgetEntry, Cursor, Page},
    engine::core::{Model, PageStatus, PaginatedList},
};

pub const NO_CATEGORY: &str = "Без категории";

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct HistoryEntry {
    pub id: String,
    pub date: NaiveDate,
    pub source_name: String,
    pub source_kind: IncomeKind,
    pub income_amount: Money,
    pub rest: Money,
    pub categories: Vec<Category>,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct Category {
    pub name: String,
    pub entries: Vec<ExpenseEntry>,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct ExpenseEntry {
    pub name: String,
    pub amount: Money,
}

impl From<&BudgetEntry> for HistoryEntry {
    fn from(storage_budget: &BudgetEntry) -> Self {
        let budget = &storage_budget.budget;

        let mut categories_map: HashMap<String, Vec<ExpenseEntry>> = HashMap::new();

        if !budget.no_category.is_empty() {
            let entries: Vec<ExpenseEntry> = budget
                .no_category
                .iter()
                .map(|entry| ExpenseEntry {
                    name: entry.expense.name.clone(),
                    amount: entry.amount,
                })
                .collect();
            categories_map.insert(NO_CATEGORY.to_string(), entries);
        }

        for (category_name, entries) in &budget.categories {
            let expense_entries: Vec<ExpenseEntry> = entries
                .iter()
                .map(|entry| ExpenseEntry {
                    name: entry.expense.name.clone(),
                    amount: entry.amount,
                })
                .collect();
            categories_map.insert(category_name.clone(), expense_entries);
        }

        let mut categories: Vec<Category> = categories_map
            .into_iter()
            .map(|(name, mut entries)| {
                entries.sort_by(|a, b| a.name.cmp(&b.name));
                Category { name, entries }
            })
            .collect();

        categories.sort_by(|a, b| match (a.name.as_str(), b.name.as_str()) {
            (NO_CATEGORY, _) => std::cmp::Ordering::Less,
            (_, NO_CATEGORY) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Self {
            id: storage_budget.id.clone(),
            date: budget.income.date,
            source_name: budget.income.source.name.clone(),
            source_kind: budget.income.source.kind.clone(),
            income_amount: budget.income.amount,
            rest: budget.rest,
            categories,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct HistoryModel {
    pub(crate) data: PaginatedList<HistoryEntry>,
}

#[derive(Deserialize, Serialize)]
pub enum Msg {
    Load,
    Loaded(Result<Page<BudgetEntry>, String>),
}

#[derive(Serialize)]
pub enum Cmd {
    Fetch { cursor: Option<Cursor> },
}

impl Model for HistoryModel {
    type Msg = Msg;
    type Cmd = Cmd;

    fn handle(self, msg: Self::Msg) -> (Self, Vec<Self::Cmd>) {
        match msg {
            Msg::Load => {
                let cursor = self.data.next_cursor.clone();
                let new = HistoryModel {
                    data: PaginatedList {
                        status: PageStatus::Loading,
                        ..self.data.clone()
                    },
                };
                (new, vec![Cmd::Fetch { cursor }])
            }
            Msg::Loaded(result) => match result {
                Ok(page) => {
                    let new_entries: Vec<HistoryEntry> =
                        page.items.iter().map(HistoryEntry::from).collect();
                    let mut items = self.data.items.clone();
                    items.extend(new_entries);
                    (
                        HistoryModel {
                            data: PaginatedList {
                                items,
                                next_cursor: page.next_cursor,
                                status: PageStatus::Idle,
                            },
                        },
                        vec![],
                    )
                }
                Err(e) => (
                    HistoryModel {
                        data: PaginatedList {
                            status: PageStatus::Error(e),
                            ..self.data.clone()
                        },
                    },
                    vec![],
                ),
            },
        }
    }
}
