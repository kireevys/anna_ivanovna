use crate::presentation::{
    components::plan::expense_card::ExpenseCard,
    plan::read::{CategoryKey, Expense},
};
use std::collections::BTreeMap;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ExpenseCategoriesProps {
    pub categories: BTreeMap<CategoryKey, Vec<Expense>>,
}

pub struct ExpenseCategories;

impl Component for ExpenseCategories {
    type Message = ();
    type Properties = ExpenseCategoriesProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {for ctx.props().categories.iter().map(|(key, expenses)| {
                    let name = key.display_name();
                    html! {
                        <div class="card bg-base-200 shadow">
                            <div class="card-body">
                                <h3 class="card-title text-xl mb-4">{ name }</h3>
                                <div class="space-y-2">
                                    {for expenses.iter().map(|expense| {
                                        html! {
                                            <ExpenseCard expense={expense.clone()} />
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
