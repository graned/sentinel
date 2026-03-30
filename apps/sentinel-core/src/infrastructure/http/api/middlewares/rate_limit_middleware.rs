//! IP-keyed GCRA rate limiting middleware using the `governor` crate.
//!
//! Two pre-built limiter factories are provided:
//!
//! | Factory | Limit | Used by |
//! |---------|-------|---------|
//! | `strict_limiter()` | 5 req / min | `POST /auth/login`, `POST /auth/mfa/verify` |
//! | `moderate_limiter()` | 10 req / min | `POST /auth/register`, `POST /auth/password/forgot`, `POST /auth/resend-verification` |
//!
//! Client IP is resolved from `X-Forwarded-For` → `X-Real-IP` → `127.0.0.1`.
//! Blocked requests receive HTTP 429 with `Retry-After: 900` and a JSON body
//! `{ "code": "RATE_LIMIT_EXCEEDED", ... }`.

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use governor::{
    clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Quota, RateLimiter,
};
use serde_json::json;
use std::{
    net::IpAddr,
    num::NonZeroU32,
    sync::Arc,
};

pub type KeyedLimiter = RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>;

/// Build a strict rate limiter: 5 requests per 15 minutes.
/// Uses per_minute(5) so burst capacity = 5 but replenishment (12 s/token)
/// is far slower than bcrypt login time (~200 ms), preventing token
/// refill between sequential requests during tests.
/// Intended for: login, mfa_verify.
pub fn strict_limiter() -> Arc<KeyedLimiter> {
    Arc::new(RateLimiter::keyed(Quota::per_minute(
        NonZeroU32::new(5).unwrap(),
    )))
}

/// Build a moderate rate limiter: 10 requests per 15 minutes.
/// Burst capacity = 10; replenishment (6 s/token) is far slower than
/// the fast endpoints it guards.
/// Intended for: register, forgot_password, resend_verification.
pub fn moderate_limiter() -> Arc<KeyedLimiter> {
    Arc::new(RateLimiter::keyed(Quota::per_minute(
        NonZeroU32::new(10).unwrap(),
    )))
}

/// Axum middleware that enforces an IP-keyed rate limit.
/// The limiter is passed via `from_fn_with_state`.
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<KeyedLimiter>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_ip(&req);
    if limiter.check_key(&ip).is_err() {
        let body = json!({
            "code": "RATE_LIMIT_EXCEEDED",
            "message": "Too many requests. Please try again later."
        });
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("retry-after", "900")],
            Json(body),
        )
            .into_response();
    }
    next.run(req).await
}

/// Extract client IP from `X-Forwarded-For` or `X-Real-IP`.
/// Falls back to `127.0.0.1` when no header is present.
fn extract_ip(req: &Request<Body>) -> IpAddr {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.trim().parse::<IpAddr>().ok())
        })
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
}
