mod rebuild;
pub(crate) mod validate;

use ai_core::plan::Plan as CorePlan;

use crate::{
    api::ApiError,
    engine::{
        core::DataState,
        plan::{
            cmd::Cmd,
            editable,
            model::{EditState, PlanModel, PlanValidation, SaveState},
            msg::{EditMsg, LoadingMsg, Msg, PersistMsg, TemplateMsg},
        },
    },
};

pub(crate) fn handle(model: PlanModel, msg: Msg) -> (PlanModel, Vec<Cmd>) {
    match msg {
        Msg::Loading(m) => handle_loading(model, m),
        Msg::Template(m) => handle_template(model, m),
        Msg::Edit(m) => handle_edit(model, m),
        Msg::Persist(m) => handle_persist(model, m),
    }
}

fn handle_loading(_model: PlanModel, msg: LoadingMsg) -> (PlanModel, Vec<Cmd>) {
    match msg {
        LoadingMsg::Reload => (PlanModel::Loading, vec![Cmd::LoadPlan]),
        LoadingMsg::Loaded(result) => match result {
            Ok(storage_plan) => (
                PlanModel::Viewing {
                    origin: storage_plan,
                },
                vec![],
            ),
            Err(ApiError::Http(404, _)) => (
                PlanModel::SelectingTemplate {
                    templates: DataState::Loading,
                },
                vec![Cmd::LoadTemplates],
            ),
            Err(e) => (PlanModel::Error(e.to_string()), vec![]),
        },
    }
}

fn handle_template(model: PlanModel, msg: TemplateMsg) -> (PlanModel, Vec<Cmd>) {
    match msg {
        TemplateMsg::TemplatesLoaded(result) => {
            if let PlanModel::SelectingTemplate { .. } = model {
                match result {
                    Ok(templates) => (
                        PlanModel::SelectingTemplate {
                            templates: DataState::Loaded(templates),
                        },
                        vec![],
                    ),
                    Err(e) => (
                        PlanModel::SelectingTemplate {
                            templates: DataState::Error(e),
                        },
                        vec![],
                    ),
                }
            } else {
                (model, vec![])
            }
        }
        TemplateMsg::Select(plan) => {
            if let PlanModel::SelectingTemplate { .. } = model {
                let edit = edit_state_from_core_plan(&plan);
                let edit = rebuild_edit(edit, &plan);
                (PlanModel::Creating { edit }, vec![Cmd::ScrollToTop])
            } else {
                (model, vec![])
            }
        }
        TemplateMsg::CreateFromScratch => {
            if let PlanModel::SelectingTemplate { .. } = model {
                let empty_plan = CorePlan::build(&[], &[]);
                let edit = edit_state_from_core_plan(&empty_plan);
                let edit = rebuild_edit(edit, &empty_plan);
                (PlanModel::Creating { edit }, vec![Cmd::ScrollToTop])
            } else {
                (model, vec![])
            }
        }
        TemplateMsg::Back => {
            if let PlanModel::Creating { .. } = model {
                (
                    PlanModel::SelectingTemplate {
                        templates: DataState::Loading,
                    },
                    vec![Cmd::LoadTemplates],
                )
            } else {
                (model, vec![])
            }
        }
    }
}

fn handle_edit(model: PlanModel, msg: EditMsg) -> (PlanModel, Vec<Cmd>) {
    match msg {
        EditMsg::Enter => {
            if let PlanModel::Viewing { origin, .. } = model {
                let edit = edit_state_from_core_plan(&origin.plan);
                let edit = rebuild_edit(edit, &origin.plan);
                (PlanModel::Editing { origin, edit }, vec![])
            } else {
                (model, vec![])
            }
        }
        EditMsg::Cancel => {
            if let PlanModel::Editing { origin, .. } = model {
                (PlanModel::Viewing { origin }, vec![])
            } else {
                (model, vec![])
            }
        }
        EditMsg::IncomesChanged(incomes) => {
            update_edit(model, |edit| EditState { incomes, ..edit })
        }
        EditMsg::ExpensesChanged(expenses) => {
            update_edit(model, |edit| EditState { expenses, ..edit })
        }
    }
}

fn handle_persist(model: PlanModel, msg: PersistMsg) -> (PlanModel, Vec<Cmd>) {
    match msg {
        PersistMsg::Save => {
            let PlanModel::Editing { origin, edit } = model else {
                return (model, vec![]);
            };
            if !matches!(edit.validation, PlanValidation::Valid)
                || !matches!(edit.save_state, SaveState::CanSave)
            {
                return (
                    PlanModel::Editing {
                        origin,
                        edit: EditState {
                            save_state: SaveState::Disabled,
                            ..edit
                        },
                    },
                    vec![],
                );
            }
            let Some(core_plan) = edit.core_plan.clone() else {
                return (PlanModel::Editing { origin, edit }, vec![]);
            };
            let id = origin.id.clone();
            (
                PlanModel::Editing {
                    origin,
                    edit: EditState {
                        save_state: SaveState::Saving,
                        ..edit
                    },
                },
                vec![Cmd::SavePlan {
                    id,
                    plan: core_plan,
                }],
            )
        }
        PersistMsg::SaveFinished(result) => {
            let PlanModel::Editing { origin, edit } = model else {
                return (model, vec![]);
            };
            match result {
                Ok(()) => (PlanModel::Loading, vec![Cmd::LoadPlan]),
                Err(ApiError::Http(422, _)) => (
                    PlanModel::Editing {
                        origin,
                        edit: apply_422(edit),
                    },
                    vec![],
                ),
                Err(e) => (PlanModel::Error(e.to_string()), vec![]),
            }
        }
        PersistMsg::Create => {
            let PlanModel::Creating { edit } = model else {
                return (model, vec![]);
            };
            if !matches!(edit.validation, PlanValidation::Valid)
                || !matches!(edit.save_state, SaveState::CanSave)
            {
                return (
                    PlanModel::Creating {
                        edit: EditState {
                            save_state: SaveState::Disabled,
                            ..edit
                        },
                    },
                    vec![],
                );
            }
            let Some(core_plan) = edit.core_plan.clone() else {
                return (PlanModel::Creating { edit }, vec![]);
            };
            (
                PlanModel::Creating {
                    edit: EditState {
                        save_state: SaveState::Saving,
                        ..edit
                    },
                },
                vec![Cmd::CreatePlan { plan: core_plan }],
            )
        }
        PersistMsg::CreateFinished(result) => {
            let PlanModel::Creating { edit } = model else {
                return (model, vec![]);
            };
            match result {
                Ok(_) => (PlanModel::Loading, vec![Cmd::LoadPlan]),
                Err(ApiError::Http(422, _)) => (
                    PlanModel::Creating {
                        edit: apply_422(edit),
                    },
                    vec![],
                ),
                Err(e) => (PlanModel::Error(e.to_string()), vec![]),
            }
        }
    }
}

fn edit_state_from_core_plan(plan: &CorePlan) -> EditState {
    let incomes = editable::incomes_from_core_plan(plan);
    let expenses = editable::expenses_from_core_plan(plan);
    EditState {
        incomes,
        expenses,
        validation: PlanValidation::Valid,
        save_state: SaveState::Idle,
        core_plan: Some(plan.clone()),
    }
}

fn rebuild_edit(edit: EditState, base_plan: &CorePlan) -> EditState {
    rebuild::rebuild_and_validate(&edit, base_plan)
}

fn update_edit(
    model: PlanModel,
    f: impl FnOnce(EditState) -> EditState,
) -> (PlanModel, Vec<Cmd>) {
    match model {
        PlanModel::Editing { origin, edit } => {
            let new_edit = f(edit);
            let base_plan = new_edit
                .core_plan
                .clone()
                .unwrap_or_else(|| origin.plan.clone());
            let new_edit = rebuild::rebuild_and_validate(&new_edit, &base_plan);
            (
                PlanModel::Editing {
                    origin,
                    edit: new_edit,
                },
                vec![],
            )
        }
        PlanModel::Creating { edit } => {
            let new_edit = f(edit);
            if let Some(base_plan) = &new_edit.core_plan {
                let base = base_plan.clone();
                let new_edit = rebuild::rebuild_and_validate(&new_edit, &base);
                (PlanModel::Creating { edit: new_edit }, vec![])
            } else {
                let (validation, save_state) =
                    validate::recompute_validation(&new_edit, false);
                let new_edit = EditState {
                    validation,
                    save_state,
                    ..new_edit
                };
                (PlanModel::Creating { edit: new_edit }, vec![])
            }
        }
        _ => (model, vec![]),
    }
}

pub(crate) const EXPENSES_EXCEED_INCOME: &str =
    "План некорректен: расходы превышают доходы";

fn apply_422(edit: EditState) -> EditState {
    EditState {
        validation: PlanValidation::BusinessInvalid {
            messages: vec![EXPENSES_EXCEED_INCOME.into()],
        },
        save_state: SaveState::Disabled,
        ..edit
    }
}
