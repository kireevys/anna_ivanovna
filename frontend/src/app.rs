use crate::{
    api::{ApiClient, BudgetEntry, Cursor, Page},
    components::{AppLayout, Error, HistoryView, Loading, PlanView},
    config::API_V1_BASE_URL,
    presentation::{history::HistoryEntry, plan::Plan},
};
use ai_core::plan::Plan as CorePlan;
use std::rc::Rc;
use yew::{Component, Context, Html, html};

#[derive(Clone, PartialEq)]
pub enum View {
    Plan,
    History,
}

#[derive(Clone, PartialEq)]
enum DataState<T> {
    Loading,
    Loaded(T),
    Error(String),
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

struct PlanState {
    data: DataState<Plan>,
}

impl PlanState {
    fn set_data(&mut self, data: DataState<Plan>) {
        self.data = data;
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
    view: View,
    plan: PlanState,
    history: HistoryState,
    api: Rc<ApiClient>,
}

pub enum AppMsg {
    SwitchView(View),
    LoadPlan,
    PlanLoaded(Result<CorePlan, String>),
    LoadHistory,
    HistoryLoaded(Result<Page<BudgetEntry>, String>),
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(AppMsg::LoadPlan);
        Self {
            view: View::Plan,
            plan: PlanState {
                data: DataState::Loading,
            },
            history: HistoryState {
                data: PaginatableDataState::Loading,
            },
            api: Rc::new(ApiClient::new(API_V1_BASE_URL.clone())),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::SwitchView(view) => {
                self.view = view;
                if self.view == View::History {
                    // Всегда загружаем историю заново при переключении на вкладку
                    self.history.set_data(PaginatableDataState::Loading);
                    ctx.link().send_message(AppMsg::LoadHistory);
                }
                true
            }
            AppMsg::LoadPlan => {
                self.plan.set_data(DataState::Loading);
                self.load_plan_async(ctx.link());
                true
            }
            AppMsg::PlanLoaded(Ok(plan)) => {
                self.plan.set_data(DataState::Loaded(Plan::from(&plan)));
                true
            }
            AppMsg::PlanLoaded(Err(e)) => {
                self.plan.set_data(DataState::Error(e));
                true
            }
            AppMsg::LoadHistory => {
                let cursor = self.history.prepare_load();
                self.load_history_async(cursor, ctx.link());
                true
            }
            AppMsg::HistoryLoaded(Ok(page)) => {
                self.history.merge_page(page);
                true
            }
            AppMsg::HistoryLoaded(Err(e)) => {
                self.history.set_data(PaginatableDataState::Error(e));
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <AppLayout
                current_view={self.view.clone()}
                on_switch_view={ctx.link().callback(AppMsg::SwitchView)}
            >
                {self.render_content(ctx)}
            </AppLayout>
        }
    }
}

impl App {
    fn load_plan_async(&self, link: &yew::html::Scope<Self>) {
        let api = self.api.clone();
        let link = link.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.get_plan().await.map_err(|e| e.to_string());
            link.send_message(AppMsg::PlanLoaded(result));
        });
    }

    fn load_history_async(
        &self,
        cursor: Option<Cursor>,
        link: &yew::html::Scope<Self>,
    ) {
        let api = self.api.clone();
        let link = link.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = api.get_history(cursor).await.map_err(|e| e.to_string());
            link.send_message(AppMsg::HistoryLoaded(result));
        });
    }

    fn render_content(&self, ctx: &Context<Self>) -> Html {
        match self.view {
            View::Plan => match &self.plan.data {
                DataState::Loading => html! { <Loading /> },
                DataState::Loaded(view_model) => html! {
                    <PlanView
                        view_model={view_model.clone()}
                        on_plan_updated={ctx.link().callback(|_| AppMsg::LoadPlan)}
                        api={self.api.clone()}
                    />
                },
                DataState::Error(error) => html! {
                    <Error
                        message={format!("Ошибка: {}", error)}
                        on_retry={ctx.link().callback(|_| AppMsg::LoadPlan)}
                    />
                },
            },
            View::History => match &self.history.data {
                PaginatableDataState::Loading => html! { <Loading /> },
                PaginatableDataState::Error(error) => html! {
                    <Error
                        message={format!("Ошибка: {}", error)}
                        on_retry={ctx.link().callback(|_| AppMsg::LoadHistory)}
                    />
                },
                PaginatableDataState::Loaded { items, next_cursor }
                | PaginatableDataState::LoadingMore { items, next_cursor } => {
                    html! {
                        <>
                            <HistoryView entries={items.clone()} />
                            {if self.history.data.is_paginating() {
                                html! { <div class="text-center mt-4"><span class="loading loading-spinner loading-md"></span></div> }
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
            },
        }
    }
}
