mod health;

use axum::{Json, Router, routing::get};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::{errors::ApiError, middleware::attach_trace_context, state::AppState};

pub fn router(state: AppState) -> Router {
    let api = Router::new().route("/", get(api_root));

    Router::new()
        .route("/health/live", get(health::live))
        .route("/health/ready", get(health::ready))
        .nest("/api/v1", api)
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(attach_trace_context))
        .with_state(state)
}

async fn not_found() -> ApiError {
    ApiError::not_found()
}

#[derive(Serialize)]
struct ApiInfo {
    name: &'static str,
    version: &'static str,
    api_version: &'static str,
}

async fn api_root() -> Json<ApiInfo> {
    Json(ApiInfo {
        name: "URP",
        version: env!("CARGO_PKG_VERSION"),
        api_version: "v1",
    })
}
