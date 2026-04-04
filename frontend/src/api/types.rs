use std::ops::Deref;

use serde::{Deserialize, Serialize};

use ai_core::{distribute::Budget, plan::Plan};

pub type Cursor = String;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Active,
    Deleted,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct StoragePlanFrontend {
    pub id: String,
    pub version: i64,
    pub status: PlanStatus,
    pub plan: Plan,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Tag {
    Recommended,
    Stability,
    Debt,
    Future,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CollectionContent {
    Book { book_url: String, audio_url: String },
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: CollectionContent,
    pub templates: Vec<PlanTemplate>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PlanTemplate {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub situation: String,
    pub tagline: String,
    pub description: String,
    pub tag: Tag,
    pub plan: Plan,
}
