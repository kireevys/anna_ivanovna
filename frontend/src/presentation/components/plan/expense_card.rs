use crate::presentation::{
    components::icons::{LandmarkIcon, MailIcon},
    plan::read::{AccountingUnit, Expense, ExpenseKindView},
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ExpenseCardProps {
    pub expense: Expense,
}

pub struct ExpenseCard;

impl Component for ExpenseCard {
    type Message = ();
    type Properties = ExpenseCardProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let expense = &ctx.props().expense;
        let expense_name = expense.name.clone();
        let value = &expense.value;

        let type_badge = match &expense.kind {
            ExpenseKindView::Envelope => {
                let (active_unit, tooltip) = if value.unit == AccountingUnit::Money {
                    ("₽", "Конверт в рублях")
                } else {
                    ("%", "Конверт в процентах")
                };
                html! {
                    <div class="relative group/envelope">
                        <span class="badge badge-sm badge-primary w-12 justify-center gap-1 cursor-help">
                            <MailIcon class="w-3 h-3" />
                            { active_unit }
                        </span>
                        <div class="absolute right-0 top-full mt-1 hidden group-hover/envelope:block bg-base-300 text-base-content text-xs rounded-lg py-1 px-2 z-20 whitespace-nowrap shadow-lg">
                            { tooltip }
                        </div>
                    </div>
                }
            }
            ExpenseKindView::Credit {
                total_amount,
                interest_rate,
                term_months,
                start_date,
                ..
            } => {
                html! {
                    <div class="relative group/credit">
                        <span class="badge badge-sm badge-primary w-12 justify-center cursor-help">
                            <LandmarkIcon class="w-3 h-3" />
                        </span>
                        <div class="absolute right-0 top-full mt-1 hidden group-hover/credit:block bg-base-300 text-base-content text-xs rounded-lg py-2 px-3 z-20 whitespace-nowrap shadow-lg">
                            <div class="font-semibold mb-1">{"Кредит"}</div>
                            <div>{ format!("Сумма: {total_amount}") }</div>
                            <div>{ format!("Ставка: {interest_rate}") }</div>
                            <div>{ format!("Срок: {term_months} мес.") }</div>
                            <div>{ format!("С {start_date}") }</div>
                        </div>
                    </div>
                }
            }
        };

        html! {
            <div class="relative group">
                <div class="flex justify-between items-center p-2 bg-base-100 rounded gap-2">
                    <span
                        class="font-medium truncate flex-1 min-w-0"
                        title={expense_name.clone()}
                    >
                        { &expense.name }
                    </span>
                    <span class="flex items-center gap-2 flex-shrink-0">
                        <span class="font-bold">{ value.money.to_string() }</span>
                        <span class="text-base-content/60">{ value.rate.to_string() }</span>
                        { type_badge }
                    </span>
                </div>
                <div class="absolute left-0 bottom-full mb-2 hidden group-hover:block bg-base-300 text-base-content text-xs rounded-lg py-1 px-2 z-10 whitespace-pre-wrap max-w-xs shadow-lg">
                    { expense_name }
                </div>
            </div>
        }
    }
}
