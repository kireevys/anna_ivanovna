use yew::prelude::*;

pub struct Loading;

impl Component for Loading {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="flex justify-center items-center min-h-[400px]">
                <span class="loading loading-spinner loading-lg"></span>
            </div>
        }
    }
}
