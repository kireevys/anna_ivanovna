use crate::presentation::plan::Expense;
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
        let expense_name = ctx.props().expense.name.clone();
        html! {
            <div class="relative group">
                <div class="flex justify-between items-center p-2 bg-base-100 rounded gap-2 max-w-xs">
                    <span
                        class="font-medium truncate flex-1 min-w-0"
                        title={expense_name.clone()}
                    >
                        { &ctx.props().expense.name }
                    </span>
                    <span class="font-bold flex-shrink-0">{ ctx.props().expense.value.to_string() }</span>
                </div>
                <div class="absolute left-0 bottom-full mb-2 hidden group-hover:block bg-gray-800 text-white text-xs rounded py-1 px-2 z-10 whitespace-pre-wrap max-w-xs">
                    { expense_name }
                    <div class="absolute left-1/2 -translate-x-1/2 top-full border-l-4 border-r-4 border-t-4 border-t-gray-800 border-l-transparent border-r-transparent w-0 h-0"></div>
                </div>
            </div>
        }
    }
}
