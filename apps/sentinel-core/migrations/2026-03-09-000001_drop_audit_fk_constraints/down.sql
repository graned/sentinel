-- Restore FK constraints on audit columns (reverts to RESTRICT default).

ALTER TABLE users
    ADD CONSTRAINT users_created_by_fkey FOREIGN KEY (created_by) REFERENCES users(user_id),
    ADD CONSTRAINT users_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES users(user_id);

ALTER TABLE user_identities
    ADD CONSTRAINT user_identities_created_by_fkey FOREIGN KEY (created_by) REFERENCES users(user_id),
    ADD CONSTRAINT user_identities_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES users(user_id);

ALTER TABLE sessions
    ADD CONSTRAINT sessions_created_by_fkey FOREIGN KEY (created_by) REFERENCES users(user_id),
    ADD CONSTRAINT sessions_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES users(user_id);

ALTER TABLE email_verifications
    ADD CONSTRAINT email_verifications_created_by_fkey FOREIGN KEY (created_by) REFERENCES users(user_id),
    ADD CONSTRAINT email_verifications_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES users(user_id);

ALTER TABLE auth_configs
    ADD CONSTRAINT auth_configs_created_by_fkey FOREIGN KEY (created_by) REFERENCES users(user_id),
    ADD CONSTRAINT auth_configs_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES users(user_id);

ALTER TABLE email_templates
    ADD CONSTRAINT email_templates_created_by_fkey FOREIGN KEY (created_by) REFERENCES users(user_id),
    ADD CONSTRAINT email_templates_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES users(user_id);

ALTER TABLE password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_created_by_fkey FOREIGN KEY (created_by) REFERENCES users(user_id),
    ADD CONSTRAINT password_reset_tokens_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES users(user_id);
