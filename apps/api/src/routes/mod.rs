mod health;

use axum::{routing::get, Json, Router};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let api = Router::new().route("/", get(api_root));

    Router::new()
        .route("/health/live", get(health::live))
        .route("/health/ready", get(health::ready))
        .nest("/api/v1", api)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
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
