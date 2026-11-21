use chrono::{Duration, Utc};
use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::{ChatPermissions, Message, MessageId};

use crate::db::audit::record_audit;
use crate::db::groups::GroupSettings;
use crate::db::moderation::{add_temp_punishment, add_warning, clear_warnings, deactivate_user_punishments};
use crate::engine::normalizer::{html_escape, to_persian_num, RLM};
use crate::engine::permissions::{is_target_protected, verify_bot_permission};
use crate::engine::target::resolve_target;
use crate::errors::AppError;
use crate::services::cache::AppCache;
use crate::services::scheduler::default_member_permissions;
use crate::i18n::get_i18n;
use crate::ui::messages::{ban_notice, kick_notice, mute_notice, unban_notice, unmute_notice, warn_notice};
use crate::ui::styled_keyboard::send_styled_message;

/// بررسی اینکه آیا کاربر هدف هم‌اکنون عضو فعال گروه است یا خیر (نه اینکه قبلاً لفت داده یا بن شده باشد)
pub async fn check_target_is_active_member(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    target_id: teloxide::types::UserId,
) -> bool {
    match bot.get_chat_member(chat_id, target_id).await {
        Ok(member) => !matches!(
            member.status(),
            teloxide::types::ChatMemberStatus::Left | teloxide::types::ChatMemberStatus::Banned
        ),
        Err(_) => false,
    }
}

pub async fn handle_ban(
    bot: &Bot,
    msg: &Message,
    pool: &PgPool,
    cache: &AppCache,
    settings: &GroupSettings,
    duration: Option<Duration>,
    reason: Option<String>,
    owner_id: i64,
) -> Result<(), AppError> {
    verify_bot_permission(bot, msg.chat.id, "can_restrict_members").await?;

    let target = resolve_target(bot, msg, pool, cache)
        .await
        .ok_or(AppError::TargetNotFound)?;
    let sender = msg.from.as_ref().ok_or(AppError::PermissionDenied)?;

    let me = bot.get_me().await.map_err(|e| AppError::Telegram(e.to_string()))?;
    if is_target_protected(bot, msg.chat.id, target.id, me.id, owner_id).await? {
        let warn_msg = get_i18n().moderation.cannot_punish_admin.replace("{RLM}", RLM);
        let _ = send_styled_message(bot, msg.chat.id, &warn_msg, None).await;
        return Ok(());
    }

    if !check_target_is_active_member(bot, msg.chat.id, target.id).await {
        let warn_msg = get_i18n().moderation.user_not_member.replace("{RLM}", RLM);
        let _ = send_styled_message(bot, msg.chat.id, &warn_msg, None).await;
        return Ok(());
    }

    bot.ban_chat_member(msg.chat.id, target.id)
        .await
        .map_err(|e| AppError::Telegram(format!("عدم امکان بن در تلگرام: {}", e)))?;

    if let Some(dur) = duration {
        let expires_at = Utc::now() + dur;
        let _ = add_temp_punishment(
            pool,
            msg.chat.id.0,
            target.id.0 as i64,
            "ban",
            expires_at,
            reason.as_deref(),
            sender.id.0 as i64,
        )
        .await;
    }

    record_audit(
        pool,
        msg.chat.id.0,
        sender.id.0 as i64,
        Some(target.id.0 as i64),
        "ban",
        reason.as_deref(),
    )
    .await?;

    if !settings.silent_punishment {
        let notice = ban_notice(&target, duration, reason.as_deref());
        let _ = send_styled_message(bot, msg.chat.id, &notice, None).await;
    }

    Ok(())
}

pub async fn handle_kick(
    bot: &Bot,
    msg: &Message,
    pool: &PgPool,
    cache: &AppCache,
    _settings: &GroupSettings,
    reason: Option<String>,
    owner_id: i64,
) -> Result<(), AppError> {
    verify_bot_permission(bot, msg.chat.id, "can_restrict_members").await?;

    let target = resolve_target(bot, msg, pool, cache)
        .await
        .ok_or(AppError::TargetNotFound)?;
    let sender = msg.from.as_ref().ok_or(AppError::PermissionDenied)?;

    let me = bot.get_me().await.map_err(|e| AppError::Telegram(e.to_string()))?;
    if is_target_protected(bot, msg.chat.id, target.id, me.id, owner_id).await? {
        let warn_msg = get_i18n().moderation.cannot_kick_admin.replace("{RLM}", RLM);
        let _ = send_styled_message(bot, msg.chat.id, &warn_msg, None).await;
        return Ok(());
    }

    if !check_target_is_active_member(bot, msg.chat.id, target.id).await {
        let warn_msg = get_i18n().moderation.user_not_member.replace("{RLM}", RLM);
        let _ = send_styled_message(bot, msg.chat.id, &warn_msg, None).await;
        return Ok(());
    }

    bot.ban_chat_member(msg.chat.id, target.id).await
        .map_err(|e| AppError::Telegram(e.to_string()))?;
    let _ = bot.unban_chat_member(msg.chat.id, target.id).await;

    record_audit(
        pool,
        msg.chat.id.0,
        sender.id.0 as i64,
        Some(target.id.0 as i64),
        "kick",
        reason.as_deref(),
    )
    .await?;

    let notice = kick_notice(&target, reason.as_deref());
    let _ = send_styled_message(bot, msg.chat.id, &notice, None).await;

    Ok(())
}

pub async fn handle_mute(
    bot: &Bot,
    msg: &Message,
    pool: &PgPool,
    cache: &AppCache,
    settings: &GroupSettings,
    duration: Option<Duration>,
    reason: Option<String>,
    owner_id: i64,
) -> Result<(), AppError> {
    verify_bot_permission(bot, msg.chat.id, "can_restrict_members").await?;

    let target = resolve_target(bot, msg, pool, cache)
        .await
        .ok_or(AppError::TargetNotFound)?;
    let sender = msg.from.as_ref().ok_or(AppError::PermissionDenied)?;

    let me = bot.get_me().await.map_err(|e| AppError::Telegram(e.to_string()))?;
    if is_target_protected(bot, msg.chat.id, target.id, me.id, owner_id).await? {
        let warn_msg = get_i18n().moderation.cannot_mute_admin.replace("{RLM}", RLM);
        let _ = send_styled_message(bot, msg.chat.id, &warn_msg, None).await;
        return Ok(());
    }

    if !check_target_is_active_member(bot, msg.chat.id, target.id).await {
        let warn_msg = get_i18n().moderation.user_not_member.replace("{RLM}", RLM);
        let _ = send_styled_message(bot, msg.chat.id, &warn_msg, None).await;
        return Ok(());
    }

    // بررسی اینکه آیا کاربر از قبل در حالت سکوت / تبعید قرار دارد یا خیر
    let is_telegram_muted = match bot.get_chat_member(msg.chat.id, target.id).await {
        Ok(member) => match &member.kind {
            teloxide::types::ChatMemberKind::Restricted(r) => !r.can_send_messages,
            _ => false,
        },
        Err(_) => false,
    };
    let is_db_muted = crate::db::moderation::is_user_muted_active(pool, msg.chat.id.0, target.id.0 as i64).await.unwrap_or(false);

    // اگر کاربر در تلگرام آزاد است ولی در دیتابیس سابقه سکوت فعال باقی مانده، آن را پاکسازی می‌کنیم
    if !is_telegram_muted && is_db_muted {
        let _ = deactivate_user_punishments(pool, msg.chat.id.0, target.id.0 as i64, "mute").await;
    }

    if is_telegram_muted {
        let token = cache.store_pending_mute(
            target.id.0,
            target.first_name.clone(),
            target.username.clone(),
            duration,
            reason.clone(),
            sender.id.0,
        );
        let target_display = if let Some(ref uname) = target.username {
            format!("@{}", uname)
        } else {
            html_escape(&target.first_name)
        };
        let prompt_text = get_i18n().moderation.mute_update_prompt
            .replace("{target_display}", &target_display)
            .replace("{RLM}", RLM);
        let kb = crate::ui::keyboards::confirm_mute_update_styled(&token);
        let _ = send_styled_message(bot, msg.chat.id, &prompt_text, Some(&kb)).await;
        return Ok(());
    }

    let empty_perms = ChatPermissions::empty();
    bot.restrict_chat_member(msg.chat.id, target.id, empty_perms).await
        .map_err(|e| AppError::Telegram(format!("خطا در سلب دسترسی تلگرام: {}", e)))?;

    if let Some(dur) = duration {
        let expires_at = Utc::now() + dur;
        let _ = add_temp_punishment(
            pool,
            msg.chat.id.0,
            target.id.0 as i64,
            "mute",
            expires_at,
            reason.as_deref(),
            sender.id.0 as i64,
        )
        .await;
    }

    record_audit(
        pool,
        msg.chat.id.0,
        sender.id.0 as i64,
        Some(target.id.0 as i64),
        "mute",
        reason.as_deref(),
    )
    .await?;

    if !settings.silent_punishment {
        let notice = mute_notice(&target, duration, reason.as_deref());
        let _ = send_styled_message(bot, msg.chat.id, &notice, None).await;
    }

    Ok(())
}

pub async fn handle_unban(
    bot: &Bot,
    msg: &Message,
    pool: &PgPool,
    cache: &AppCache,
) -> Result<(), AppError> {
    verify_bot_permission(bot, msg.chat.id, "can_restrict_members").await?;

    let target = resolve_target(bot, msg, pool, cache)
        .await
        .ok_or(AppError::TargetNotFound)?;
    let sender = msg.from.as_ref().ok_or(AppError::PermissionDenied)?;

    bot.unban_chat_member(msg.chat.id, target.id).await
        .map_err(|e| AppError::Telegram(e.to_string()))?;

    let _ = deactivate_user_punishments(pool, msg.chat.id.0, target.id.0 as i64, "ban").await;

    record_audit(
        pool,
        msg.chat.id.0,
        sender.id.0 as i64,
        Some(target.id.0 as i64),
        "unban",
        None,
    )
    .await?;

    let notice = unban_notice(&target.mention_display());
    let _ = send_styled_message(bot, msg.chat.id, &notice, None).await;

    Ok(())
}

pub async fn handle_unmute(
    bot: &Bot,
    msg: &Message,
    pool: &PgPool,
    cache: &AppCache,
) -> Result<(), AppError> {
    verify_bot_permission(bot, msg.chat.id, "can_restrict_members").await?;

    let target = resolve_target(bot, msg, pool, cache)
        .await
        .ok_or(AppError::TargetNotFound)?;
    let sender = msg.from.as_ref().ok_or(AppError::PermissionDenied)?;

    let default_perms = default_member_permissions();
    bot.restrict_chat_member(msg.chat.id, target.id, default_perms).await
        .map_err(|e| AppError::Telegram(e.to_string()))?;

    let _ = deactivate_user_punishments(pool, msg.chat.id.0, target.id.0 as i64, "mute").await;

    record_audit(
        pool,
        msg.chat.id.0,
        sender.id.0 as i64,
        Some(target.id.0 as i64),
        "unmute",
        None,
    )
    .await?;

    let notice = unmute_notice(&target.mention_display());
    let _ = send_styled_message(bot, msg.chat.id, &notice, None).await;

    Ok(())
}

pub async fn handle_warn(
    bot: &Bot,
    msg: &Message,
    pool: &PgPool,
    cache: &AppCache,
    settings: &GroupSettings,
    reason: Option<String>,
    owner_id: i64,
) -> Result<(), AppError> {
    let target = resolve_target(bot, msg, pool, cache)
        .await
        .ok_or(AppError::TargetNotFound)?;
    let sender = msg.from.as_ref().ok_or(AppError::PermissionDenied)?;

    let me = bot.get_me().await.map_err(|e| AppError::Telegram(e.to_string()))?;
    if is_target_protected(bot, msg.chat.id, target.id, me.id, owner_id).await? {
        let warn_msg = get_i18n().moderation.cannot_warn_admin.replace("{RLM}", RLM);
        let _ = send_styled_message(bot, msg.chat.id, &warn_msg, None).await;
        return Ok(());
    }

    if !check_target_is_active_member(bot, msg.chat.id, target.id).await {
        let warn_msg = get_i18n().moderation.user_not_member.replace("{RLM}", RLM);
        let _ = send_styled_message(bot, msg.chat.id, &warn_msg, None).await;
        return Ok(());
    }

    let r_text = reason.unwrap_or("نقض قوانین".to_string());
    let current_warns = add_warning(
        pool,
        msg.chat.id.0,
        target.id.0 as i64,
        &r_text,
        sender.id.0 as i64,
    )
    .await?;

    record_audit(
        pool,
        msg.chat.id.0,
        sender.id.0 as i64,
        Some(target.id.0 as i64),
        "warn",
        Some(&r_text),
    )
    .await?;

    if current_warns >= settings.warn_limit as i64 {
        let _ = clear_warnings(pool, msg.chat.id.0, target.id.0 as i64).await;
        
        match settings.warn_escalation_action.as_str() {
            "ban" => {
                let _ = bot.ban_chat_member(msg.chat.id, target.id).await;
                let notice = get_i18n().moderation.warn_escalation_ban
                    .replace("{mention}", &target.mention_display())
                    .replace("{limit}", &to_persian_num(settings.warn_limit))
                    .replace("{RLM}", RLM);
                let _ = send_styled_message(bot, msg.chat.id, &notice, None).await;
            }
            "kick" => {
                let _ = bot.ban_chat_member(msg.chat.id, target.id).await;
                let _ = bot.unban_chat_member(msg.chat.id, target.id).await;
                let notice = get_i18n().moderation.warn_escalation_kick
                    .replace("{mention}", &target.mention_display())
                    .replace("{limit}", &to_persian_num(settings.warn_limit))
                    .replace("{RLM}", RLM);
                let _ = send_styled_message(bot, msg.chat.id, &notice, None).await;
            }
            "mute_1h" => {
                let empty_perms = ChatPermissions::empty();
                let _ = bot.restrict_chat_member(msg.chat.id, target.id, empty_perms).await;
                let expires = Utc::now() + Duration::hours(1);
                let _ = add_temp_punishment(
                    pool,
                    msg.chat.id.0,
                    target.id.0 as i64,
                    "mute",
                    expires,
                    Some("تکمیل سقف اخطارها"),
                    sender.id.0 as i64,
                ).await;
                let notice = get_i18n().moderation.warn_escalation_mute_1h
                    .replace("{mention}", &target.mention_display())
                    .replace("{limit}", &to_persian_num(settings.warn_limit))
                    .replace("{RLM}", RLM);
                let _ = send_styled_message(bot, msg.chat.id, &notice, None).await;
            }
            _ => {
                // پیش‌فرض: سکوت ۲۴ ساعت
                let empty_perms = ChatPermissions::empty();
                let _ = bot.restrict_chat_member(msg.chat.id, target.id, empty_perms).await;
                let expires = Utc::now() + Duration::hours(24);
                let _ = add_temp_punishment(
                    pool,
                    msg.chat.id.0,
                    target.id.0 as i64,
                    "mute",
                    expires,
                    Some("تکمیل سقف اخطارها"),
                    sender.id.0 as i64,
                ).await;
                let notice = get_i18n().moderation.warn_escalation_mute_24h
                    .replace("{mention}", &target.mention_display())
                    .replace("{limit}", &to_persian_num(settings.warn_limit))
                    .replace("{RLM}", RLM);
                let _ = send_styled_message(bot, msg.chat.id, &notice, None).await;
            }
        }
    } else {
        let notice = warn_notice(&target, current_warns, settings.warn_limit, Some(&r_text));
        let _ = send_styled_message(bot, msg.chat.id, &notice, None).await;
    }

    Ok(())
}

pub async fn handle_purge(
    bot: &Bot,
    msg: &Message,
    count: usize,
) -> Result<(), AppError> {
    let current_id = msg.id.0;
    let chat_id = msg.chat.id;

    for i in 0..count {
        if current_id > i as i32 {
            let target_msg_id = MessageId(current_id - (i as i32));
            let _ = bot.delete_message(chat_id, target_msg_id).await;
        }
    }

    Ok(())
}
