use crate::{
    components::{
        EditLayout,
        Error,
        HistoryView,
        Loading,
        PlanView,
        TemplateSelector,
        Totals,
    },
    presentation::plan::read::Plan,
};
use yew::{Context, Html, html};

use super::{
    App,
    AppMsg,
    DataState,
    PaginatableDataState,
    PlanMode,
    PlanValidation,
    SaveState,
    View,
};

impl App {
    pub(crate) fn render_sticky_header(&self) -> Html {
        match self.view {
            View::Plan => {
                if let DataState::Loaded(view_model) = &self.plan.data {
                    let (bar_class, bar_content): (String, Html) = match &self
                        .plan
                        .validation
                    {
                        PlanValidation::Valid => {
                            if matches!(
                                self.plan.mode,
                                PlanMode::Edit | PlanMode::Creating
                            ) {
                                match self.plan.save_state {
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
                                    SaveState::Disabled => (
                                        "bg-base-200 text-base-content".to_string(),
                                        html! {},
                                    ),
                                }
                            } else {
                                ("".to_string(), html! {})
                            }
                        }
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
                    };

                    html! {
                        <>
                            <Totals
                                total_income={view_model.total_income.clone()}
                                total_expenses={view_model.total_expenses.clone()}
                                balance={view_model.balance.clone()}
                            />
                            {
                                if matches!(self.plan.mode, PlanMode::Edit | PlanMode::Creating) && !bar_class.is_empty() {
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
                } else {
                    html! {}
                }
            }
            View::History => html! {
                <h2 class="text-2xl font-bold mb-2 text-center">
                    { "История распределений" }
                </h2>
            },
        }
    }

    pub(crate) fn render_content(&self, ctx: &Context<Self>) -> Html {
        match self.view {
            View::Plan => self.render_plan_content(ctx),
            View::History => self.render_history_content(ctx),
        }
    }

    pub(crate) fn render_plan_content(&self, ctx: &Context<Self>) -> Html {
        if self.plan.mode == PlanMode::Creating {
            return match &self.plan.data {
                DataState::Loaded(_) => self.render_plan_edit_mode(ctx),
                _ => self.render_template_selection(ctx),
            };
        }

        match &self.plan.data {
            DataState::Loading => html! { <Loading /> },
            DataState::Error(error) => self.render_plan_error(ctx, error),
            DataState::Loaded(view_model) => self.render_plan_loaded(ctx, view_model),
        }
    }

    pub(crate) fn render_plan_error(&self, ctx: &Context<Self>, error: &str) -> Html {
        html! {
            <Error
                message={format!("Ошибка: {}", error)}
                on_retry={ctx.link().callback(|_| AppMsg::LoadPlan)}
            />
        }
    }

    pub(crate) fn render_plan_loaded(
        &self,
        ctx: &Context<Self>,
        view_model: &Plan,
    ) -> Html {
        match self.plan.mode {
            PlanMode::View => self.render_plan_view_mode(ctx, view_model),
            PlanMode::Edit | PlanMode::Creating => self.render_plan_edit_mode(ctx),
        }
    }

    pub(crate) fn render_plan_view_mode(
        &self,
        ctx: &Context<Self>,
        view_model: &Plan,
    ) -> Html {
        html! {
            <div class="space-y-4">
                <div class="flex justify-end">
                    <button
                        class="btn btn-outline btn-sm"
                        onclick={ctx.link().callback(|_| AppMsg::EnterEditMode)}
                    >
                        { "Редактировать план" }
                    </button>
                </div>
                <PlanView
                    view_model={view_model.clone()}
                    on_plan_updated={ctx.link().callback(|_| AppMsg::LoadPlan)}
                    api={self.api.clone()}
                />
            </div>
        }
    }

    pub(crate) fn render_template_selection(&self, ctx: &Context<Self>) -> Html {
        match &self.plan.templates {
            DataState::Loading => html! { <Loading /> },
            DataState::Error(error) => html! {
                <Error
                    message={format!("Ошибка загрузки шаблонов: {}", error)}
                    on_retry={ctx.link().callback(|_| AppMsg::LoadPlan)}
                />
            },
            DataState::Loaded(collections) => html! {
                <TemplateSelector
                    collections={collections.clone()}
                    on_select={ctx.link().callback(AppMsg::SelectTemplate)}
                    on_create_empty={ctx.link().callback(|_| AppMsg::CreateFromScratch)}
                />
            },
        }
    }

    pub(crate) fn render_plan_edit_mode(&self, ctx: &Context<Self>) -> Html {
        let disable_save = !matches!(self.plan.save_state, SaveState::CanSave);
        let is_new_plan = self.plan.meta.is_none();
        let on_save = if is_new_plan {
            ctx.link().callback(|_| AppMsg::CreatePlan)
        } else {
            ctx.link().callback(|_| AppMsg::SavePlan)
        };
        let on_cancel = if is_new_plan {
            ctx.link().callback(|_| AppMsg::BackToTemplates)
        } else {
            ctx.link().callback(|_| AppMsg::CancelEditMode)
        };
        let total_income = self
            .plan
            .edited_core_plan
            .as_ref()
            .map(|p| p.total_incomes().value);
        html! {
            <EditLayout
                incomes={self.plan.incomes.clone()}
                expenses={self.plan.expenses.clone()}
                total_income={total_income}
                disable_save={disable_save}
                on_cancel={on_cancel}
                on_save={on_save}
                on_incomes_change={ctx
                    .link()
                    .callback(AppMsg::IncomeSourcesChanged)}
                on_expenses_change={ctx
                    .link()
                    .callback(AppMsg::ExpensesChanged)}
            />
        }
    }

    pub(crate) fn render_history_content(&self, ctx: &Context<Self>) -> Html {
        match &self.history.data {
            PaginatableDataState::Loading => html! { <Loading /> },
            PaginatableDataState::Error(error) => html! {
                <Error
                    message={format!("Ошибка: {}", error)}
                    on_retry={ctx.link().callback(|_| AppMsg::LoadHistory)}
                />
            },
            PaginatableDataState::Loaded { items, next_cursor }
            | PaginatableDataState::LoadingMore { items, next_cursor } => {
                if items.is_empty() {
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
                        <HistoryView entries={items.clone()} />
                        {if self.history.data.is_paginating() {
                            html! {
                                <div class="text-center mt-4">
                                    <span class="loading loading-spinner loading-md"></span>
                                </div>
                            }
                        } else if next_cursor.is_some() {
                            html! {
                                <div class="text-center mt-4">
                                    <button
                                        class="btn btn-primary"
                                        onclick={ctx.link().callback(|_| AppMsg::LoadHistory)}
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
        }
    }
}
