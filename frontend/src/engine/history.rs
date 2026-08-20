use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use ai_core::{
    distribute::BudgetEntry as CoreBudgetEntry,
    finance::Money,
    planning::IncomeKind,
};

use crate::{
    api::{BudgetEntry, Cursor, Page},
    engine::{
        category::CategoryKey,
        core::{Model, PageStatus, PaginatedList},
    },
};

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
    pub key: CategoryKey,
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

        let mut grouped: BTreeMap<CategoryKey, Vec<ExpenseEntry>> = BTreeMap::new();

        if !budget.no_category.is_empty() {
            grouped
                .entry(CategoryKey::NoCategory)
                .or_default()
                .extend(budget.no_category.iter().map(ExpenseEntry::from));
        }

        for (category_name, entries) in &budget.categories {
            grouped
                .entry(CategoryKey::named(category_name))
                .or_default()
                .extend(entries.iter().map(ExpenseEntry::from));
        }

        let categories = grouped
            .into_iter()
            .map(|(key, mut entries)| {
                entries.sort_by(|a, b| a.name.cmp(&b.name));
                Category { key, entries }
            })
            .collect();

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

impl From<&CoreBudgetEntry> for ExpenseEntry {
    fn from(entry: &CoreBudgetEntry) -> Self {
        Self {
            name: entry.expense.name.clone(),
            amount: entry.amount,
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
