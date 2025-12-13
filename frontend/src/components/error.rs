use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ErrorProps {
    pub message: String,
    pub on_retry: Callback<()>,
}

pub struct Error;

impl Component for Error {
    type Message = ();
    type Properties = ErrorProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="alert alert-error">
                <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>{ &ctx.props().message }</span>
                <div>
                    <button class="btn btn-sm" onclick={ctx.props().on_retry.reform(|_| ())}>
                        { "Повторить" }
                    </button>
                </div>
            </div>
        }
    }
}
