use crate::presentation::editable_plan::EditableIncomeSource;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct IncomeSourcesEditorProps {
    pub sources: Vec<EditableIncomeSource>,
    pub on_change: Callback<Vec<EditableIncomeSource>>,
}

pub enum IncomeSourcesEditorMsg {
    AmountChanged { pos: usize, value: String },
    NameChanged { pos: usize, value: String },
    StartAdding,
    ConfirmNew,
    CancelNew,
    DeleteSource { pos: usize },
}

pub struct IncomeSourcesEditor {
    adding: bool,
}

impl Component for IncomeSourcesEditor {
    type Message = IncomeSourcesEditorMsg;
    type Properties = IncomeSourcesEditorProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { adding: false }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let mut updated = ctx.props().sources.clone();
        match msg {
            IncomeSourcesEditorMsg::AmountChanged { pos, value } => {
                if let Some(source) = updated.get_mut(pos) {
                    source.amount = value;
                }
                ctx.props().on_change.emit(updated);
            }
            IncomeSourcesEditorMsg::NameChanged { pos, value } => {
                if let Some(source) = updated.get_mut(pos) {
                    source.name = value;
                }
                ctx.props().on_change.emit(updated);
            }
            IncomeSourcesEditorMsg::StartAdding => {
                self.adding = true;
                updated.insert(0, EditableIncomeSource::empty());
                ctx.props().on_change.emit(updated);
            }
            IncomeSourcesEditorMsg::ConfirmNew => {
                self.adding = false;
            }
            IncomeSourcesEditorMsg::CancelNew => {
                self.adding = false;
                updated.remove(0);
                ctx.props().on_change.emit(updated);
            }
            IncomeSourcesEditorMsg::DeleteSource { pos } => {
                if pos < updated.len() {
                    if self.adding && pos == 0 {
                        self.adding = false;
                    }
                    updated.remove(pos);
                }
                ctx.props().on_change.emit(updated);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let sources = &ctx.props().sources;

        html! {
            <div class="space-y-4">
                {if self.adding {
                    if let Some(first) = sources.first() {
                        self.render_new_source_card(ctx, first)
                    } else {
                        html! {}
                    }
                } else {
                    html! {
                        <button
                            class="btn btn-outline btn-primary w-full"
                            onclick={ctx.link().callback(|_| IncomeSourcesEditorMsg::StartAdding)}
                        >
                            {"+ Добавить источник дохода"}
                        </button>
                    }
                }}

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {for sources.iter().enumerate()
                        .skip(if self.adding { 1 } else { 0 })
                        .map(|(pos, source)| {
                            self.render_source_card(ctx, pos, source)
                        })
                    }
                </div>
            </div>
        }
    }
}

impl IncomeSourcesEditor {
    fn render_new_source_card(
        &self,
        ctx: &Context<Self>,
        source: &EditableIncomeSource,
    ) -> Html {
        let amount_class = if source.is_valid || source.amount.is_empty() {
            "input input-bordered w-full"
        } else {
            "input input-bordered input-error w-full"
        };
        html! {
            <div class="card bg-base-200 shadow border-2 border-primary">
                <div class="card-body p-4 space-y-2">
                    <h4 class="font-medium text-sm text-primary">{"Новый источник дохода"}</h4>
                    <input
                        class="input input-bordered input-sm w-full"
                        placeholder="Название"
                        value={source.name.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {
                            let value = e.target_unchecked_into::<HtmlInputElement>().value();
                            IncomeSourcesEditorMsg::NameChanged { pos: 0, value }
                        })}
                    />
                    <input
                        class={amount_class}
                        placeholder="Сумма"
                        value={source.amount.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {
                            let value = e.target_unchecked_into::<HtmlInputElement>().value();
                            IncomeSourcesEditorMsg::AmountChanged { pos: 0, value }
                        })}
                    />
                    <div class="flex gap-2 justify-end">
                        <button
                            class="btn btn-sm btn-ghost"
                            onclick={ctx.link().callback(|_| IncomeSourcesEditorMsg::CancelNew)}
                        >
                            {"Отмена"}
                        </button>
                        <button
                            class="btn btn-sm btn-primary"
                            onclick={ctx.link().callback(|_| IncomeSourcesEditorMsg::ConfirmNew)}
                        >
                            {"Добавить"}
                        </button>
                    </div>
                </div>
            </div>
        }
    }

    fn render_source_card(
        &self,
        ctx: &Context<Self>,
        pos: usize,
        source: &EditableIncomeSource,
    ) -> Html {
        let amount_class = if source.is_valid || source.amount.is_empty() {
            "input input-bordered w-full"
        } else {
            "input input-bordered input-error w-full"
        };
        html! {
            <div class="card bg-base-200 shadow">
                <div class="card-body p-4 space-y-2">
                    <div class="flex items-center gap-2">
                        <input
                            class="input input-bordered input-sm flex-1"
                            placeholder="Название"
                            value={source.name.clone()}
                            oninput={ctx.link().callback(move |e: InputEvent| {
                                let value = e.target_unchecked_into::<HtmlInputElement>().value();
                                IncomeSourcesEditorMsg::NameChanged { pos, value }
                            })}
                        />
                        <button
                            class="btn btn-sm btn-ghost btn-square text-error"
                            onclick={ctx.link().callback(move |_| IncomeSourcesEditorMsg::DeleteSource { pos })}
                        >
                            {"✕"}
                        </button>
                    </div>
                    <input
                        class={amount_class}
                        placeholder="Сумма"
                        value={source.amount.clone()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let value = e.target_unchecked_into::<HtmlInputElement>().value();
                            IncomeSourcesEditorMsg::AmountChanged { pos, value }
                        })}
                    />
                </div>
            </div>
        }
    }
}
