use std::{collections::HashSet, str::FromStr};

use rust_decimal::Decimal;

use crate::{
    engine::plan::model::{PlanValidation, SaveState},
    presentation::plan::editable,
};

pub(crate) fn recompute_validation(
    edit: &crate::engine::plan::model::EditState,
    business_invalid: bool,
) -> (PlanValidation, SaveState) {
    let mut format_messages = Vec::new();

    validate_named_items(
        edit.incomes
            .iter()
            .map(|i| (i.name.as_str(), i.amount.as_str())),
        "дохода",
        &mut format_messages,
    );

    validate_named_items(
        edit.expenses
            .iter()
            .map(|e| (e.name.as_str(), e.primary_amount())),
        "расхода",
        &mut format_messages,
    );

    for expense in &edit.expenses {
        if expense.active_type == editable::ActiveType::Credit {
            for error in expense.credit.validation_errors() {
                let label = item_display_name(&expense.name, "Расход");
                format_messages.push(format!("{label}: {error}"));
            }
        }
    }

    if !format_messages.is_empty() {
        let validation = PlanValidation::FormatInvalid {
            messages: format_messages,
        };
        return (validation, SaveState::Disabled);
    }

    if business_invalid {
        let mut business_messages = Vec::new();

        for income in &edit.incomes {
            if !income.amount.is_empty()
                && let Ok(v) = Decimal::from_str(&income.amount)
                && v <= Decimal::ZERO
            {
                business_messages
                    .push(format!("Доход \"{}\" должен быть больше 0", income.name));
            }
        }

        for expense in &edit.expenses {
            let amount_str = expense.primary_amount();
            if !amount_str.is_empty()
                && let Ok(v) = Decimal::from_str(amount_str)
                && v <= Decimal::ZERO
            {
                business_messages
                    .push(format!("Расход \"{}\" должен быть больше 0", expense.name));
            }
        }

        if business_messages.is_empty() {
            business_messages
                .push(crate::engine::plan::update::EXPENSES_EXCEED_INCOME.into());
        }

        let validation = PlanValidation::BusinessInvalid {
            messages: business_messages,
        };
        return (validation, SaveState::Disabled);
    }

    let save_state = match edit.save_state {
        SaveState::Idle | SaveState::Disabled => SaveState::CanSave,
        SaveState::Saving => SaveState::Saving,
        SaveState::CanSave => SaveState::CanSave,
    };
    (PlanValidation::Valid, save_state)
}

fn validate_named_items<'a>(
    items: impl Iterator<Item = (&'a str, &'a str)>,
    label: &str,
    messages: &mut Vec<String>,
) {
    let items: Vec<_> = items.collect();

    for &(name, amount) in &items {
        if !amount.is_empty() && Decimal::from_str(amount).is_err() {
            messages.push(format!(
                "{}: некорректное число",
                item_display_name(name, label),
            ));
        }
    }

    if items.iter().any(|&(name, _)| name.trim().is_empty()) {
        messages.push(format!("Не указано название {label}"));
    }

    if items.iter().any(|&(_, amount)| amount.is_empty()) {
        messages.push(format!("Не указана сумма {label}"));
    }

    let mut seen = HashSet::new();
    for &(name, _) in &items {
        let trimmed = name.trim().to_lowercase();
        if !trimmed.is_empty() && !seen.insert(trimmed) {
            messages.push(format!(
                "Дублирующееся название {label}: \"{}\"",
                name.trim()
            ));
        }
    }
}

fn item_display_name(name: &str, label: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        format!("(без названия {label})")
    } else {
        format!("{label} \"{trimmed}\"")
    }
}
