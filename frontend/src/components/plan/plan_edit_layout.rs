use rust_decimal::Decimal;

use crate::{
    components::plan::{
        EditActionsBar,
        ExpensesEditor,
        IncomeSourcesEditor,
        SectionCard,
    },
    presentation::plan::editable,
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct EditLayoutProps {
    pub incomes: Vec<editable::IncomeSource>,
    pub expenses: Vec<editable::Expense>,
    pub total_income: Option<Decimal>,
    pub disable_save: bool,
    pub on_cancel: Callback<()>,
    pub on_save: Callback<()>,
    pub on_incomes_change: Callback<Vec<editable::IncomeSource>>,
    pub on_expenses_change: Callback<Vec<editable::Expense>>,
}

pub struct EditLayout;

impl Component for EditLayout {
    type Message = ();
    type Properties = EditLayoutProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let incomes = ctx.props().incomes.clone();
        let expenses = ctx.props().expenses.clone();

        let incomes_on_change = ctx.props().on_incomes_change.clone();
        let expenses_on_change = ctx.props().on_expenses_change.clone();

        let actions = html! {
            <EditActionsBar
                disable_save={ctx.props().disable_save}
                on_cancel={ctx.props().on_cancel.clone()}
                on_save={ctx.props().on_save.clone()}
            />
        };

        html! {
            <div class="space-y-6">
                <SectionCard
                    title={"Редактирование доходов плана"}
                    header_right={Some(actions)}
                >
                    <IncomeSourcesEditor
                        sources={incomes}
                        on_change={incomes_on_change}
                    />
                </SectionCard>
                <SectionCard
                    title={"Редактирование расходов плана"}
                >
                    <ExpensesEditor
                        expenses={expenses}
                        total_income={ctx.props().total_income}
                        on_change={expenses_on_change}
                    />
                </SectionCard>
            </div>
        }
    }
}
