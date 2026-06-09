use std::{collections::HashSet, time::Instant};

use axum::{
    Router,
    body::{Body, Bytes, to_bytes},
    extract::{OriginalUri, State},
    http::{
        HeaderMap, HeaderName, Method, StatusCode, Uri,
        header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, HOST},
    },
    response::Response,
    routing::any,
};
use chrono::Utc;
use futures_util::{StreamExt, stream::BoxStream};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    audit,
    models::{ApiType, AuditStatus, LogEntry, LogLevel, RiskLevel},
    payload,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/{*path}", any(proxy_openai_default))
        .route("/openai/{*path}", any(proxy_openai_prefixed))
        .route("/anthropic/{*path}", any(proxy_anthropic_prefixed))
}

async fn proxy_openai_default(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Body,
) -> Response {
    proxy(state, ApiType::OpenAi, None, method, uri, headers, body).await
}

async fn proxy_openai_prefixed(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Body,
) -> Response {
    proxy(
        state,
        ApiType::OpenAi,
        Some("/openai"),
        method,
        uri,
        headers,
        body,
    )
    .await
}

async fn proxy_anthropic_prefixed(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Body,
) -> Response {
    proxy(
        state,
        ApiType::Anthropic,
        Some("/anthropic"),
        method,
        uri,
        headers,
        body,
    )
    .await
}

struct ProxyContext {
    state: AppState,
    api_type: ApiType,
    method: String,
    path: String,
    model: Option<String>,
    account_id: Uuid,
    account_name: String,
    started_at: Instant,
    request_payload: Value,
}

async fn finalize_log(
    context: ProxyContext,
    status_code: Option<u16>,
    response_payload: Value,
    error: Option<String>,
) {
    let ProxyContext {
        state,
        api_type,
        method,
        path,
        model,
        account_id,
        account_name,
        started_at,
        request_payload,
    } = context;

    if let Some(message) = error.as_ref() {
        let _ = state
            .store
            .mark_failure(account_id, status_code, message.clone())
            .await;
    } else {
        let _ = state.store.mark_success(account_id).await;
    }

    let level = if let Some(code) = status_code.and_then(|c| StatusCode::from_u16(c).ok()) {
        response_level(code)
    } else {
        LogLevel::Error
    };

    let log = LogEntry {
        id: Uuid::new_v4(),
        created_at: Utc::now(),
        level,
        api_type,
        method,
        path,
        model: model.clone(),
        account_id: Some(account_id),
        account_name: Some(account_name),
        status_code,
        latency_ms: Some(started_at.elapsed().as_millis() as u64),
        request_payload: if error.is_none() {
            request_payload.clone()
        } else {
            Value::Null
        },
        response_payload: if error.is_none() {
            response_payload.clone()
        } else {
            Value::Null
        },
        risk_level: RiskLevel::Unknown,
        audit_status: if error.is_none() {
            AuditStatus::Queued
        } else {
            AuditStatus::Skipped
        },
        error,
    };

    if let Ok(log) = state.store.append_log(log).await {
        if log.audit_status == AuditStatus::Queued {
            audit::spawn_audit(state, log.id, request_payload, response_payload, model);
        }
    }
}

async fn proxy(
    state: AppState,
    api_type: ApiType,
    strip_prefix: Option<&str>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Response {
    let path = uri.path().to_string();
    let target_path = strip_path_prefix(uri.path(), strip_prefix);
    let query = uri.query().map(str::to_string);
    let request_bytes = match to_bytes(body, state.config.max_body_bytes).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return json_response(
                StatusCode::PAYLOAD_TOO_LARGE,
                json!({ "error": format!("request body is too large or invalid: {err}") }),
            );
        }
    };

    let model = payload::extract_model(&request_bytes);
    let request_payload =
        payload::extract_request_payload(&api_type, &request_bytes, state.config.preview_bytes);
    let mut tried = HashSet::<Uuid>::new();

    loop {
        let tried_ids: Vec<Uuid> = tried.iter().copied().collect();
        let Some(account) = state
            .store
            .choose_account(api_type.clone(), model.as_deref(), &tried_ids)
            .await
        else {
            return json_response(
                StatusCode::SERVICE_UNAVAILABLE,
                json!({ "error": "no available upstream account" }),
            );
        };
        tried.insert(account.id);

        let upstream_url = join_url(&account.base_url, &target_path, query.as_deref());
        let mut request = state.client.request(method.clone(), upstream_url);
        for (name, value) in headers.iter() {
            if should_forward_request_header(name) {
                request = request.header(name, value);
            }
        }
        request = match api_type {
            ApiType::OpenAi => request.bearer_auth(&account.api_key),
            ApiType::Anthropic => {
                let request = request.header("x-api-key", &account.api_key);
                if headers.contains_key("anthropic-version") {
                    request
                } else {
                    request.header("anthropic-version", "2023-06-01")
                }
            }
        };

        let context = ProxyContext {
            state: state.clone(),
            api_type: api_type.clone(),
            method: method.to_string(),
            path: path.clone(),
            model: model.clone(),
            account_id: account.id,
            account_name: account.name.clone(),
            started_at: Instant::now(),
            request_payload: request_payload.clone(),
        };

        let result = request.body(request_bytes.clone()).send().await;
        match result {
            Ok(response) => {
                let status = StatusCode::from_u16(response.status().as_u16())
                    .unwrap_or(StatusCode::BAD_GATEWAY);
                let response_headers = response.headers().clone();

                if status.is_client_error() || status.is_server_error() {
                    finalize_log(
                        context,
                        Some(status.as_u16()),
                        Value::Null,
                        Some(format!("upstream returned HTTP {}", status.as_u16())),
                    )
                    .await;
                    continue;
                }

                if status.is_success() && response_is_streaming(&response_headers) {
                    return upstream_streaming_response(
                        status,
                        response_headers,
                        response.bytes_stream().boxed(),
                        context,
                    );
                }

                let response_bytes = match response.bytes().await {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        finalize_log(
                            context,
                            Some(status.as_u16()),
                            Value::Null,
                            Some(format!("failed to read upstream response: {err}")),
                        )
                        .await;
                        continue;
                    }
                };

                let response_payload = payload::extract_response_payload(
                    &api_type,
                    &response_bytes,
                    state.config.preview_bytes,
                );
                finalize_log(
                    context,
                    Some(status.as_u16()),
                    response_payload.clone(),
                    None,
                )
                .await;

                return upstream_response(status, response_headers, response_bytes);
            }
            Err(err) => {
                finalize_log(
                    context,
                    None,
                    Value::Null,
                    Some(format!("upstream request failed: {err}")),
                )
                .await;
                continue;
            }
        }
    }
}

struct StreamingLogState {
    stream: BoxStream<'static, Result<Bytes, reqwest::Error>>,
    collected: Vec<u8>,
    status_code: u16,
    context: Option<ProxyContext>,
}

fn strip_path_prefix(path: &str, prefix: Option<&str>) -> String {
    let stripped = prefix
        .and_then(|prefix| path.strip_prefix(prefix))
        .unwrap_or(path);
    if stripped.is_empty() {
        "/".to_string()
    } else if stripped.starts_with('/') {
        stripped.to_string()
    } else {
        format!("/{stripped}")
    }
}

pub fn join_url(base_url: &str, path: &str, query: Option<&str>) -> String {
    let base_url = base_url.trim_end_matches('/');
    let path = if base_url.ends_with("/v1") {
        if path == "/v1" {
            ""
        } else {
            path.strip_prefix("/v1").unwrap_or(path)
        }
    } else {
        path
    };

    let mut url = format!("{base_url}{path}");
    if let Some(query) = query {
        url.push('?');
        url.push_str(query);
    }
    url
}

fn should_forward_request_header(name: &HeaderName) -> bool {
    !matches!(*name, HOST | AUTHORIZATION | CONTENT_LENGTH)
}

fn should_forward_response_header(name: &HeaderName) -> bool {
    !matches!(
        name.as_str(),
        "connection"
            | "content-length"
            | "transfer-encoding"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "upgrade"
    )
}

fn response_is_streaming(headers: &HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("text/event-stream"))
}

fn response_level(status: StatusCode) -> LogLevel {
    if status.is_server_error() {
        LogLevel::Error
    } else if !status.is_success() {
        LogLevel::Warn
    } else {
        LogLevel::Info
    }
}

fn upstream_streaming_response(
    status: StatusCode,
    headers: HeaderMap,
    stream: BoxStream<'static, Result<Bytes, reqwest::Error>>,
    context: ProxyContext,
) -> Response {
    let stream = futures_util::stream::unfold(
        Some(StreamingLogState {
            stream,
            collected: Vec::new(),
            status_code: status.as_u16(),
            context: Some(context),
        }),
        |state| async move {
            let StreamingLogState {
                mut stream,
                mut collected,
                status_code,
                mut context,
            } = state?;

            match stream.next().await {
                Some(Ok(bytes)) => {
                    collected.extend_from_slice(&bytes);
                    Some((
                        Ok(bytes),
                        Some(StreamingLogState {
                            stream,
                            collected,
                            status_code,
                            context,
                        }),
                    ))
                }
                Some(Err(err)) => {
                    let message = format!("upstream stream failed: {err}");
                    if let Some(context) = context.take() {
                        finalize_stream_log(
                            context,
                            status_code,
                            Bytes::from(collected),
                            Some(message),
                        )
                        .await;
                    }
                    Some((Err(err), None))
                }
                None => {
                    if let Some(context) = context.take() {
                        finalize_stream_log(context, status_code, Bytes::from(collected), None)
                            .await;
                    }
                    None
                }
            }
        },
    );

    let mut builder = Response::builder().status(status);
    for (name, value) in headers.iter() {
        if should_forward_response_header(name) {
            builder = builder.header(name, value);
        }
    }
    builder
        .body(Body::from_stream(stream))
        .unwrap_or_else(|err| {
            json_response(
                StatusCode::BAD_GATEWAY,
                json!({ "error": format!("failed to build streaming response: {err}") }),
            )
        })
}

async fn finalize_stream_log(
    context: ProxyContext,
    status_code: u16,
    response_bytes: Bytes,
    stream_error: Option<String>,
) {
    let preview_bytes = context.state.config.preview_bytes;
    let response_payload =
        payload::extract_response_payload(&context.api_type, &response_bytes, preview_bytes);
    finalize_log(context, Some(status_code), response_payload, stream_error).await;
}

fn upstream_response(status: StatusCode, headers: HeaderMap, bytes: Bytes) -> Response {
    let mut builder = Response::builder().status(status);
    for (name, value) in headers.iter() {
        if should_forward_response_header(name) {
            builder = builder.header(name, value);
        }
    }
    builder.body(Body::from(bytes)).unwrap_or_else(|err| {
        json_response(
            StatusCode::BAD_GATEWAY,
            json!({ "error": format!("failed to build response: {err}") }),
        )
    })
}

fn json_response(status: StatusCode, body: serde_json::Value) -> Response {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}
