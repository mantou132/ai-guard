mod api;
mod app_state;
mod assets;
mod audit;
mod config;
mod models;
mod payload;
mod proxy;
mod store;

use std::{sync::Arc, time::Duration};

use anyhow::Context;
use axum::{Router, routing::get};
use clap::Parser;
use config::Config;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::{app_state::AppState, store::Store};

fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ai_guard=info,tower_http=info".into()),
        )
        .init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to create Tokio runtime")?
        .block_on(run())
}

async fn run() -> anyhow::Result<()> {
    let config = Config::parse();
    let store = Store::open(config.state_path())
        .await
        .context("failed to open local state store")?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .context("failed to create HTTP client")?;

    let bind = config.bind.clone();
    let state = AppState {
        config: Arc::new(config),
        store,
        client,
    };

    requeue_pending_audits(&state).await;

    let app = Router::new()
        .merge(api::router())
        .merge(proxy::router())
        .route("/", get(assets::index))
        .fallback(assets::fallback)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("failed to bind {bind}"))?;
    info!("ai-guard listening on http://{bind}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server failed")?;

    Ok(())
}

async fn requeue_pending_audits(state: &AppState) {
    let pending = state.store.pending_audit_logs().await;
    if pending.is_empty() {
        return;
    }
    info!(
        count = pending.len(),
        "re-queuing pending audits from store"
    );
    for log in pending {
        audit::spawn_audit(
            state.clone(),
            log.id,
            log.request_payload,
            log.response_payload,
            log.model,
        );
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::warn!(%err, "failed to install Ctrl+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => tracing::warn!(%err, "failed to install terminate handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
