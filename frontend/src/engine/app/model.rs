use crate::engine::{
    app::{cmd, msg},
    core::Model,
    history,
    onboarding,
    plan,
};

#[derive(Clone)]
pub struct AppModel {
    pub(crate) onboarding: onboarding::OnboardingModel,
    pub(crate) view: View,
    pub(crate) plan: plan::model::PlanModel,
    pub(crate) history: history::HistoryModel,
}

#[derive(Clone, PartialEq)]
pub enum View {
    Plan,
    History,
}

impl Model for AppModel {
    type Msg = msg::Msg;
    type Cmd = cmd::Cmd;

    fn handle(self, msg: Self::Msg) -> (Self, Vec<Self::Cmd>) {
        crate::engine::app::update::handle(self, msg)
    }
}
