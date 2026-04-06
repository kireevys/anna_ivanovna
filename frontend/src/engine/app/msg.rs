use crate::engine::{app::model::View, history, onboarding, plan};

pub enum Msg {
    Onboarding(onboarding::Msg),
    SwitchView(View),
    Plan(plan::msg::Msg),
    History(history::Msg),
}
