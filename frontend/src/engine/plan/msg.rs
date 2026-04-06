use serde::{Deserialize, Serialize};

use ai_core::plan::Plan;

use crate::{
    api::{ApiError, Collection, StoragePlanFrontend},
    presentation::plan::editable,
};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum Msg {
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

impl From<LoadingMsg> for Msg {
    fn from(msg: LoadingMsg) -> Self {
        Msg::Loading(msg)
    }
}

impl From<TemplateMsg> for Msg {
    fn from(msg: TemplateMsg) -> Self {
        Msg::Template(msg)
    }
}

impl From<EditMsg> for Msg {
    fn from(msg: EditMsg) -> Self {
        Msg::Edit(msg)
    }
}

impl From<PersistMsg> for Msg {
    fn from(msg: PersistMsg) -> Self {
        Msg::Persist(msg)
    }
}
