use std::{ops::Deref, sync::Arc};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::instrument;

use crate::{
    distribute::{Budget, Income, distribute as core_dist},
    plan::Plan,
    planning::DistributionWeights,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("distribution error")]
    CantDistribute { message: String },
    #[error("cant save budget")]
    CantSaveBudget,
}

pub type PlanId = String;
pub type BudgetId = String;

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

pub trait CoreRepo {
    fn location(&self) -> &str;
    fn get_plan(&self) -> Option<Plan>;
    fn save_budget(
        &self,
        budget_id: BudgetId,
        budget: Budget,
    ) -> Result<BudgetId, Error>;
    fn budget_by_id(&self, id: &BudgetId) -> Option<StorageBudget>;
    fn budgets(&self, from: Option<Cursor>, limit: usize) -> Page<StorageBudget>;
    fn build_budget_id(b: &Budget) -> BudgetId {
        format!(
            "{}-{}.json",
            b.income_date().format("%Y-%m-%d"),
            b.income.source.name
        )
    }
}

pub type Cursor = String;

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

#[derive(Clone)]
pub struct CoreApi<R: CoreRepo> {
    repo: Arc<R>,
}

impl<R: CoreRepo> CoreApi<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub fn get_plan(&self) -> Option<Plan> {
        self.repo.get_plan()
    }

    #[instrument(skip(plan, income, self))]
    pub fn distribute(
        &self,
        plan: &DistributionWeights,
        income: &Income,
    ) -> Result<Budget, Error> {
        core_dist(plan, income).map_err(|e| Error::CantDistribute {
            message: e.to_string(),
        })
    }

    #[instrument(skip(budget, self))]
    pub fn save_budget(
        &self,
        budget_id: BudgetId,
        budget: Budget,
    ) -> Result<BudgetId, Error> {
        self.repo
            .save_budget(budget_id, budget)
            .map_err(|_| Error::CantSaveBudget)
    }

    pub fn budget_list(
        &self,
        from: Option<Cursor>,
        limit: usize,
    ) -> Page<StorageBudget> {
        self.repo.budgets(from, limit)
    }

    #[instrument(skip(self))]
    pub fn budget_by_id(&self, id: &BudgetId) -> Option<StorageBudget> {
        self.repo.budget_by_id(id)
    }

    pub fn build_budget_id(b: &Budget) -> BudgetId {
        format!(
            "{}-{}.json",
            b.income_date().format("%Y-%m-%d"),
            b.income.source.name
        )
    }

    pub fn location(&self) -> &str {
        self.repo.location()
    }
}
