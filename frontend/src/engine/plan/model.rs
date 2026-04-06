use serde::{Deserialize, Serialize};

use ai_core::plan::Plan as CorePlan;

use crate::{
    api::{Collection, StoragePlanFrontend},
    engine::core::{DataState, Model},
    presentation::plan::editable,
};

use crate::engine::plan::{cmd, msg};

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
    pub(crate) incomes: Vec<editable::IncomeSource>,
    pub(crate) expenses: Vec<editable::Expense>,
    pub(crate) validation: PlanValidation,
    pub(crate) save_state: SaveState,
    pub(crate) core_plan: Option<CorePlan>,
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
        origin: StoragePlanFrontend,
    },
    Editing {
        origin: StoragePlanFrontend,
        edit: EditState,
    },
}

impl Model for PlanModel {
    type Msg = msg::Msg;
    type Cmd = cmd::Cmd;

    fn handle(self, msg: Self::Msg) -> (Self, Vec<Self::Cmd>) {
        crate::engine::plan::update::handle(self, msg)
    }
}
