//! URP HTTP API process and composition root.

mod config;
mod errors;
pub mod extractors;
mod middleware;
mod routes;
mod state;

use std::{error::Error, net::SocketAddr};

use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;
use urp_storage::{Database, DatabaseConfig};

use crate::{config::ApiConfig, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = ApiConfig::from_env()?;
    init_tracing(&config.log_filter);

    let database = Database::connect(DatabaseConfig {
        url: config.database_url.clone(),
        max_connections: config.database_max_connections,
    })
    .await?;

    let state = AppState {
        database,
        version: env!("CARGO_PKG_VERSION"),
    };
    let app = routes::router(state);

    let address = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(address).await?;
    tracing::info!(%address, "URP API listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing(filter: &str) {
    let env_filter =
        EnvFilter::try_new(filter).unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,hyper=warn"));
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(env_filter)
        .with_current_span(true)
        .with_span_list(false)
        .init();
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let terminate = signal(SignalKind::terminate());
        match terminate {
            Ok(mut terminate) => {
                tokio::select! {
                    result = tokio::signal::ctrl_c() => {
                        if let Err(error) = result {
                            tracing::warn!(%error, "failed to listen for Ctrl+C");
                        }
                    }
                    _ = terminate.recv() => {}
                }
            }
            Err(error) => {
                tracing::warn!(%error, "failed to install SIGTERM handler");
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }

    #[cfg(not(unix))]
    {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::warn!(%error, "failed to listen for Ctrl+C");
        }
    }

    tracing::info!("shutdown signal received");
}
