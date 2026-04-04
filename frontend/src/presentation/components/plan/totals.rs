use crate::presentation::formatting::FormattedMoney;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TotalsProps {
    pub total_income: FormattedMoney,
    pub total_expenses: FormattedMoney,
    pub balance: FormattedMoney,
}

pub struct Totals;

impl Component for Totals {
    type Message = ();
    type Properties = TotalsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="stats shadow w-full">
                <div class="stat">
                    <div class="stat-title">{ "Доходы" }</div>
                    <div class="stat-value text-success text-2xl">
                        { ctx.props().total_income.to_string() }
                    </div>
                </div>
                <div class="stat">
                    <div class="stat-title">{ "Расходы" }</div>
                    <div class="stat-value text-error text-2xl">
                        { ctx.props().total_expenses.to_string() }
                    </div>
                </div>
                <div class="stat">
                    <div class="stat-title">{ "Остаток" }</div>
                    <div class="stat-value text-2xl text-warning">
                        { ctx.props().balance.to_string() }
                    </div>
                </div>
            </div>
        }
    }
}
