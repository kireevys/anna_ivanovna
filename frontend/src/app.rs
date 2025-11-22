use crate::api;
use ai_core::editor::Plan;
use yew::{Component, Context, Html, html};

pub struct App {
    plan: Option<Plan>,
    loading: bool,
    error: Option<String>,
}

pub enum AppMsg {
    LoadPlan,
    PlanLoaded(Result<Plan, String>),
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(AppMsg::LoadPlan);
        Self {
            plan: None,
            loading: true,
            error: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::LoadPlan => {
                self.loading = true;
                self.error = None;
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = api::get_plan().await;
                    link.send_message(AppMsg::PlanLoaded(result));
                });
                true
            }
            AppMsg::PlanLoaded(Ok(plan)) => {
                self.plan = Some(plan);
                self.loading = false;
                true
            }
            AppMsg::PlanLoaded(Err(e)) => {
                self.error = Some(e);
                self.loading = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <h1>{ "Anna Ivanovna - План бюджета" }</h1>
                {self.render_content(ctx)}
            </div>
        }
    }
}

impl App {
    fn render_content(&self, ctx: &Context<Self>) -> Html {
        if self.loading {
            return html! { <p>{ "Загрузка..." }</p> };
        }

        if let Some(ref error) = self.error {
            return html! {
                <div>
                    <p style="color: red;">{ format!("Ошибка: {}", error) }</p>
                    <button onclick={ctx.link().callback(|_| AppMsg::LoadPlan)}>
                        { "Повторить" }
                    </button>
                </div>
            };
        }

        if let Some(ref plan) = self.plan {
            html! {
                <div>
                    <h2>{ "Источники дохода" }</h2>
                    <ul>
                        {for plan.sources.values().map(|source| {
                            html! {
                                <li>{ format!("{}: {} {}", source.name, source.expected.value, source.expected.currency) }</li>
                            }
                        })}
                    </ul>
                    <h2>{ "Расходы" }</h2>
                    <ul>
                        {for plan.expenses.values().map(|expense| {
                            html! {
                                <li>
                                    { format!("{}: {:?}", expense.name, expense.value) }
                                    {if let Some(ref cat) = expense.category {
                                        html! { <span>{ format!(" [{}]", cat) }</span> }
                                    } else {
                                        html! {}
                                    }}
                                </li>
                            }
                        })}
                    </ul>
                </div>
            }
        } else {
            html! { <p>{ "План не найден" }</p> }
        }
    }
}
