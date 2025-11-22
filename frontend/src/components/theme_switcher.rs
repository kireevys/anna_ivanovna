use web_sys::window;
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

pub struct ThemeSwitcher;

pub enum ThemeMsg {
    SetTheme(String),
}

impl Component for ThemeSwitcher {
    type Message = ThemeMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ThemeMsg::SetTheme(theme) => {
                    set_theme(&theme);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="dropdown dropdown-end">
                <div tabindex="0" role="button" class="btn btn-ghost">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    { "Тема" }
                </div>
                <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-[1] w-52 p-2 shadow-2xl" style="max-height: 24rem; overflow-y: auto; overflow-x: hidden;">
                    {for THEMES.iter().map(|theme| {
                        let theme_clone = theme.to_string();
                        html! {
                            <li>
                                <button
                                    class="btn btn-sm btn-ghost justify-start w-full"
                                    onclick={ctx.link().callback(move |_| ThemeMsg::SetTheme(theme_clone.clone()))}
                                >
                                    { *theme }
                                </button>
                            </li>
                        }
                    })}
                </ul>
            </div>
        }
    }
}
