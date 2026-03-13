use tracing::info;

use crate::storage::{CoreRepo, build_id};

const DEFAULT_USER: &str = "default";

pub fn migrate<S: CoreRepo, T: CoreRepo>(source: &S, target: &T) -> Result<(), String> {
    let user_id = DEFAULT_USER.to_string();
    if let Some(sp) = source.get_plan(&user_id) {
        let plan_id = build_id();
        target
            .create_plan(&user_id, plan_id.clone(), sp.plan)
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
        let id = build_id();
        target
            .save_budget(id, sb.budget.clone())
            .map_err(|e| e.to_string())?;
    }
    info!("Мигрировано бюджетов: {}", all_budgets.len());
    Ok(())
}
