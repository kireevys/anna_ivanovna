use crate::{
    api::{ApiError, StoragePlanFrontend},
    presentation::plan::{editable, read::Plan},
};
use ai_core::{finance::Percentage, planning::ExpenseValue};
use rust_decimal::Decimal;
use yew::Context;

use super::{App, AppMsg, DataState, PlanMode, PlanValidation, SaveState};

fn scroll_to_top() {
    if let Some(window) = web_sys::window() {
        window.scroll_to_with_x_and_y(0.0, 0.0);
    }
}

impl App {
    pub(crate) fn handle_load_plan(&mut self, ctx: &Context<Self>) -> bool {
        self.plan.set_data(DataState::Loading);
        self.plan.set_meta(None);
        self.plan.set_mode(PlanMode::View);
        self.plan.set_incomes(vec![]);
        self.plan.set_expenses(vec![]);
        self.plan.reset_validation();
        self.load_plan_async(ctx.link());
        true
    }

    pub(crate) fn handle_plan_loaded(
        &mut self,
        ctx: &Context<Self>,
        result: Result<StoragePlanFrontend, ApiError>,
    ) -> bool {
        match result {
            Ok(storage_plan) => {
                self.plan.set_meta(Some(storage_plan.clone()));
                let incomes = editable::incomes_from_core_plan(&storage_plan.plan);
                let expenses = editable::expenses_from_core_plan(&storage_plan.plan);
                self.plan.set_incomes(incomes);
                self.plan.set_expenses(expenses);
                self.plan.set_mode(PlanMode::View);
                self.plan.reset_validation();
                self.plan
                    .set_data(DataState::Loaded(Plan::from(&storage_plan.plan)));
            }
            Err(ApiError::Http(404, _)) => {
                self.plan.set_meta(None);
                self.plan.set_mode(PlanMode::Creating);
                self.plan.templates = DataState::Loading;
                self.load_templates_async(ctx);
            }
            Err(e) => {
                self.plan.set_meta(None);
                self.plan.set_mode(PlanMode::View);
                self.plan.set_incomes(vec![]);
                self.plan.set_expenses(vec![]);
                self.plan.reset_validation();
                self.plan.set_data(DataState::Error(e.to_string()));
            }
        }
        true
    }

    pub(crate) fn handle_enter_edit_mode(&mut self) -> bool {
        if let Some(storage_plan) = self.plan.meta.clone() {
            let incomes = editable::incomes_from_core_plan(&storage_plan.plan);
            let expenses = editable::expenses_from_core_plan(&storage_plan.plan);
            self.plan.set_incomes(incomes);
            self.plan.set_expenses(expenses);
            self.plan.set_mode(PlanMode::Edit);
            self.plan.reset_validation();
            self.rebuild_plan_and_validate();
        }
        true
    }

    pub(crate) fn handle_cancel_edit_mode(&mut self) -> bool {
        if let Some(storage_plan) = self.plan.meta.clone() {
            let incomes = editable::incomes_from_core_plan(&storage_plan.plan);
            self.plan.set_incomes(incomes);
            let expenses = editable::expenses_from_core_plan(&storage_plan.plan);
            self.plan.set_expenses(expenses);
            self.plan
                .set_data(DataState::Loaded(Plan::from(&storage_plan.plan)));
        }
        self.plan.edited_core_plan = None;
        self.plan.set_mode(PlanMode::View);
        self.plan.reset_validation();
        true
    }

    pub(crate) fn handle_income_sources_changed(
        &mut self,
        incomes: Vec<editable::IncomeSource>,
    ) -> bool {
        self.plan.set_incomes(incomes);
        self.rebuild_plan_and_validate();
        true
    }

    pub(crate) fn handle_expenses_changed(
        &mut self,
        expenses: Vec<editable::Expense>,
    ) -> bool {
        self.plan.set_expenses(expenses);
        self.rebuild_plan_and_validate();
        true
    }

    fn rebuild_plan_and_validate(&mut self) {
        let base_plan = if let Some(storage_plan) = &self.plan.meta {
            storage_plan.plan.clone()
        } else if let Some(edited) = &self.plan.edited_core_plan {
            edited.clone()
        } else {
            self.plan.recompute_validation_after_edit(false);
            return;
        };

        let updated_plan = editable::build_updated_plan(
            &base_plan,
            &self.plan.incomes,
            &self.plan.expenses,
        );

        self.plan
            .set_data(DataState::Loaded(Plan::from(&updated_plan)));
        self.plan.edited_core_plan = Some(updated_plan.clone());

        let incomes_total = updated_plan.total_incomes();
        let expenses_total = updated_plan.total_expenses();
        let non_positive_incomes = updated_plan
            .sources
            .iter()
            .any(|s| s.net().value <= Decimal::ZERO);
        let non_positive_expenses_money = updated_plan
            .expenses
            .iter()
            .filter_map(|e| match e.value() {
                ExpenseValue::MONEY { value } => Some(value.value),
                _ => None,
            })
            .any(|v| v <= Decimal::ZERO);
        let non_positive_expenses_rate = updated_plan
            .expenses
            .iter()
            .filter_map(|e| match e.value() {
                ExpenseValue::RATE { value } => Some(value),
                _ => None,
            })
            .any(|p| p <= Percentage::ZERO);
        let is_empty =
            updated_plan.sources.is_empty() || updated_plan.expenses.is_empty();
        let business_invalid = is_empty
            || expenses_total.value > incomes_total.value
            || non_positive_incomes
            || non_positive_expenses_money
            || non_positive_expenses_rate;

        self.plan.recompute_validation_after_edit(business_invalid);
    }

    pub(crate) fn handle_save_plan(&mut self, ctx: &Context<Self>) -> bool {
        if !matches!(self.plan.validation, PlanValidation::Valid)
            || !matches!(self.plan.save_state, SaveState::CanSave)
        {
            self.plan.save_state = SaveState::Disabled;
            return true;
        }

        if let (Some(storage_plan), Some(core_plan)) =
            (&self.plan.meta, &self.plan.edited_core_plan)
        {
            self.plan.save_state = SaveState::Saving;
            self.save_plan_async(
                storage_plan.id.clone(),
                core_plan.clone(),
                ctx.link(),
            );
        }
        true
    }

    pub(crate) fn handle_plan_save_finished(
        &mut self,
        ctx: &Context<Self>,
        result: Result<(), ApiError>,
    ) -> bool {
        match result {
            Ok(()) => {
                self.plan.reset_validation();
                ctx.link().send_message(AppMsg::LoadPlan);
            }
            Err(e) => match e {
                ApiError::Http(422, _) => {
                    self.plan.validation = PlanValidation::BusinessInvalid {
                        messages: vec![
                            "План некорректен: расходы превышают доходы".into(),
                        ],
                    };
                    self.plan.save_state = SaveState::Disabled;
                }
                other => {
                    self.plan.set_data(DataState::Error(other.to_string()));
                    self.plan.set_mode(PlanMode::View);
                    self.plan.save_state = SaveState::Disabled;
                }
            },
        }
        true
    }

    pub(crate) fn load_plan_async(&self, link: &yew::html::Scope<Self>) {
        let api = self.api.clone();
        let link = link.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.get_plan().await;
            link.send_message(AppMsg::PlanLoaded(result));
        });
    }

    pub(crate) fn save_plan_async(
        &self,
        id: String,
        core_plan: ai_core::plan::Plan,
        link: &yew::html::Scope<Self>,
    ) {
        let api = self.api.clone();
        let link = link.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.update_plan(&id, &core_plan).await;
            link.send_message(AppMsg::PlanSaveFinished(result));
        });
    }

    pub(crate) fn handle_templates_loaded(
        &mut self,
        result: Result<Vec<crate::api::Collection>, String>,
    ) -> bool {
        match result {
            Ok(templates) => {
                self.plan.templates = DataState::Loaded(templates);
            }
            Err(e) => {
                self.plan.templates = DataState::Error(e);
            }
        }
        true
    }

    pub(crate) fn handle_select_template(&mut self, plan: ai_core::plan::Plan) -> bool {
        let incomes = editable::incomes_from_core_plan(&plan);
        let expenses = editable::expenses_from_core_plan(&plan);
        self.plan.set_incomes(incomes);
        self.plan.set_expenses(expenses);
        self.plan.edited_core_plan = Some(plan.clone());
        self.plan.set_data(DataState::Loaded(Plan::from(&plan)));
        self.plan.set_mode(PlanMode::Creating);
        self.plan.reset_validation();
        self.rebuild_plan_and_validate();
        scroll_to_top();
        true
    }

    pub(crate) fn handle_back_to_templates(&mut self) -> bool {
        self.plan.edited_core_plan = None;
        self.plan.set_data(DataState::Loading);
        self.plan.set_mode(PlanMode::Creating);
        self.plan.reset_validation();
        true
    }

    pub(crate) fn handle_create_from_scratch(&mut self) -> bool {
        let empty_plan = ai_core::plan::Plan::build(&[], &[]);
        self.handle_select_template(empty_plan)
    }

    pub(crate) fn handle_create_plan(&mut self, ctx: &Context<Self>) -> bool {
        if !matches!(self.plan.validation, PlanValidation::Valid)
            || !matches!(self.plan.save_state, SaveState::CanSave)
        {
            self.plan.save_state = SaveState::Disabled;
            return true;
        }

        if let Some(core_plan) = &self.plan.edited_core_plan {
            self.plan.save_state = SaveState::Saving;
            self.create_plan_async(core_plan.clone(), ctx.link());
        }
        true
    }

    pub(crate) fn handle_plan_create_finished(
        &mut self,
        ctx: &Context<Self>,
        result: Result<String, crate::api::ApiError>,
    ) -> bool {
        match result {
            Ok(_plan_id) => {
                self.plan.reset_validation();
                ctx.link().send_message(AppMsg::LoadPlan);
            }
            Err(e) => match e {
                crate::api::ApiError::Http(422, _) => {
                    self.plan.validation = PlanValidation::BusinessInvalid {
                        messages: vec![
                            "План некорректен: расходы превышают доходы".into(),
                        ],
                    };
                    self.plan.save_state = SaveState::Disabled;
                }
                other => {
                    self.plan.set_data(DataState::Error(other.to_string()));
                    self.plan.set_mode(PlanMode::View);
                    self.plan.save_state = SaveState::Disabled;
                }
            },
        }
        true
    }

    pub(crate) fn load_templates_async(&self, ctx: &Context<Self>) {
        let api = self.api.clone();
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.get_collections().await.map_err(|e| e.to_string());
            link.send_message(AppMsg::TemplatesLoaded(result));
        });
    }

    pub(crate) fn create_plan_async(
        &self,
        core_plan: ai_core::plan::Plan,
        link: &yew::html::Scope<Self>,
    ) {
        let api = self.api.clone();
        let link = link.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.create_plan(&core_plan).await;
            link.send_message(AppMsg::PlanCreateFinished(result));
        });
    }
}
