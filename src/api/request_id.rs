use axum::{
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use tracing::Instrument;
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";
const MAX_REQUEST_ID_LEN: usize = 128;

pub async fn request_id_middleware(mut req: Request<axum::body::Body>, next: Next) -> Response {
    let request_id = req
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(sanitize_request_id)
        .unwrap_or_else(generate_request_id);
    req.extensions_mut().insert(request_id.clone());
    let span = tracing::info_span!("http_request", request_id = %request_id);
    let mut response = next.run(req).instrument(span).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }
    response
}

pub(super) fn request_id<B>(request: &Request<B>) -> String {
    request
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(generate_request_id)
}
fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}
fn sanitize_request_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_REQUEST_ID_LEN {
        return None;
    }
    trimmed
        .bytes()
        .all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
        })
        .then(|| trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::sanitize_request_id;
    #[test]
    fn request_ids_are_sanitized() {
        assert_eq!(
            sanitize_request_id("abc-123_DEF:/v1.req").as_deref(),
            Some("abc-123_DEF:/v1.req")
        );
        assert!(sanitize_request_id("abc\n123").is_none());
        assert!(sanitize_request_id(&"a".repeat(129)).is_none());
    }
}
