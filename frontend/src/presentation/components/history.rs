use yew::prelude::*;

use ai_core::{finance::Money, planning::IncomeKind};

use crate::{
    engine::history::HistoryEntry,
    presentation::{
        formatting::FormattedMoney,
        income::{OTHER_LABEL, SALARY_LABEL},
    },
};

#[derive(Properties, PartialEq)]
pub struct HistoryProps {
    pub entries: Vec<HistoryEntry>,
}

pub struct HistoryView;

impl Component for HistoryView {
    type Message = ();
    type Properties = HistoryProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="space-y-6">
                <div class="join join-vertical w-full">
                    {for ctx.props().entries.iter().map(|entry| {
                        let kind_label = match &entry.source_kind {
                            IncomeKind::Salary { .. } => SALARY_LABEL,
                            IncomeKind::Other { .. } => OTHER_LABEL,
                        };
                        html! {
                            <div class="collapse collapse-arrow join-item border border-base-300 bg-base-100">
                                <input type="checkbox" />
                                <div class="collapse-title text-xl font-medium">
                                    <div class="flex justify-between items-center w-full pr-8">
                                        <div>
                                            <h3 class="text-xl font-bold">{ entry.date.format("%Y-%m-%d").to_string() }</h3>
                                            <p class="text-sm text-base-content/70">
                                                { &entry.source_name }
                                                <span class="badge badge-sm badge-ghost ml-1">{ kind_label }</span>
                                            </p>
                                        </div>
                                        <div class="text-right">
                                            <p class="text-lg font-semibold text-success">{ "Доход: " }{ FormattedMoney::from_money(entry.income_amount).to_string() }</p>
                                            <p class="text-lg font-semibold text-warning">{ "Остаток: " }{ FormattedMoney::from_money(entry.rest).to_string() }</p>
                                        </div>
                                    </div>
                                </div>
                                <div class="collapse-content">
                                    {if let IncomeKind::Salary { gross, tax_rate } = &entry.source_kind {
                                        let tax_money = Money::new(tax_rate.apply_to(gross.value), gross.currency);
                                        let rate_display = tax_rate
                                            .to_string()
                                            .trim_end_matches('%')
                                            .trim()
                                            .to_string();
                                        html! {
                                            <div class="card bg-warning/10 border border-warning/30 shadow mb-4 mt-4">
                                                <div class="card-body p-4">
                                                    <h4 class="font-semibold text-warning">{ "Налоги" }</h4>
                                                    <div class="space-y-1 text-sm">
                                                        <div class="flex justify-between">
                                                            <span>{ "Gross" }</span>
                                                            <span class="font-bold">{ FormattedMoney::from_money(*gross).to_string() }</span>
                                                        </div>
                                                        <div class="flex justify-between">
                                                            <span>{ format!("Налог ({rate_display}%)") }</span>
                                                            <span class="font-bold text-warning">{ FormattedMoney::from_money(tax_money).to_string() }</span>
                                                        </div>
                                                        <div class="divider my-1"></div>
                                                        <div class="flex justify-between">
                                                            <span>{ "На руки" }</span>
                                                            <span class="font-bold text-success">{ FormattedMoney::from_money(entry.income_amount).to_string() }</span>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }}
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 pt-4">
                                        {for entry.categories.iter().map(|category| {
                                            html! {
                                                <div class="card bg-base-200 shadow">
                                                    <div class="card-body p-4">
                                                        <h4 class="font-semibold text-lg mb-2">{ &category.name }</h4>
                                                        <div class="space-y-1">
                                                            {for category.entries.iter().map(|expense| {
                                                                html! {
                                                                    <div class="flex justify-between items-center text-sm">
                                                                        <span>{ &expense.name }</span>
                                                                        <span class="font-bold">{ FormattedMoney::from_money(expense.amount).to_string() }</span>
                                                                    </div>
                                                                }
                                                            })}
                                                        </div>
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
            </div>
        }
    }
}
