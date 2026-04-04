use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct EditActionsBarProps {
    pub disable_save: bool,
    pub on_cancel: Callback<()>,
    pub on_save: Callback<()>,
}

pub struct EditActionsBar;

impl Component for EditActionsBar {
    type Message = ();
    type Properties = EditActionsBarProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let disable_save = ctx.props().disable_save;

        let on_cancel = {
            let cb = ctx.props().on_cancel.clone();
            Callback::from(move |_| cb.emit(()))
        };

        let on_save = {
            let cb = ctx.props().on_save.clone();
            Callback::from(move |_| cb.emit(()))
        };

        html! {
            <div class="space-x-2">
                <button
                    class="btn btn-ghost btn-sm"
                    onclick={on_cancel}
                >
                    { "Отмена" }
                </button>
                <button
                    class={if disable_save {
                        "btn btn-primary btn-sm btn-disabled"
                    } else {
                        "btn btn-primary btn-sm"
                    }}
                    disabled={disable_save}
                    onclick={on_save}
                >
                    { "Сохранить" }
                </button>
            </div>
        }
    }
}
