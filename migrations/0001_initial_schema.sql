-- Gordafarid Initial Schema
-- PostgreSQL Migration 0001

CREATE TABLE IF NOT EXISTS groups (
    chat_id BIGINT PRIMARY KEY,
    title TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS group_settings (
    chat_id BIGINT PRIMARY KEY REFERENCES groups(chat_id) ON DELETE CASCADE,
    welcome_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    welcome_text TEXT NOT NULL DEFAULT 'سلام {mention}، به گروه {group} خوش آمدید! 🌸',
    goodbye_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    goodbye_text TEXT NOT NULL DEFAULT 'کاربر {name} از گروه خارج شد.',
    rules_text TEXT NOT NULL DEFAULT 'قوانین گروه هنوز تنظیم نشده است.',
    
    anti_flood_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    anti_flood_rate INT NOT NULL DEFAULT 5,
    anti_flood_window_secs INT NOT NULL DEFAULT 5,
    anti_flood_action TEXT NOT NULL DEFAULT 'mute_5m',
    
    link_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    link_filter_action TEXT NOT NULL DEFAULT 'delete',
    
    forward_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    media_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    photo_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    video_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    voice_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    document_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    gif_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    sticker_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    service_msg_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    
    night_mode_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    night_mode_start TIME NOT NULL DEFAULT '23:00:00',
    night_mode_end TIME NOT NULL DEFAULT '07:00:00',
    night_mode_timezone TEXT NOT NULL DEFAULT 'Asia/Tehran',
    
    captcha_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    captcha_timeout_secs INT NOT NULL DEFAULT 120,
    captcha_fail_action TEXT NOT NULL DEFAULT 'kick',
    
    warn_limit INT NOT NULL DEFAULT 3,
    warn_escalation_action TEXT NOT NULL DEFAULT 'mute_24h',
    
    forced_channel TEXT,
    silent_punishment BOOLEAN NOT NULL DEFAULT FALSE
);

ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS photo_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS video_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS voice_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS document_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS gif_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS service_msg_filter_enabled BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE IF NOT EXISTS banned_words (
    id SERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES groups(chat_id) ON DELETE CASCADE,
    word TEXT NOT NULL,
    list_type INT NOT NULL DEFAULT 1, -- 1: حذف پیام, 2: اخطار + حذف
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (chat_id, word, list_type)
);

CREATE TABLE IF NOT EXISTS allowed_domains (
    id SERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES groups(chat_id) ON DELETE CASCADE,
    domain TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (chat_id, domain)
);

CREATE TABLE IF NOT EXISTS warnings (
    id SERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES groups(chat_id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL,
    reason TEXT NOT NULL DEFAULT '',
    warned_by BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS temporary_punishments (
    id SERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES groups(chat_id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL,
    punishment_type TEXT NOT NULL, -- 'mute', 'ban'
    expires_at TIMESTAMPTZ NOT NULL,
    reason TEXT,
    applied_by BIGINT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS custom_commands (
    id SERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES groups(chat_id) ON DELETE CASCADE,
    trigger TEXT NOT NULL,
    response TEXT NOT NULL,
    match_type TEXT NOT NULL DEFAULT 'exact', -- 'exact', 'contains'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (chat_id, trigger)
);

CREATE TABLE IF NOT EXISTS scheduled_messages (
    id SERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES groups(chat_id) ON DELETE CASCADE,
    message_text TEXT NOT NULL,
    interval_secs INT, -- NULL برای پیام یک‌باره
    next_run_at TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS custom_roles (
    id SERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES groups(chat_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    can_warn BOOLEAN NOT NULL DEFAULT TRUE,
    can_mute BOOLEAN NOT NULL DEFAULT FALSE,
    can_ban BOOLEAN NOT NULL DEFAULT FALSE,
    can_purge BOOLEAN NOT NULL DEFAULT FALSE,
    can_manage_settings BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (chat_id, name)
);

CREATE TABLE IF NOT EXISTS user_roles (
    id SERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES groups(chat_id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL,
    role_id INT NOT NULL REFERENCES custom_roles(id) ON DELETE CASCADE,
    UNIQUE (chat_id, user_id, role_id)
);

CREATE TABLE IF NOT EXISTS captcha_sessions (
    id SERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES groups(chat_id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL,
    message_id INT,
    correct_answer TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE (chat_id, user_id)
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id SERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    admin_id BIGINT NOT NULL,
    target_id BIGINT,
    action TEXT NOT NULL,
    details TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for high-frequency queries
CREATE INDEX IF NOT EXISTS idx_warnings_chat_user ON warnings(chat_id, user_id);
CREATE INDEX IF NOT EXISTS idx_temp_punish_active ON temporary_punishments(is_active, expires_at);
CREATE INDEX IF NOT EXISTS idx_scheduled_active ON scheduled_messages(is_active, next_run_at);
CREATE INDEX IF NOT EXISTS idx_banned_words_chat ON banned_words(chat_id);
CREATE INDEX IF NOT EXISTS idx_custom_commands_chat ON custom_commands(chat_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_chat ON audit_logs(chat_id, created_at DESC);

CREATE TABLE IF NOT EXISTS known_users (
    user_id BIGINT PRIMARY KEY,
    first_name TEXT NOT NULL,
    username TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_known_users_username ON known_users(LOWER(username));

ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS group_lock_mode TEXT NOT NULL DEFAULT 'unlocked';
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS channel_track_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS track_photo BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS track_video BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS track_voice BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS track_gif BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS track_sticker BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS track_document BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS track_text BOOLEAN NOT NULL DEFAULT TRUE;

CREATE TABLE IF NOT EXISTS channel_post_rules (
    chat_id BIGINT NOT NULL,
    post_id INT NOT NULL,
    allowed_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (chat_id, post_id)
);
CREATE INDEX IF NOT EXISTS idx_channel_post_rules ON channel_post_rules(chat_id, post_id);

ALTER TABLE group_settings ADD COLUMN IF NOT EXISTS log_topic_id BIGINT DEFAULT NULL;


