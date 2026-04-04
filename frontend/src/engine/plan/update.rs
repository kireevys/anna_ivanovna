mod rebuild;
pub(crate) mod validate;

use ai_core::plan::Plan as CorePlan;

use crate::{
    api::ApiError,
    engine::plan::{
        cmd::PlanCmd,
        model::{DataState, EditState, PlanModel, PlanValidation, SaveState},
        msg::{EditMsg, LoadingMsg, PersistMsg, PlanMsg, TemplateMsg},
    },
    presentation::plan::{editable, read::Plan},
};

pub fn handle(model: &PlanModel, msg: PlanMsg) -> (PlanModel, Vec<PlanCmd>) {
    match msg {
        PlanMsg::Loading(m) => handle_loading(model, m),
        PlanMsg::Template(m) => handle_template(model, m),
        PlanMsg::Edit(m) => handle_edit(model, m),
        PlanMsg::Persist(m) => handle_persist(model, m),
    }
}

fn handle_loading(_model: &PlanModel, msg: LoadingMsg) -> (PlanModel, Vec<PlanCmd>) {
    match msg {
        LoadingMsg::Reload => (PlanModel::Loading, vec![PlanCmd::LoadPlan]),
        LoadingMsg::Loaded(result) => match result {
            Ok(storage_plan) => {
                let plan = Plan::from(&storage_plan.plan);
                (
                    PlanModel::Viewing {
                        plan,
                        origin: storage_plan,
                    },
                    vec![],
                )
            }
            Err(ApiError::Http(404, _)) => (
                PlanModel::SelectingTemplate {
                    templates: DataState::Loading,
                },
                vec![PlanCmd::LoadTemplates],
            ),
            Err(e) => (PlanModel::Error(e.to_string()), vec![]),
        },
    }
}

fn handle_template(model: &PlanModel, msg: TemplateMsg) -> (PlanModel, Vec<PlanCmd>) {
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
                (model.clone(), vec![])
            }
        }
        TemplateMsg::Select(plan) => {
            if let PlanModel::SelectingTemplate { .. } = model {
                let edit = edit_state_from_core_plan(&plan);
                let edit = rebuild_edit(edit, &plan);
                (PlanModel::Creating { edit }, vec![PlanCmd::ScrollToTop])
            } else {
                (model.clone(), vec![])
            }
        }
        TemplateMsg::CreateFromScratch => {
            if let PlanModel::SelectingTemplate { .. } = model {
                let empty_plan = CorePlan::build(&[], &[]);
                let edit = edit_state_from_core_plan(&empty_plan);
                let edit = rebuild_edit(edit, &empty_plan);
                (PlanModel::Creating { edit }, vec![PlanCmd::ScrollToTop])
            } else {
                (model.clone(), vec![])
            }
        }
        TemplateMsg::Back => {
            if let PlanModel::Creating { .. } = model {
                (
                    PlanModel::SelectingTemplate {
                        templates: DataState::Loading,
                    },
                    vec![PlanCmd::LoadTemplates],
                )
            } else {
                (model.clone(), vec![])
            }
        }
    }
}

fn handle_edit(model: &PlanModel, msg: EditMsg) -> (PlanModel, Vec<PlanCmd>) {
    match msg {
        EditMsg::Enter => {
            if let PlanModel::Viewing { origin, .. } = model {
                let edit = edit_state_from_core_plan(&origin.plan);
                let edit = rebuild_edit(edit, &origin.plan);
                (
                    PlanModel::Editing {
                        origin: origin.clone(),
                        edit,
                    },
                    vec![],
                )
            } else {
                (model.clone(), vec![])
            }
        }
        EditMsg::Cancel => {
            if let PlanModel::Editing { origin, .. } = model {
                let plan = Plan::from(&origin.plan);
                (
                    PlanModel::Viewing {
                        plan,
                        origin: origin.clone(),
                    },
                    vec![],
                )
            } else {
                (model.clone(), vec![])
            }
        }
        EditMsg::IncomesChanged(incomes) => update_edit(model, |edit| EditState {
            incomes,
            ..edit.clone()
        }),
        EditMsg::ExpensesChanged(expenses) => update_edit(model, |edit| EditState {
            expenses,
            ..edit.clone()
        }),
    }
}

fn handle_persist(model: &PlanModel, msg: PersistMsg) -> (PlanModel, Vec<PlanCmd>) {
    match msg {
        PersistMsg::Save => {
            if let PlanModel::Editing { origin, edit } = model {
                if !matches!(edit.validation, PlanValidation::Valid)
                    || !matches!(edit.save_state, SaveState::CanSave)
                {
                    let new_edit = EditState {
                        save_state: SaveState::Disabled,
                        ..edit.clone()
                    };
                    return (
                        PlanModel::Editing {
                            origin: origin.clone(),
                            edit: new_edit,
                        },
                        vec![],
                    );
                }

                if let Some(core_plan) = &edit.core_plan {
                    let new_edit = EditState {
                        save_state: SaveState::Saving,
                        ..edit.clone()
                    };
                    (
                        PlanModel::Editing {
                            origin: origin.clone(),
                            edit: new_edit,
                        },
                        vec![PlanCmd::SavePlan {
                            id: origin.id.clone(),
                            plan: core_plan.clone(),
                        }],
                    )
                } else {
                    (model.clone(), vec![])
                }
            } else {
                (model.clone(), vec![])
            }
        }
        PersistMsg::SaveFinished(result) => {
            if let PlanModel::Editing { origin, edit } = model {
                match result {
                    Ok(()) => (PlanModel::Loading, vec![PlanCmd::LoadPlan]),
                    Err(ApiError::Http(422, _)) => (
                        PlanModel::Editing {
                            origin: origin.clone(),
                            edit: apply_422(edit),
                        },
                        vec![],
                    ),
                    Err(e) => (PlanModel::Error(e.to_string()), vec![]),
                }
            } else {
                (model.clone(), vec![])
            }
        }
        PersistMsg::Create => {
            if let PlanModel::Creating { edit } = model {
                if !matches!(edit.validation, PlanValidation::Valid)
                    || !matches!(edit.save_state, SaveState::CanSave)
                {
                    let new_edit = EditState {
                        save_state: SaveState::Disabled,
                        ..edit.clone()
                    };
                    return (PlanModel::Creating { edit: new_edit }, vec![]);
                }

                if let Some(core_plan) = &edit.core_plan {
                    let new_edit = EditState {
                        save_state: SaveState::Saving,
                        ..edit.clone()
                    };
                    (
                        PlanModel::Creating { edit: new_edit },
                        vec![PlanCmd::CreatePlan {
                            plan: core_plan.clone(),
                        }],
                    )
                } else {
                    (model.clone(), vec![])
                }
            } else {
                (model.clone(), vec![])
            }
        }
        PersistMsg::CreateFinished(result) => {
            if let PlanModel::Creating { edit } = model {
                match result {
                    Ok(_) => (PlanModel::Loading, vec![PlanCmd::LoadPlan]),
                    Err(ApiError::Http(422, _)) => (
                        PlanModel::Creating {
                            edit: apply_422(edit),
                        },
                        vec![],
                    ),
                    Err(e) => (PlanModel::Error(e.to_string()), vec![]),
                }
            } else {
                (model.clone(), vec![])
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
    model: &PlanModel,
    f: impl FnOnce(&EditState) -> EditState,
) -> (PlanModel, Vec<PlanCmd>) {
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
                    origin: origin.clone(),
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
        _ => (model.clone(), vec![]),
    }
}

pub(crate) const EXPENSES_EXCEED_INCOME: &str =
    "План некорректен: расходы превышают доходы";

fn apply_422(edit: &EditState) -> EditState {
    EditState {
        validation: PlanValidation::BusinessInvalid {
            messages: vec![EXPENSES_EXCEED_INCOME.into()],
        },
        save_state: SaveState::Disabled,
        ..edit.clone()
    }
}
