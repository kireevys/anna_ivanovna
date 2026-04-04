use wasm_bindgen::prelude::wasm_bindgen;

pub mod api;
mod config;
pub mod engine;
pub mod presentation;
mod runtime;
#[cfg(feature = "tauri")] mod tauri_ipc;

#[wasm_bindgen(start)]
pub fn run_app() {
    presentation::components::set_theme(
        &presentation::components::user_prefer_theme()
            .unwrap_or(presentation::components::DEFAULT_THEME.to_string()),
    );
    yew::Renderer::<runtime::App>::new().render();
}
