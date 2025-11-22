use crate::presentation::history::HistoryEntry;
use yew::prelude::*;

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
                <h2 class="text-2xl font-bold mb-6 text-center">{ "История распределений" }</h2>
                <div class="join join-vertical w-full">
                    {for ctx.props().entries.iter().map(|entry| {
                        html! {
                            <div class="collapse collapse-arrow join-item border border-base-300 bg-base-100">
                                <input type="checkbox" />
                                <div class="collapse-title text-xl font-medium">
                                    <div class="flex justify-between items-center w-full pr-8">
                                        <div>
                                            <h3 class="text-xl font-bold">{ &entry.date }</h3>
                                            <p class="text-sm text-base-content/70">{ &entry.source_name }</p>
                                        </div>
                                        <div class="text-right">
                                            <p class="text-lg font-semibold text-success">{ "Доход: " }{ &entry.income_amount }</p>
                                            <p class="text-lg font-semibold text-warning">{ "Остаток: " }{ &entry.rest }</p>
                                        </div>
                                    </div>
                                </div>
                                <div class="collapse-content">
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
                                                                        <span class="font-bold">{ &expense.amount }</span>
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
