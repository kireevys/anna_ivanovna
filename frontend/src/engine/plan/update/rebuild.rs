use rust_decimal::Decimal;

use ai_core::{finance::Percentage, plan::Plan as CorePlan, planning::ExpenseValue};

use crate::{
    engine::plan::{model::EditState, update::validate::recompute_validation},
    presentation::plan::editable,
};

pub fn rebuild_and_validate(edit: &EditState, base_plan: &CorePlan) -> EditState {
    let updated_plan =
        editable::build_updated_plan(base_plan, &edit.incomes, &edit.expenses);

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
    let is_empty = updated_plan.sources.is_empty() || updated_plan.expenses.is_empty();
    let business_invalid = is_empty
        || expenses_total.value > incomes_total.value
        || non_positive_incomes
        || non_positive_expenses_money
        || non_positive_expenses_rate;

    let (validation, save_state) = recompute_validation(edit, business_invalid);

    EditState {
        incomes: edit.incomes.clone(),
        expenses: edit.expenses.clone(),
        validation,
        save_state,
        core_plan: Some(updated_plan),
    }
}
