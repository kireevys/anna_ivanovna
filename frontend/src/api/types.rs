use std::ops::Deref;

use ai_core::distribute::Budget;
use serde::Deserialize;

pub type Cursor = String;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BudgetEntry {
    pub id: String,
    pub budget: Budget,
}

#[derive(Debug, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<Cursor>,
}

impl<T> Deref for Page<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}
