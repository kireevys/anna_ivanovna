use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct WelcomeProps {
    pub default_path: String,
    pub on_pick_folder: Callback<()>,
    pub on_complete: Callback<()>,
    pub chosen_path: Option<String>,
    pub error: Option<String>,
    pub saving: bool,
}

pub struct WelcomeScreen;

impl Component for WelcomeScreen {
    type Message = ();
    type Properties = WelcomeProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let display_path = props.chosen_path.as_deref().unwrap_or(&props.default_path);

        let on_pick = {
            let cb = props.on_pick_folder.clone();
            Callback::from(move |_: MouseEvent| cb.emit(()))
        };

        let on_start = {
            let cb = props.on_complete.clone();
            Callback::from(move |_: MouseEvent| cb.emit(()))
        };

        html! {
            <div class="min-h-screen flex items-center justify-center bg-base-200">
                <div class="card w-96 bg-base-100 shadow-xl">
                    <div class="card-body items-center text-center">
                        <h2 class="card-title text-2xl mb-4">
                            {"Anna Ivanovna"}
                        </h2>
                        <p class="text-base-content/70 mb-6">
                            {"Планировщик бюджета по методу конвертов"}
                        </p>

                        <div class="form-control w-full mb-4">
                            <label class="label">
                                <span class="label-text">{"Папка для данных"}</span>
                            </label>
                            <div class="flex gap-2">
                                <input
                                    type="text"
                                    class="input input-bordered flex-1 text-sm"
                                    value={display_path.to_string()}
                                    readonly=true
                                />
                                <button
                                    class="btn btn-outline btn-sm"
                                    onclick={on_pick}
                                    disabled={props.saving}
                                >
                                    {"..."}
                                </button>
                            </div>
                        </div>

                        if let Some(err) = &props.error {
                            <div class="alert alert-error text-sm mb-4">
                                {err}
                            </div>
                        }

                        <div class="card-actions w-full">
                            <button
                                class="btn btn-primary w-full"
                                onclick={on_start}
                                disabled={props.saving}
                            >
                                if props.saving {
                                    <span class="loading loading-spinner loading-sm"></span>
                                    {"Настройка..."}
                                } else {
                                    {"Начать"}
                                }
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
