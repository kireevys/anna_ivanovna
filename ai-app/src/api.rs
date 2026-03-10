use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use ai_core::{
    distribute::{Budget, Income, distribute as core_dist},
    plan::Plan,
    planning::DistributionWeights,
};

use crate::storage::{BudgetId, CoreRepo, Cursor, Page, PlanId, StorageBudget};

#[derive(Debug, Error)]
pub enum Error {
    #[error("distribution error: {message}")]
    CantDistribute { message: String },
    #[error("cant save budget")]
    CantSaveBudget,
    #[error("cant save plan")]
    CantSavePlan,
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

    #[instrument(skip(self, plan))]
    pub fn save_plan(&self, plan_id: PlanId, plan: Plan) -> Result<PlanId, Error> {
        self.repo
            .save_plan(plan_id, plan)
            .map_err(|_| Error::CantSavePlan)
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

    #[must_use]
    pub fn build_budget_id() -> BudgetId {
        uuid::Uuid::now_v7().to_string()
    }
}
