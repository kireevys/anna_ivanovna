use wasm_bindgen::prelude::wasm_bindgen;

mod api;
mod app;

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<app::App>::new().render();
}
