use crate::api::ApiClient;
use crate::components::IncomeModal;
use crate::presentation::plan::IncomeSource;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct IncomeSourcesProps {
    pub sources: Vec<IncomeSource>,
    pub on_saved: Callback<()>,
    pub api: Rc<ApiClient>,
}

pub enum IncomeSourcesMsg {
    OpenModal(String),
    CloseModal,
    Saved,
}

pub struct IncomeSources {
    modal_source_id: Option<String>,
}

impl Component for IncomeSources {
    type Message = IncomeSourcesMsg;
    type Properties = IncomeSourcesProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { modal_source_id: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            IncomeSourcesMsg::OpenModal(source_id) => {
                self.modal_source_id = Some(source_id);
                true
            }
            IncomeSourcesMsg::CloseModal => {
                self.modal_source_id = None;
                true
            }
            IncomeSourcesMsg::Saved => {
                self.modal_source_id = None;
                ctx.props().on_saved.emit(());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {for ctx.props().sources.iter().map(|source| {
                        let source_id = source.id.clone();
                        html! {
                            <div class="card bg-base-200 shadow">
                                <div class="card-body p-4">
                                    <div class="flex justify-between items-center">
                                        <div>
                                            <h3 class="font-semibold text-lg">{ &source.name }</h3>
                                            <p class="text-2xl font-bold text-primary">
                                                { source.amount.to_string() }
                                            </p>
                                        </div>
                                        <button
                                            class="btn btn-primary btn-sm"
                                            onclick={ctx.link().callback(move |_| IncomeSourcesMsg::OpenModal(source_id.clone()))}
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
                            on_close={ctx.link().callback(|_| IncomeSourcesMsg::CloseModal)}
                            on_saved={ctx.link().callback(|_| IncomeSourcesMsg::Saved)}
                            api={ctx.props().api.clone()}
                        />
                    }
                } else {
                    html! {}
                }}
            </>
        }
    }
}
