use wasm_bindgen::prelude::wasm_bindgen;

mod api;
mod app;
mod components;
mod presentation;

#[wasm_bindgen(start)]
pub fn run_app() {
    if let Some(theme) = components::user_prefer_theme() {
        components::set_theme(&theme);
    }
    yew::Renderer::<app::App>::new().render();
}
