use wasm_bindgen::JsCast;
use web_sys::{FocusEvent, Node, window};
use yew::prelude::*;

const THEMES: &[&str] = &[
    "light",
    "dark",
    "forest",
    "cupcake",
    "bumblebee",
    "emerald",
    "corporate",
    "synthwave",
    "retro",
    "cyberpunk",
    "valentine",
    "halloween",
    "garden",
    "aqua",
    "lofi",
    "pastel",
    "fantasy",
    "wireframe",
    "black",
    "luxury",
    "dracula",
    "cmyk",
    "autumn",
    "business",
    "acid",
    "lemonade",
    "night",
    "coffee",
    "winter",
];

pub const DEFAULT_THEME: &str = "winter";

pub fn user_prefer_theme() -> Option<String> {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|ls| ls.get_item("theme").ok().flatten())
}

pub fn set_theme(theme: &str) {
    let _ = window()
        .and_then(|w| w.document())
        .and_then(|doc| doc.document_element())
        .map(|html| html.set_attribute("data-theme", theme));

    // Сохранить в localStorage
    let _ = window()
        .and_then(|w| w.local_storage().ok().flatten())
        .map(|ls| ls.set_item("theme", theme));
}

pub struct ThemeSwitcher {
    is_open: bool,
    current_theme: String,
    dropdown_root: NodeRef,
}

pub enum ThemeMsg {
    Toggle,
    SetTheme(String),
}

impl Component for ThemeSwitcher {
    type Message = ThemeMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            is_open: false,
            current_theme: user_prefer_theme().unwrap_or(DEFAULT_THEME.to_string()),
            dropdown_root: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ThemeMsg::Toggle => {
                self.is_open = !self.is_open;
                true
            }
            ThemeMsg::SetTheme(theme) => {
                set_theme(&theme);
                self.current_theme = theme;
                self.close_dropdown();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let dropdown_class = if self.is_open {
            "dropdown dropdown-end dropdown-open"
        } else {
            "dropdown dropdown-end"
        };

        let link = ctx.link().clone();
        let dropdown_ref = self.dropdown_root.clone();
        let onblur = Callback::from(move |e: FocusEvent| {
            if Self::should_close_on_blur(&dropdown_ref, e.related_target().as_ref()) {
                link.send_message(ThemeMsg::Toggle);
            }
        });

        html! {
            <div ref={self.dropdown_root.clone()} class={dropdown_class}>
                <div
                    tabindex="0"
                    role="button"
                    class="btn btn-ghost"
                    onclick={ctx.link().callback(|_| ThemeMsg::Toggle)}
                    onblur={onblur}
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    { "Тема" }
                </div>
                {if self.is_open {
                    html! {
                        <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-[1] w-52 p-2 shadow-2xl" style="max-height: 24rem; overflow-y: auto; overflow-x: hidden;">
                            {for THEMES.iter().map(|theme| {
                                let is_active = *theme == self.current_theme;
                                let button_class = if is_active {
                                    "btn btn-sm btn-ghost justify-start w-full btn-active"
                                } else {
                                    "btn btn-sm btn-ghost justify-start w-full"
                                };
                                html! {
                                    <li>
                                        <button
                                            class={button_class}
                                            onclick={ctx.link().callback(move |_| ThemeMsg::SetTheme(theme.to_string()))}
                                        >
                                            { *theme }
                                        </button>
                                    </li>
                                }
                            })}
                        </ul>
                    }
                } else {
                    html! {}
                }}
            </div>
        }
    }
}

impl ThemeSwitcher {
    fn close_dropdown(&mut self) {
        self.is_open = false;
    }

    fn should_close_on_blur(
        dropdown_root: &NodeRef,
        related_target: Option<&web_sys::EventTarget>,
    ) -> bool {
        let root = match dropdown_root.get() {
            Some(r) => r,
            None => return true,
        };

        let related = match related_target {
            Some(r) => r,
            None => return true,
        };

        let root_node = match root.dyn_ref::<Node>() {
            Some(n) => n,
            None => return true,
        };

        let related_node = match related.dyn_ref::<Node>() {
            Some(n) => n,
            None => return true,
        };

        !root_node.contains(Some(related_node))
    }
}
