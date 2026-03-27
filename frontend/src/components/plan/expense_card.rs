use crate::presentation::plan::{AccountingUnit, Expense};
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

        let money_active = value.unit == AccountingUnit::Money;
        let rate_active = value.unit == AccountingUnit::Rate;

        let money_badge = if money_active {
            "btn btn-xs join-item btn-primary"
        } else {
            "btn btn-xs join-item"
        };
        let rate_badge = if rate_active {
            "btn btn-xs join-item btn-primary"
        } else {
            "btn btn-xs join-item"
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
                        <span class="join">
                            <span class={money_badge}>{"₽"}</span>
                            <span class={rate_badge}>{"%"}</span>
                        </span>
                    </span>
                </div>
                <div class="absolute left-0 bottom-full mb-2 hidden group-hover:block bg-gray-800 text-white text-xs rounded py-1 px-2 z-10 whitespace-pre-wrap max-w-xs">
                    { expense_name }
                    <div class="absolute left-1/2 -translate-x-1/2 top-full border-l-4 border-r-4 border-t-4 border-t-gray-800 border-l-transparent border-r-transparent w-0 h-0"></div>
                </div>
            </div>
        }
    }
}
