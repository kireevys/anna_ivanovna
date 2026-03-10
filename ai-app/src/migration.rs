use tracing::info;

use crate::{api::CoreApi, storage::CoreRepo};

pub fn migrate<S: CoreRepo, T: CoreRepo>(source: &S, target: &T) -> Result<(), String> {
    if let Some(plan) = source.get_plan() {
        let plan_id = CoreApi::<T>::build_budget_id();
        target
            .save_plan(plan_id.clone(), plan)
            .map_err(|e| e.to_string())?;
        info!("Мигрирован план: {plan_id}");
    }

    let mut all_budgets = Vec::new();
    let mut cursor = None;
    loop {
        let page = source.budgets(cursor, 50);
        if page.items.is_empty() {
            break;
        }
        all_budgets.extend(page.items);
        cursor = page.next_cursor.clone();
        if cursor.is_none() {
            break;
        }
    }

    all_budgets.sort_by(|a, b| a.budget.income_date().cmp(b.budget.income_date()));

    for sb in &all_budgets {
        let id = CoreApi::<T>::build_budget_id();
        target
            .save_budget(id, sb.budget.clone())
            .map_err(|e| e.to_string())?;
    }
    info!("Мигрировано бюджетов: {}", all_budgets.len());
    Ok(())
}
