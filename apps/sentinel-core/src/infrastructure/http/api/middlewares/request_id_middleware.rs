//! Request-ID middleware — assigns or forwards a correlation ID for every request.
//!
//! If the incoming request already contains an `X-Request-Id` header (valid ASCII,
//! ≤ 128 chars), that value is adopted. Otherwise a new UUID v4 is generated.
//!
//! The ID is:
//! 1. Inserted into Axum request extensions as a [`RequestId`] so handlers and
//!    middleware can read it without re-parsing the header.
//! 2. Echoed back in the `X-Request-Id` response header for client-side correlation.

use crate::http::api::RequestId;
use axum::{
    body::Body,
    http::{HeaderMap, HeaderName, HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use std::sync::OnceLock;

static REQ_ID_HEADER: OnceLock<HeaderName> = OnceLock::new();

fn request_id_header() -> HeaderName {
    REQ_ID_HEADER
        .get_or_init(|| HeaderName::from_static("x-request-id"))
        .clone()
}

fn extract_request_id(headers: &HeaderMap) -> Option<RequestId> {
    let name = request_id_header();
    let value = headers.get(&name);
    // Only accept valid visible ASCII to avoid weird log injection.
    let s = value?.to_str().ok()?.trim();
    if s.is_empty() || s.len() > 128 {
        return None;
    }
    Some(RequestId(s.to_string()))
}

pub async fn request_id_middleware(mut req: Request<Body>, next: Next) -> Response {
    let rid = extract_request_id(req.headers()).unwrap_or_else(RequestId::new);

    // Put into request context so handlers/extractors can read it
    req.extensions_mut().insert(rid.clone());

    // Continue
    let mut res = next.run(req).await;

    // Echo back so clients & logs can correlate
    let name = request_id_header();
    if let Ok(val) = HeaderValue::from_str(rid.as_str()) {
        res.headers_mut().insert(name, val);
    }

    res
}
