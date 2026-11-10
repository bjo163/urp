mod health;

use axum::{
    Json, Router,
    extract::Extension,
    routing::get,
};
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::{
    errors::ApiError,
    middleware::{RequestTrace, attach_trace_context},
    state::AppState,
};

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

async fn not_found(Extension(trace): Extension<RequestTrace>) -> ApiError {
    ApiError::not_found().with_request_id(trace.request_id)
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

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json::Value;
    use tower::ServiceExt;
    use urp_storage::{Database, DatabaseConfig};
    use uuid::Uuid;

    use super::router;
    use crate::state::AppState;

    async fn test_router() -> Result<axum::Router, Box<dyn std::error::Error>> {
        let database = Database::connect(DatabaseConfig {
            url: "sqlite::memory:".to_owned(),
            max_connections: 1,
        })
        .await?;

        Ok(router(AppState {
            database,
            version: "0.1.0-test",
        }))
    }

    #[tokio::test]
    async fn health_live_is_available_without_database_query()
    -> Result<(), Box<dyn std::error::Error>> {
        let app = test_router().await?;
        let response = app
            .oneshot(Request::builder().uri("/health/live").body(Body::empty())?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("x-request-id"));
        Ok(())
    }

    #[tokio::test]
    async fn health_ready_checks_database_dependency() -> Result<(), Box<dyn std::error::Error>> {
        let app = test_router().await?;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn api_root_exposes_versioned_contract() -> Result<(), Box<dyn std::error::Error>> {
        let app = test_router().await?;
        let response = app
            .oneshot(Request::builder().uri("/api/v1/").body(Body::empty())?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 4096).await?;
        let json: Value = serde_json::from_slice(&bytes)?;
        assert_eq!(json["api_version"], "v1");
        assert_eq!(json["name"], "URP");
        Ok(())
    }

    #[tokio::test]
    async fn not_found_uses_canonical_error_envelope_and_request_id()
    -> Result<(), Box<dyn std::error::Error>> {
        let app = test_router().await?;
        let request_id = Uuid::new_v4();
        let expected_request_id = request_id.to_string();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/does-not-exist")
                    .header("x-request-id", expected_request_id.as_str())
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response
                .headers()
                .get("x-request-id")
                .and_then(|value| value.to_str().ok()),
            Some(expected_request_id.as_str())
        );
        let bytes = to_bytes(response.into_body(), 4096).await?;
        let json: Value = serde_json::from_slice(&bytes)?;
        assert_eq!(json["error"]["code"], "not_found");
        assert_eq!(json["request_id"], request_id.to_string());
        Ok(())
    }
}
