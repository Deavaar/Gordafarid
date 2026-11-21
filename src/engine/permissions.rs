use teloxide::prelude::*;
use teloxide::types::{ChatId, ChatMemberStatus, UserId};
use crate::errors::AppError;
use crate::services::cache::AppCache;

/// بررسی دسترسی‌های فرستنده پیام به همراه کشینگ پرسرعت در حافظه (RBAC Cache)
pub async fn check_sender_is_admin_cached(
    bot: &Bot,
    cache: &AppCache,
    chat_id: ChatId,
    user_id: UserId,
    owner_id: i64,
) -> Result<bool, AppError> {
    if let Some(admins) = cache.get_group_admins(chat_id.0) {
        return Ok(admins.contains(&user_id.0));
    }

    match bot.get_chat_administrators(chat_id).await {
        Ok(members) => {
            let admin_ids: Vec<u64> = members.into_iter().map(|m| m.user.id.0).collect();
            let is_admin = admin_ids.contains(&user_id.0);
            cache.set_group_admins(chat_id.0, admin_ids);
            Ok(is_admin)
        }
        Err(_) => check_sender_is_admin(bot, chat_id, user_id, owner_id).await,
    }
}

/// بررسی دسترسی‌های فرستنده پیام (بدون کش)
pub async fn check_sender_is_admin(
    bot: &Bot,
    chat_id: ChatId,
    user_id: UserId,
    _owner_id: i64,
) -> Result<bool, AppError> {
    let member = bot
        .get_chat_member(chat_id, user_id)
        .await
        .map_err(|e| AppError::Telegram(e.to_string()))?;

    match member.status() {
        ChatMemberStatus::Administrator | ChatMemberStatus::Owner => Ok(true),
        _ => Ok(false),
    }
}

/// بررسی اینکه آیا هدف عملیات خود ربات یا ادمین دیگری است یا خیر با استفاده از کش
pub async fn is_target_protected_cached(
    bot: &Bot,
    cache: &AppCache,
    chat_id: ChatId,
    target_id: UserId,
    bot_id: UserId,
    owner_id: i64,
) -> Result<bool, AppError> {
    if target_id == bot_id {
        return Ok(true);
    }

    if let Some(admins) = cache.get_group_admins(chat_id.0) {
        return Ok(admins.contains(&target_id.0));
    }

    is_target_protected(bot, chat_id, target_id, bot_id, owner_id).await
}

/// بررسی اینکه آیا هدف عملیات خود ربات یا ادمین دیگری است یا خیر (محافظت از مدیران)
pub async fn is_target_protected(
    bot: &Bot,
    chat_id: ChatId,
    target_id: UserId,
    bot_id: UserId,
    _owner_id: i64,
) -> Result<bool, AppError> {
    if target_id == bot_id {
        return Ok(true);
    }

    let member = match bot.get_chat_member(chat_id, target_id).await {
        Ok(m) => m,
        Err(_) => return Ok(false),
    };

    match member.status() {
        ChatMemberStatus::Administrator | ChatMemberStatus::Owner => Ok(true),
        _ => Ok(false),
    }
}

/// بررسی دسترسی‌های واقعی ربات در گروه
pub async fn verify_bot_permission(
    bot: &Bot,
    chat_id: ChatId,
    _required: &str,
) -> Result<(), AppError> {
    let me = bot.get_me().await.map_err(|e| AppError::Telegram(e.to_string()))?;
    let bot_member = bot
        .get_chat_member(chat_id, me.id)
        .await
        .map_err(|e| AppError::Telegram(e.to_string()))?;

    match bot_member.status() {
        ChatMemberStatus::Administrator | ChatMemberStatus::Owner => Ok(()),
        _ => Err(AppError::BotLacksPermission(
            "ربات باید به عنوان مدیر (Admin) با دسترسی‌های لازم در گروه باشد."
        )),
    }
}
