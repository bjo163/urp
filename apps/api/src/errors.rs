use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;
use urp_kernel::KernelError;

/// Stable JSON error response returned by the HTTP application.
#[derive(Debug, Serialize)]
pub struct ApiErrorEnvelope {
    /// Error details.
    pub error: ApiErrorDetail,
    /// Request identifier when the caller attaches one to the error.
    pub request_id: Option<Uuid>,
}

/// Machine-readable and safe public error details.
#[derive(Debug, Serialize)]
pub struct ApiErrorDetail {
    /// Stable error code.
    pub code: &'static str,
    /// Safe public message.
    pub message: String,
    /// Field associated with a validation error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<&'static str>,
}

/// An HTTP error created from a safe application error.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    detail: ApiErrorDetail,
    request_id: Option<Uuid>,
}

impl ApiError {
    /// Maps a kernel error to a stable HTTP status and public response.
    pub fn from_kernel(error: KernelError) -> Self {
        let (status, detail) = match error {
            KernelError::Validation { field, message } => (
                StatusCode::BAD_REQUEST,
                ApiErrorDetail {
                    code: "validation_error",
                    message,
                    field: Some(field),
                },
            ),
            KernelError::NotFound => (
                StatusCode::NOT_FOUND,
                ApiErrorDetail {
                    code: "not_found",
                    message: "The requested resource was not found.".to_owned(),
                    field: None,
                },
            ),
            KernelError::Conflict => (
                StatusCode::CONFLICT,
                ApiErrorDetail {
                    code: "conflict",
                    message: "The operation conflicts with the current resource state.".to_owned(),
                    field: None,
                },
            ),
            KernelError::Forbidden => (
                StatusCode::FORBIDDEN,
                ApiErrorDetail {
                    code: "forbidden",
                    message: "You are not permitted to perform this operation.".to_owned(),
                    field: None,
                },
            ),
            KernelError::Unauthenticated => (
                StatusCode::UNAUTHORIZED,
                ApiErrorDetail {
                    code: "unauthenticated",
                    message: "Authentication is required.".to_owned(),
                    field: None,
                },
            ),
            KernelError::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                ApiErrorDetail {
                    code: "unavailable",
                    message: "A required service is temporarily unavailable.".to_owned(),
                    field: None,
                },
            ),
        };

        Self {
            status,
            detail,
            request_id: None,
        }
    }

    /// Attaches the HTTP request identifier to the public response.
    pub fn with_request_id(mut self, request_id: Uuid) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Creates the standard not-found response.
    pub fn not_found() -> Self {
        Self::from_kernel(KernelError::NotFound)
    }

    #[cfg(test)]
    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            detail: ApiErrorDetail {
                code: "internal_error",
                message: "An unexpected error occurred.".to_owned(),
                field: None,
            },
            request_id: None,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorEnvelope {
                error: self.detail,
                request_id: self.request_id,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, response::IntoResponse};
    use serde_json::Value;
    use uuid::Uuid;
    use urp_kernel::KernelError;

    use super::ApiError;

    #[tokio::test]
    async fn validation_errors_map_to_stable_field_error_envelope() {
        let request_id = Uuid::new_v4();
        let response = ApiError::from_kernel(KernelError::Validation {
            field: "email",
            message: "email format is invalid".to_owned(),
        })
        .with_request_id(request_id)
        .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), 4096).await;
        assert!(body.is_ok());
        let parsed = body
            .ok()
            .and_then(|value| serde_json::from_slice::<Value>(&value).ok());
        assert!(parsed.is_some_and(|value| {
            value["error"]["code"] == "validation_error"
                && value["error"]["field"] == "email"
                && value["request_id"] == request_id.to_string()
        }));
    }

    #[tokio::test]
    async fn internal_error_does_not_expose_internal_details() {
        let response = ApiError::internal().into_response();
        assert_eq!(
            response.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        let body = to_bytes(response.into_body(), 4096).await;
        let parsed = body
            .ok()
            .and_then(|value| serde_json::from_slice::<Value>(&value).ok());
        assert!(parsed.is_some_and(|value| {
            value["error"]["code"] == "internal_error"
                && value["error"]["message"] == "An unexpected error occurred."
        }));
    }
}
