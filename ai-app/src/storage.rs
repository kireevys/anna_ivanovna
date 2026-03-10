use std::ops::Deref;

use ai_core::{distribute::Budget, plan::Plan};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type PlanId = String;
pub type BudgetId = String;
pub type Cursor = String;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to save plan")]
    SavePlan,
    #[error("failed to save budget")]
    SaveBudget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageBudget {
    pub id: BudgetId,
    pub budget: Budget,
}

impl From<(BudgetId, Budget)> for StorageBudget {
    fn from(value: (BudgetId, Budget)) -> Self {
        Self {
            id: value.0,
            budget: value.1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

impl<T> Page<T> {
    pub fn new(items: Vec<T>, next_cursor: Option<Cursor>) -> Self {
        Self { items, next_cursor }
    }
}

pub trait CoreRepo {
    fn get_plan(&self) -> Option<Plan>;
    fn save_plan(&self, plan_id: PlanId, plan: Plan) -> Result<PlanId, StorageError>;
    fn save_budget(
        &self,
        budget_id: BudgetId,
        budget: Budget,
    ) -> Result<BudgetId, StorageError>;
    fn budget_by_id(&self, id: &BudgetId) -> Option<StorageBudget>;
    fn budgets(&self, from: Option<Cursor>, limit: usize) -> Page<StorageBudget>;
}
