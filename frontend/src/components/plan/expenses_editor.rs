use crate::presentation::editable_plan::{EditableExpense, EditableExpenseKind};
use std::collections::BTreeMap;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ExpensesEditorProps {
    pub expenses: Vec<EditableExpense>,
    pub on_change: Callback<Vec<EditableExpense>>,
}

pub enum ExpensesEditorMsg {
    AmountChanged {
        index: usize,
        value: String,
    },
    KindChanged {
        index: usize,
        kind: EditableExpenseKind,
    },
}

pub struct ExpensesEditor;

impl Component for ExpensesEditor {
    type Message = ExpensesEditorMsg;
    type Properties = ExpensesEditorProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ExpensesEditorMsg::AmountChanged { index, value } => {
                let mut updated = ctx.props().expenses.clone();
                if let Some(expense) = updated.iter_mut().find(|e| e.index == index) {
                    expense.amount = value;
                }
                ctx.props().on_change.emit(updated);
                true
            }
            ExpensesEditorMsg::KindChanged { index, kind } => {
                let mut updated = ctx.props().expenses.clone();
                if let Some(expense) = updated.iter_mut().find(|e| e.index == index) {
                    expense.kind = kind;
                }
                ctx.props().on_change.emit(updated);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut by_category: BTreeMap<String, Vec<EditableExpense>> = BTreeMap::new();
        for e in &ctx.props().expenses {
            let key = e
                .category
                .clone()
                .unwrap_or_else(|| "Без категории".to_string());
            by_category.entry(key).or_default().push(e.clone());
        }

        html! {
            <div class="space-y-4">
                {for by_category.into_iter().map(|(category, expenses)| {
                    html! {
                        <div class="card bg-base-100 shadow">
                            <div class="card-body space-y-3">
                                <h3 class="card-title text-lg">{ category }</h3>
                                <div class="space-y-2">
                                    {for expenses.into_iter().map(|expense| {
                                        let index = expense.index;
                                        let input_class =
                                            if expense.is_valid || expense.amount.is_empty() {
                                                "input input-bordered w-full"
                                            } else {
                                                "input input-bordered input-error w-full"
                                            };
                                        html! {
                                            <div class="flex flex-col gap-1">
                                                <span class="font-medium">{ &expense.name }</span>
                                                <div class="flex items-center gap-2">
                                                    <div class="join">
                                                        { Self::render_kind_button(ctx, index, EditableExpenseKind::Money, "₽", expense.kind == EditableExpenseKind::Money) }
                                                        { Self::render_kind_button(ctx, index, EditableExpenseKind::Rate, "%", expense.kind == EditableExpenseKind::Rate) }
                                                    </div>
                                                    <input
                                                        class={input_class}
                                                        value={expense.amount.clone()}
                                                        oninput={ctx.link().callback(move |e: InputEvent| {
                                                            let value = e
                                                                .target_unchecked_into::<HtmlInputElement>()
                                                                .value();
                                                            ExpensesEditorMsg::AmountChanged {
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
                            </div>
                        </div>
                    }
                })}
            </div>
        }
    }
}

impl ExpensesEditor {
    fn render_kind_button(
        ctx: &Context<Self>,
        index: usize,
        kind: EditableExpenseKind,
        label: &str,
        active: bool,
    ) -> Html {
        let class = if active {
            "btn btn-xs join-item btn-primary"
        } else {
            "btn btn-xs join-item"
        };
        let onclick = ctx
            .link()
            .callback(move |_| ExpensesEditorMsg::KindChanged { index, kind });
        html! {
            <button class={class} {onclick}>
                { label }
            </button>
        }
    }
}
