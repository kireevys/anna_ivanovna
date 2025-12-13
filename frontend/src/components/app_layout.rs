use crate::app::View;
use crate::components::ThemeSwitcher;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AppLayoutProps {
    pub current_view: View,
    pub on_switch_view: Callback<View>,
    pub children: Children,
}

pub struct AppLayout;

impl Component for AppLayout {
    type Message = ();
    type Properties = AppLayoutProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="min-h-screen bg-base-200">
                <div class="container mx-auto px-4 py-8">
                    <div class="flex justify-between items-center mb-8">
                        <h1 class="text-4xl font-bold">
                            { "Anna Ivanovna" }
                        </h1>
                        <ThemeSwitcher />
                    </div>
                    <div class="tabs tabs-boxed mb-6">
                        <button
                            class={format!("tab {}", if ctx.props().current_view == View::Plan { "tab-active" } else { "" })}
                            onclick={ctx.props().on_switch_view.reform(|_| View::Plan)}
                        >
                            { "План" }
                        </button>
                        <button
                            class={format!("tab {}", if ctx.props().current_view == View::History { "tab-active" } else { "" })}
                            onclick={ctx.props().on_switch_view.reform(|_| View::History)}
                        >
                            { "История" }
                        </button>
                    </div>
                    { ctx.props().children.clone() }
                </div>
            </div>
        }
    }
}
