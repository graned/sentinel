use dotenvy::dotenv;
use std::env;

#[allow(dead_code)]
pub fn get_server_url() -> String {
    dotenv().ok(); // load .env
    let addr = env::var("APP_HOST").unwrap_or_else(|_| "sentinel-core".to_string());
    let port = env::var("APP_PORT").unwrap_or_else(|_| "8000".to_string());
    format!("http://{}:{}", addr, port)
}

#[allow(dead_code)]
pub fn get_login_user_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/login")
}

#[allow(dead_code)]
pub fn get_register_user_url() -> String {
    let server_url = get_server_url();
    let api_endpoint = "/v1/api/auth/register".to_string();
    format!("{server_url}{api_endpoint}")
}
#[allow(dead_code)]
pub fn get_authenticate_token_url() -> String {
    let server_url = get_server_url();
    let api_endpoint = "/v1/api/auth/authenticate".to_string();
    format!("{server_url}{api_endpoint}")
}
#[allow(dead_code)]
pub fn get_config_email_url() -> String {
    let server_url = get_server_url();
    let api_endpoint = "/v1/api/system/config/email".to_string();
    format!("{server_url}{api_endpoint}")
}

#[allow(dead_code)]
pub fn get_config_email_item_url(config_id: uuid::Uuid) -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/system/config/email/{config_id}")
}

#[allow(dead_code)]
pub fn get_config_email_reveal_url(config_id: uuid::Uuid) -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/system/config/email/{config_id}/reveal")
}

#[allow(dead_code)]
pub fn get_create_policy_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/policies")
}

#[allow(dead_code)]
pub fn get_update_policy_rules_url(policy_id: uuid::Uuid) -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/policies/{policy_id}/rules")
}

#[allow(dead_code)]
pub fn get_admin_policies_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/policies")
}

#[allow(dead_code)]
pub fn get_admin_policy_url(policy_id: uuid::Uuid) -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/policies/{policy_id}")
}

#[allow(dead_code)]
pub fn get_user_canary_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/user/canary")
}

#[allow(dead_code)]
pub fn get_user_me_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/user/me")
}

#[allow(dead_code)]
pub fn get_logout_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/logout")
}

#[allow(dead_code)]
pub fn get_logout_all_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/logout-all")
}

#[allow(dead_code)]
pub fn get_token_refresh_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/token/refresh")
}

// ── MFA URLs ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn get_mfa_totp_start_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/mfa/totp/start")
}

#[allow(dead_code)]
pub fn get_mfa_totp_confirm_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/mfa/totp/confirm")
}

#[allow(dead_code)]
pub fn get_mfa_verify_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/mfa/verify")
}

// ── API Token URLs ─────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn get_api_tokens_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/api-tokens")
}

#[allow(dead_code)]
pub fn get_api_token_url(token_id: uuid::Uuid) -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/api-tokens/{token_id}")
}

// ── Password URLs ──────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn get_forgot_password_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/password/forgot")
}

#[allow(dead_code)]
pub fn get_reset_password_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/auth/password/reset")
}

#[allow(dead_code)]
pub fn get_change_password_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/user/password/change")
}

// ── Email Template URLs ─────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn get_email_templates_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/email-templates")
}

#[allow(dead_code)]
pub fn get_update_email_template_url(template_id: uuid::Uuid) -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/email-templates/{template_id}")
}

// ── User Session / Permission URLs ─────────────────────────────────────────

#[allow(dead_code)]
pub fn get_user_sessions_url() -> String {
    format!("{}/v1/api/user/sessions", get_server_url())
}

#[allow(dead_code)]
pub fn get_user_session_url(session_id: uuid::Uuid) -> String {
    format!("{}/v1/api/user/sessions/{session_id}", get_server_url())
}

#[allow(dead_code)]
pub fn get_user_permissions_url() -> String {
    format!("{}/v1/api/user/permissions", get_server_url())
}

// ── OIDC URLs ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn get_oidc_generate_key_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/oidc/keys/generate")
}

#[allow(dead_code)]
pub fn get_oidc_create_client_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/oidc/clients")
}

#[allow(dead_code)]
pub fn get_oauth_authorize_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/oauth/authorize")
}

#[allow(dead_code)]
pub fn get_oauth_token_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/oauth/token")
}

#[allow(dead_code)]
pub fn get_jwks_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/oauth/jwks.json")
}

#[allow(dead_code)]
pub fn get_oidc_discovery_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/.well-known/openid-configuration")
}

// ── Admin Session URLs ─────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn get_admin_sessions_url() -> String {
    format!("{}/v1/api/admin/sessions", get_server_url())
}

#[allow(dead_code)]
pub fn get_admin_session_url(session_id: uuid::Uuid) -> String {
    format!("{}/v1/api/admin/sessions/{session_id}", get_server_url())
}

#[allow(dead_code)]
pub fn get_admin_sessions_revoke_url() -> String {
    format!("{}/v1/api/admin/sessions/revoke", get_server_url())
}

#[allow(dead_code)]
pub fn get_admin_users_url() -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/users")
}

#[allow(dead_code)]
pub fn get_admin_user_url(user_id: uuid::Uuid) -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/users/{user_id}")
}

#[allow(dead_code)]
pub fn get_admin_user_status_url(user_id: uuid::Uuid) -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/users/{user_id}/status")
}

#[allow(dead_code)]
pub fn get_admin_user_send_invite_url(user_id: uuid::Uuid) -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/users/{user_id}/send-invite")
}

#[allow(dead_code)]
pub fn get_admin_user_invite_link_url(user_id: uuid::Uuid) -> String {
    let server_url = get_server_url();
    format!("{server_url}/v1/api/admin/users/{user_id}/invite-link")
}

// ── Insights / Analytics URLs ──────────────────────────────────────────────

#[allow(dead_code)]
pub fn get_insights_stats_url() -> String {
    format!("{}/v1/api/system/stats", get_server_url())
}

#[allow(dead_code)]
pub fn get_insights_user_growth_url() -> String {
    format!("{}/v1/api/system/analytics/user-growth", get_server_url())
}

#[allow(dead_code)]
pub fn get_insights_sessions_url() -> String {
    format!("{}/v1/api/system/analytics/sessions", get_server_url())
}
