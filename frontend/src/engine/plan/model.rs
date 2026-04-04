use serde::{Deserialize, Serialize};

use ai_core::plan::Plan as CorePlan;

use crate::{
    api::{Collection, StoragePlanFrontend},
    presentation::plan::{editable, read::Plan},
};

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum DataState<T> {
    Loading,
    Loaded(T),
    Error(String),
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub enum PlanValidation {
    Valid,
    FormatInvalid { messages: Vec<String> },
    BusinessInvalid { messages: Vec<String> },
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum SaveState {
    Idle,
    CanSave,
    Disabled,
    Saving,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct EditState {
    pub incomes: Vec<editable::IncomeSource>,
    pub expenses: Vec<editable::Expense>,
    pub validation: PlanValidation,
    pub save_state: SaveState,
    pub core_plan: Option<CorePlan>,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "state", content = "payload")]
pub enum PlanModel {
    Loading,
    Error(String),
    SelectingTemplate {
        templates: DataState<Vec<Collection>>,
    },
    Creating {
        edit: EditState,
    },
    Viewing {
        plan: Plan,
        origin: StoragePlanFrontend,
    },
    Editing {
        origin: StoragePlanFrontend,
        edit: EditState,
    },
}
