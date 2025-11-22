use ai_core::api::{Cursor, Page, StorageBudget};
use ai_core::distribute::Budget;
use ai_core::editor::Plan;
use gloo_net::http::Request;
use once_cell::sync::Lazy;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use url::Url;

const API_BASE: &str = "http://localhost:3000/v1/";

static API_V1_BASE_URL: Lazy<Url> =
    Lazy::new(|| Url::parse(API_BASE).expect("Invalid API_BASE URL"));

#[derive(Deserialize)]
struct Success<T> {
    response: T,
}

pub async fn get_plan() -> Result<Plan, String> {
    let url = API_V1_BASE_URL
        .join("plan")
        .map_err(|e| format!("Failed to build URL: {e}"))?;

    let response = Request::get(url.as_str())
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

const DEFAULT_HISTORY_LIMIT: usize = 20;

pub async fn get_history(from: Option<Cursor>) -> Result<Page<StorageBudget>, String> {
    let mut url = API_V1_BASE_URL
        .join("history")
        .map_err(|e| format!("Failed to build URL: {e}"))?;

    url.query_pairs_mut()
        .append_pair("limit", &DEFAULT_HISTORY_LIMIT.to_string());

    if let Some(cursor) = from {
        url.query_pairs_mut().append_pair("from", &cursor);
    }

    let response = Request::get(url.as_str())
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let success: Success<Page<StorageBudget>> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {e}"))?;

    Ok(success.response)
}

#[derive(Serialize)]
struct AddIncomeReq {
    source_id: String,
    amount: Decimal,
    date: chrono::NaiveDate,
}

pub async fn add_income(source_id: String, amount: Decimal, date: chrono::NaiveDate) -> Result<Budget, String> {
    let url = API_V1_BASE_URL
        .join("add_income")
        .map_err(|e| format!("Failed to build URL: {e}"))?;

    let req = AddIncomeReq { source_id, amount, date };
    let response = Request::post(url.as_str())
        .json(&req)
        .map_err(|e| format!("Failed to serialize request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let success: Success<Budget> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {e}"))?;

    Ok(success.response)
}

pub async fn save_budget(budget: &Budget) -> Result<String, String> {
    let url = API_V1_BASE_URL
        .join("save_budget")
        .map_err(|e| format!("Failed to build URL: {e}"))?;

    let response = Request::post(url.as_str())
        .json(budget)
        .map_err(|e| format!("Failed to serialize request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let success: Success<String> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {e}"))?;

    Ok(success.response)
}
