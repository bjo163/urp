use axum::{
    body::Body,
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";
const CORRELATION_ID_HEADER: &str = "x-correlation-id";

/// Request/correlation identifiers validated at the HTTP boundary.
#[derive(Clone, Copy, Debug)]
pub struct RequestTrace {
    /// Unique ID for the current HTTP request.
    pub request_id: Uuid,
    /// ID propagated across related requests and background work.
    pub correlation_id: Uuid,
}

/// Adds validated request/correlation IDs to request extensions and response headers.
///
/// Incoming IDs are tracing metadata only; they never establish identity, tenant,
/// company or authorization context.
pub async fn attach_trace_context(mut request: Request<Body>, next: Next) -> Response {
    let request_id = parse_uuid_header(request.headers().get(REQUEST_ID_HEADER))
        .unwrap_or_else(Uuid::new_v4);
    let correlation_id =
        parse_uuid_header(request.headers().get(CORRELATION_ID_HEADER)).unwrap_or(request_id);

    request.extensions_mut().insert(RequestTrace {
        request_id,
        correlation_id,
    });

    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&request_id.to_string()) {
        response.headers_mut().insert(
            HeaderName::from_static(REQUEST_ID_HEADER),
            value,
        );
    }
    if let Ok(value) = HeaderValue::from_str(&correlation_id.to_string()) {
        response.headers_mut().insert(
            HeaderName::from_static(CORRELATION_ID_HEADER),
            value,
        );
    }

    response
}

fn parse_uuid_header(value: Option<&HeaderValue>) -> Option<Uuid> {
    value
        .and_then(|header| header.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;
    use uuid::Uuid;

    use super::parse_uuid_header;

    #[test]
    fn valid_uuid_header_is_preserved() {
        let expected = Uuid::new_v4();
        let header = HeaderValue::from_str(&expected.to_string());
        assert!(header.is_ok_and(|value| parse_uuid_header(Some(&value)) == Some(expected)));
    }

    #[test]
    fn malformed_or_missing_header_is_rejected() {
        let malformed = HeaderValue::from_static("not-a-uuid");
        assert_eq!(parse_uuid_header(Some(&malformed)), None);
        assert_eq!(parse_uuid_header(None), None);
    }
}
