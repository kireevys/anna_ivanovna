use wasm_bindgen::prelude::wasm_bindgen;

mod api;
mod app;
mod components;
mod config;
mod presentation;
#[cfg(feature = "tauri")] mod tauri_ipc;

#[wasm_bindgen(start)]
pub fn run_app() {
    components::set_theme(
        &components::user_prefer_theme()
            .unwrap_or(components::DEFAULT_THEME.to_string()),
    );
    yew::Renderer::<app::App>::new().render();
}
