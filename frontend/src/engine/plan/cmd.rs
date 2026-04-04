use serde::Serialize;

use ai_core::plan::Plan;

#[derive(Clone, PartialEq, Serialize)]
pub enum PlanCmd {
    LoadPlan,
    LoadTemplates,
    SavePlan { id: String, plan: Plan },
    CreatePlan { plan: Plan },
    ScrollToTop,
}
