//! Safe JSON request extraction and DTO validation boundary.

use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;

use crate::{
    errors::{ApiError, FieldError},
    middleware::RequestTrace,
};

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
        let trace = request.extensions().get::<RequestTrace>().copied();
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(|rejection| {
                let error = ApiError::from_json_rejection(rejection);
                match trace {
                    Some(value) => error.with_request_id(value.request_id),
                    None => error,
                }
            })?;

        value.validate_request().map_err(|failures| {
            let error = ApiError::validation_failures(failures);
            match trace {
                Some(value) => error.with_request_id(value.request_id),
                None => error,
            }
        })?;

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
    use std::error::Error;

    fn json_request(body: &'static str) -> Result<Request<Body>, axum::http::Error> {
        Request::builder()
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
    }
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
    async fn validated_json_rejects_invalid_dto_with_field_paths() -> Result<(), Box<dyn Error>> {
        let request = json_request(r#"{"email":"invalid","name":" "}"#)?;

        let result = ValidatedJson::<CreateContact>::from_request(request, &()).await;
        assert!(result.is_err());

        let response = result
            .err()
            .unwrap_or_else(ApiError::internal)
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(response.into_body(), 4096).await;
        let parsed = bytes
            .ok()
            .and_then(|body| serde_json::from_slice::<Value>(&body).ok());
        assert!(parsed.is_some_and(|value| {
            value["error"]["field_errors"][0]["path"] == "email"
                && value["error"]["field_errors"][1]["path"] == "name"
        }));
        Ok(())
    }

    #[tokio::test]
    async fn validated_json_rejects_malformed_body_safely() -> Result<(), Box<dyn Error>> {
        let request = json_request("{")?;

        let result = ValidatedJson::<CreateContact>::from_request(request, &()).await;
        assert!(result.is_err());
        let response = result
            .err()
            .unwrap_or_else(ApiError::internal)
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }

    #[tokio::test]
    async fn validated_json_accepts_valid_dto() -> Result<(), Box<dyn Error>> {
        let request = json_request(r#"{"email":"person@example.test","name":"Ada"}"#)?;

        let result = ValidatedJson::<CreateContact>::from_request(request, &()).await;
        assert!(result.is_ok());
        Ok(())
    }
}
