use crate::{
    api::{ApiError, StoragePlanFrontend},
    presentation::plan::Plan,
};
use ai_core::{finance::Percentage, planning::ExpenseValue};
use rust_decimal::Decimal;
use yew::Context;

use super::{App, AppMsg, DataState, PlanMode, PlanValidation, SaveState};

impl App {
    pub(super) fn handle_load_plan(&mut self, ctx: &Context<Self>) -> bool {
        self.plan.set_data(DataState::Loading);
        self.plan.set_meta(None);
        self.plan.set_mode(PlanMode::View);
        self.plan.set_incomes(vec![]);
        self.plan.set_expenses(vec![]);
        self.plan.reset_validation();
        self.load_plan_async(ctx.link());
        true
    }

    pub(super) fn handle_plan_loaded(
        &mut self,
        result: Result<StoragePlanFrontend, String>,
    ) -> bool {
        match result {
            Ok(storage_plan) => {
                self.plan.set_meta(Some(storage_plan.clone()));
                let incomes =
                    crate::presentation::editable_plan::incomes_from_core_plan(
                        &storage_plan.plan,
                    );
                let expenses =
                    crate::presentation::editable_plan::expenses_from_core_plan(
                        &storage_plan.plan,
                    );
                self.plan.set_incomes(incomes);
                self.plan.set_expenses(expenses);
                self.plan.set_mode(PlanMode::View);
                self.plan.reset_validation();
                self.plan
                    .set_data(DataState::Loaded(Plan::from(&storage_plan.plan)));
            }
            Err(e) => {
                self.plan.set_meta(None);
                self.plan.set_mode(PlanMode::View);
                self.plan.set_incomes(vec![]);
                self.plan.set_expenses(vec![]);
                self.plan.reset_validation();
                self.plan.set_data(DataState::Error(e));
            }
        }
        true
    }

    pub(super) fn handle_enter_edit_mode(&mut self) -> bool {
        if let Some(storage_plan) = self.plan.meta.clone() {
            let incomes = crate::presentation::editable_plan::incomes_from_core_plan(
                &storage_plan.plan,
            );
            let expenses = crate::presentation::editable_plan::expenses_from_core_plan(
                &storage_plan.plan,
            );
            self.plan.set_incomes(incomes);
            self.plan.set_expenses(expenses);
            self.plan.set_mode(PlanMode::Edit);
            self.plan.reset_validation();
        }
        true
    }

    pub(super) fn handle_cancel_edit_mode(&mut self) -> bool {
        if let Some(storage_plan) = self.plan.meta.clone() {
            let incomes = crate::presentation::editable_plan::incomes_from_core_plan(
                &storage_plan.plan,
            );
            self.plan.set_incomes(incomes);
            let expenses = crate::presentation::editable_plan::expenses_from_core_plan(
                &storage_plan.plan,
            );
            self.plan.set_expenses(expenses);
        }
        self.plan.set_mode(PlanMode::View);
        self.plan.reset_validation();
        true
    }

    pub(super) fn handle_income_sources_changed(
        &mut self,
        incomes: Vec<crate::presentation::editable_plan::EditableIncomeSource>,
    ) -> bool {
        let validated: Vec<_> = incomes
            .into_iter()
            .map(|mut income| {
                income.is_valid =
                    rust_decimal::Decimal::from_str_exact(&income.amount).is_ok();
                income
            })
            .collect();

        self.plan.set_incomes(validated);
        self.rebuild_plan_and_validate();
        true
    }

    pub(super) fn handle_expenses_changed(
        &mut self,
        expenses: Vec<crate::presentation::editable_plan::EditableExpense>,
    ) -> bool {
        let validated: Vec<_> = expenses
            .into_iter()
            .map(|mut expense| {
                expense.is_valid =
                    rust_decimal::Decimal::from_str_exact(&expense.amount).is_ok();
                expense
            })
            .collect();

        self.plan.set_expenses(validated);
        self.rebuild_plan_and_validate();
        true
    }

    fn rebuild_plan_and_validate(&mut self) {
        let Some(storage_plan) = &self.plan.meta else {
            self.plan.recompute_validation_after_edit(false);
            return;
        };

        let updated_plan = crate::presentation::editable_plan::build_updated_plan(
            &storage_plan.plan,
            &self.plan.incomes,
            &self.plan.expenses,
        );

        self.plan
            .set_data(DataState::Loaded(Plan::from(&updated_plan)));

        let incomes_total = updated_plan.total_incomes();
        let expenses_total = updated_plan.total_expenses();
        let non_positive_incomes = updated_plan
            .sources
            .iter()
            .any(|s| s.expected.value <= Decimal::ZERO);
        let non_positive_expenses_money = updated_plan
            .expenses
            .iter()
            .filter_map(|e| match &e.value {
                ExpenseValue::MONEY { value } => Some(value.value),
                _ => None,
            })
            .any(|v| v <= Decimal::ZERO);
        let non_positive_expenses_rate = updated_plan
            .expenses
            .iter()
            .filter_map(|e| match &e.value {
                ExpenseValue::RATE { value } => Some(value),
                _ => None,
            })
            .any(|p| *p <= Percentage::ZERO);
        let business_invalid = expenses_total.value > incomes_total.value
            || non_positive_incomes
            || non_positive_expenses_money
            || non_positive_expenses_rate;

        self.plan.recompute_validation_after_edit(business_invalid);
    }

    pub(super) fn handle_save_plan(&mut self, ctx: &Context<Self>) -> bool {
        if !matches!(self.plan.validation, PlanValidation::Valid)
            || !matches!(self.plan.save_state, SaveState::CanSave)
        {
            self.plan.save_state = SaveState::Disabled;
            return true;
        }

        if let Some(storage_plan) = &self.plan.meta {
            self.plan.save_state = SaveState::Saving;
            self.save_plan_async(storage_plan.clone(), ctx.link());
        }
        true
    }

    pub(super) fn handle_plan_save_finished(
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

    pub(super) fn load_plan_async(&self, link: &yew::html::Scope<Self>) {
        let api = self.api.clone();
        let link = link.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.get_plan().await.map_err(|e| e.to_string());
            link.send_message(AppMsg::PlanLoaded(result));
        });
    }

    pub(super) fn save_plan_async(
        &self,
        storage_plan: StoragePlanFrontend,
        link: &yew::html::Scope<Self>,
    ) {
        let api = self.api.clone();
        let link = link.clone();
        let incomes = self.plan.incomes.clone();
        let expenses = self.plan.expenses.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let updated_plan = crate::presentation::editable_plan::build_updated_plan(
                &storage_plan.plan,
                &incomes,
                &expenses,
            );
            let result = api.update_plan(&storage_plan.id, &updated_plan).await;
            link.send_message(AppMsg::PlanSaveFinished(result));
        });
    }
}
