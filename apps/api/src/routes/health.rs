use axum::{Json, extract::{Extension, State}, http::StatusCode};
use serde::Serialize;
use urp_kernel::KernelError;

use crate::{
    errors::ApiError,
    middleware::RequestTrace,
    state::AppState,
};

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
    Extension(trace): Extension<RequestTrace>,
) -> Result<Json<HealthResponse>, ApiError> {
    match state.database.health_check().await {
        Ok(()) => Ok(Json(HealthResponse {
            status: "ready",
            version: state.version,
        })),
        Err(error) => {
            tracing::error!(%error, request_id = %trace.request_id, "database readiness check failed");
            Err(ApiError::from_kernel(KernelError::Unavailable)
                .with_request_id(trace.request_id))
        }
    }
}
