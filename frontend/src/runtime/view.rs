use std::rc::Rc;

use yew::{Context, Html, html};

use crate::{
    api::ApiClient,
    engine::{
        app::{
            model::{AppModel, View},
            msg,
        },
        core::{DataState, PageStatus},
        history,
        onboarding::{self, OnboardingModel},
        plan::{
            model::{EditState, PlanModel, PlanValidation, SaveState},
            msg::{EditMsg, LoadingMsg, PersistMsg, TemplateMsg},
        },
    },
    presentation::{
        components::{
            AppLayout,
            EditLayout,
            Error,
            HistoryView,
            Loading,
            PlanView,
            TemplateSelector,
            Totals,
            WelcomeScreen,
        },
        plan::read::Plan,
    },
    runtime::App,
};

pub fn render(
    model: Option<&AppModel>,
    api: &Rc<ApiClient>,
    ctx: &Context<App>,
) -> Html {
    let Some(model) = model else {
        return render_fatal(ctx);
    };
    match &model.onboarding {
        OnboardingModel::Checking => html! { <Loading /> },
        OnboardingModel::Setup {
            default_path,
            chosen_path,
            error,
            saving,
        } => html! {
            <WelcomeScreen
                default_path={default_path.clone()}
                chosen_path={chosen_path.clone()}
                error={error.clone()}
                saving={*saving}
                on_pick_folder={ctx.link().callback(|_| msg::Msg::Onboarding(onboarding::Msg::PickFolder))}
                on_complete={ctx.link().callback(|_| msg::Msg::Onboarding(onboarding::Msg::CompleteSetup))}
            />
        },
        OnboardingModel::Ready => html! {
            <AppLayout
                current_view={model.view.clone()}
                on_switch_view={ctx.link().callback(msg::Msg::SwitchView)}
                sticky_header={render_sticky_header(model)}
            >
                {render_content(model, api, ctx)}
            </AppLayout>
        },
    }
}

fn render_sticky_header(model: &AppModel) -> Html {
    match model.view {
        View::Plan => render_plan_sticky_header(model),
        View::History => html! {
            <h2 class="text-2xl font-bold mb-2 text-center">
                { "История распределений" }
            </h2>
        },
    }
}

fn render_plan_sticky_header(model: &AppModel) -> Html {
    let (view_model, edit_state) = match &model.plan {
        PlanModel::Editing { edit, .. } => {
            let plan = edit.core_plan.as_ref().map(Plan::from);
            (plan, Some(edit))
        }
        PlanModel::Creating { edit } => {
            let plan = edit.core_plan.as_ref().map(Plan::from);
            (plan, Some(edit))
        }
        PlanModel::Viewing { origin } => (Some(Plan::from(&origin.plan)), None),
        _ => (None, None),
    };

    let Some(view_model) = view_model else {
        return html! {};
    };

    let (bar_class, bar_content) = if let Some(edit) = edit_state {
        render_validation_bar(edit)
    } else {
        ("".to_string(), html! {})
    };

    html! {
        <>
            <Totals
                total_income={view_model.total_income.clone()}
                total_expenses={view_model.total_expenses.clone()}
                balance={view_model.balance.clone()}
            />
            {
                if edit_state.is_some() && !bar_class.is_empty() {
                    html! {
                        <div class={format!("mt-3 rounded-box px-4 py-2 {}", bar_class)}>
                            { bar_content }
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </>
    }
}

fn render_content(model: &AppModel, api: &Rc<ApiClient>, ctx: &Context<App>) -> Html {
    match model.view {
        View::Plan => render_plan_content(model, api, ctx),
        View::History => render_history_content(model, ctx),
    }
}

fn render_plan_content(
    model: &AppModel,
    api: &Rc<ApiClient>,
    ctx: &Context<App>,
) -> Html {
    match &model.plan {
        PlanModel::Loading => html! { <Loading /> },
        PlanModel::Error(error) => html! {
            <Error
                message={format!("Ошибка: {}", error)}
                on_retry={ctx.link().callback(|_| msg::Msg::Plan(LoadingMsg::Reload.into()))}
            />
        },
        PlanModel::SelectingTemplate { templates } => {
            render_template_selection(templates, ctx)
        }
        PlanModel::Creating { edit } => render_plan_edit_mode(edit, true, ctx),
        PlanModel::Viewing { origin } => {
            let plan = Plan::from(&origin.plan);
            render_plan_view_mode(&plan, api, ctx)
        }
        PlanModel::Editing { edit, .. } => render_plan_edit_mode(edit, false, ctx),
    }
}

fn render_plan_view_mode(
    view_model: &Plan,
    api: &Rc<ApiClient>,
    ctx: &Context<App>,
) -> Html {
    html! {
        <div class="space-y-4">
            <div class="flex justify-end">
                <button
                    class="btn btn-outline btn-sm"
                    onclick={ctx.link().callback(|_| msg::Msg::Plan(EditMsg::Enter.into()))}
                >
                    { "Редактировать план" }
                </button>
            </div>
            <PlanView
                view_model={view_model.clone()}
                on_plan_updated={ctx.link().callback(|_| msg::Msg::Plan(LoadingMsg::Reload.into()))}
                api={api.clone()}
            />
        </div>
    }
}

fn render_template_selection(
    templates: &DataState<Vec<crate::api::Collection>>,
    ctx: &Context<App>,
) -> Html {
    match templates {
        DataState::Loading => html! { <Loading /> },
        DataState::Error(error) => html! {
            <Error
                message={format!("Ошибка загрузки шаблонов: {}", error)}
                on_retry={ctx.link().callback(|_| msg::Msg::Plan(LoadingMsg::Reload.into()))}
            />
        },
        DataState::Loaded(collections) => html! {
            <TemplateSelector
                collections={collections.clone()}
                on_select={ctx.link().callback(|p| msg::Msg::Plan(TemplateMsg::Select(p).into()))}
                on_create_empty={ctx.link().callback(|_| msg::Msg::Plan(TemplateMsg::CreateFromScratch.into()))}
            />
        },
    }
}

fn render_plan_edit_mode(
    edit: &EditState,
    is_creating: bool,
    ctx: &Context<App>,
) -> Html {
    let disable_save = !matches!(edit.save_state, SaveState::CanSave);
    let on_save = if is_creating {
        ctx.link()
            .callback(|_| msg::Msg::Plan(PersistMsg::Create.into()))
    } else {
        ctx.link()
            .callback(|_| msg::Msg::Plan(PersistMsg::Save.into()))
    };
    let on_cancel = if is_creating {
        ctx.link()
            .callback(|_| msg::Msg::Plan(TemplateMsg::Back.into()))
    } else {
        ctx.link()
            .callback(|_| msg::Msg::Plan(EditMsg::Cancel.into()))
    };
    let total_income = edit.core_plan.as_ref().map(|p| p.total_incomes().value);
    html! {
        <EditLayout
            incomes={edit.incomes.clone()}
            expenses={edit.expenses.clone()}
            total_income={total_income}
            disable_save={disable_save}
            on_cancel={on_cancel}
            on_save={on_save}
            on_incomes_change={ctx
                .link()
                .callback(|v| msg::Msg::Plan(EditMsg::IncomesChanged(v).into()))}
            on_expenses_change={ctx
                .link()
                .callback(|v| msg::Msg::Plan(EditMsg::ExpensesChanged(v).into()))}
        />
    }
}

fn render_history_content(model: &AppModel, ctx: &Context<App>) -> Html {
    let data = &model.history.data;

    if let PageStatus::Error(error) = &data.status {
        return html! {
            <Error
                message={format!("Ошибка: {}", error)}
                on_retry={ctx.link().callback(|_| msg::Msg::History(history::Msg::Load))}
            />
        };
    }

    if data.items.is_empty() && data.is_loading() {
        return html! { <Loading /> };
    }

    if data.items.is_empty() {
        return html! {
            <div class="flex flex-col items-center justify-center py-20 gap-4">
                <p class="text-4xl">{"🏛"}</p>
                <h3 class="text-xl font-semibold text-base-content/70">
                    {"Каждое великое состояние начиналось с первого решения"}
                </h3>
                <p class="text-sm text-base-content/40 max-w-md text-center">
                    {"Распределите первый доход — и история ваших финансовых решений начнётся здесь"}
                </p>
            </div>
        };
    }

    html! {
        <>
            <HistoryView entries={data.items.clone()} />
            {if data.is_loading() {
                html! {
                    <div class="text-center mt-4">
                        <span class="loading loading-spinner loading-md"></span>
                    </div>
                }
            } else if data.next_cursor.is_some() {
                html! {
                    <div class="text-center mt-4">
                        <button
                            class="btn btn-primary"
                            onclick={ctx.link().callback(|_| msg::Msg::History(history::Msg::Load))}
                        >
                            { "Загрузить еще" }
                        </button>
                    </div>
                }
            } else {
                html! {}
            }}
        </>
    }
}

fn render_validation_bar(edit: &EditState) -> (String, Html) {
    match &edit.validation {
        PlanValidation::Valid => match edit.save_state {
            SaveState::Idle => (
                "bg-base-200 text-base-content".to_string(),
                html! { <span class="text-sm text-base-content/70">
                    { "Внесите первое изменение" }
                </span> },
            ),
            SaveState::CanSave => (
                "bg-success text-success-content".to_string(),
                html! { <span class="text-sm">
                    { "План валиден, можно сохранить изменения" }
                </span> },
            ),
            SaveState::Saving => (
                "bg-info text-info-content".to_string(),
                html! {
                    <span class="flex items-center gap-2 text-sm">
                        <span class="loading loading-spinner loading-xs"></span>
                        { "Сохранение..." }
                    </span>
                },
            ),
            SaveState::Disabled => {
                ("bg-base-200 text-base-content".to_string(), html! {})
            }
        },
        PlanValidation::FormatInvalid { messages } => {
            if messages.len() > 3 {
                (
                    "bg-error text-error-content".to_string(),
                    html! { <span class="text-sm">
                        { "Введены некорректные значения" }
                    </span> },
                )
            } else {
                (
                    "bg-error text-error-content".to_string(),
                    html! {
                        <div class="flex flex-col gap-1">
                            { for messages.iter().map(|m| html! {
                                <span class="text-sm">{ m }</span>
                            }) }
                        </div>
                    },
                )
            }
        }
        PlanValidation::BusinessInvalid { messages } => (
            "bg-error text-error-content".to_string(),
            html! {
                <div class="flex flex-wrap gap-2">
                    { for messages.iter().map(|m| html! {
                        <span class="badge badge-error badge-sm text-xs">
                            { m }
                        </span>
                    }) }
                </div>
            },
        ),
    }
}

fn render_fatal(ctx: &Context<App>) -> Html {
    html! {
        <div class="flex flex-col items-center justify-center min-h-screen gap-4 p-8">
            <p class="text-lg text-base-content/70 text-center max-w-md">
                {"Мы считали, что это невозможно, но вы всё же сломали этот софт"}
            </p>
            <button
                class="btn btn-primary"
                onclick={ctx.link().callback(|_| {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().reload();
                    }
                    msg::Msg::SwitchView(View::Plan)
                })}
            >
                {"Перезагрузить"}
            </button>
        </div>
    }
}
