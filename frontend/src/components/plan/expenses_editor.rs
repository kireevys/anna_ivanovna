use std::str::FromStr;

use rust_decimal::Decimal;

use ai_core::finance::{Money, Percentage};

use crate::presentation::{
    editable_plan::{EditableExpense, EditableExpenseKind},
    formatting::FormattedMoney,
};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ExpensesEditorProps {
    pub expenses: Vec<EditableExpense>,
    pub total_income: Option<Decimal>,
    pub on_change: Callback<Vec<EditableExpense>>,
}

pub enum ExpensesEditorMsg {
    AmountChanged {
        pos: usize,
        value: String,
    },
    KindChanged {
        pos: usize,
        kind: EditableExpenseKind,
    },
    NameChanged {
        pos: usize,
        value: String,
    },
    CategoryChanged {
        pos: usize,
        value: String,
    },
    StartAdding,
    ConfirmNew,
    CancelNew,
    DeleteExpense {
        pos: usize,
    },
}

pub struct ExpensesEditor {
    adding: bool,
}

impl Component for ExpensesEditor {
    type Message = ExpensesEditorMsg;
    type Properties = ExpensesEditorProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { adding: false }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let mut updated = ctx.props().expenses.clone();
        match msg {
            ExpensesEditorMsg::AmountChanged { pos, value } => {
                if let Some(expense) = updated.get_mut(pos) {
                    expense.amount = value;
                }
                ctx.props().on_change.emit(updated);
            }
            ExpensesEditorMsg::KindChanged { pos, kind } => {
                if let Some(expense) = updated.get_mut(pos) {
                    expense.kind = kind;
                }
                ctx.props().on_change.emit(updated);
            }
            ExpensesEditorMsg::NameChanged { pos, value } => {
                if let Some(expense) = updated.get_mut(pos) {
                    expense.name = value;
                }
                ctx.props().on_change.emit(updated);
            }
            ExpensesEditorMsg::CategoryChanged { pos, value } => {
                if let Some(expense) = updated.get_mut(pos) {
                    expense.category =
                        if value.is_empty() { None } else { Some(value) };
                }
                ctx.props().on_change.emit(updated);
            }
            ExpensesEditorMsg::StartAdding => {
                self.adding = true;
                updated.insert(0, EditableExpense::empty());
                ctx.props().on_change.emit(updated);
            }
            ExpensesEditorMsg::ConfirmNew => {
                self.adding = false;
            }
            ExpensesEditorMsg::CancelNew => {
                self.adding = false;
                updated.remove(0);
                ctx.props().on_change.emit(updated);
            }
            ExpensesEditorMsg::DeleteExpense { pos } => {
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
        let expenses = &ctx.props().expenses;

        let existing_categories: Vec<String> = {
            let mut cats: Vec<String> =
                expenses.iter().filter_map(|e| e.category.clone()).collect();
            cats.sort();
            cats.dedup();
            cats
        };

        html! {
            <div class="space-y-4">
                <datalist id="expense-categories">
                    {for existing_categories.iter().map(|cat| html! {
                        <option value={cat.clone()} />
                    })}
                </datalist>

                {if self.adding {
                    if let Some(first) = expenses.first() {
                        self.render_new_expense_card(ctx, first)
                    } else {
                        html! {}
                    }
                } else {
                    html! {
                        <button
                            class="btn btn-outline btn-primary w-full"
                            onclick={ctx.link().callback(|_| ExpensesEditorMsg::StartAdding)}
                        >
                            {"+ Добавить расход"}
                        </button>
                    }
                }}

                {for expenses.iter().enumerate()
                    .skip(if self.adding { 1 } else { 0 })
                    .map(|(pos, expense)| {
                        self.render_expense_card(ctx, pos, expense)
                    })
                }
            </div>
        }
    }
}

impl ExpensesEditor {
    fn render_new_expense_card(
        &self,
        ctx: &Context<Self>,
        expense: &EditableExpense,
    ) -> Html {
        let amount_class = if expense.is_valid || expense.amount.is_empty() {
            "input input-bordered w-full"
        } else {
            "input input-bordered input-error w-full"
        };
        html! {
            <div class="card bg-base-200 shadow border-2 border-primary">
                <div class="card-body p-3 space-y-2">
                    <h4 class="font-medium text-sm text-primary">{"Новый расход"}</h4>
                    <div class="flex items-center gap-2">
                        <input
                            class="input input-bordered input-sm flex-1"
                            placeholder="Название"
                            value={expense.name.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                let value = e.target_unchecked_into::<HtmlInputElement>().value();
                                ExpensesEditorMsg::NameChanged { pos: 0, value }
                            })}
                        />
                        <input
                            class="input input-bordered input-sm w-40"
                            placeholder="Категория"
                            list="expense-categories"
                            value={expense.category.clone().unwrap_or_default()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                let value = e.target_unchecked_into::<HtmlInputElement>().value();
                                ExpensesEditorMsg::CategoryChanged { pos: 0, value }
                            })}
                        />
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="join">
                            { Self::render_kind_button(ctx, 0, EditableExpenseKind::Money, "₽", expense.kind == EditableExpenseKind::Money) }
                            { Self::render_kind_button(ctx, 0, EditableExpenseKind::Rate, "%", expense.kind == EditableExpenseKind::Rate) }
                        </div>
                        <input
                            class={amount_class}
                            placeholder="Сумма"
                            value={expense.amount.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                let value = e.target_unchecked_into::<HtmlInputElement>().value();
                                ExpensesEditorMsg::AmountChanged { pos: 0, value }
                            })}
                        />
                    </div>
                    {if expense.kind == EditableExpenseKind::Rate {
                        Self::render_rate_hint(ctx.props().total_income, &expense.amount)
                    } else {
                        html! {}
                    }}
                    <div class="flex gap-2 justify-end">
                        <button
                            class="btn btn-sm btn-ghost"
                            onclick={ctx.link().callback(|_| ExpensesEditorMsg::CancelNew)}
                        >
                            {"Отмена"}
                        </button>
                        <button
                            class="btn btn-sm btn-primary"
                            onclick={ctx.link().callback(|_| ExpensesEditorMsg::ConfirmNew)}
                        >
                            {"Добавить"}
                        </button>
                    </div>
                </div>
            </div>
        }
    }

    fn render_expense_card(
        &self,
        ctx: &Context<Self>,
        pos: usize,
        expense: &EditableExpense,
    ) -> Html {
        let amount_class = if expense.is_valid || expense.amount.is_empty() {
            "input input-bordered w-full"
        } else {
            "input input-bordered input-error w-full"
        };
        html! {
            <div class="card bg-base-100 shadow">
                <div class="card-body p-3 space-y-2">
                    <div class="flex items-center gap-2">
                        <input
                            class="input input-bordered input-sm flex-1"
                            placeholder="Название расхода"
                            value={expense.name.clone()}
                            oninput={ctx.link().callback(move |e: InputEvent| {
                                let value = e.target_unchecked_into::<HtmlInputElement>().value();
                                ExpensesEditorMsg::NameChanged { pos, value }
                            })}
                        />
                        <input
                            class="input input-bordered input-sm w-40"
                            placeholder="Категория"
                            list="expense-categories"
                            value={expense.category.clone().unwrap_or_default()}
                            oninput={ctx.link().callback(move |e: InputEvent| {
                                let value = e.target_unchecked_into::<HtmlInputElement>().value();
                                ExpensesEditorMsg::CategoryChanged { pos, value }
                            })}
                        />
                        <button
                            class="btn btn-sm btn-ghost btn-square text-error"
                            onclick={ctx.link().callback(move |_| ExpensesEditorMsg::DeleteExpense { pos })}
                        >
                            {"✕"}
                        </button>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="join">
                            { Self::render_kind_button(ctx, pos, EditableExpenseKind::Money, "₽", expense.kind == EditableExpenseKind::Money) }
                            { Self::render_kind_button(ctx, pos, EditableExpenseKind::Rate, "%", expense.kind == EditableExpenseKind::Rate) }
                        </div>
                        <input
                            class={amount_class}
                            placeholder="Сумма"
                            value={expense.amount.clone()}
                            oninput={ctx.link().callback(move |e: InputEvent| {
                                let value = e.target_unchecked_into::<HtmlInputElement>().value();
                                ExpensesEditorMsg::AmountChanged { pos, value }
                            })}
                        />
                    </div>
                    {if expense.kind == EditableExpenseKind::Rate {
                        Self::render_rate_hint(ctx.props().total_income, &expense.amount)
                    } else {
                        html! {}
                    }}
                </div>
            </div>
        }
    }

    fn render_rate_hint(total_income: Option<Decimal>, amount_str: &str) -> Html {
        let preview = total_income.and_then(|total| {
            let rate = Decimal::from_str(amount_str).ok()?;
            let value = Percentage::from(rate).apply_to(total);
            Some(FormattedMoney::from_money(Money::new_rub(value)))
        });
        match preview {
            Some(money) => html! {
                <span class="text-xs text-base-content/60">
                    { format!("= {money}") }
                </span>
            },
            None => html! {},
        }
    }

    fn render_kind_button(
        ctx: &Context<Self>,
        pos: usize,
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
            .callback(move |_| ExpensesEditorMsg::KindChanged { pos, kind });
        html! {
            <button class={class} {onclick}>
                { label }
            </button>
        }
    }
}
