use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

pub async fn live(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "live",
        version: state.version,
    })
}

pub async fn ready(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<HealthResponse>)> {
    match state.database.health_check().await {
        Ok(()) => Ok(Json(HealthResponse {
            status: "ready",
            version: state.version,
        })),
        Err(error) => {
            tracing::error!(%error, "database readiness check failed");
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "not_ready",
                    version: state.version,
                }),
            ))
        }
    }
}
