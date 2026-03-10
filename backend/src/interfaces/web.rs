use ai_core::{
    api::{self, BudgetId, CoreApi, CoreRepo, Cursor, Page, StorageBudget},
    distribute::{Budget, Income},
    finance::Money,
    plan::Plan,
};
use axum::{
    Json,
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Debug)]
enum ApiError {
    NotFound,
    Storage(String),
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not found".into()),
            ApiError::Storage(e) => (StatusCode::BAD_REQUEST, e),
            ApiError::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
        };
        let body = Json(serde_json::json!({ "error": message }));
        (status, body).into_response()
    }
}

async fn health_handler() -> &'static str {
    "ok"
}

async fn plan_handler<R: CoreRepo>(
    State(api): State<CoreApi<R>>,
) -> Result<Success<Plan>, ApiError> {
    api.get_plan().map(Success::new).ok_or(ApiError::NotFound)
}

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    from: Option<Cursor>,
    limit: usize,
}

async fn history<R: CoreRepo>(
    State(api): State<CoreApi<R>>,
    Query(params): Query<HistoryQuery>,
) -> Success<Page<StorageBudget>> {
    let HistoryQuery { from, limit } = params;
    let page = api.budget_list(from, limit);
    Success::new(page)
}

#[derive(Debug, Deserialize)]
struct NewIncome {
    source_id: String,
    amount: Decimal,
    date: NaiveDate,
}

async fn add_income<R: CoreRepo>(
    State(api): State<CoreApi<R>>,
    Json(income): Json<NewIncome>,
) -> Result<Success<Budget>, ApiError> {
    let plan = api.get_plan().ok_or(ApiError::NotFound)?;
    let source = plan
        .sources
        .iter()
        .find(|s| s.name == income.source_id)
        .ok_or(ApiError::NotFound)?;
    let NewIncome {
        source_id,
        amount,
        date,
    } = income;
    info!(source_id = source_id, date = %date, amount = %amount);
    let income = Income::new(source.clone(), Money::new_rub(amount), date);
    let weights = plan.try_into().map_err(|_| ApiError::Internal)?;
    let budget = api
        .distribute(&weights, &income)
        .map_err(|_| ApiError::Internal)?;
    Ok(Success::new(budget))
}

#[derive(Debug, Serialize)]
struct Success<T: Serialize> {
    response: T,
}

impl<T: Serialize> Success<T> {
    fn new(response: T) -> Self {
        Self { response }
    }
}

impl<T: Serialize> IntoResponse for Success<T> {
    fn into_response(self) -> axum::response::Response {
        let body = Json(self);
        (StatusCode::OK, body).into_response()
    }
}

async fn save_budget<R: CoreRepo>(
    State(api): State<CoreApi<R>>,
    Json(budget): Json<Budget>,
) -> Result<Success<BudgetId>, ApiError> {
    let budget_id = api
        .save_budget(CoreApi::<R>::build_budget_id(), budget)
        .map_err(|e| ApiError::Storage(e.to_string()))?;
    Ok(Success::new(budget_id))
}

async fn budget<R: CoreRepo>(
    State(api): State<CoreApi<R>>,
    Path(id): Path<BudgetId>,
) -> Result<Success<StorageBudget>, ApiError> {
    api.budget_by_id(&id)
        .map(Success::new)
        .ok_or(ApiError::NotFound)
}

pub async fn run<R>(api: CoreApi<R>, addr: &str) -> Result<(), std::io::Error>
where
    R: api::CoreRepo + Clone + Send + Sync + 'static,
{
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/v1/plan", get(plan_handler))
        .route("/v1/history", get(history::<R>))
        .route("/v1/add_income", post(add_income::<R>))
        .route("/v1/save_budget", post(save_budget::<R>))
        .route("/v1/budget/{id}", get(budget::<R>))
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                .allow_headers(tower_http::cors::Any),
        )
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
                .on_request(
                    |_request: &axum::http::Request<_>, _span: &tracing::Span| {
                        tracing::info!("request started");
                    },
                )
                .on_response(
                    |_response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!("response generated in {:?}", latency);
                    },
                )
                .on_failure(
                    |_error: tower_http::classify::ServerErrorsFailureClass,
                     _latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::error!("request failed");
                    },
                ),
        )
        .with_state(api);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("web interface is listening on {addr}");
    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}
