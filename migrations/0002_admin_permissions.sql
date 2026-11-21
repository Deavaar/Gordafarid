-- Gordafarid Migration 0002: Group Owner & Admin Permissions

ALTER TABLE groups ADD COLUMN IF NOT EXISTS added_by BIGINT DEFAULT NULL;

CREATE TABLE IF NOT EXISTS admin_permissions (
    chat_id BIGINT NOT NULL,
    admin_id BIGINT NOT NULL, -- 0 indicates GLOBAL (سراسری) for all admins in chat
    permission_key TEXT NOT NULL,
    allowed BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (chat_id, admin_id, permission_key)
);

CREATE INDEX IF NOT EXISTS idx_admin_perms_chat ON admin_permissions(chat_id, admin_id);
