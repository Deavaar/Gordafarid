use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::{Message, MessageEntityKind, MessageKind, MessageOrigin, UserId};
use tracing::debug;

use crate::services::cache::AppCache;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TargetIdentifier {
    User {
        id: UserId,
        first_name: String,
        username: Option<String>,
    },
    Username(String),
    Channel(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TargetUser {
    pub id: UserId,
    pub first_name: String,
    pub username: Option<String>,
}

impl TargetUser {
    pub fn mention_display(&self) -> String {
        if let Some(ref uname) = self.username {
            format!("<b>@{}</b>", uname.trim_start_matches('@'))
        } else {
            let safe_name = crate::engine::normalizer::html_escape(&self.first_name);
            format!("<a href=\"tg://user?id={}\">{}</a>", self.id.0, safe_name)
        }
    }
}

/// استخراج هدف به صورت همگام (برای تست‌ها و موارد بدون نیاز به دیتابیس)
#[allow(dead_code)]
pub fn resolve_target_sync(message: &Message) -> Option<TargetUser> {
    let full_text = message.text().or_else(|| message.caption());
    let entities = message.entities().or_else(|| message.caption_entities());

    // 1. بررسی پیام ریپلای‌شده
    if let Some(reply) = message.reply_to_message() {
        if matches!(reply.kind, MessageKind::ForumTopicCreated(_)) {
            // نادیده گرفتن پیام سیستمی ایجاد تاپیک
        } else if let MessageKind::NewChatMembers(m) = &reply.kind {
            if let Some(first_user) = m.new_chat_members.first() {
                return Some(TargetUser {
                    id: first_user.id,
                    first_name: first_user.first_name.clone(),
                    username: first_user.username.clone(),
                });
            }
        } else if let MessageKind::LeftChatMember(m) = &reply.kind {
            return Some(TargetUser {
                id: m.left_chat_member.id,
                first_name: m.left_chat_member.first_name.clone(),
                username: m.left_chat_member.username.clone(),
            });
        } else if let Some(chat) = reply.sender_chat.as_ref() {
            if reply.from.as_ref().map(|u| u.id.0 == 136817688).unwrap_or(true) {
                return Some(TargetUser {
                    id: UserId(chat.id.0.unsigned_abs()),
                    first_name: chat.title().unwrap_or("کانال").to_string(),
                    username: chat.username().map(|s| s.to_string()),
                });
            }
        } else if let Some(from) = reply.from.as_ref() {
            return Some(TargetUser {
                id: from.id,
                first_name: from.first_name.clone(),
                username: from.username.clone(),
            });
        } else if let Some(origin) = reply.forward_origin() {
            match origin {
                MessageOrigin::User { ref sender_user, .. } => {
                    return Some(TargetUser {
                        id: sender_user.id,
                        first_name: sender_user.first_name.clone(),
                        username: sender_user.username.clone(),
                    });
                }
                MessageOrigin::Channel { ref chat, .. } => {
                    return Some(TargetUser {
                        id: UserId(chat.id.0.unsigned_abs()),
                        first_name: chat.title().unwrap_or("کانال").to_string(),
                        username: chat.username().map(|s| s.to_string()),
                    });
                }
                _ => {}
            }
        }
    }

    // 2. بررسی Telegram 7.0+ External Reply / Quotes
    if let MessageKind::Common(ref common) = message.kind {
        if let Some(ref ext) = common.external_reply {
            match &ext.origin {
                MessageOrigin::User { ref sender_user, .. } => {
                    return Some(TargetUser {
                        id: sender_user.id,
                        first_name: sender_user.first_name.clone(),
                        username: sender_user.username.clone(),
                    });
                }
                MessageOrigin::Channel { ref chat, .. } => {
                    return Some(TargetUser {
                        id: UserId(chat.id.0.unsigned_abs()),
                        first_name: chat.title().unwrap_or("کانال").to_string(),
                        username: chat.username().map(|s| s.to_string()),
                    });
                }
                _ => {}
            }
        }
    }

    // 3. بررسی منشن متنی تلگرام (TextMention)
    if let Some(ents) = entities {
        for entity in ents {
            if let MessageEntityKind::TextMention { ref user } = entity.kind {
                return Some(TargetUser {
                    id: user.id,
                    first_name: user.first_name.clone(),
                    username: user.username.clone(),
                });
            }
        }
    }

    // 4. بررسی شناسه عددی در متن پیام
    if let Some(text) = full_text {
        let parts: Vec<&str> = text.split_whitespace().collect();
        for part in parts {
            let normalized = crate::engine::normalizer::normalize_digits(part);
            if let Ok(num) = normalized.parse::<u64>() {
                if num > 1000 && !normalized.starts_with('/') {
                    return Some(TargetUser {
                        id: UserId(num),
                        first_name: "کاربر".to_string(),
                        username: None,
                    });
                }
            }
        }
    }

    None
}

/// موتور جامع شناسایی و استخراج کاربر هدف
/// پشتیبانی کامل از:
/// 1. ریپلای مستقیم به پیام کاربر
/// 2. ریپلای به پیام ربات حاوی اطلاعات کاربر (مثل اخطار یا اطلاعیه)
/// 3. ریپلای به پیام عضویت عضو جدید (Join service message)
/// 4. ریپلای به پیام‌های فوروارد شده
/// 5. ریپلای نوع دوم تلگرام (Telegram 7.0+ Quote & External Reply)
/// 6. منشن متنی (TextMention)
/// 7. منشن نام کاربری (@username) با بررسی کش، دیتابیس و تلگرام API
/// 8. شناسه‌های عددی (User ID) در متن
pub async fn resolve_target(
    bot: &Bot,
    message: &Message,
    pool: &PgPool,
    cache: &AppCache,
) -> Option<TargetUser> {
    let my_bot_id = match cache.get_bot_id() {
        Some(id) => Some(id),
        None => bot.get_me().await.map(|m| m.id).ok(),
    };

    let full_text = message.text().or_else(|| message.caption());
    let entities = message.entities().or_else(|| message.caption_entities());

    // ۱. بررسی پیام ریپلای‌شده
    if let Some(reply) = message.reply_to_message() {
        debug!(
            "بررسی ریپلای: msg_id={}, from={:?}, sender_chat={:?}",
            reply.id.0,
            reply.from.as_ref().map(|u| (u.id.0, &u.first_name)),
            reply.sender_chat.as_ref().map(|c| c.id.0)
        );

        if !matches!(reply.kind, MessageKind::ForumTopicCreated(_)) {
            // ۱-الف: پیام عضویت جدید در گروه (کاربر عضو شده هدف است، نه دعوت‌کننده)
            if let MessageKind::NewChatMembers(m) = &reply.kind {
                if let Some(first_user) = m.new_chat_members.first() {
                    let target = TargetUser {
                        id: first_user.id,
                        first_name: first_user.first_name.clone(),
                        username: first_user.username.clone(),
                    };
                    cache.record_user(&target);
                    return Some(target);
                }
            }

            // ۱-ب: پیام خروج عضو از گروه
            if let MessageKind::LeftChatMember(m) = &reply.kind {
                let target = TargetUser {
                    id: m.left_chat_member.id,
                    first_name: m.left_chat_member.first_name.clone(),
                    username: m.left_chat_member.username.clone(),
                };
                cache.record_user(&target);
                return Some(target);
            }

            // ۱-ج: پیام ارسالی از طرف کانال در سوپرگروه
            if let Some(chat) = reply.sender_chat.as_ref() {
                if reply.from.as_ref().map(|u| u.id.0 == 136817688).unwrap_or(true) {
                    return Some(TargetUser {
                        id: UserId(chat.id.0.unsigned_abs()),
                        first_name: chat.title().unwrap_or("کانال").to_string(),
                        username: chat.username().map(|s| s.to_string()),
                    });
                }
            }

            // ۱-د: پیام ارسالی از طرف کاربر عادی یا ربات
            if let Some(from) = reply.from.as_ref() {
                // اگر ریپلای روی پیام خود ربات باشد (مثلاً ریپلای به پیام اخطار یا میوت قبلی ربات)
                if let Some(bot_id) = my_bot_id {
                    if from.id == bot_id {
                        // استخراج اطلاعات کاربری که ربات درباره‌اش پیام فرستاده بوده
                        if let Some(extracted) = extract_target_from_bot_notice(bot, reply, pool, cache).await {
                            return Some(extracted);
                        }
                        // اگر پیامی که ریپلای شده پیام عمومی ربات بود، خارج نشو و به بررسی متن ادامه بده
                    } else {
                        let target = TargetUser {
                            id: from.id,
                            first_name: from.first_name.clone(),
                            username: from.username.clone(),
                        };
                        cache.record_user(&target);
                        return Some(target);
                    }
                } else {
                    let target = TargetUser {
                        id: from.id,
                        first_name: from.first_name.clone(),
                        username: from.username.clone(),
                    };
                    cache.record_user(&target);
                    return Some(target);
                }
            }

            // ۱-ه: پیام فوروارد شده
            if let Some(origin) = reply.forward_origin() {
                match origin {
                    MessageOrigin::User { ref sender_user, .. } => {
                        let target = TargetUser {
                            id: sender_user.id,
                            first_name: sender_user.first_name.clone(),
                            username: sender_user.username.clone(),
                        };
                        cache.record_user(&target);
                        return Some(target);
                    }
                    MessageOrigin::Channel { ref chat, .. } => {
                        return Some(TargetUser {
                            id: UserId(chat.id.0.unsigned_abs()),
                            first_name: chat.title().unwrap_or("کانال").to_string(),
                            username: chat.username().map(|s| s.to_string()),
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    // ۲. بررسی Telegram 7.0+ External Reply / Quotes
    if let MessageKind::Common(ref common) = message.kind {
        if let Some(ref ext) = common.external_reply {
            match &ext.origin {
                MessageOrigin::User { ref sender_user, .. } => {
                    let target = TargetUser {
                        id: sender_user.id,
                        first_name: sender_user.first_name.clone(),
                        username: sender_user.username.clone(),
                    };
                    cache.record_user(&target);
                    return Some(target);
                }
                MessageOrigin::Channel { ref chat, .. } => {
                    return Some(TargetUser {
                        id: UserId(chat.id.0.unsigned_abs()),
                        first_name: chat.title().unwrap_or("کانال").to_string(),
                        username: chat.username().map(|s| s.to_string()),
                    });
                }
                _ => {}
            }
        }
    }

    // ۳. بررسی منشن متنی تلگرام (TextMention)
    if let Some(ents) = entities {
        for entity in ents {
            if let MessageEntityKind::TextMention { ref user } = entity.kind {
                debug!("یافتن کاربر از TextMention: {} ({})", user.first_name, user.id.0);
                let target = TargetUser {
                    id: user.id,
                    first_name: user.first_name.clone(),
                    username: user.username.clone(),
                };
                cache.record_user(&target);
                return Some(target);
            }
        }
    }

    // ۴. بررسی منشن یوزرنیم (@username) از طریق Entity
    if let Some(text) = full_text {
        if let Some(ents) = entities {
            let utf16_units: Vec<u16> = text.encode_utf16().collect();
            for entity in ents {
                if let MessageEntityKind::Mention = entity.kind {
                    let start = entity.offset;
                    let end = start + entity.length;
                    if end <= utf16_units.len() {
                        let mention_slice = String::from_utf16_lossy(&utf16_units[start..end]);
                        let username = mention_slice.trim_start_matches('@');
                        if let Some(target) = resolve_username(bot, username, pool, cache).await {
                            return Some(target);
                        }
                    }
                }
            }
        }

        // ۵. بررسی یوزرنیم در کلمات متن پیام (حتی اگر تلگرام به عنوان Entity ثبت نکرده باشد)
        let words: Vec<&str> = text.split_whitespace().collect();
        for word in words {
            if word.starts_with('@') && word.len() > 1 {
                let clean = word
                    .trim_start_matches('@')
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if !clean.is_empty() {
                    if let Some(target) = resolve_username(bot, clean, pool, cache).await {
                        return Some(target);
                    }
                }
            }
        }

        // ۶. بررسی شناسه عددی در متن پیام (مثلاً: «میوت ۱۲۳۴۵۶۷۸ ۳۰ دقیقه»)
        for part in text.split_whitespace() {
            let normalized = crate::engine::normalizer::normalize_digits(part);
            if let Ok(num) = normalized.parse::<u64>() {
                if num > 1000 && !normalized.starts_with('/') {
                    // بررسی اینکه آیا قبلاً این شناسه در کش یا دیتابیس با نام مشخص ثبت شده است
                    if let Some(cached) = cache.get_user_by_id(num) {
                        return Some(cached);
                    }
                    if let Ok(Some(db_user)) = crate::db::users::find_user_by_id(pool, num as i64).await {
                        let target = TargetUser {
                            id: UserId(num),
                            first_name: db_user.first_name,
                            username: db_user.username,
                        };
                        cache.record_user(&target);
                        return Some(target);
                    }

                    debug!("یافتن شناسه عددی کاربر در متن: {}", num);
                    return Some(TargetUser {
                        id: UserId(num),
                        first_name: "کاربر".to_string(),
                        username: None,
                    });
                }
            }
        }
    }

    debug!("هیچ کاربری به عنوان هدف شناسایی نشد.");
    None
}

/// حل نام کاربری (@username) از کش، دیتابیس، یا دریافت اطلاعات چت از تلگرام
async fn resolve_username(
    bot: &Bot,
    username: &str,
    pool: &PgPool,
    cache: &AppCache,
) -> Option<TargetUser> {
    let clean = username.trim_start_matches('@');
    if clean.is_empty() {
        return None;
    }

    // ۱. بررسی کش در حافظه
    if let Some(cached) = cache.get_user_by_username(clean) {
        return Some(cached);
    }

    // ۲. بررسی دیتابیس کاربران شناخته‌شده
    if let Ok(Some(db_user)) = crate::db::users::find_user_by_username(pool, clean).await {
        let target = TargetUser {
            id: UserId(db_user.user_id as u64),
            first_name: db_user.first_name,
            username: db_user.username,
        };
        cache.record_user(&target);
        return Some(target);
    }

    // ۳. تلاش برای استعلام مستقیم از API تلگرام
    if let Ok(chat) = bot.get_chat(format!("@{}", clean)).await {
        let title_or_name = chat
            .title()
            .map(|t| t.to_string())
            .or_else(|| {
                chat.first_name().map(|f| match chat.last_name() {
                    Some(l) => format!("{} {}", f, l),
                    None => f.to_string(),
                })
            })
            .unwrap_or_else(|| clean.to_string());

        let target = TargetUser {
            id: UserId(chat.id.0.unsigned_abs()),
            first_name: title_or_name,
            username: chat.username().map(|u| u.to_string()),
        };
        cache.record_user(&target);
        let _ = crate::db::users::upsert_user(
            pool,
            chat.id.0,
            &target.first_name,
            target.username.as_deref(),
        )
        .await;
        return Some(target);
    }

    None
}

/// اگر مدیر روی پیام ارسالی خود ربات ریپلای کند، اطلاعات کاربری که موضوع پیام ربات بوده استخراج می‌شود
async fn extract_target_from_bot_notice(
    bot: &Bot,
    reply: &Message,
    pool: &PgPool,
    cache: &AppCache,
) -> Option<TargetUser> {
    let full_text = reply.text().or_else(|| reply.caption());
    let entities = reply.entities().or_else(|| reply.caption_entities());

    // ۱. بررسی TextMention در پیام اعلان
    if let Some(ents) = entities {
        for entity in ents {
            if let MessageEntityKind::TextMention { ref user } = entity.kind {
                let target = TargetUser {
                    id: user.id,
                    first_name: user.first_name.clone(),
                    username: user.username.clone(),
                };
                cache.record_user(&target);
                return Some(target);
            }
        }
    }

    // ۲. بررسی لینک‌های منشن tg://user?id=X در TextLink
    if let Some(ents) = entities {
        for entity in ents {
            if let MessageEntityKind::TextLink { ref url } = entity.kind {
                if url.as_str().starts_with("tg://user?id=") {
                    let id_str = url.as_str().trim_start_matches("tg://user?id=");
                    if let Ok(num) = id_str.parse::<u64>() {
                        if let Some(cached) = cache.get_user_by_id(num) {
                            return Some(cached);
                        }
                        if let Ok(Some(db_user)) = crate::db::users::find_user_by_id(pool, num as i64).await {
                            let target = TargetUser {
                                id: UserId(num),
                                first_name: db_user.first_name,
                                username: db_user.username,
                            };
                            cache.record_user(&target);
                            return Some(target);
                        }
                        return Some(TargetUser {
                            id: UserId(num),
                            first_name: "کاربر".to_string(),
                            username: None,
                        });
                    }
                }
            }
        }
    }

    // ۳. بررسی منشن @username از طریق Entity در پیام اعلان
    if let Some(text) = full_text {
        if let Some(ents) = entities {
            let utf16_units: Vec<u16> = text.encode_utf16().collect();
            for entity in ents {
                if let MessageEntityKind::Mention = entity.kind {
                    let start = entity.offset;
                    let end = start + entity.length;
                    if end <= utf16_units.len() {
                        let mention_slice = String::from_utf16_lossy(&utf16_units[start..end]);
                        let username = mention_slice.trim_start_matches('@');
                        if let Some(target) = resolve_username(bot, username, pool, cache).await {
                            return Some(target);
                        }
                    }
                }
            }
        }

        // ۴. بررسی متن پیام ریپلای‌شده برای یافتن الگوی tg://user?id=
        if let Some(pos) = text.find("tg://user?id=") {
            let rest = &text[pos + 13..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = digits.parse::<u64>() {
                if let Some(cached) = cache.get_user_by_id(num) {
                    return Some(cached);
                }
                if let Ok(Some(db_user)) = crate::db::users::find_user_by_id(pool, num as i64).await {
                    let target = TargetUser {
                        id: UserId(num),
                        first_name: db_user.first_name,
                        username: db_user.username,
                    };
                    cache.record_user(&target);
                    return Some(target);
                }
                return Some(TargetUser {
                    id: UserId(num),
                    first_name: "کاربر".to_string(),
                    username: None,
                });
            }
        }

        // ۵. جستجوی @username در کلمات اعلان
        for word in text.split_whitespace() {
            if word.starts_with('@') && word.len() > 1 {
                let clean = word
                    .trim_start_matches('@')
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if !clean.is_empty() {
                    let is_bot = cache.get_bot_id().map(|b| {
                        cache.get_user_by_username(clean).map(|u| u.id == b).unwrap_or(false)
                    }).unwrap_or(false);
                    if !is_bot {
                        if let Some(target) = resolve_username(bot, clean, pool, cache).await {
                            return Some(target);
                        }
                    }
                }
            }
        }

        // ۶. جستجوی شناسه عددی در متن اعلان (مثلاً: شناسه عددی: ۱۲۳۴۵۶)
        for part in text.split_whitespace() {
            let normalized = crate::engine::normalizer::normalize_digits(part);
            let digits: String = normalized.chars().filter(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = digits.parse::<u64>() {
                if num > 10000 {
                    let is_bot = cache.get_bot_id().map(|b| b.0 == num).unwrap_or(false);
                    if !is_bot {
                        if let Some(cached) = cache.get_user_by_id(num) {
                            return Some(cached);
                        }
                        if let Ok(Some(db_user)) = crate::db::users::find_user_by_id(pool, num as i64).await {
                            let target = TargetUser {
                                id: UserId(num),
                                first_name: db_user.first_name,
                                username: db_user.username,
                            };
                            cache.record_user(&target);
                            return Some(target);
                        }
                        return Some(TargetUser {
                            id: UserId(num),
                            first_name: "کاربر".to_string(),
                            username: None,
                        });
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_target_from_reply() {
        let json = r#"{
            "message_id": 100,
            "date": 1600000000,
            "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
            "from": {"id": 111, "is_bot": false, "first_name": "Admin"},
            "text": "میوت",
            "reply_to_message": {
                "message_id": 99,
                "date": 1600000000,
                "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
                "from": {"id": 222, "is_bot": false, "first_name": "Spammer"},
                "text": "hello"
            }
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        let target = resolve_target_sync(&msg);
        assert!(target.is_some());
        assert_eq!(target.unwrap().id.0, 222);
    }

    #[test]
    fn test_resolve_target_from_id_in_text() {
        let json = r#"{
            "message_id": 101,
            "date": 1600000000,
            "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
            "from": {"id": 111, "is_bot": false, "first_name": "Admin"},
            "text": "میوت ۸۲۱۱۹۱۱۵۱ ۲ ساعت"
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        let target = resolve_target_sync(&msg);
        assert!(target.is_some());
        assert_eq!(target.unwrap().id.0, 821191151);
    }

    #[test]
    fn test_resolve_target_from_photo_reply() {
        let json = r#"{
            "message_id": 102,
            "date": 1600000000,
            "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
            "from": {"id": 111, "is_bot": false, "first_name": "Admin"},
            "text": "بن",
            "reply_to_message": {
                "message_id": 98,
                "date": 1600000000,
                "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
                "from": {"id": 333, "is_bot": false, "first_name": "PhotoSender"},
                "photo": [
                    {"file_id": "p1", "file_unique_id": "u1", "width": 100, "height": 100}
                ]
            }
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        let target = resolve_target_sync(&msg).expect("target should be resolved");
        assert_eq!(target.id.0, 333);
        assert_eq!(target.first_name, "PhotoSender");
    }

    #[test]
    fn test_resolve_target_from_text_mention() {
        let json = r#"{
            "message_id": 103,
            "date": 1600000000,
            "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
            "from": {"id": 111, "is_bot": false, "first_name": "Admin"},
            "text": "اخطار کاربر",
            "entities": [
                {
                    "type": "text_mention",
                    "offset": 6,
                    "length": 5,
                    "user": {"id": 444, "is_bot": false, "first_name": "MentionedUser"}
                }
            ]
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        let target = resolve_target_sync(&msg).expect("target should be resolved");
        assert_eq!(target.id.0, 444);
        assert_eq!(target.first_name, "MentionedUser");
    }

    #[test]
    fn test_resolve_target_from_caption_id() {
        let json = r#"{
            "message_id": 104,
            "date": 1600000000,
            "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
            "from": {"id": 111, "is_bot": false, "first_name": "Admin"},
            "caption": "میوت ۸۲۱۱۹۱۱۵۱",
            "photo": [
                {"file_id": "p2", "file_unique_id": "u2", "width": 100, "height": 100}
            ]
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        let target = resolve_target_sync(&msg).expect("target should be resolved from caption");
        assert_eq!(target.id.0, 821191151);
    }

    #[test]
    fn test_resolve_target_from_join_service_message() {
        let json = r#"{
            "message_id": 105,
            "date": 1600000000,
            "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
            "from": {"id": 111, "is_bot": false, "first_name": "Admin"},
            "text": "بن",
            "reply_to_message": {
                "message_id": 95,
                "date": 1600000000,
                "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
                "from": {"id": 999, "is_bot": false, "first_name": "Inviter"},
                "new_chat_members": [
                    {"id": 555, "is_bot": false, "first_name": "NewSpammer", "username": "spambot"}
                ]
            }
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        let target = resolve_target_sync(&msg).expect("target should resolve new member");
        assert_eq!(target.id.0, 555);
        assert_eq!(target.first_name, "NewSpammer");
        assert_eq!(target.username.as_deref(), Some("spambot"));
    }

    #[test]
    fn test_resolve_target_from_channel_reply() {
        let json = r#"{
            "message_id": 106,
            "date": 1600000000,
            "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
            "from": {"id": 111, "is_bot": false, "first_name": "Admin"},
            "text": "بن",
            "reply_to_message": {
                "message_id": 94,
                "date": 1600000000,
                "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
                "from": {"id": 136817688, "is_bot": true, "first_name": "Telegram", "username": "Channel_Bot"},
                "sender_chat": {"id": -100998877, "type": "channel", "title": "AdChannel", "username": "adchannel"},
                "text": "Buy crypto"
            }
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        let target = resolve_target_sync(&msg).expect("target should resolve channel");
        assert_eq!(target.id.0, 100998877);
        assert_eq!(target.first_name, "AdChannel");
        assert_eq!(target.username.as_deref(), Some("adchannel"));
    }

    #[test]
    fn test_resolve_target_from_left_service_message() {
        let json = r#"{
            "message_id": 107,
            "date": 1600000000,
            "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
            "from": {"id": 111, "is_bot": false, "first_name": "Admin"},
            "text": "بن",
            "reply_to_message": {
                "message_id": 93,
                "date": 1600000000,
                "chat": {"id": -10012345, "type": "supergroup", "title": "Test"},
                "from": {"id": 111, "is_bot": false, "first_name": "Admin"},
                "left_chat_member": {
                    "id": 666,
                    "is_bot": false,
                    "first_name": "BadUser",
                    "username": "bad_user"
                }
            }
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        let target = resolve_target_sync(&msg).expect("target should resolve left chat member");
        assert_eq!(target.id.0, 666);
        assert_eq!(target.first_name, "BadUser");
        assert_eq!(target.username.as_deref(), Some("bad_user"));
    }
}
