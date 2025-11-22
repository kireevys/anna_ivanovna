use crate::api;
use crate::components::{Error, HistoryView, Loading, PlanView, ThemeSwitcher};
use crate::presentation::history::HistoryEntry;
use crate::presentation::plan::Plan;
use ai_core::api::{Cursor, Page, StorageBudget};
use ai_core::editor::Plan as CorePlan;
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

struct PlanState {
    data: DataState<Plan>,
}

struct HistoryState {
    data: PaginatableDataState<HistoryEntry>,
}

pub struct App {
    view: View,
    plan: PlanState,
    history: HistoryState,
}

pub enum AppMsg {
    SwitchView(View),
    LoadPlan,
    PlanLoaded(Result<CorePlan, String>),
    LoadHistory,
    HistoryLoaded(Result<Page<StorageBudget>, String>),
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
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::SwitchView(view) => {
                self.view = view.clone();
                if view == View::History && matches!(&self.history.data, PaginatableDataState::Loading) {
                    ctx.link().send_message(AppMsg::LoadHistory);
                }
                true
            }
            AppMsg::LoadPlan => {
                self.plan.data = DataState::Loading;
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = api::get_plan().await;
                    link.send_message(AppMsg::PlanLoaded(result));
                });
                true
            }
            AppMsg::PlanLoaded(Ok(plan)) => {
                self.plan.data = DataState::Loaded(Plan::from(&plan));
                true
            }
            AppMsg::PlanLoaded(Err(e)) => {
                self.plan.data = DataState::Error(e);
                true
            }
            AppMsg::LoadHistory => {
                let cursor = match self.history.data.clone() {
                    PaginatableDataState::Loaded { items, next_cursor } => {
                        self.history.data = PaginatableDataState::LoadingMore {
                            items,
                            next_cursor: next_cursor.clone(),
                        };
                        next_cursor
                    }
                    PaginatableDataState::LoadingMore { next_cursor, .. } => next_cursor,
                    _ => {
                        self.history.data = PaginatableDataState::Loading;
                        None
                    }
                };

                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = api::get_history(cursor).await;
                    link.send_message(AppMsg::HistoryLoaded(result));
                });
                true
            }
            AppMsg::HistoryLoaded(Ok(page)) => {
                let new_entries: Vec<HistoryEntry> =
                    page.items.iter().map(HistoryEntry::from).collect();

                match &mut self.history.data {
                    PaginatableDataState::Loading => {
                        self.history.data = PaginatableDataState::Loaded {
                            items: new_entries,
                            next_cursor: page.next_cursor,
                        };
                    }
                    PaginatableDataState::Loaded { items, next_cursor }
                    | PaginatableDataState::LoadingMore { items, next_cursor } => {
                        items.extend(new_entries);
                        *next_cursor = page.next_cursor;
                        self.history.data = PaginatableDataState::Loaded {
                            items: items.clone(),
                            next_cursor: next_cursor.clone(),
                        };
                    }
                    _ => {}
                }
                true
            }
            AppMsg::HistoryLoaded(Err(e)) => {
                self.history.data = PaginatableDataState::Error(e);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="min-h-screen bg-base-200">
                <div class="container mx-auto px-4 py-8">
                    <div class="flex justify-between items-center mb-8">
                        <h1 class="text-4xl font-bold">
                            { "Anna Ivanovna" }
                        </h1>
                        <ThemeSwitcher />
                    </div>
                    <div class="tabs tabs-boxed mb-6">
                        <button
                            class={format!("tab {}", if self.view == View::Plan { "tab-active" } else { "" })}
                            onclick={ctx.link().callback(|_| AppMsg::SwitchView(View::Plan))}
                        >
                            { "План" }
                        </button>
                        <button
                            class={format!("tab {}", if self.view == View::History { "tab-active" } else { "" })}
                            onclick={ctx.link().callback(|_| AppMsg::SwitchView(View::History))}
                        >
                            { "История" }
                        </button>
                    </div>
                    {self.render_content(ctx)}
                </div>
            </div>
        }
    }
}

impl App {
    fn render_content(&self, ctx: &Context<Self>) -> Html {
        match self.view {
            View::Plan => match &self.plan.data {
                DataState::Loading => html! { <Loading /> },
                DataState::Loaded(view_model) => html! {
                    <PlanView
                        view_model={view_model.clone()}
                        on_plan_updated={ctx.link().callback(|_| AppMsg::LoadPlan)}
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
                            {if matches!(&self.history.data, PaginatableDataState::LoadingMore { .. }) {
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
