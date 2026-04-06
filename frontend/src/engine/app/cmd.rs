use crate::engine::{history, onboarding, plan};

pub enum Cmd {
    Plan(plan::cmd::Cmd),
    History(history::Cmd),
    Onboarding(onboarding::Cmd),
}
