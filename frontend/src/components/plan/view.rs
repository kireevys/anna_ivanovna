use std::rc::Rc;

use yew::prelude::*;

use crate::{
    api::ApiClient,
    components::plan::{ExpenseCategories, IncomeSources},
    presentation::plan::Plan,
};

#[derive(Properties, PartialEq)]
pub struct PlanProps {
    pub view_model: Plan,
    pub on_plan_updated: Callback<()>,
    pub api: Rc<ApiClient>,
}

pub struct PlanView;

impl Component for PlanView {
    type Message = ();
    type Properties = PlanProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="space-y-6">
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <h2 class="card-title text-2xl mb-4 text-center justify-center text-success">
                            { "Доходы" }
                        </h2>
                        <IncomeSources
                            sources={ctx.props().view_model.sources.clone()}
                            on_saved={ctx.props().on_plan_updated.clone()}
                            api={ctx.props().api.clone()}
                        />
                    </div>
                </div>

                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body">
                        <h2 class="card-title text-2xl mb-6 text-center justify-center text-error">
                            { "Расходы" }
                        </h2>
                        <ExpenseCategories categories={ctx.props().view_model.categories.clone()} />
                    </div>
                </div>
            </div>
        }
    }
}
