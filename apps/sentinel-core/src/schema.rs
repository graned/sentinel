// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "email_template_type"))]
    pub struct EmailTemplateType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "identity_provider"))]
    pub struct IdentityProvider;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "revocation_reason"))]
    pub struct RevocationReason;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "role_type"))]
    pub struct RoleType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_status"))]
    pub struct UserStatus;
}

diesel::table! {
    api_tokens (api_token_id) {
        api_token_id -> Uuid,
        user_id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        token_hash -> Text,
        expires_at -> Nullable<Timestamptz>,
        last_used_at -> Nullable<Timestamptz>,
        revoked_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        created_by -> Nullable<Uuid>,
        updated_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    auth_configs (config_id) {
        config_id -> Uuid,
        user_id -> Uuid,
        key_hash -> Text,
        #[max_length = 20]
        algorithm -> Varchar,
        #[max_length = 255]
        issuer -> Varchar,
        default_expiry_seconds -> Int4,
        is_active -> Bool,
        rotated_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        created_by -> Nullable<Uuid>,
        updated_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::EmailTemplateType;

    email_templates (template_id) {
        template_id -> Uuid,
        template_type -> EmailTemplateType,
        subject -> Text,
        body_text -> Text,
        body_html -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        created_by -> Nullable<Uuid>,
        updated_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    email_verifications (verification_id) {
        verification_id -> Uuid,
        identity_id -> Uuid,
        token_hash -> Text,
        expires_at -> Timestamptz,
        verified_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        created_by -> Nullable<Uuid>,
        updated_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    oidc_auth_codes (oidc_auth_code_id) {
        oidc_auth_code_id -> Uuid,
        code_hash -> Text,
        oidc_client_id -> Uuid,
        user_id -> Uuid,
        redirect_uri -> Text,
        scope -> Text,
        nonce -> Nullable<Text>,
        code_challenge -> Text,
        code_challenge_method -> Text,
        expires_at -> Timestamptz,
        consumed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    oidc_clients (oidc_client_id) {
        oidc_client_id -> Uuid,
        tenant_id -> Nullable<Uuid>,
        client_id -> Text,
        client_secret_hash -> Nullable<Text>,
        name -> Text,
        redirect_uris -> Array<Text>,
        allowed_scopes -> Array<Text>,
        pkce_required -> Bool,
        is_confidential -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    oidc_signing_keys (oidc_signing_key_id) {
        oidc_signing_key_id -> Uuid,
        kid -> Text,
        alg -> Text,
        public_jwk_json -> Jsonb,
        private_key_encrypted -> Bytea,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    password_reset_tokens (reset_token_id) {
        reset_token_id -> Uuid,
        identity_id -> Uuid,
        token_hash -> Text,
        expires_at -> Timestamptz,
        used_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        created_by -> Nullable<Uuid>,
        updated_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    policies (policy_id) {
        policy_id -> Uuid,
        tenant_id -> Nullable<Uuid>,
        environment -> Text,
        name -> Text,
        description -> Nullable<Text>,
        active_version -> Int8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        is_active -> Bool,
    }
}

diesel::table! {
    policy_versions (policy_version_id) {
        policy_version_id -> Uuid,
        policy_id -> Uuid,
        version -> Int8,
        rules -> Jsonb,
        rules_hash -> Text,
        compiled_rules -> Bytea,
        compiled_hash -> Text,
        compiler_version -> Text,
        compiled_at -> Timestamptz,
        created_at -> Timestamptz,
        is_active -> Bool,
    }
}

diesel::table! {
    provider_configurations (configuration_id) {
        configuration_id -> Uuid,
        tenant_id -> Nullable<Uuid>,
        provider -> Text,
        config_encrypted -> Bytea,
        config_redacted -> Jsonb,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Uuid>,
        updated_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RoleType;

    roles (role_id) {
        role_id -> Uuid,
        #[sql_name = "type"]
        type_ -> RoleType,
        name -> Text,
        description -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Uuid>,
        updated_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RevocationReason;

    sessions (session_id) {
        session_id -> Uuid,
        user_id -> Uuid,
        identity_id -> Uuid,
        refresh_token_hash -> Text,
        refresh_token_family -> Uuid,
        user_agent -> Nullable<Text>,
        ip_address -> Nullable<Text>,
        #[max_length = 20]
        device_type -> Nullable<Varchar>,
        refresh_token_expires_at -> Timestamptz,
        revoked_at -> Nullable<Timestamptz>,
        revoked_reason -> Nullable<RevocationReason>,
        last_used_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        created_by -> Nullable<Uuid>,
        updated_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::IdentityProvider;

    user_identities (identity_id) {
        identity_id -> Uuid,
        user_id -> Uuid,
        provider -> IdentityProvider,
        #[max_length = 255]
        provider_user_id -> Nullable<Varchar>,
        #[max_length = 255]
        email -> Varchar,
        password_hash -> Nullable<Text>,
        password_changed_at -> Nullable<Timestamptz>,
        email_verified -> Nullable<Bool>,
        oauth_access_token -> Nullable<Text>,
        oauth_refresh_token -> Nullable<Text>,
        oauth_token_expires_at -> Nullable<Timestamptz>,
        is_primary -> Bool,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        last_login_at -> Nullable<Timestamptz>,
        created_by -> Nullable<Uuid>,
        updated_by -> Nullable<Uuid>,
        must_change_password -> Bool,
    }
}

diesel::table! {
    user_mfa_totp (user_mfa_totp_id) {
        user_mfa_totp_id -> Uuid,
        user_id -> Uuid,
        secret_encrypted -> Bytea,
        enabled -> Bool,
        enrolled_at -> Nullable<Timestamptz>,
        last_used_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_recovery_codes (user_recovery_code_id) {
        user_recovery_code_id -> Uuid,
        user_id -> Uuid,
        code_hash -> Text,
        used_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_roles (user_role_id) {
        user_role_id -> Uuid,
        user_id -> Uuid,
        role_id -> Uuid,
        created_at -> Timestamptz,
        created_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserStatus;

    users (user_id) {
        user_id -> Uuid,
        #[max_length = 100]
        first_name -> Nullable<Varchar>,
        #[max_length = 100]
        last_name -> Nullable<Varchar>,
        avatar_url -> Nullable<Text>,
        status -> UserStatus,
        token_version -> Int4,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        created_by -> Nullable<Uuid>,
        updated_by -> Nullable<Uuid>,
        mfa_required -> Bool,
        #[max_length = 255]
        display_name -> Nullable<Varchar>,
    }
}

diesel::joinable!(api_tokens -> users (user_id));
diesel::joinable!(auth_configs -> users (user_id));
diesel::joinable!(email_verifications -> user_identities (identity_id));
diesel::joinable!(oidc_auth_codes -> oidc_clients (oidc_client_id));
diesel::joinable!(oidc_auth_codes -> users (user_id));
diesel::joinable!(password_reset_tokens -> user_identities (identity_id));
diesel::joinable!(policy_versions -> policies (policy_id));
diesel::joinable!(sessions -> user_identities (identity_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(user_identities -> users (user_id));
diesel::joinable!(user_mfa_totp -> users (user_id));
diesel::joinable!(user_recovery_codes -> users (user_id));
diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(user_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    api_tokens,auth_configs,email_templates,email_verifications,oidc_auth_codes,oidc_clients,oidc_signing_keys,password_reset_tokens,policies,policy_versions,provider_configurations,roles,sessions,user_identities,user_mfa_totp,user_recovery_codes,user_roles,users,);
