use serde::{Deserialize, Serialize};

use ai_core::plan::Plan;

use crate::{
    api::{ApiError, Collection, StoragePlanFrontend},
    presentation::plan::editable,
};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum PlanMsg {
    Loading(LoadingMsg),
    Template(TemplateMsg),
    Edit(EditMsg),
    Persist(PersistMsg),
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum LoadingMsg {
    Reload,
    Loaded(Result<StoragePlanFrontend, ApiError>),
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum TemplateMsg {
    TemplatesLoaded(Result<Vec<Collection>, String>),
    Select(Plan),
    CreateFromScratch,
    Back,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum EditMsg {
    Enter,
    Cancel,
    IncomesChanged(Vec<editable::IncomeSource>),
    ExpensesChanged(Vec<editable::Expense>),
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum PersistMsg {
    Save,
    SaveFinished(Result<(), ApiError>),
    Create,
    CreateFinished(Result<String, ApiError>),
}

impl From<LoadingMsg> for PlanMsg {
    fn from(msg: LoadingMsg) -> Self {
        PlanMsg::Loading(msg)
    }
}

impl From<TemplateMsg> for PlanMsg {
    fn from(msg: TemplateMsg) -> Self {
        PlanMsg::Template(msg)
    }
}

impl From<EditMsg> for PlanMsg {
    fn from(msg: EditMsg) -> Self {
        PlanMsg::Edit(msg)
    }
}

impl From<PersistMsg> for PlanMsg {
    fn from(msg: PersistMsg) -> Self {
        PlanMsg::Persist(msg)
    }
}
