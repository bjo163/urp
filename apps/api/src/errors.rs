use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use urp_kernel::KernelError;
use uuid::Uuid;

/// Safe, machine-readable field validation detail.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FieldError {
    /// JSON/request path for the invalid field.
    pub path: String,
    /// Stable validation code.
    pub code: String,
    /// Safe, user-facing validation detail.
    pub message: String,
}

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
    /// Singular field path for backwards-compatible simple validation responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// Full field-level validation errors for request DTOs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_errors: Option<Vec<FieldError>>,
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
        if matches!(&error, KernelError::Internal) {
            return Self::internal();
        }

        let (status, detail) = match error {
            KernelError::Validation { field, message } => (
                StatusCode::BAD_REQUEST,
                ApiErrorDetail {
                    code: "validation_error",
                    message: message.clone(),
                    field: Some(field.to_owned()),
                    field_errors: Some(vec![FieldError {
                        path: field.to_owned(),
                        code: "invalid_value".to_owned(),
                        message,
                    }]),
                },
            ),
            KernelError::NotFound => (
                StatusCode::NOT_FOUND,
                ApiErrorDetail {
                    code: "not_found",
                    message: "The requested resource was not found.".to_owned(),
                    field: None,
                    field_errors: None,
                },
            ),
            KernelError::Conflict => (
                StatusCode::CONFLICT,
                ApiErrorDetail {
                    code: "conflict",
                    message: "The operation conflicts with the current resource state.".to_owned(),
                    field: None,
                    field_errors: None,
                },
            ),
            KernelError::Forbidden => (
                StatusCode::FORBIDDEN,
                ApiErrorDetail {
                    code: "forbidden",
                    message: "You are not permitted to perform this operation.".to_owned(),
                    field: None,
                    field_errors: None,
                },
            ),
            KernelError::Unauthenticated => (
                StatusCode::UNAUTHORIZED,
                ApiErrorDetail {
                    code: "unauthenticated",
                    message: "Authentication is required.".to_owned(),
                    field: None,
                    field_errors: None,
                },
            ),
            KernelError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                ApiErrorDetail {
                    code: "rate_limited",
                    message: "Too many requests. Please retry later.".to_owned(),
                    field: None,
                    field_errors: None,
                },
            ),
            KernelError::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                ApiErrorDetail {
                    code: "unavailable",
                    message: "A required service is temporarily unavailable.".to_owned(),
                    field: None,
                    field_errors: None,
                },
            ),
            KernelError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorDetail {
                    code: "internal_error",
                    message: "An unexpected error occurred.".to_owned(),
                    field: None,
                    field_errors: None,
                },
            ),
        };

        Self {
            status,
            detail,
            request_id: None,
        }
    }

    /// Creates a safe validation response with one or more field paths.
    pub fn validation_failures(mut failures: Vec<FieldError>) -> Self {
        if failures.is_empty() {
            return Self::internal();
        }

        let field = if failures.len() == 1 {
            failures.first().map(|failure| failure.path.clone())
        } else {
            None
        };
        let message = if failures.len() == 1 {
            failures[0].message.clone()
        } else {
            "One or more request fields are invalid.".to_owned()
        };

        Self {
            status: StatusCode::BAD_REQUEST,
            detail: ApiErrorDetail {
                code: "validation_error",
                message,
                field,
                field_errors: Some(std::mem::take(&mut failures)),
            },
            request_id: None,
        }
    }

    /// Creates a safe malformed-JSON response without exposing parser internals.
    pub fn malformed_json() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            detail: ApiErrorDetail {
                code: "invalid_json",
                message: "The request body must contain valid JSON matching the expected shape."
                    .to_owned(),
                field: None,
                field_errors: None,
            },
            request_id: None,
        }
    }

    /// Maps Axum JSON extraction failures to a safe public response.
    pub fn from_json_rejection(_rejection: JsonRejection) -> Self {
        Self::malformed_json()
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

    /// Creates a generic internal response. The underlying error belongs in server logs only.
    pub fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            detail: ApiErrorDetail {
                code: "internal_error",
                message: "An unexpected error occurred.".to_owned(),
                field: None,
                field_errors: None,
            },
            request_id: None,
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::from_json_rejection(rejection)
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
    use urp_kernel::KernelError;
    use uuid::Uuid;

    use super::{ApiError, FieldError};

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
                && value["error"]["field_errors"][0]["path"] == "email"
        }));
    }

    #[tokio::test]
    async fn multiple_validation_failures_preserve_field_paths() {
        let response = ApiError::validation_failures(vec![
            FieldError {
                path: "email".to_owned(),
                code: "invalid_format".to_owned(),
                message: "email is invalid".to_owned(),
            },
            FieldError {
                path: "name".to_owned(),
                code: "required".to_owned(),
                message: "name is required".to_owned(),
            },
        ])
        .into_response();

        let body = to_bytes(response.into_body(), 4096).await;
        let parsed = body
            .ok()
            .and_then(|value| serde_json::from_slice::<Value>(&value).ok());
        assert!(parsed.is_some_and(|value| {
            value["error"]["field_errors"][0]["path"] == "email"
                && value["error"]["field_errors"][1]["path"] == "name"
        }));
    }

    #[tokio::test]
    async fn malformed_json_maps_to_generic_safe_error() {
        let response = ApiError::malformed_json().into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), 4096).await;
        let parsed = body
            .ok()
            .and_then(|value| serde_json::from_slice::<Value>(&value).ok());
        assert!(parsed.is_some_and(|value| {
            value["error"]["code"] == "invalid_json"
                && !value["error"]["message"].as_str().unwrap_or_default().contains("serde")
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

    #[tokio::test]
    async fn every_kernel_error_has_a_stable_status_and_code() {
        let cases = [
            (KernelError::NotFound, axum::http::StatusCode::NOT_FOUND, "not_found"),
            (KernelError::Conflict, axum::http::StatusCode::CONFLICT, "conflict"),
            (KernelError::Forbidden, axum::http::StatusCode::FORBIDDEN, "forbidden"),
            (KernelError::Unauthenticated, axum::http::StatusCode::UNAUTHORIZED, "unauthenticated"),
            (KernelError::RateLimited, axum::http::StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            (KernelError::Unavailable, axum::http::StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
        ];

        for (error, expected_status, expected_code) in cases {
            let response = ApiError::from_kernel(error).into_response();
            assert_eq!(response.status(), expected_status);
            let body = to_bytes(response.into_body(), 4096).await;
            let parsed = body.ok().and_then(|value| serde_json::from_slice::<Value>(&value).ok());
            assert!(parsed.is_some_and(|value| value["error"]["code"] == expected_code));
        }
    }
}
