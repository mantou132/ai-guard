use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    models::{ConfigView, CreateAccount, LogQuery, ReportQuery, UpdateAccount},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/config", get(config))
        .route("/api/stats", get(stats))
        .route("/api/accounts", get(list_accounts).post(create_account))
        .route(
            "/api/accounts/{id}",
            put(update_account).delete(delete_account),
        )
        .route("/api/accounts/{id}/check", post(check_account))
        .route("/api/logs", get(list_logs))
        .route("/api/reports", get(list_reports))
}

async fn health() -> impl IntoResponse {
    Json(json!({ "ok": true }))
}

async fn config(State(state): State<AppState>) -> impl IntoResponse {
    Json(ConfigView {
        bind: state.config.bind.clone(),
        data_dir: state.config.data_dir.display().to_string(),
        audit_enabled: state.config.audit_enabled(),
        audit_model: state.config.audit_model.clone(),
        max_body_bytes: state.config.max_body_bytes,
        preview_bytes: state.config.preview_bytes,
    })
}

async fn stats(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.store.stats().await)
}

async fn list_accounts(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.store.list_accounts().await)
}

async fn create_account(
    State(state): State<AppState>,
    Json(payload): Json<CreateAccount>,
) -> Result<impl IntoResponse, ApiError> {
    Ok((
        StatusCode::CREATED,
        Json(state.store.create_account(payload).await?),
    ))
}

async fn update_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAccount>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(state.store.update_account(id, payload).await?))
}

async fn delete_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.store.delete_account(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn check_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let account = state
        .store
        .get_account(id)
        .await
        .ok_or_else(|| anyhow::anyhow!("account not found"))?;

    let endpoint = crate::proxy::join_url(&account.base_url, "/v1/models", None);
    let response = match account.api_type {
        crate::models::ApiType::OpenAi => {
            state
                .client
                .get(endpoint)
                .bearer_auth(&account.api_key)
                .send()
                .await
        }
        crate::models::ApiType::Anthropic => {
            state
                .client
                .get(endpoint)
                .header("anthropic-version", "2023-06-01")
                .header("x-api-key", &account.api_key)
                .send()
                .await
        }
    };

    match response {
        Ok(response) => Ok(Json(json!({
            "ok": response.status().is_success(),
            "status": response.status().as_u16()
        }))),
        Err(err) => Ok(Json(json!({
            "ok": false,
            "error": err.to_string()
        }))),
    }
}

async fn list_logs(
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> impl IntoResponse {
    Json(state.store.logs(query).await)
}

async fn list_reports(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    Json(state.store.reports(query).await)
}

struct ApiError(anyhow::Error);

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let message = self.0.to_string();
        let status = if message.contains("not found") {
            StatusCode::NOT_FOUND
        } else if message.contains("required") || message.contains("base_url") {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
