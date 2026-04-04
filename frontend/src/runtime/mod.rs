use std::rc::Rc;

use yew::{Component, Context, Html, html};

use crate::{
    api::{ApiClient, BudgetEntry, Cursor, Page},
    config::API_V1_BASE_URL,
    engine::plan::{model::PlanModel, msg::PlanMsg},
    presentation::{
        components::{AppLayout, WelcomeScreen},
        history::HistoryEntry,
    },
};

mod history;
mod onboarding;
mod plan;
mod view;

#[derive(Clone, PartialEq)]
#[cfg_attr(not(feature = "tauri"), allow(dead_code))]
pub enum AppPhase {
    /// Checking if app is configured (Tauri only)
    Checking,
    /// First run — show welcome/onboarding screen
    Onboarding {
        default_path: String,
        chosen_path: Option<String>,
        error: Option<String>,
        saving: bool,
    },
    /// App is ready — backend running, show main UI
    Ready,
}

#[derive(Clone, PartialEq)]
pub enum View {
    Plan,
    History,
}

#[derive(Clone, PartialEq)]
enum PaginatableDataState<T> {
    Loading,
    Loaded {
        items: Vec<T>,
        next_cursor: Option<Cursor>,
    },
    LoadingMore {
        items: Vec<T>,
        next_cursor: Option<Cursor>,
    },
    Error(String),
}

impl<T> PaginatableDataState<T> {
    fn is_paginating(&self) -> bool {
        matches!(self, Self::LoadingMore { .. })
    }
}

struct HistoryState {
    data: PaginatableDataState<HistoryEntry>,
}

impl HistoryState {
    fn set_data(&mut self, data: PaginatableDataState<HistoryEntry>) {
        self.data = data;
    }

    fn prepare_load(&mut self) -> Option<Cursor> {
        match &self.data {
            PaginatableDataState::Loaded { items, next_cursor } => {
                let items = items.clone();
                let next_cursor = next_cursor.clone();
                self.set_data(PaginatableDataState::LoadingMore {
                    items,
                    next_cursor: next_cursor.clone(),
                });
                next_cursor
            }
            PaginatableDataState::LoadingMore { next_cursor, .. } => {
                next_cursor.clone()
            }
            _ => {
                self.set_data(PaginatableDataState::Loading);
                None
            }
        }
    }

    fn merge_page(&mut self, page: Page<BudgetEntry>) {
        let new_entries: Vec<HistoryEntry> =
            page.items.iter().map(HistoryEntry::from).collect();

        let was_loading_more =
            matches!(self.data, PaginatableDataState::LoadingMore { .. });

        match &mut self.data {
            PaginatableDataState::Loading => {
                self.set_data(PaginatableDataState::Loaded {
                    items: new_entries,
                    next_cursor: page.next_cursor,
                });
            }
            PaginatableDataState::Loaded { items, next_cursor }
            | PaginatableDataState::LoadingMore { items, next_cursor } => {
                items.extend(new_entries);
                *next_cursor = page.next_cursor.clone();

                // Переводим из LoadingMore в Loaded
                if was_loading_more {
                    let items = items.clone();
                    let next_cursor = next_cursor.clone();
                    self.set_data(PaginatableDataState::Loaded { items, next_cursor });
                }
            }
            _ => {}
        }
    }
}

pub struct App {
    phase: AppPhase,
    view: View,
    plan: PlanModel,
    history: HistoryState,
    api: Rc<ApiClient>,
}

#[cfg_attr(not(feature = "tauri"), allow(dead_code))]
pub enum OnboardingMsg {
    PhaseResolved(AppPhase),
    PickFolder,
    FolderPicked(Option<String>),
    CompleteSetup,
    SetupFinished(Result<(), String>),
}

pub enum AppMsg {
    Onboarding(OnboardingMsg),
    SwitchView(View),
    Plan(PlanMsg),
    LoadHistory,
    HistoryLoaded(Result<Page<BudgetEntry>, String>),
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let initial_phase = onboarding::resolve_initial_phase(ctx);

        Self {
            phase: initial_phase,
            view: View::Plan,
            plan: PlanModel::Loading,
            history: HistoryState {
                data: PaginatableDataState::Loading,
            },
            api: Rc::new(ApiClient::new(API_V1_BASE_URL.clone())),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::Onboarding(msg) => onboarding::handle(self, ctx, msg),
            AppMsg::SwitchView(view) => self.handle_switch_view(ctx, view),
            AppMsg::Plan(plan_msg) => {
                let (new_model, cmds) =
                    crate::engine::plan::update::handle(&self.plan, plan_msg);
                self.plan = new_model;
                self.execute_plan_cmds(cmds, ctx);
                true
            }
            AppMsg::LoadHistory => self.handle_load_history(ctx),
            AppMsg::HistoryLoaded(result) => self.handle_history_loaded(result),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.phase {
            AppPhase::Checking => html! {
                <crate::presentation::components::Loading />
            },
            AppPhase::Onboarding {
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
                    on_pick_folder={ctx.link().callback(|_| AppMsg::Onboarding(OnboardingMsg::PickFolder))}
                    on_complete={ctx.link().callback(|_| AppMsg::Onboarding(OnboardingMsg::CompleteSetup))}
                />
            },
            AppPhase::Ready => html! {
                <AppLayout
                    current_view={self.view.clone()}
                    on_switch_view={ctx.link().callback(AppMsg::SwitchView)}
                    sticky_header={self.render_sticky_header()}
                >
                    {self.render_content(ctx)}
                </AppLayout>
            },
        }
    }
}

impl App {}
