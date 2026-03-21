use std::{ops::Deref, str::FromStr};

use ai_core::{distribute::Budget, plan::Plan};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type PlanDraft = Plan;
pub type PlanId = String;
pub type UserId = String;
pub type BudgetId = String;
pub type Cursor = String;
pub type Version = i64;

#[must_use]
pub fn build_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to create plan")]
    CreatePlan,
    #[error("plan already exists")]
    PlanAlreadyExists,
    #[error("failed to update plan")]
    UpdatePlan,
    #[error("failed to delete plan")]
    DeletePlan,
    #[error("failed to save budget")]
    SaveBudget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Active,
    Deleted,
}

impl PlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlanStatus::Active => "active",
            PlanStatus::Deleted => "deleted",
        }
    }
}

impl FromStr for PlanStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(PlanStatus::Active),
            "deleted" => Ok(PlanStatus::Deleted),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanAction {
    Created,
    Updated,
    Deleted,
}

impl PlanAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlanAction::Created => "created",
            PlanAction::Updated => "updated",
            PlanAction::Deleted => "deleted",
        }
    }
}

impl FromStr for PlanAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created" => Ok(PlanAction::Created),
            "updated" => Ok(PlanAction::Updated),
            "deleted" => Ok(PlanAction::Deleted),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePlan {
    pub user_id: UserId,
    pub id: PlanId,
    pub plan: Plan,
    pub version: Version,
    pub status: PlanStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanEvent {
    pub id: i64,
    pub plan_id: PlanId,
    pub version: Version,
    pub action: PlanAction,
    pub content: Option<Plan>,
    pub created_at: String,
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
pub trait CoreRepo: Send + Sync {
    /// Возвращает активный план пользователя
    fn get_plan(
        &self,
        user_id: &UserId,
    ) -> impl Future<Output = Option<StoragePlan>> + Send;

    /// Создаёт новый план для пользователя с указанным внешним идентификатором.
    fn create_plan(
        &self,
        user_id: &UserId,
        plan_id: PlanId,
        plan: Plan,
    ) -> impl Future<Output = Result<PlanId, StorageError>> + Send;

    /// Обновляет указанный план пользователя и добавляет событие об изменении.
    fn update_plan(
        &self,
        user_id: &UserId,
        plan_id: &PlanId,
        plan: Plan,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    /// Помечает план пользователя как удалённый (soft delete) и добавляет событие.
    fn delete_plan(
        &self,
        user_id: &UserId,
        plan_id: &PlanId,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    /// Возвращает страницу событий указанного плана пользователя.
    fn plan_events(
        &self,
        user_id: &UserId,
        plan_id: &PlanId,
        from: Option<Cursor>,
        limit: usize,
    ) -> impl Future<Output = Page<PlanEvent>> + Send;

    fn save_budget(
        &self,
        budget_id: BudgetId,
        budget: Budget,
    ) -> impl Future<Output = Result<BudgetId, StorageError>> + Send;

    fn budget_by_id(
        &self,
        id: &BudgetId,
    ) -> impl Future<Output = Option<StorageBudget>> + Send;

    fn budgets(
        &self,
        from: Option<Cursor>,
        limit: usize,
    ) -> impl Future<Output = Page<StorageBudget>> + Send;
}
