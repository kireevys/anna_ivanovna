use crate::api::{
    error::ApiError,
    types::{BudgetEntry, Cursor, Page, StoragePlanFrontend},
};
use ai_core::{distribute::Budget, plan::Plan};
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

    /// Проверяет HTTP-ответ на ошибки, возвращая тело как текст при успехе
    async fn read_response(
        &self,
        response: gloo_net::http::Response,
    ) -> Result<String, ApiError> {
        let status = response.status();
        let status_text = response.status_text();

        let body_text = response.text().await.map_err(|e| {
            ApiError::Parse(format!("Failed to read response body: {e}"))
        })?;

        if !response.ok() {
            if let Ok(error_response) =
                serde_json::from_str::<ErrorResponse>(&body_text)
            {
                return Err(ApiError::Http(status, error_response.error));
            }
            return Err(ApiError::Http(status, format!("{status} {status_text}")));
        }

        Ok(body_text)
    }

    /// Обработка ответов с JSON-телом в обёртке `{ "response": T }`
    async fn parse_response<T>(
        &self,
        response: gloo_net::http::Response,
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let body_text = self.read_response(response).await?;

        let wrapper: ResponseWrapper<T> = serde_json::from_str(&body_text)
            .map_err(|e| ApiError::Parse(format!("Failed to parse JSON: {e}")))?;

        Ok(wrapper.response)
    }

    /// Проверка ответа без разбора тела (для 204 No Content и аналогичных)
    async fn check_response(
        &self,
        response: gloo_net::http::Response,
    ) -> Result<(), ApiError> {
        self.read_response(response).await.map(|_| ())
    }

    pub async fn get_plan(&self) -> Result<StoragePlanFrontend, ApiError> {
        let url = self.build_url("plan")?;
        let response = Request::get(url.as_str())
            .send()
            .await
            .map_err(|e| ApiError::Network(format!("Request failed: {e}")))?;
        self.parse_response(response).await
    }

    pub async fn update_plan(&self, id: &str, plan: &Plan) -> Result<(), ApiError> {
        let url = self.build_url(&format!("plan/{id}"))?;
        let response = Request::put(url.as_str())
            .json(plan)
            .map_err(|e| {
                ApiError::Serialization(format!("Failed to serialize request: {e}"))
            })?
            .send()
            .await
            .map_err(|e| ApiError::Network(format!("Request failed: {e}")))?;

        self.check_response(response).await
    }

    pub async fn get_history(
        &self,
        from: Option<Cursor>,
    ) -> Result<Page<BudgetEntry>, ApiError> {
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
