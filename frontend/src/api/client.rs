use crate::api::error::ApiError;
use ai_core::{
    api::{Cursor, Page, StorageBudget},
    distribute::Budget,
    plan::Plan,
};
use chrono::NaiveDate;
use gloo_net::http::Request;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use url::Url;

#[derive(Deserialize)]
struct ResponseWrapper<T> {
    response: T,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
pub struct AddIncomeRequest {
    pub source_id: String,
    pub amount: Decimal,
    pub date: NaiveDate,
}

#[derive(PartialEq)]
pub struct ApiClient {
    base_url: Url,
}

impl ApiClient {
    pub fn new(base_url: Url) -> Self {
        Self { base_url }
    }

    fn build_url(&self, path: &str) -> Result<Url, ApiError> {
        self.base_url
            .join(path)
            .map_err(|e| ApiError::InvalidUrl(format!("Failed to build URL: {e}")))
    }

    /// Централизованный метод обработки ответов
    async fn parse_response<T>(
        &self,
        response: gloo_net::http::Response,
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let status_text = response.status_text();

        // Читаем тело ответа как текст
        let body_text = response.text().await.map_err(|e| {
            ApiError::Parse(format!("Failed to read response body: {e}"))
        })?;

        if !response.ok() {
            // Пытаемся извлечь error из тела ответа
            if let Ok(error_response) =
                serde_json::from_str::<ErrorResponse>(&body_text)
            {
                return Err(ApiError::Http(status, error_response.error));
            }
            return Err(ApiError::Http(status, format!("{status} {status_text}")));
        }

        // Десериализуем успешный ответ
        let wrapper: ResponseWrapper<T> = serde_json::from_str(&body_text)
            .map_err(|e| ApiError::Parse(format!("Failed to parse JSON: {e}")))?;

        Ok(wrapper.response)
    }

    pub async fn get_plan(&self) -> Result<Plan, ApiError> {
        let url = self.build_url("plan")?;
        let response = Request::get(url.as_str())
            .send()
            .await
            .map_err(|e| ApiError::Network(format!("Request failed: {e}")))?;
        self.parse_response(response).await
    }

    pub async fn get_history(
        &self,
        from: Option<Cursor>,
    ) -> Result<Page<StorageBudget>, ApiError> {
        let mut url = self.build_url("history")?;

        url.query_pairs_mut().append_pair("limit", "20");

        if let Some(cursor) = from {
            url.query_pairs_mut().append_pair("from", &cursor);
        }

        let response = Request::get(url.as_str())
            .send()
            .await
            .map_err(|e| ApiError::Network(format!("Request failed: {e}")))?;
        self.parse_response(response).await
    }

    pub async fn add_income(
        &self,
        request: AddIncomeRequest,
    ) -> Result<Budget, ApiError> {
        let url = self.build_url("add_income")?;
        let response = Request::post(url.as_str())
            .json(&request)
            .map_err(|e| {
                ApiError::Serialization(format!("Failed to serialize request: {e}"))
            })?
            .send()
            .await
            .map_err(|e| ApiError::Network(format!("Request failed: {e}")))?;

        self.parse_response(response).await
    }

    pub async fn save_budget(&self, budget: &Budget) -> Result<String, ApiError> {
        let url = self.build_url("save_budget")?;
        let response = Request::post(url.as_str())
            .json(budget)
            .map_err(|e| {
                ApiError::Serialization(format!("Failed to serialize request: {e}"))
            })?
            .send()
            .await
            .map_err(|e| ApiError::Network(format!("Request failed: {e}")))?;

        self.parse_response(response).await
    }
}
