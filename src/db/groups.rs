use sqlx::PgPool;
use chrono::NaiveTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GroupSettings {
    pub chat_id: i64,
    pub welcome_enabled: bool,
    pub welcome_text: String,
    pub goodbye_enabled: bool,
    pub goodbye_text: String,
    pub rules_text: String,
    
    pub anti_flood_enabled: bool,
    pub anti_flood_rate: i32,
    pub anti_flood_window_secs: i32,
    pub anti_flood_action: String,
    
    pub link_filter_enabled: bool,
    pub link_filter_action: String,
    
    pub forward_filter_enabled: bool,
    pub media_filter_enabled: bool,
    pub photo_filter_enabled: bool,
    pub video_filter_enabled: bool,
    pub voice_filter_enabled: bool,
    pub document_filter_enabled: bool,
    pub gif_filter_enabled: bool,
    pub sticker_filter_enabled: bool,
    pub service_msg_filter_enabled: bool,
    
    pub night_mode_enabled: bool,
    pub night_mode_start: NaiveTime,
    pub night_mode_end: NaiveTime,
    pub night_mode_timezone: String,
    
    pub captcha_enabled: bool,
    pub captcha_timeout_secs: i32,
    pub captcha_fail_action: String,
    
    pub warn_limit: i32,
    pub warn_escalation_action: String,
    
    pub forced_channel: Option<String>,
    pub silent_punishment: bool,
    pub group_lock_mode: String,
    
    pub channel_track_enabled: bool,
    pub track_photo: bool,
    pub track_video: bool,
    pub track_voice: bool,
    pub track_gif: bool,
    pub track_sticker: bool,
    pub track_document: bool,
    pub track_text: bool,
    pub log_topic_id: Option<i64>,
}

pub async fn get_or_create_group(
    pool: &PgPool,
    chat_id: i64,
    title: &str,
) -> Result<GroupSettings, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO groups (chat_id, title)
        VALUES ($1, $2)
        ON CONFLICT (chat_id) DO UPDATE SET title = EXCLUDED.title, updated_at = NOW()
        "#,
        chat_id,
        title
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO group_settings (chat_id)
        VALUES ($1)
        ON CONFLICT (chat_id) DO NOTHING
        "#,
        chat_id
    )
    .execute(pool)
    .await?;

    get_settings(pool, chat_id).await
}

pub async fn get_settings(pool: &PgPool, chat_id: i64) -> Result<GroupSettings, sqlx::Error> {
    sqlx::query_as!(
        GroupSettings,
        r#"
        SELECT
            chat_id, welcome_enabled, welcome_text, goodbye_enabled, goodbye_text,
            rules_text, anti_flood_enabled, anti_flood_rate, anti_flood_window_secs,
            anti_flood_action, link_filter_enabled, link_filter_action,
            forward_filter_enabled, media_filter_enabled, photo_filter_enabled,
            video_filter_enabled, voice_filter_enabled, document_filter_enabled,
            gif_filter_enabled, sticker_filter_enabled, service_msg_filter_enabled,
            night_mode_enabled, night_mode_start, night_mode_end, night_mode_timezone,
            captcha_enabled, captcha_timeout_secs, captcha_fail_action,
            warn_limit, warn_escalation_action, forced_channel, silent_punishment,
            COALESCE(group_lock_mode, 'unlocked') AS "group_lock_mode!",
            COALESCE(channel_track_enabled, FALSE) AS "channel_track_enabled!",
            COALESCE(track_photo, TRUE) AS "track_photo!",
            COALESCE(track_video, TRUE) AS "track_video!",
            COALESCE(track_voice, TRUE) AS "track_voice!",
            COALESCE(track_gif, TRUE) AS "track_gif!",
            COALESCE(track_sticker, TRUE) AS "track_sticker!",
            COALESCE(track_document, TRUE) AS "track_document!",
            COALESCE(track_text, TRUE) AS "track_text!",
            log_topic_id
        FROM group_settings
        WHERE chat_id = $1
        "#,
        chat_id
    )
    .fetch_one(pool)
    .await
}

pub async fn update_welcome_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET welcome_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_goodbye_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET goodbye_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_anti_flood_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET anti_flood_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_anti_flood_params(pool: &PgPool, chat_id: i64, rate: i32, window: i32) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET anti_flood_rate = $1, anti_flood_window_secs = $2 WHERE chat_id = $3",
        rate,
        window,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_link_filter_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET link_filter_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_forward_filter_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET forward_filter_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_media_filter_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET media_filter_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_sticker_filter_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET sticker_filter_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_night_mode_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET night_mode_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_night_mode_hours(pool: &PgPool, chat_id: i64, start: NaiveTime, end: NaiveTime) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET night_mode_start = $1, night_mode_end = $2 WHERE chat_id = $3",
        start,
        end,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_captcha_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET captcha_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_captcha_timeout(pool: &PgPool, chat_id: i64, timeout: i32) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET captcha_timeout_secs = $1 WHERE chat_id = $2",
        timeout,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_warn_limit(pool: &PgPool, chat_id: i64, limit: i32) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET warn_limit = $1 WHERE chat_id = $2",
        limit,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_forced_channel(pool: &PgPool, chat_id: i64, channel: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET forced_channel = $1 WHERE chat_id = $2",
        channel,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_rules_text(pool: &PgPool, chat_id: i64, rules: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET rules_text = $1 WHERE chat_id = $2",
        rules,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_silent_punishment(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET silent_punishment = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_anti_flood_action(pool: &PgPool, chat_id: i64, action: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET anti_flood_action = $1 WHERE chat_id = $2",
        action,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_warn_escalation_action(pool: &PgPool, chat_id: i64, action: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET warn_escalation_action = $1 WHERE chat_id = $2",
        action,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_captcha_fail_action(pool: &PgPool, chat_id: i64, action: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET captcha_fail_action = $1 WHERE chat_id = $2",
        action,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_link_filter_action(pool: &PgPool, chat_id: i64, action: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET link_filter_action = $1 WHERE chat_id = $2",
        action,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn set_welcome_text(pool: &PgPool, chat_id: i64, text: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET welcome_text = $1 WHERE chat_id = $2",
        text,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn set_goodbye_text(pool: &PgPool, chat_id: i64, text: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET goodbye_text = $1 WHERE chat_id = $2",
        text,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_photo_filter_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET photo_filter_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_video_filter_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET video_filter_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_voice_filter_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET voice_filter_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_document_filter_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET document_filter_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_gif_filter_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET gif_filter_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_service_msg_filter_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET service_msg_filter_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_all_night_mode_groups(pool: &PgPool) -> Result<Vec<GroupSettings>, sqlx::Error> {
    sqlx::query_as!(
        GroupSettings,
        r#"
        SELECT
            chat_id, welcome_enabled, welcome_text, goodbye_enabled, goodbye_text,
            rules_text, anti_flood_enabled, anti_flood_rate, anti_flood_window_secs,
            anti_flood_action, link_filter_enabled, link_filter_action,
            forward_filter_enabled, media_filter_enabled, photo_filter_enabled,
            video_filter_enabled, voice_filter_enabled, document_filter_enabled,
            gif_filter_enabled, sticker_filter_enabled, service_msg_filter_enabled,
            night_mode_enabled, night_mode_start, night_mode_end, night_mode_timezone,
            captcha_enabled, captcha_timeout_secs, captcha_fail_action,
            warn_limit, warn_escalation_action, forced_channel, silent_punishment,
            COALESCE(group_lock_mode, 'unlocked') AS "group_lock_mode!",
            COALESCE(channel_track_enabled, FALSE) AS "channel_track_enabled!",
            COALESCE(track_photo, TRUE) AS "track_photo!",
            COALESCE(track_video, TRUE) AS "track_video!",
            COALESCE(track_voice, TRUE) AS "track_voice!",
            COALESCE(track_gif, TRUE) AS "track_gif!",
            COALESCE(track_sticker, TRUE) AS "track_sticker!",
            COALESCE(track_document, TRUE) AS "track_document!",
            COALESCE(track_text, TRUE) AS "track_text!",
            log_topic_id
        FROM group_settings
        WHERE night_mode_enabled = TRUE
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn update_group_lock_mode(pool: &PgPool, chat_id: i64, mode: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET group_lock_mode = $1 WHERE chat_id = $2",
        mode,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_channel_track_toggle(pool: &PgPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET channel_track_enabled = $1 WHERE chat_id = $2",
        enabled,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_track_category_toggle(pool: &PgPool, chat_id: i64, category: &str, enabled: bool) -> Result<(), sqlx::Error> {
    let sql = match category {
        "photo" => "UPDATE group_settings SET track_photo = $1 WHERE chat_id = $2",
        "video" => "UPDATE group_settings SET track_video = $1 WHERE chat_id = $2",
        "voice" => "UPDATE group_settings SET track_voice = $1 WHERE chat_id = $2",
        "gif" => "UPDATE group_settings SET track_gif = $1 WHERE chat_id = $2",
        "sticker" => "UPDATE group_settings SET track_sticker = $1 WHERE chat_id = $2",
        "document" => "UPDATE group_settings SET track_document = $1 WHERE chat_id = $2",
        "text" => "UPDATE group_settings SET track_text = $1 WHERE chat_id = $2",
        _ => return Ok(()),
    };
    sqlx::query(sql)
        .bind(enabled)
        .bind(chat_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn save_channel_post_rule(pool: &PgPool, chat_id: i64, post_id: i32, allowed: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO channel_post_rules (chat_id, post_id, allowed_type)
        VALUES ($1, $2, $3)
        ON CONFLICT (chat_id, post_id) DO UPDATE SET allowed_type = EXCLUDED.allowed_type
        "#,
        chat_id,
        post_id,
        allowed
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_channel_post_rule(pool: &PgPool, chat_id: i64, post_id: i32) -> Result<Option<String>, sqlx::Error> {
    let rec = sqlx::query!(
        "SELECT allowed_type FROM channel_post_rules WHERE chat_id = $1 AND post_id = $2",
        chat_id,
        post_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(rec.map(|r| r.allowed_type))
}

pub async fn update_log_topic(pool: &PgPool, chat_id: i64, topic_id: Option<i64>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE group_settings SET log_topic_id = $1 WHERE chat_id = $2",
        topic_id,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_group_added_by(
    pool: &PgPool,
    chat_id: i64,
    added_by: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE groups
        SET added_by = $1, updated_at = NOW()
        WHERE chat_id = $2
        "#,
        added_by,
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_groups_by_added_by(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<(i64, String)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT chat_id, title
        FROM groups
        WHERE is_active = TRUE AND added_by = $1
        ORDER BY title ASC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| (r.chat_id, r.title)).collect())
}

pub async fn get_all_active_groups(
    pool: &PgPool,
) -> Result<Vec<(i64, String, Option<i64>)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT chat_id, title, added_by
        FROM groups
        WHERE is_active = TRUE
        ORDER BY title ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| (r.chat_id, r.title, r.added_by)).collect())
}

pub async fn get_group_added_by(
    pool: &PgPool,
    chat_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT added_by
        FROM groups
        WHERE chat_id = $1
        "#,
        chat_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.and_then(|r| r.added_by))
}

pub async fn check_is_group_owner(
    bot: &teloxide::Bot,
    pool: &PgPool,
    chat_id: i64,
    user_id: u64,
) -> bool {
    use teloxide::prelude::*;
    if let Ok(Some(added_by)) = get_group_added_by(pool, chat_id).await {
        if added_by == user_id as i64 {
            return true;
        }
    }

    match bot.get_chat_member(teloxide::types::ChatId(chat_id), teloxide::types::UserId(user_id)).await {
        Ok(m) => {
            if matches!(m.kind, teloxide::types::ChatMemberKind::Owner(_)) {
                let _ = update_group_added_by(pool, chat_id, user_id as i64).await;
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// گروه‌هایی که کاربر دقیقاً مالک آن‌هاست (ربات را اد کرده یا مالک اصلی تلگرامی گروه است)
/// این متد برای دکمه «دسترسی ادمین‌ها» در پیوی استفاده می‌شود
pub async fn resolve_user_strictly_owned_groups(
    bot: &teloxide::Bot,
    pool: &PgPool,
    user_id: u64,
) -> Result<Vec<(i64, String)>, sqlx::Error> {
    use teloxide::prelude::*;
    let mut owned = get_groups_by_added_by(pool, user_id as i64).await?;

    let all = get_all_active_groups(pool).await?;
    for (chat_id, title, added_by) in all {
        if added_by == Some(user_id as i64) {
            continue;
        }

        if added_by.is_none() {
            if let Ok(member) = bot.get_chat_member(teloxide::types::ChatId(chat_id), teloxide::types::UserId(user_id)).await {
                if matches!(member.kind, teloxide::types::ChatMemberKind::Owner(_)) {
                    let _ = update_group_added_by(pool, chat_id, user_id as i64).await;
                    if !owned.iter().any(|(cid, _)| *cid == chat_id) {
                        owned.push((chat_id, title));
                    }
                }
            }
        }
    }

    Ok(owned)
}

/// گروه‌هایی که کاربر در آن‌ها ادمین یا مالک است
/// این متد برای دستور «پنل» در پیوی استفاده می‌شود تا فقط گروه‌هایی که کاربر در آنها ادمین است نمایش یابند
pub async fn resolve_user_admin_groups(
    bot: &teloxide::Bot,
    pool: &PgPool,
    user_id: u64,
) -> Result<Vec<(i64, String)>, sqlx::Error> {
    use teloxide::prelude::*;
    let mut admin_groups = Vec::new();

    let all = get_all_active_groups(pool).await?;
    for (chat_id, title, added_by) in all {
        if added_by == Some(user_id as i64) {
            admin_groups.push((chat_id, title));
            continue;
        }

        if let Ok(member) = bot.get_chat_member(teloxide::types::ChatId(chat_id), teloxide::types::UserId(user_id)).await {
            match member.status() {
                teloxide::types::ChatMemberStatus::Administrator => {
                    admin_groups.push((chat_id, title));
                }
                teloxide::types::ChatMemberStatus::Owner => {
                    if added_by.is_none() {
                        let _ = update_group_added_by(pool, chat_id, user_id as i64).await;
                    }
                    admin_groups.push((chat_id, title));
                }
                _ => {}
            }
        }
    }

    Ok(admin_groups)
}

pub async fn resolve_user_owned_groups(
    bot: &teloxide::Bot,
    pool: &PgPool,
    user_id: u64,
    _bot_owner_id: i64,
) -> Result<Vec<(i64, String)>, sqlx::Error> {
    resolve_user_strictly_owned_groups(bot, pool, user_id).await
}

