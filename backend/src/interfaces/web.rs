use ai_app::{
    api::{CoreApi, Error as AppError},
    storage::{
        BudgetId,
        CoreRepo,
        Page,
        PlanDraft,
        PlanId,
        StorageBudget,
        StoragePlan,
        UserId,
        build_id,
    },
};
use ai_core::{
    distribute::{Budget, Income},
    finance::Money,
};
use axum::{
    Json,
    Router,
    extract::{FromRequestParts, Path, Query, State},
    http::{StatusCode, request::Parts},
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
    Conflict(String),
    Validation(String),
    Storage(String),
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not found".into()),
            ApiError::Conflict(e) => (StatusCode::CONFLICT, e),
            ApiError::Validation(e) => (StatusCode::UNPROCESSABLE_ENTITY, e),
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
    CurrentUser(user_id): CurrentUser,
) -> Result<Success<StoragePlan>, ApiError> {
    api.get_plan(&user_id)
        .await
        .map(Success::new)
        .ok_or(ApiError::NotFound)
}

async fn create_plan_handler<R: CoreRepo>(
    State(api): State<CoreApi<R>>,
    CurrentUser(user_id): CurrentUser,
    Json(draft): Json<PlanDraft>,
) -> Result<Success<PlanId>, ApiError> {
    api.create_plan(&user_id, build_id(), draft)
        .await
        .map(Success::new)
        .map_err(|e| match e {
            AppError::PlanAlreadyExists => ApiError::Conflict(e.to_string()),
            AppError::InvalidPlan { .. } => ApiError::Validation(e.to_string()),
            _ => ApiError::Internal,
        })
}

async fn update_plan_handler<R: CoreRepo>(
    State(api): State<CoreApi<R>>,
    CurrentUser(user_id): CurrentUser,
    Path(plan_id): Path<PlanId>,
    Json(draft): Json<PlanDraft>,
) -> Result<StatusCode, ApiError> {
    api.update_plan(&user_id, plan_id, draft)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| match e {
            AppError::PlanNotFound => ApiError::NotFound,
            AppError::InvalidPlan { .. } => ApiError::Validation(e.to_string()),
            _ => ApiError::Internal,
        })
}

async fn delete_plan_handler<R: CoreRepo>(
    State(api): State<CoreApi<R>>,
    CurrentUser(user_id): CurrentUser,
    Path(plan_id): Path<PlanId>,
) -> Result<StatusCode, ApiError> {
    api.delete_plan(&user_id, plan_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|_| ApiError::Internal)
}

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    from: Option<ai_app::storage::Cursor>,
    limit: usize,
}

async fn history<R: CoreRepo>(
    State(api): State<CoreApi<R>>,
    Query(params): Query<PaginationQuery>,
) -> Success<Page<StorageBudget>> {
    let PaginationQuery { from, limit } = params;
    let page = api.budget_list(from, limit).await;
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
    CurrentUser(user_id): CurrentUser,
    Json(income): Json<NewIncome>,
) -> Result<Success<Budget>, ApiError> {
    let sp = api.get_plan(&user_id).await.ok_or(ApiError::NotFound)?;
    let source = sp
        .plan
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
    let weights = sp.plan.try_into().map_err(|_| ApiError::Internal)?;
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
        .save_budget(build_id(), budget)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;
    Ok(Success::new(budget_id))
}

async fn budget<R: CoreRepo>(
    State(api): State<CoreApi<R>>,
    Path(id): Path<BudgetId>,
) -> Result<Success<StorageBudget>, ApiError> {
    api.budget_by_id(&id)
        .await
        .map(Success::new)
        .ok_or(ApiError::NotFound)
}

async fn collections_handler() -> Success<Vec<ai_core::templates::Collection>> {
    Success {
        response: ai_core::templates::collections(),
    }
}

pub fn create_router<R>(api: CoreApi<R>) -> Router
where
    R: CoreRepo + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/collections", get(collections_handler))
        .route(
            "/v1/plan",
            get(plan_handler::<R>).post(create_plan_handler::<R>),
        )
        .route(
            "/v1/plan/{plan_id}",
            axum::routing::put(update_plan_handler::<R>)
                .delete(delete_plan_handler::<R>),
        )
        .route("/v1/history", get(history::<R>))
        .route("/v1/add_income", post(add_income::<R>))
        .route("/v1/save_budget", post(save_budget::<R>))
        .route("/v1/budget/{id}", get(budget::<R>))
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                ])
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
        .with_state(api)
}

pub async fn run<R>(api: CoreApi<R>, addr: &str) -> Result<(), std::io::Error>
where
    R: CoreRepo + Clone + Send + Sync + 'static,
{
    let app = create_router(api);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("web interface is listening on {addr}");
    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}

#[derive(Clone, Debug)]
struct CurrentUser(UserId);

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Временно: пока нет auth, user_id приходит из заголовка,
        // но оставляем default для совместимости со старым клиентом.
        let user_id = parts
            .headers
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "default".to_string());
        Ok(Self(user_id))
    }
}
