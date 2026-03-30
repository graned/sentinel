ALTER TABLE policies
    ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE policy_versions
    ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT FALSE;

-- Backfill: mark the currently active version for each existing policy
UPDATE policy_versions pv
SET    is_active = TRUE
FROM   policies p
WHERE  pv.policy_id = p.policy_id
  AND  pv.version   = p.active_version;
