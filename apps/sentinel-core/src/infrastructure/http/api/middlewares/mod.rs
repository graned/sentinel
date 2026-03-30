//! Tower/Axum middleware stack for the `/v1/api/*` router.
//!
//! | Middleware | Purpose |
//! |-----------|---------|
//! | `request_id_middleware` | Assigns / forwards `X-Request-Id` |
//! | `rate_limit_middleware` | IP-keyed GCRA rate limiter (strict / moderate tiers) |
//! | `response_wrapper` | Wraps responses in the standard Sentinel envelope |
//! | `authenticate_middleware` | Validates Bearer token → inserts `AuthenticatedUserContext` |
//! | `authorize_middleware` | RBAC policy evaluation against the policy engine |

pub mod authenticate_middleware;
pub mod authorize_middleware;
pub mod rate_limit_middleware;
pub mod request_id_middleware;
pub mod response_wrapper;
