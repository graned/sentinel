//! Dependency-injection container — wires every repository, service, and application
//! together and returns the fully configured Axum [`Router`].
//!
//! `build_app` is the single place where concrete types are instantiated.
//! The Clean Architecture layering rule (infrastructure → application → service →
//! domain) is enforced here: lower-layer `Arc<T>` handles are passed into
//! higher-layer constructors, never the other way around.

use crate::{
    http::api::routes::build_router, http::server::AppState, AdminApplication,
    AdminSessionApplication, ApiTokenApplication, ApiTokenRepository, ApiTokenService,
    AuthApplication, EmailService, EmailTemplateApplication, EmailTemplateRepository,
    EmailTemplateService, EmailVerificationRepository, EmailVerificationService,
    ExternalIdentityRepository, FederationApplication, FederationService, IdentitiesRepository,
    IdentityService, InsightsApplication, MfaApplication, MfaTotpService, OidcApplication,
    OidcAuthCodeRepository, OidcAuthCodeService, OidcClientRepository, OidcClientService,
    OidcKeyService, OidcSigningKeyRepository, OidcTokenService, PasswordResetService,
    PasswordResetTokenRepository, PolicyApplication, PolicyRepository, PolicyService,
    PolicyVersionRepository, PostgresClient, ProviderConfigurationReposiory,
    ProviderConfigurationService, RoleRepository, SessionRepository, SessionService,
    SupabaseFederationConfig, SupabaseJwtVerifier, SystemApplication, UserApplication,
    UserMfaTotpRepository, UserPasswordApplication, UserRecoveryCodeRepository, UserRepository,
    UserRoleRepository, UserRoleService, UserService,
};
use axum::Router;
use std::sync::Arc;

/// Dependency-injection container — constructs every repository, service, and application
/// object and wires them together, then builds and returns the Axum [`Router`].
///
/// This is the single place where all concrete types are instantiated.
/// The layering rule (infrastructure → application → service → domain) is enforced here:
/// lower-layer `Arc<T>` handles are passed *into* higher-layer constructors.
///
/// # Parameters
/// - `database_url`          — PostgreSQL connection string.
/// - `sesson_encryption_key` — 32-byte key for PASETO session token encryption.
/// - `config_encryption_key` — 32-byte key for SMTP config and OIDC key encryption.
/// - `oidc_issuer_url`        — Advertised OIDC issuer URL used in JWT claims.
pub async fn build_app(
    database_url: &str,
    sesson_encryption_key: [u8; 32],
    config_encryption_key: [u8; 32],
    oidc_issuer_url: String,
) -> anyhow::Result<Router> {
    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_default();
    let pg_client = Arc::new(PostgresClient::new(database_url).await?);

    let user_repo = Arc::new(UserRepository::new());
    let identities_repo = Arc::new(IdentitiesRepository::new());
    let session_repo = Arc::new(SessionRepository::new());
    let config_repo = Arc::new(ProviderConfigurationReposiory::new());
    let user_role_repo = Arc::new(UserRoleRepository::new());
    let role_repo = Arc::new(RoleRepository::new());

    let mfa_totp_repo = Arc::new(UserMfaTotpRepository::new());
    let recovery_code_repo = Arc::new(UserRecoveryCodeRepository::new());

    // Clone repos needed by InsightsApplication before they're consumed
    let user_repo_insights = user_repo.clone();
    let identities_repo_insights = identities_repo.clone();
    let session_repo_insights = session_repo.clone();
    let mfa_totp_repo_insights = mfa_totp_repo.clone();

    let user_service = Arc::new(UserService::new(user_repo));
    let user_role_service = Arc::new(UserRoleService::new(role_repo, user_role_repo));
    let identity_service = Arc::new(IdentityService::new(identities_repo));
    let session_service = Arc::new(SessionService::new(session_repo, sesson_encryption_key));
    let config_service = Arc::new(ProviderConfigurationService::new(
        config_repo.clone(),
        config_encryption_key,
    ));
    let mfa_totp_service = Arc::new(MfaTotpService::new(
        mfa_totp_repo,
        recovery_code_repo,
        config_encryption_key,
    ));

    let policy_repo = Arc::new(PolicyRepository::new());
    let policy_version_repo = Arc::new(PolicyVersionRepository::new());
    let policy_service = Arc::new(PolicyService::new(policy_repo, policy_version_repo));

    // OIDC repositories
    let oidc_client_repo = Arc::new(OidcClientRepository::new());
    let oidc_auth_code_repo = Arc::new(OidcAuthCodeRepository::new());
    let oidc_signing_key_repo = Arc::new(OidcSigningKeyRepository::new());

    // OIDC services
    let oidc_client_service = Arc::new(OidcClientService::new(oidc_client_repo));
    let oidc_auth_code_service = Arc::new(OidcAuthCodeService::new(oidc_auth_code_repo));
    let oidc_key_service = Arc::new(OidcKeyService::new(
        oidc_signing_key_repo,
        config_encryption_key,
    ));
    let oidc_token_service = Arc::new(OidcTokenService::new(oidc_issuer_url));

    let user_application = Arc::new(UserApplication::new(
        pg_client.clone(),
        user_service.clone(),
        identity_service.clone(),
        session_service.clone(),
        user_role_service.clone(),
        mfa_totp_service.clone(),
    ));

    let api_token_repo = Arc::new(ApiTokenRepository::new());
    let api_token_service = Arc::new(ApiTokenService::new(api_token_repo));

    let email_verification_repo = Arc::new(EmailVerificationRepository::new());
    let email_verification_service =
        Arc::new(EmailVerificationService::new(email_verification_repo));

    let email_template_repo = Arc::new(EmailTemplateRepository::new());
    let email_template_service = Arc::new(EmailTemplateService::new(email_template_repo));

    let password_reset_repo = Arc::new(PasswordResetTokenRepository::new());
    let password_reset_service = Arc::new(PasswordResetService::new(password_reset_repo));

    let email_service = Arc::new(EmailService::new(
        config_repo.clone(),
        config_service.clone(),
        email_template_service.clone(),
        frontend_url,
    ));

    let auth_application = Arc::new(AuthApplication::new(
        pg_client.clone(),
        identity_service.clone(),
        user_service.clone(),
        user_role_service.clone(),
        session_service.clone(),
        mfa_totp_service.clone(),
        api_token_service.clone(),
        email_verification_service.clone(),
        email_service.clone(),
        password_reset_service,
    ));
    let system_application = Arc::new(SystemApplication::new(
        pg_client.clone(),
        config_service,
        oidc_client_service.clone(),
        email_service.clone(),
    ));
    let policy_application = Arc::new(PolicyApplication::new(
        pg_client.clone(),
        policy_service,
        session_service.clone(),
    ));

    let oidc_application = Arc::new(OidcApplication::new(
        pg_client.clone(),
        oidc_client_service,
        oidc_auth_code_service,
        oidc_key_service,
        oidc_token_service,
        user_service.clone(),
        identity_service.clone(),
        mfa_totp_service.clone(),
    ));

    let mfa_totp_service_for_admin = mfa_totp_service.clone();

    let mfa_application = Arc::new(MfaApplication::new(
        pg_client.clone(),
        mfa_totp_service,
        session_service.clone(),
        user_service.clone(),
        identity_service.clone(),
        user_role_service.clone(),
    ));

    let api_token_application = Arc::new(ApiTokenApplication::new(
        pg_client.clone(),
        api_token_service,
    ));

    let user_password_application = Arc::new(UserPasswordApplication::new(
        pg_client.clone(),
        identity_service.clone(),
        session_service.clone(),
        user_service.clone(),
        email_service.clone(),
    ));

    let email_template_application = Arc::new(EmailTemplateApplication::new(
        email_template_service,
        pg_client.clone(),
    ));

    // Clone services for federation before they're moved
    let identity_service_for_federation = identity_service.clone();
    let user_service_for_federation = user_service.clone();
    let session_service_for_federation = session_service.clone();

    let admin_application = Arc::new(AdminApplication::new(
        pg_client.clone(),
        user_service,
        identity_service,
        user_role_service.clone(),
        email_verification_service,
        email_service,
        session_service.clone(),
        mfa_totp_service_for_admin,
    ));

    let admin_session_application = Arc::new(AdminSessionApplication::new(
        pg_client.clone(),
        session_service,
    ));

    let insights_application = Arc::new(InsightsApplication::new(
        pg_client.clone(),
        user_repo_insights,
        session_repo_insights,
        mfa_totp_repo_insights,
        identities_repo_insights,
    ));

    // Federation components
    let supabase_federation_enabled = std::env::var("SUPABASE_FEDERATION_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    let supabase_jwks_url =
        std::env::var("SUPABASE_JWKS_URL").unwrap_or_else(|_| String::new());

    let supabase_jwt_issuer =
        std::env::var("SUPABASE_JWT_ISSUER").unwrap_or_else(|_| String::new());

    let supabase_jwt_audience =
        std::env::var("SUPABASE_JWT_AUDIENCE").unwrap_or_else(|_| String::new());

    let external_identity_repo = Arc::new(ExternalIdentityRepository::new());

    let federation_service = Arc::new(FederationService::new(
        external_identity_repo,
        identity_service_for_federation,
        user_service_for_federation,
        user_role_service.clone(),
        session_service_for_federation,
    ));

    let federation_config = SupabaseFederationConfig {
        enabled: supabase_federation_enabled,
        jwks_url: supabase_jwks_url,
        jwt_issuer: supabase_jwt_issuer,
        jwt_audience: supabase_jwt_audience,
    };

    let federation_application = Arc::new(FederationApplication::new(
        pg_client.clone(),
        federation_service,
        federation_config,
    ));

    let app_state = Arc::new(AppState {
        auth_application,
        system_application,
        policy_application,
        user_application,
        oidc_application,
        mfa_application,
        api_token_application,
        user_password_application,
        email_template_application,
        admin_application,
        admin_session_application,
        insights_application,
        federation_application,
    });

    Ok(build_router(app_state))
}
