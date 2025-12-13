use crate::api::{ApiClient, ApiError, AddIncomeRequest};
use std::rc::Rc;
use crate::presentation::history::HistoryEntry;
use ai_core::api::StorageBudget;
use ai_core::distribute::Budget;
use chrono::{Duration, Local, NaiveDate};
use rust_decimal::Decimal;
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct IncomeModalProps {
    pub on_close: Callback<()>,
    pub on_saved: Callback<()>,
    pub source_id: String,
    pub api: Rc<ApiClient>,
}

pub enum IncomeModalMsg {
    SetAmount(String),
    SetDate(String),
    Calculate,
    Calculated(Result<Budget, String>),
    Save,
    Saved(Result<String, String>),
    Close,
}

pub struct IncomeModal {
    amount: String,
    date: NaiveDate,
    state: IncomeModalState,
}

#[derive(Clone, PartialEq)]
enum IncomeModalState {
    Input,
    Calculating,
    Result(Budget),
    Saving,
    Saved,
    Error(String),
}

impl Component for IncomeModal {
    type Message = IncomeModalMsg;
    type Properties = IncomeModalProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            amount: String::new(),
            date: Local::now().date_naive(),
            state: IncomeModalState::Input,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            IncomeModalMsg::SetAmount(amount) => {
                self.amount = amount;
                true
            }
            IncomeModalMsg::SetDate(date_str) => {
                if let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                    self.date = date;
                }
                true
            }
            IncomeModalMsg::Calculate => {
                if let Ok(amount) = self.amount.parse::<Decimal>() {
                    self.state = IncomeModalState::Calculating;
                    let api = ctx.props().api.clone();
                    let request = AddIncomeRequest {
                        source_id: ctx.props().source_id.clone(),
                        amount,
                        date: self.date,
                    };
                    let link = ctx.link().clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let result = api.add_income(request).await.map_err(|e: ApiError| e.to_string());
                        link.send_message(IncomeModalMsg::Calculated(result));
                    });
                    true
                } else {
                    self.state = IncomeModalState::Error("Неверная сумма".to_string());
                    true
                }
            }
            IncomeModalMsg::Calculated(Ok(budget)) => {
                self.state = IncomeModalState::Result(budget);
                true
            }
            IncomeModalMsg::Calculated(Err(e)) => {
                self.state = IncomeModalState::Error(e);
                true
            }
            IncomeModalMsg::Save => {
                let budget = match &self.state {
                    IncomeModalState::Result(budget) => budget.clone(),
                    _ => return false,
                };
                self.state = IncomeModalState::Saving;
                let api = ctx.props().api.clone();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = api.save_budget(&budget).await.map_err(|e: ApiError| e.to_string());
                    link.send_message(IncomeModalMsg::Saved(result));
                });
                true
            }
            IncomeModalMsg::Saved(Ok(budget_id)) => {
                self.state = IncomeModalState::Saved;
                ctx.props().on_saved.emit(());
                // Показываем toast с ID
                Self::show_toast(&budget_id);
                true
            }
            IncomeModalMsg::Saved(Err(e)) => {
                self.state = IncomeModalState::Error(e);
                true
            }
            IncomeModalMsg::Close => {
                ctx.props().on_close.emit(());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let date_str = self.date.format("%Y-%m-%d").to_string();

        html! {
            <div class="modal modal-open">
                <div class="modal-box max-w-4xl">
                    <h3 class="font-bold text-lg mb-4">{ "Поступление дохода" }</h3>

                    <div class="form-control w-full mb-4">
                        <div class="flex gap-2">
                            <input
                                type="number"
                                step="0.01"
                                placeholder="Введите сумму (₽)"
                                class="input input-bordered flex-1"
                                value={self.amount.clone()}
                                oninput={ctx.link().callback(|e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    IncomeModalMsg::SetAmount(input.value())
                                })}
                            />
                            <input
                                type="date"
                                class="input input-bordered"
                                value={date_str}
                                oninput={ctx.link().callback(|e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    IncomeModalMsg::SetDate(input.value())
                                })}
                            />
                            <button
                                class="btn btn-success"
                                onclick={ctx.link().callback(|_| IncomeModalMsg::Calculate)}
                                disabled={self.amount.is_empty() || matches!(self.state, IncomeModalState::Calculating | IncomeModalState::Saving)}
                            >
                                { "Посчитать" }
                            </button>
                        </div>
                    </div>

                    {match &self.state {
                        IncomeModalState::Calculating => html! {
                            <div class="flex justify-center items-center py-8">
                                <span class="loading loading-spinner loading-lg"></span>
                            </div>
                        },
                        IncomeModalState::Result(budget) => html! {
                            <div class="collapse collapse-open">
                                <div class="collapse-content p-0">
                                    {self.render_result(ctx, budget)}
                                </div>
                            </div>
                        },
                        IncomeModalState::Saving => html! {
                            <div class="flex justify-center items-center py-8">
                                <span class="loading loading-spinner loading-lg"></span>
                                <span class="ml-4">{ "Сохранение..." }</span>
                            </div>
                        },
                        IncomeModalState::Saved => html! {},
                        IncomeModalState::Error(e) => html! {
                            <div class="alert alert-error">
                                <span>{ e }</span>
                            </div>
                        },
                        IncomeModalState::Input => html! {},
                    }}

                    <div class="modal-action">
                        <button
                            class="btn"
                            onclick={ctx.link().callback(|_| IncomeModalMsg::Close)}
                        >
                            { if matches!(self.state, IncomeModalState::Saved) { "Закрыть" } else { "Отмена" } }
                        </button>
                        {if matches!(self.state, IncomeModalState::Result(_)) {
                            html! {
                                <button
                                    class="btn btn-success"
                                    onclick={ctx.link().callback(|_| IncomeModalMsg::Save)}
                                >
                                    { "Сохранить" }
                                </button>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                </div>
            </div>
        }
    }
}

const TOAST_LIVE: Duration = Duration::seconds(2);

impl IncomeModal {
    fn show_toast(budget_id: &str) {
        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
        {
            // Создаем toast элемент
            let toast = document.create_element("div").unwrap();
            toast.set_class_name("toast toast-top toast-end");
            toast.set_inner_html(&format!(
                r#"
                <div class="alert alert-success shadow-lg">
                    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    <span>Сохранено распределение {}</span>
                </div>
                "#,
                budget_id
            ));

            // Добавляем в body
            if let Some(body) = document.body() {
                let _ = body.append_child(&toast);

                let closure = Closure::wrap(Box::new(move || {
                    toast.remove();
                }) as Box<dyn FnMut()>);
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    TOAST_LIVE.num_milliseconds() as i32,
                );
                closure.forget();
            }
        }
    }

    fn render_result(&self, _ctx: &Context<Self>, budget: &Budget) -> Html {
        let storage_budget = StorageBudget {
            id: String::new(),
            budget: budget.clone(),
        };
        let entry = HistoryEntry::from(&storage_budget);

        html! {
            <div class="space-y-4">
                <div class="card bg-base-100 shadow mb-6">
                    <div class="card-body p-4">
                        <div class="flex justify-between items-center">
                            <div>
                                <div class="text-sm text-base-content/70">{ "Доход" }</div>
                                <div class="text-2xl font-bold text-success">
                                    { entry.income_amount.to_string() }
                                </div>
                            </div>
                            <div class="text-right">
                                <div class="text-sm text-base-content/70">{ "Остаток" }</div>
                                <div class="text-2xl font-bold text-warning">
                                    { entry.rest.to_string() }
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {for entry.categories.iter().map(|category| {
                        html! {
                            <div class="card bg-base-200 shadow">
                                <div class="card-body p-4">
                                    <h4 class="font-semibold text-lg mb-2">{ &category.name }</h4>
                                    <div class="space-y-1">
                                        {for category.entries.iter().map(|expense| {
                                            html! {
                                                <div class="flex justify-between items-center text-sm">
                                                    <span>{ &expense.name }</span>
                                                    <span class="font-bold">{ expense.amount.to_string() }</span>
                                                </div>
                                            }
                                        })}
                                    </div>
                                </div>
                            </div>
                        }
                    })}
                </div>
            </div>
        }
    }
}
