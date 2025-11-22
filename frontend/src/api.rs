use ai_core::editor::Plan;
use gloo_net::http::Request;
use serde::Deserialize;

const API_BASE: &str = "http://localhost:3000";

#[derive(Deserialize)]
struct Success<T> {
    response: T,
}

pub async fn get_plan() -> Result<Plan, String> {
    let response = Request::get(&format!("{}/v1/plan", API_BASE))
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let success: Success<Plan> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {e}"))?;

    Ok(success.response)
}
