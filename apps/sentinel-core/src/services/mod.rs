//! Service layer — single-responsibility business logic units.
//!
//! Each service owns exactly one domain concern (identity management, session
//! tokens, MFA, email, etc.) and exposes async methods that take a
//! `&mut DbConnection<'_>`.  Services never call other services; composition
//! happens in the application layer.

pub mod api_token_service;
pub mod email_service;
pub mod email_template_service;
pub mod email_verification_service;
pub mod federation_service;
pub mod identity_service;
pub mod mfa_totp_service;
pub mod oidc_auth_code_service;
pub mod oidc_client_service;
pub mod oidc_key_service;
pub mod oidc_token_service;
pub mod password_reset_service;
pub mod policy_service;
pub mod provider_config_service;
pub mod session_service;
pub mod supabase_jwt_verifier;
pub mod user_role_service;
pub mod user_service;

pub use api_token_service::ApiTokenService;
pub use email_service::EmailService;
pub use email_template_service::{
    CreateEmailTemplateServiceRequest, EmailTemplateService, RenderedEmail,
    UpdateEmailTemplateServiceRequest,
};
pub use email_verification_service::EmailVerificationService;
pub use federation_service::FederationService;
pub use identity_service::IdentityService;
pub use mfa_totp_service::MfaTotpService;
pub use oidc_auth_code_service::OidcAuthCodeService;
pub use oidc_client_service::OidcClientService;
pub use oidc_key_service::OidcKeyService;
pub use oidc_token_service::OidcTokenService;
pub use password_reset_service::PasswordResetService;
pub use policy_service::PolicyService;
pub use provider_config_service::ProviderConfigurationService;
pub use session_service::{SessionService, SessionTokens};
pub use supabase_jwt_verifier::{SupabaseFederationConfig, SupabaseJwtVerifier, VerifiedSupabaseToken};
pub use user_role_service::UserRoleService;
pub use user_service::UserService;
