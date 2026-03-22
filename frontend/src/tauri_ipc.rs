use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    fn tauri_invoke(cmd: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    fn tauri_invoke_with_args(cmd: &str, args: &JsValue) -> js_sys::Promise;
}

pub async fn is_configured() -> Result<bool, String> {
    let promise = tauri_invoke("is_configured");
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("{e:?}"))?;
    result.as_bool().ok_or_else(|| "expected bool".to_string())
}

pub async fn get_default_buh_home() -> Result<String, String> {
    let promise = tauri_invoke("get_default_buh_home");
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("{e:?}"))?;
    result
        .as_string()
        .ok_or_else(|| "expected string".to_string())
}

pub async fn pick_data_folder() -> Result<Option<String>, String> {
    let promise = tauri_invoke("pick_data_folder");
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("{e:?}"))?;
    if result.is_null() || result.is_undefined() {
        Ok(None)
    } else {
        Ok(result.as_string())
    }
}

pub async fn complete_setup(buh_home: &str) -> Result<(), String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &"buhHome".into(), &buh_home.into())
        .map_err(|e| format!("{e:?}"))?;
    let promise = tauri_invoke_with_args("complete_setup", &args);
    JsFuture::from(promise)
        .await
        .map_err(|e| format!("{e:?}"))?;
    Ok(())
}

pub async fn start_app_backend() -> Result<(), String> {
    let promise = tauri_invoke("start_app_backend");
    JsFuture::from(promise)
        .await
        .map_err(|e| format!("{e:?}"))?;
    Ok(())
}
