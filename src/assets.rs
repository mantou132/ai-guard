use axum::{
    body::Body,
    extract::OriginalUri,
    http::{StatusCode, Uri},
    response::Response,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct FrontendAssets;

pub async fn index() -> Response {
    asset_response("index.html").unwrap_or_else(not_found)
}

pub async fn fallback(OriginalUri(uri): OriginalUri) -> Response {
    if should_return_backend_404(&uri) {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "application/json")
            .body(Body::from(r#"{"error":"not found"}"#))
            .unwrap();
    }

    let path = uri.path().trim_start_matches('/');
    if let Some(response) = asset_response(path) {
        return response;
    }

    if path.contains('.') {
        return not_found();
    }

    asset_response("index.html").unwrap_or_else(not_found)
}

fn should_return_backend_404(uri: &Uri) -> bool {
    let path = uri.path();
    path.starts_with("/api/")
        || path == "/api"
        || path.starts_with("/v1/")
        || path.starts_with("/openai/")
        || path.starts_with("/anthropic/")
}

fn asset_response(path: &str) -> Option<Response> {
    let path = if path.is_empty() { "index.html" } else { path };
    let asset = FrontendAssets::get(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", mime.as_ref())
        .body(Body::from(asset.data.into_owned()))
        .ok()
}

fn not_found() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("content-type", "text/plain; charset=utf-8")
        .body(Body::from("not found"))
        .unwrap()
}
