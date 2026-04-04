use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use ai_core::{
    distribute::{Budget, Income, distribute as core_dist},
    plan::Plan,
    planning::DistributionWeights,
};

use crate::storage::{
    BudgetId,
    CoreRepo,
    Cursor,
    Page,
    PlanDraft,
    PlanId,
    StorageBudget,
    StoragePlan,
    UserId,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("distribution error: {message}")]
    CantDistribute { message: String },
    #[error("cant save budget")]
    CantSaveBudget,
    #[error("cant create plan")]
    CantCreatePlan,
    #[error("plan already exists")]
    PlanAlreadyExists,
    #[error("invalid plan: {message}")]
    InvalidPlan { message: String },
    #[error("cant delete plan")]
    CantDeletePlan,
    #[error("cant update plan")]
    CantUpdatePlan,
    #[error("plan not found")]
    PlanNotFound,
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
    pub async fn get_plan(&self, user_id: &UserId) -> Option<StoragePlan> {
        self.repo.get_plan(user_id).await
    }

    #[instrument(skip(self, draft))]
    pub async fn create_plan(
        &self,
        user_id: &UserId,
        plan_id: PlanId,
        draft: PlanDraft,
    ) -> Result<PlanId, Error> {
        let plan = Self::validate(draft)?;
        self.repo
            .create_plan(user_id, plan_id, plan)
            .await
            .map_err(|e| match e {
                crate::storage::StorageError::PlanAlreadyExists => {
                    Error::PlanAlreadyExists
                }
                _ => Error::CantCreatePlan,
            })
    }

    #[instrument(skip(self, draft))]
    pub async fn update_plan(
        &self,
        user_id: &UserId,
        plan_id: PlanId,
        draft: PlanDraft,
    ) -> Result<(), Error> {
        let plan = Self::validate(draft)?;
        self.repo
            .update_plan(user_id, &plan_id, plan)
            .await
            .map_err(|_| Error::CantUpdatePlan)
    }

    fn validate(draft: PlanDraft) -> Result<Plan, Error> {
        DistributionWeights::try_from(draft.clone()).map_err(|e| {
            Error::InvalidPlan {
                message: e.to_string(),
            }
        })?;
        Ok(draft)
    }

    #[instrument(skip(self))]
    pub async fn delete_plan(
        &self,
        user_id: &UserId,
        plan_id: PlanId,
    ) -> Result<(), Error> {
        self.repo
            .delete_plan(user_id, &plan_id)
            .await
            .map_err(|_| Error::CantDeletePlan)
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
    pub async fn save_budget(
        &self,
        budget_id: BudgetId,
        budget: Budget,
    ) -> Result<BudgetId, Error> {
        self.repo
            .save_budget(budget_id, budget)
            .await
            .map_err(|_| Error::CantSaveBudget)
    }

    pub async fn budget_list(
        &self,
        from: Option<Cursor>,
        limit: usize,
    ) -> Page<StorageBudget> {
        self.repo.budgets(from, limit).await
    }

    #[instrument(skip(self))]
    pub async fn budget_by_id(&self, id: &BudgetId) -> Option<StorageBudget> {
        self.repo.budget_by_id(id).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use ai_core::{
        distribute::Budget,
        finance::{Money, Percentage},
        plan::Plan,
        planning::{Expense, ExpenseValue, IncomeKind, IncomeSource},
    };
    use rust_decimal_macros::dec;
    use serde::Serialize;

    use super::*;
    use crate::storage::*;

    fn other_source(name: &str, expected: Money) -> IncomeSource {
        IncomeSource::new(name.to_string(), IncomeKind::Other { expected })
    }

    struct InMemoryCoreRepo {
        plan: Mutex<Option<StoragePlan>>,
        events: Mutex<Vec<PlanEvent>>,
    }

    impl InMemoryCoreRepo {
        fn new() -> Self {
            Self {
                plan: Mutex::new(None),
                events: Mutex::new(vec![]),
            }
        }
    }

    impl CoreRepo for InMemoryCoreRepo {
        async fn get_plan(&self, _user_id: &UserId) -> Option<StoragePlan> {
            self.plan.lock().unwrap().clone()
        }

        async fn create_plan(
            &self,
            user_id: &UserId,
            plan_id: PlanId,
            plan: Plan,
        ) -> Result<PlanId, StorageError> {
            if self.plan.lock().unwrap().is_some() {
                return Err(StorageError::PlanAlreadyExists);
            }
            let sp = StoragePlan {
                user_id: user_id.clone(),
                id: plan_id.clone(),
                plan,
                version: 1,
                status: PlanStatus::Active,
            };
            self.events.lock().unwrap().push(PlanEvent {
                id: 1,
                plan_id: plan_id.clone(),
                version: 1,
                action: PlanAction::Created,
                content: Some(sp.plan.clone()),
                created_at: "2026-03-10T00:00:00".into(),
            });
            *self.plan.lock().unwrap() = Some(sp);
            Ok(plan_id)
        }

        async fn update_plan(
            &self,
            _user_id: &UserId,
            plan_id: &PlanId,
            plan: Plan,
        ) -> Result<(), StorageError> {
            let mut current = self.plan.lock().unwrap();
            let sp = current
                .as_mut()
                .filter(|sp| &sp.id == plan_id)
                .ok_or(StorageError::UpdatePlan)?;
            sp.version += 1;
            sp.plan = plan.clone();
            let mut events = self.events.lock().unwrap();
            let next_id = events.len() as i64 + 1;
            events.push(PlanEvent {
                id: next_id,
                plan_id: plan_id.clone(),
                version: sp.version,
                action: PlanAction::Updated,
                content: Some(plan),
                created_at: "2026-03-10T00:00:00".into(),
            });
            Ok(())
        }

        async fn delete_plan(
            &self,
            _user_id: &UserId,
            plan_id: &PlanId,
        ) -> Result<(), StorageError> {
            let mut current = self.plan.lock().unwrap();
            let sp = current
                .as_mut()
                .filter(|sp| &sp.id == plan_id)
                .ok_or(StorageError::DeletePlan)?;
            sp.version += 1;
            sp.status = PlanStatus::Deleted;
            let version = sp.version;
            let mut events = self.events.lock().unwrap();
            let next_id = events.len() as i64 + 1;
            events.push(PlanEvent {
                id: next_id,
                plan_id: plan_id.clone(),
                version,
                action: PlanAction::Deleted,
                content: None,
                created_at: "2026-03-10T00:00:00".into(),
            });
            *current = None;
            Ok(())
        }

        async fn plan_events(
            &self,
            _user_id: &UserId,
            plan_id: &PlanId,
            _from: Option<Cursor>,
            _limit: usize,
        ) -> Page<PlanEvent> {
            let items: Vec<_> = self
                .events
                .lock()
                .unwrap()
                .iter()
                .filter(|e| e.plan_id == *plan_id)
                .cloned()
                .collect();
            Page::new(items, None)
        }

        async fn save_budget(
            &self,
            budget_id: BudgetId,
            _budget: Budget,
        ) -> Result<BudgetId, StorageError> {
            Ok(budget_id)
        }

        async fn budget_by_id(&self, _id: &BudgetId) -> Option<StorageBudget> {
            None
        }

        async fn budgets(
            &self,
            _from: Option<Cursor>,
            _limit: usize,
        ) -> Page<StorageBudget> {
            Page::new(vec![], None)
        }
    }

    fn valid_plan() -> Plan {
        Plan::build(
            &[other_source("Зарплата", Money::new_rub(dec!(100000)))],
            &[
                Expense::envelope(
                    "Аренда".into(),
                    ExpenseValue::MONEY {
                        value: Money::new_rub(dec!(30000)),
                    },
                    Some("Жильё".into()),
                ),
                Expense::envelope(
                    "Накопления".into(),
                    ExpenseValue::RATE {
                        value: Percentage::from_int(20),
                    },
                    None,
                ),
            ],
        )
    }

    #[derive(Serialize)]
    #[serde(tag = "status")]
    enum TestResult {
        Ok { draft: Plan, stored: Plan },
        Err { draft: Plan, error: String },
    }

    const TEST_PLAN_ID: &str = "test-plan-id";
    const TEST_USER_ID: &str = "default";

    fn make_api() -> CoreApi<InMemoryCoreRepo> {
        CoreApi::new(Arc::new(InMemoryCoreRepo::new()))
    }

    #[tokio::test]
    async fn create_plan_ok() {
        let api = make_api();
        let draft = valid_plan();
        api.create_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), draft.clone())
            .await
            .unwrap();
        insta::assert_json_snapshot!(TestResult::Ok {
            draft,
            stored: api.get_plan(&TEST_USER_ID.into()).await.unwrap().plan
        });
    }

    #[tokio::test]
    async fn create_plan_already_exists() {
        let api = make_api();
        api.create_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), valid_plan())
            .await
            .unwrap();
        let err = api
            .create_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), valid_plan())
            .await
            .unwrap_err();
        insta::assert_debug_snapshot!(err);
    }

    #[tokio::test]
    async fn create_plan_invalid_empty() {
        let api = make_api();
        let draft = Plan::default();
        let err = api
            .create_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), draft.clone())
            .await
            .unwrap_err();
        insta::assert_json_snapshot!(TestResult::Err {
            draft,
            error: err.to_string()
        });
    }

    #[tokio::test]
    async fn create_plan_invalid_too_big_expenses() {
        let api = make_api();
        let draft = Plan::build(
            &[other_source("Зарплата", Money::new_rub(dec!(100000)))],
            &[Expense::envelope(
                "Всё".into(),
                ExpenseValue::RATE {
                    value: Percentage::from_int(101),
                },
                None,
            )],
        );
        let err = api
            .create_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), draft.clone())
            .await
            .unwrap_err();
        insta::assert_json_snapshot!(TestResult::Err {
            draft,
            error: err.to_string()
        });
    }

    #[tokio::test]
    async fn update_plan_ok() {
        let api = make_api();
        api.create_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), valid_plan())
            .await
            .unwrap();
        let updated = Plan::build(
            &[other_source("Фриланс", Money::new_rub(dec!(200000)))],
            &[Expense::envelope(
                "Ипотека".into(),
                ExpenseValue::MONEY {
                    value: Money::new_rub(dec!(80000)),
                },
                Some("Жильё".into()),
            )],
        );
        api.update_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), updated.clone())
            .await
            .unwrap();
        insta::assert_json_snapshot!(TestResult::Ok {
            draft: updated,
            stored: api.get_plan(&TEST_USER_ID.into()).await.unwrap().plan
        });
    }

    #[tokio::test]
    async fn update_plan_no_existing() {
        let api = make_api();
        let err = api
            .update_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), valid_plan())
            .await
            .unwrap_err();
        insta::assert_debug_snapshot!(err);
    }

    #[tokio::test]
    async fn update_plan_invalid() {
        let api = make_api();
        api.create_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), valid_plan())
            .await
            .unwrap();
        let draft = Plan::default();
        let err = api
            .update_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), draft.clone())
            .await
            .unwrap_err();
        insta::assert_json_snapshot!(TestResult::Err {
            draft,
            error: err.to_string()
        });
    }

    #[tokio::test]
    async fn update_plan_increments_version() {
        let api = make_api();
        api.create_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), valid_plan())
            .await
            .unwrap();
        assert_eq!(api.get_plan(&TEST_USER_ID.into()).await.unwrap().version, 1);

        let updated = Plan::build(
            &[other_source("Фриланс", Money::new_rub(dec!(200000)))],
            &[Expense::envelope(
                "Ипотека".into(),
                ExpenseValue::MONEY {
                    value: Money::new_rub(dec!(80000)),
                },
                Some("Жильё".into()),
            )],
        );
        api.update_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), updated)
            .await
            .unwrap();
        assert_eq!(api.get_plan(&TEST_USER_ID.into()).await.unwrap().version, 2);
    }

    #[tokio::test]
    async fn delete_plan_ok() {
        let api = make_api();
        api.create_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into(), valid_plan())
            .await
            .unwrap();
        assert!(api.get_plan(&TEST_USER_ID.into()).await.is_some());
        let result = api
            .delete_plan(&TEST_USER_ID.into(), TEST_PLAN_ID.into())
            .await;
        assert!(result.is_ok());
        assert!(api.get_plan(&TEST_USER_ID.into()).await.is_none());
    }

    // events are still stored in the repository for history,
    // but CoreApi does not expose them directly for now
}
