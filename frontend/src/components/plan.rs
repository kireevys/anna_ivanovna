use crate::components::IncomeModal;
use crate::presentation::plan::Plan;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PlanProps {
    pub view_model: Plan,
    pub on_plan_updated: Callback<()>,
}

pub enum PlanViewMsg {
    OpenModal(String),
    CloseModal,
    Saved,
}

pub struct PlanView {
    modal_source_id: Option<String>,
}

impl Component for PlanView {
    type Message = PlanViewMsg;
    type Properties = PlanProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { modal_source_id: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PlanViewMsg::OpenModal(source_id) => {
                self.modal_source_id = Some(source_id);
                true
            }
            PlanViewMsg::CloseModal => {
                self.modal_source_id = None;
                true
            }
            PlanViewMsg::Saved => {
                self.modal_source_id = None;
                ctx.props().on_plan_updated.emit(());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="space-y-6">
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <h2 class="card-title text-2xl mb-4 text-center justify-center text-success">
                            { "Доходы" }
                        </h2>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            {for ctx.props().view_model.sources.iter().map(|source| {
                                let source_id = source.id.clone();
                                html! {
                                    <div class="card bg-base-200 shadow">
                                        <div class="card-body p-4">
                                            <div class="flex justify-between items-center">
                                                <div>
                                                    <h3 class="font-semibold text-lg">{ &source.name }</h3>
                                                    <p class="text-2xl font-bold text-primary">
                                                        { &source.amount }
                                                    </p>
                                                </div>
                                                <button
                                                    class="btn btn-primary btn-sm"
                                                    onclick={ctx.link().callback(move |_| PlanViewMsg::OpenModal(source_id.clone()))}
                                                >
                                                    { "Поступление" }
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                        {if let Some(source_id) = &self.modal_source_id {
                            html! {
                                <IncomeModal
                                    source_id={source_id.clone()}
                                    on_close={ctx.link().callback(|_| PlanViewMsg::CloseModal)}
                                    on_saved={ctx.link().callback(|_| PlanViewMsg::Saved)}
                                />
                            }
                        } else {
                            html! {}
                        }}
                        <div class="divider"></div>
                        <div class="stats shadow w-full">
                            <div class="stat">
                                <div class="stat-title">{ "Доходы" }</div>
                                <div class="stat-value text-success text-2xl">
                                    { &ctx.props().view_model.total_income }
                                </div>
                            </div>
                            <div class="stat">
                                <div class="stat-title">{ "Расходы" }</div>
                                <div class="stat-value text-error text-2xl">
                                    { &ctx.props().view_model.total_expenses }
                                </div>
                            </div>
                            <div class="stat">
                                <div class="stat-title">{ "Остаток" }</div>
                                <div class="stat-value text-2xl text-warning">
                                    { &ctx.props().view_model.balance }
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                // Расходы по категориям (Pricing Cards)
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <h2 class="card-title text-2xl mb-6 text-center justify-center text-error">{ "Расходы" }</h2>
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                            {for ctx.props().view_model.categories.iter().map(|category| {
                                html! {
                                    <div class="card bg-base-200 shadow">
                                        <div class="card-body">
                                            <h3 class="card-title text-xl mb-4">{ &category.name }</h3>
                                            <div class="space-y-2">
                                                {for category.expenses.iter().map(|expense| {
                                                    let display_name = expense.name.clone();
                                                    let full_name = expense.name.clone();
                                                    html! {
                                                        <div class="relative group">
                                                            <div class="flex justify-between items-center p-2 bg-base-100 rounded gap-2 max-w-xs">
                                                                <span
                                                                    class="font-medium truncate flex-1 min-w-0"
                                                                    title={display_name.clone()}
                                                                >
                                                                    { &expense.name }
                                                                </span>
                                                                <span class="font-bold flex-shrink-0">{ &expense.value }</span>
                                                            </div>
                                                            <div class="absolute left-0 bottom-full mb-2 hidden group-hover:block bg-gray-800 text-white text-xs rounded py-1 px-2 z-10 whitespace-pre-wrap max-w-xs">
                                                                { full_name }
                                                                <div class="absolute left-1/2 -translate-x-1/2 top-full border-l-4 border-r-4 border-t-4 border-t-gray-800 border-l-transparent border-r-transparent w-0 h-0"></div>
                                                            </div>
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
                </div>
            </div>
        }
    }
}
