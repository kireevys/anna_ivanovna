use std::rc::Rc;

use yew::prelude::*;

use crate::{
    api::ApiClient,
    components::IncomeModal,
    presentation::{income::SourceKind, plan::IncomeSource},
};

#[derive(Properties, PartialEq)]
pub struct IncomeSourcesProps {
    pub sources: Vec<IncomeSource>,
    pub on_saved: Callback<()>,
    pub api: Rc<ApiClient>,
}

pub enum IncomeSourcesMsg {
    OpenModal(ModalContext),
    CloseModal,
    Saved,
}

pub struct ModalContext {
    source_id: String,
    source_kind: SourceKind,
}

pub struct IncomeSources {
    modal: Option<ModalContext>,
}

impl Component for IncomeSources {
    type Message = IncomeSourcesMsg;
    type Properties = IncomeSourcesProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { modal: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            IncomeSourcesMsg::OpenModal(modal) => {
                self.modal = Some(modal);
                true
            }
            IncomeSourcesMsg::CloseModal => {
                self.modal = None;
                true
            }
            IncomeSourcesMsg::Saved => {
                self.modal = None;
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
                        let source_kind = source.source_kind.clone();
                        html! {
                            <div class="card bg-base-200 shadow">
                                <div class="card-body p-4">
                                    <div class="flex justify-between items-start">
                                        <div>
                                            <div class="flex items-center gap-2 mb-1">
                                                <h3 class="font-semibold text-lg">{ &source.name }</h3>
                                                <span class="badge badge-sm badge-ghost">{ source.source_kind.kind_label() }</span>
                                            </div>
                                            {if let SourceKind::Salary { gross, tax_rate, tax_amount } = &source.source_kind {
                                                html! {
                                                    <>
                                                        <p class="text-sm text-base-content/60">
                                                            { format!("Gross: {gross}") }
                                                        </p>
                                                        <p class="text-sm text-base-content/60">
                                                            { format!("Налог: {tax_rate}% ({tax_amount})") }
                                                        </p>
                                                        <div class="divider my-1"></div>
                                                        <p class="text-2xl font-bold text-primary">
                                                            { format!("На руки: {}", source.amount) }
                                                        </p>
                                                    </>
                                                }
                                            } else {
                                                html! {
                                                    <p class="text-2xl font-bold text-primary">
                                                        { source.amount.to_string() }
                                                    </p>
                                                }
                                            }}
                                        </div>
                                        <button
                                            class="btn btn-primary btn-sm"
                                            onclick={ctx.link().callback(move |_| IncomeSourcesMsg::OpenModal(ModalContext {
                                                source_id: source_id.clone(),
                                                source_kind: source_kind.clone(),
                                            }))}
                                        >
                                            { "Поступление" }
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    })}
                </div>
                {if let Some(modal) = &self.modal {
                    html! {
                        <IncomeModal
                            source_id={modal.source_id.clone()}
                            source_kind={modal.source_kind.clone()}
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
