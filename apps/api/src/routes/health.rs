use axum::{
    Json,
    extract::{Extension, State},
};
use serde::Serialize;
use urp_kernel::KernelError;
use utoipa::ToSchema;

use crate::{errors::ApiError, middleware::RequestTrace, state::AppState};

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[utoipa::path(
    get,
    path = "/health/live",
    tag = "health",
    responses(
        (status = 200, description = "API process is alive", body = HealthResponse)
    )
)]
pub async fn live(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "live",
        version: state.version,
    })
}

#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "health",
    responses(
        (status = 200, description = "Required dependencies are ready", body = HealthResponse),
        (status = 503, description = "A required dependency is unavailable", body = crate::errors::ApiErrorEnvelope)
    )
)]
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
            tracing::error!(
                %error,
                request_id = %trace.request_id,
                correlation_id = %trace.correlation_id,
                "database readiness check failed"
            );
            Err(ApiError::from_kernel(KernelError::Unavailable).with_request_id(trace.request_id))
        }
    }
}
