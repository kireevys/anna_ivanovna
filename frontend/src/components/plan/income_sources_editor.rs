use crate::presentation::editable_plan::EditableIncomeSource;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct IncomeSourcesEditorProps {
    pub sources: Vec<EditableIncomeSource>,
    pub on_change: Callback<Vec<EditableIncomeSource>>,
}

pub enum IncomeSourcesEditorMsg {
    AmountChanged { index: usize, value: String },
}

pub struct IncomeSourcesEditor;

impl Component for IncomeSourcesEditor {
    type Message = IncomeSourcesEditorMsg;
    type Properties = IncomeSourcesEditorProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            IncomeSourcesEditorMsg::AmountChanged { index, value } => {
                let mut updated = ctx.props().sources.clone();
                if let Some(source) = updated.iter_mut().find(|s| s.index == index) {
                    source.amount = value;
                }
                ctx.props().on_change.emit(updated);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                {for ctx.props().sources.iter().map(|source| {
                    let index = source.index;
                    let input_class = if source.is_valid || source.amount.is_empty() {
                        "input input-bordered w-full".to_string()
                    } else {
                        "input input-bordered input-error w-full".to_string()
                    };
                    html! {
                        <div class="card bg-base-200 shadow">
                            <div class="card-body p-4 space-y-2">
                                <div class="font-semibold text-lg">{ &source.name }</div>
                                <input
                                    class={input_class}
                                    value={source.amount.clone()}
                                    oninput={ctx.link().callback(move |e: InputEvent| {
                                        let value = e
                                            .target_unchecked_into::<HtmlInputElement>()
                                            .value();
                                        IncomeSourcesEditorMsg::AmountChanged {
                                            index,
                                            value,
                                        }
                                    })}
                                />
                            </div>
                        </div>
                    }
                })}
            </div>
        }
    }
}
