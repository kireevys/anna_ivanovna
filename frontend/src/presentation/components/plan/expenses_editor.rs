use std::str::FromStr;

use rust_decimal::Decimal;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use ai_core::finance::{Money, Percentage};

use crate::{
    engine::plan::editable::{ActiveType, Expense, ExpenseType, ValueKind},
    presentation::{components::icons::XIcon, formatting::FormattedMoney},
};

#[derive(Properties, PartialEq)]
pub struct ExpensesEditorProps {
    pub expenses: Vec<Expense>,
    pub total_income: Option<Decimal>,
    pub on_change: Callback<Vec<Expense>>,
}

pub enum CreditField {
    MonthlyPayment,
    TotalAmount,
    InterestRate,
    TermMonths,
    StartDate,
}

pub enum ExpensesEditorMsg {
    AmountChanged {
        pos: usize,
        value: String,
    },
    KindChanged {
        pos: usize,
        kind: ValueKind,
    },
    NameChanged {
        pos: usize,
        value: String,
    },
    CategoryChanged {
        pos: usize,
        value: String,
    },
    ExpenseTypeChanged {
        pos: usize,
        is_credit: bool,
    },
    CreditFieldChanged {
        pos: usize,
        field: CreditField,
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
                    expense.envelope.amount = value;
                }
                ctx.props().on_change.emit(updated);
            }
            ExpensesEditorMsg::KindChanged { pos, kind } => {
                if let Some(expense) = updated.get_mut(pos) {
                    expense.envelope.value_kind = kind;
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
            ExpensesEditorMsg::ExpenseTypeChanged { pos, is_credit } => {
                if let Some(expense) = updated.get_mut(pos) {
                    expense.active_type = if is_credit {
                        ActiveType::Credit
                    } else {
                        ActiveType::Envelope
                    };
                }
                ctx.props().on_change.emit(updated);
            }
            ExpensesEditorMsg::CreditFieldChanged { pos, field, value } => {
                if let Some(expense) = updated.get_mut(pos) {
                    let credit = &mut expense.credit;
                    match field {
                        CreditField::MonthlyPayment => credit.monthly_payment = value,
                        CreditField::TotalAmount => credit.total_amount = value,
                        CreditField::InterestRate => credit.interest_rate = value,
                        CreditField::TermMonths => credit.term_months = value,
                        CreditField::StartDate => credit.start_date = value,
                    }
                }
                ctx.props().on_change.emit(updated);
            }
            ExpensesEditorMsg::StartAdding => {
                self.adding = true;
                updated.insert(0, Expense::empty());
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
    fn render_new_expense_card(&self, ctx: &Context<Self>, expense: &Expense) -> Html {
        let is_credit = expense.active_type == ActiveType::Credit;
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
                    { Self::render_type_toggle(ctx, 0, is_credit) }
                    { Self::render_expense_fields(ctx, 0, &expense.expense_type()) }
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
        expense: &Expense,
    ) -> Html {
        let is_credit = expense.active_type == ActiveType::Credit;
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
                            <XIcon />
                        </button>
                    </div>
                    { Self::render_type_toggle(ctx, pos, is_credit) }
                    { Self::render_expense_fields(ctx, pos, &expense.expense_type()) }
                </div>
            </div>
        }
    }

    fn render_type_toggle(ctx: &Context<Self>, pos: usize, is_credit: bool) -> Html {
        let envelope_class = if !is_credit {
            "btn btn-sm join-item btn-primary"
        } else {
            "btn btn-sm join-item btn-outline"
        };
        let credit_class = if is_credit {
            "btn btn-sm join-item btn-primary"
        } else {
            "btn btn-sm join-item btn-outline"
        };
        html! {
            <div class="join">
                <button
                    class={envelope_class}
                    onclick={ctx.link().callback(move |_| ExpensesEditorMsg::ExpenseTypeChanged { pos, is_credit: false })}
                >
                    {"Конверт"}
                </button>
                <button
                    class={credit_class}
                    onclick={ctx.link().callback(move |_| ExpensesEditorMsg::ExpenseTypeChanged { pos, is_credit: true })}
                >
                    {"Кредит"}
                </button>
            </div>
        }
    }

    fn render_expense_fields(
        ctx: &Context<Self>,
        pos: usize,
        expense_type: &ExpenseType,
    ) -> Html {
        match expense_type {
            ExpenseType::Envelope { value_kind, amount } => {
                Self::render_envelope_fields(ctx, pos, *value_kind, amount)
            }
            ExpenseType::Credit {
                monthly_payment,
                total_amount,
                interest_rate,
                term_months,
                start_date,
            } => Self::render_credit_fields(
                ctx,
                pos,
                monthly_payment,
                total_amount,
                interest_rate,
                term_months,
                start_date,
            ),
        }
    }

    fn render_envelope_fields(
        ctx: &Context<Self>,
        pos: usize,
        value_kind: ValueKind,
        amount: &str,
    ) -> Html {
        let amount_owned = amount.to_owned();
        html! {
            <>
                <div class="flex items-center gap-2">
                    <div class="join">
                        { Self::render_kind_button(ctx, pos, ValueKind::Money, "₽", value_kind == ValueKind::Money) }
                        { Self::render_kind_button(ctx, pos, ValueKind::Rate, "%", value_kind == ValueKind::Rate) }
                    </div>
                    <input
                        class="input input-bordered w-full"
                        placeholder="Сумма"
                        value={amount_owned.clone()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let value = e.target_unchecked_into::<HtmlInputElement>().value();
                            ExpensesEditorMsg::AmountChanged { pos, value }
                        })}
                    />
                </div>
                {if value_kind == ValueKind::Rate {
                    Self::render_rate_hint(ctx.props().total_income, &amount_owned)
                } else {
                    html! {}
                }}
            </>
        }
    }

    fn render_credit_fields(
        ctx: &Context<Self>,
        pos: usize,
        monthly_payment: &str,
        total_amount: &str,
        interest_rate: &str,
        term_months: &str,
        start_date: &str,
    ) -> Html {
        html! {
            <div class="space-y-2">
                <div>
                    <label class="text-xs text-base-content/60">{"Ежемесячный платёж"}</label>
                    <input
                        class="input input-bordered input-sm w-full"
                        value={monthly_payment.to_owned()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let value = e.target_unchecked_into::<HtmlInputElement>().value();
                            ExpensesEditorMsg::CreditFieldChanged { pos, field: CreditField::MonthlyPayment, value }
                        })}
                    />
                </div>
                <div>
                    <label class="text-xs text-base-content/60">{"Сумма кредита"}</label>
                    <input
                        class="input input-bordered input-sm w-full"
                        value={total_amount.to_owned()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let value = e.target_unchecked_into::<HtmlInputElement>().value();
                            ExpensesEditorMsg::CreditFieldChanged { pos, field: CreditField::TotalAmount, value }
                        })}
                    />
                </div>
                <div class="flex items-center gap-2">
                    <div class="flex-1">
                        <label class="text-xs text-base-content/60">{"Ставка, %"}</label>
                        <input
                            class="input input-bordered input-sm w-full"
                            value={interest_rate.to_owned()}
                            oninput={ctx.link().callback(move |e: InputEvent| {
                                let value = e.target_unchecked_into::<HtmlInputElement>().value();
                                ExpensesEditorMsg::CreditFieldChanged { pos, field: CreditField::InterestRate, value }
                            })}
                        />
                    </div>
                    <div class="w-24">
                        <label class="text-xs text-base-content/60">{"Срок, мес."}</label>
                        <input
                            class="input input-bordered input-sm w-full"
                            value={term_months.to_owned()}
                            oninput={ctx.link().callback(move |e: InputEvent| {
                                let value = e.target_unchecked_into::<HtmlInputElement>().value();
                                ExpensesEditorMsg::CreditFieldChanged { pos, field: CreditField::TermMonths, value }
                            })}
                        />
                    </div>
                </div>
                <div>
                    <label class="text-xs text-base-content/60">{"Дата оформления"}</label>
                    <input
                        type="date"
                        class="input input-bordered input-sm w-full"
                        value={start_date.to_owned()}
                        oninput={ctx.link().callback(move |e: InputEvent| {
                            let value = e.target_unchecked_into::<HtmlInputElement>().value();
                            ExpensesEditorMsg::CreditFieldChanged { pos, field: CreditField::StartDate, value }
                        })}
                    />
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
        kind: ValueKind,
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
