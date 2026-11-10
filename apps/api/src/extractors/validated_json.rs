//! Safe JSON request extraction and DTO validation boundary.

use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;

use crate::errors::{ApiError, FieldError};

/// A request DTO that validates its own business-independent input constraints.
pub trait ValidateRequest {
    /// Returns safe field-level failures, or an empty success.
    fn validate_request(&self) -> Result<(), Vec<FieldError>>;
}

/// JSON extractor that maps parse failures and DTO validation into the canonical API error.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + ValidateRequest,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(ApiError::from_json_rejection)?;

        value
            .validate_request()
            .map_err(ApiError::validation_failures)?;

        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        extract::FromRequest,
        http::{Request, StatusCode, header},
        response::IntoResponse,
    };
    use serde::Deserialize;
    use serde_json::Value;

    use super::{ValidateRequest, ValidatedJson};
    use crate::errors::{ApiError, FieldError};

    #[derive(Deserialize)]
    struct CreateContact {
        email: String,
        name: String,
    }

    impl ValidateRequest for CreateContact {
        fn validate_request(&self) -> Result<(), Vec<FieldError>> {
            let mut failures = Vec::new();

            if !self.email.contains('@') {
                failures.push(FieldError {
                    path: "email".to_owned(),
                    code: "invalid_format".to_owned(),
                    message: "email must contain @".to_owned(),
                });
            }
            if self.name.trim().is_empty() {
                failures.push(FieldError {
                    path: "name".to_owned(),
                    code: "required".to_owned(),
                    message: "name is required".to_owned(),
                });
            }

            if failures.is_empty() {
                Ok(())
            } else {
                Err(failures)
            }
        }
    }

    #[tokio::test]
    async fn validated_json_rejects_invalid_dto_with_field_paths() {
        let request = Request::builder()
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"email":"invalid","name":" "}"#))
            .expect("request builder should succeed");

        let result = ValidatedJson::<CreateContact>::from_request(request, &()).await;
        assert!(result.is_err());

        let response = result.err().unwrap_or_else(ApiError::internal).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(response.into_body(), 4096).await;
        let parsed = bytes
            .ok()
            .and_then(|body| serde_json::from_slice::<Value>(&body).ok());
        assert!(parsed.is_some_and(|value| {
            value["error"]["field_errors"][0]["path"] == "email"
                && value["error"]["field_errors"][1]["path"] == "name"
        }));
    }

    #[tokio::test]
    async fn validated_json_rejects_malformed_body_safely() {
        let request = Request::builder()
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{"))
            .expect("request builder should succeed");

        let result = ValidatedJson::<CreateContact>::from_request(request, &()).await;
        assert!(result.is_err());
        let response = result.err().unwrap_or_else(ApiError::internal).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn validated_json_accepts_valid_dto() {
        let request = Request::builder()
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"email":"person@example.test","name":"Ada"}"#))
            .expect("request builder should succeed");

        let result = ValidatedJson::<CreateContact>::from_request(request, &()).await;
        assert!(result.is_ok());
    }
}
