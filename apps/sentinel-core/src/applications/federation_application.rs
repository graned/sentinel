//! Federation application layer — orchestrates external token exchange flows.
//!
//! Coordinates Supabase JWT verification and session creation.

use crate::{
    http::api::dtos::BasicLoginResponse, DbConnection, FederationService, PostgresClient,
    ServiceError, SupabaseJwtVerifier,
};
use std::sync::Arc;

/// Configuration for Supabase federation.
pub struct SupabaseFederationConfig {
    pub enabled: bool,
    pub jwks_url: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
}

/// Orchestrates federated authentication flows.
pub struct FederationApplication {
    pg_client: Arc<PostgresClient>,
    supabase_verifier: Option<SupabaseJwtVerifier>,
    federation_service: Arc<FederationService>,
    config: SupabaseFederationConfig,
}

impl FederationApplication {
    pub fn new(
        pg_client: Arc<PostgresClient>,
        federation_service: Arc<FederationService>,
        config: SupabaseFederationConfig,
    ) -> Self {
        let supabase_verifier = if config.enabled {
            Some(SupabaseJwtVerifier::new(
                config.jwks_url.clone(),
                config.jwt_issuer.clone(),
                config.jwt_audience.clone(),
            ))
        } else {
            None
        };

        Self {
            pg_client,
            supabase_verifier,
            federation_service,
            config,
        }
    }

    /// Check if Supabase federation is enabled.
    pub fn is_supabase_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Exchange a Supabase JWT for a native Sentinel session.
    ///
    /// # Flow
    /// 1. Validate Supabase JWT (signature, iss, aud, exp, nbf, sub)
    /// 2. Extract identity from claims (use `sub` as stable identity)
    /// 3. If external identity exists → load linked user → create session
    /// 4. If external identity doesn't exist → create user + identity mapping + session
    /// 5. Return native Sentinel access_token / refresh_token
    pub async fn exchange_supabase_token(
        &self,
        access_token: String,
    ) -> Result<BasicLoginResponse, ServiceError> {
        if !self.config.enabled {
            return Err(ServiceError::FederationNotEnabled(
                "Supabase federation is not enabled".to_string(),
            ));
        }

        let verifier = self.supabase_verifier.as_ref().ok_or_else(|| {
            ServiceError::InternalError("Supabase verifier not initialized".to_string())
        })?;

        let verified = verifier.verify_token(&access_token).await?;

        let mut conn = self.pg_client.get_conn().await?;

        let subject = verified.user_id.to_string();

        let (session, tokens) = self
            .federation_service
            .exchange_external_identity(
                &mut conn,
                "supabase",
                &self.config.jwt_issuer,
                &subject,
                verified.email,
                verified.user_metadata,
            )
            .await?;

        let expires_at = chrono::Utc::now() + self.federation_service.session_service.access_ttl;

        Ok(BasicLoginResponse {
            user_id: session.user_id,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_at,
            must_change_password: false,
            mfa_setup_required: false,
        })
    }
}
