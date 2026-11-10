mod config;

use std::error::Error;

use axum::{Json, Router, routing::get};
use config::ApiConfig;
use serde::Serialize;
use tracing_subscriber::EnvFilter;
use urp_kernel::{API_VERSION, PRODUCT_NAME};

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    api_version: &'static str,
}

async fn live() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: PRODUCT_NAME,
        api_version: API_VERSION,
    })
}

async fn ready() -> Json<HealthResponse> {
    // Database and worker readiness checks are added with the storage layer.
    Json(HealthResponse {
        status: "ready",
        service: PRODUCT_NAME,
        api_version: API_VERSION,
    })
}

fn app() -> Router {
    Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        if let Ok(mut terminate) = signal(SignalKind::terminate()) {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = terminate.recv() => {},
            }
            return;
        }
    }

    let _ = tokio::signal::ctrl_c().await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = ApiConfig::from_env()?;
    let filter = EnvFilter::try_new(&config.log_filter)?;
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let listener = tokio::net::TcpListener::bind(config.bind_address).await?;
    tracing::info!(bind_address = %config.bind_address, "URP API listening");

    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn live_health_payload_identifies_service() {
        let response = live().await.0;

        assert_eq!(response.status, "ok");
        assert_eq!(response.service, PRODUCT_NAME);
        assert_eq!(response.api_version, API_VERSION);
    }

    #[tokio::test]
    async fn ready_health_payload_is_explicit() {
        let response = ready().await.0;

        assert_eq!(response.status, "ready");
    }
}
