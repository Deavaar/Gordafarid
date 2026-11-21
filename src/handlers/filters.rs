use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::{ChatMemberStatus, ChatPermissions, Message, MessageEntityKind, Recipient, UserId};
use tracing::info;

use crate::db::filters::{get_allowed_domains, get_banned_words};
use crate::db::groups::GroupSettings;
use crate::engine::normalizer::{html_escape, to_persian_num, RLM};
use crate::engine::normalizer::normalize_persian_text;
use crate::services::cache::AppCache;
use crate::ui::styled_keyboard::send_styled_message;

pub fn is_channel_comment(msg: &Message) -> bool {
    // ۱. پیام داخل تاپیک یا ترد کامنت‌های پست کانال ارسال شده است
    if msg.thread_id.is_some() {
        return true;
    }

    // ۲. پیام ریپلای مستقیم به پست ارسال‌شده از کانال است
    if let Some(reply) = msg.reply_to_message() {
        if reply.is_automatic_forward() {
            return true;
        }
        if let Some(forward_chat) = reply.forward_from_chat() {
            if forward_chat.is_channel() {
                return true;
            }
        }
        if let Some(ref sender_chat) = reply.sender_chat {
            if sender_chat.is_channel() {
                return true;
            }
        }
        if let Some(ref u) = reply.from {
            if u.id.0 == 777000 || u.id.0 == 136817688 {
                return true;
            }
        }
    }

    false
}

pub fn get_comment_post_id(msg: &Message) -> Option<i32> {
    if let Some(thread_id) = msg.thread_id {
        return Some(thread_id.0 .0);
    }
    if let Some(reply) = msg.reply_to_message() {
        if reply.is_automatic_forward() {
            return Some(reply.id.0);
        }
        if let Some(ref sc) = reply.sender_chat {
            if sc.is_channel() {
                return Some(reply.id.0);
            }
        }
        if let Some(fc) = reply.forward_from_chat() {
            if fc.is_channel() {
                return Some(reply.id.0);
            }
        }
        if let Some(ref u) = reply.from {
            if u.id.0 == 777000 || u.id.0 == 136817688 {
                return Some(reply.id.0);
            }
        }
    }
    None
}

pub async fn inspect_message_filters(
    bot: &Bot,
    msg: &Message,
    pool: &PgPool,
    cache: &AppCache,
    settings: &GroupSettings,
    is_admin: bool,
) -> bool {
    // مدیران از فیلترهای ضداسپم، قفل شب و محتوا معاف هستند
    if is_admin {
        return false;
    }

    // پیام‌های فوروارد خودکار کانال رسمی متصل به گروه یا ارسال‌شده از خود کانال هرگز حذف نشوند
    if msg.is_automatic_forward() {
        return false;
    }
    if let Some(ref sc) = msg.sender_chat {
        if sc.is_channel() {
            return false;
        }
    }

    let chat_id = msg.chat.id;
    let sender_user = match msg.from.as_ref() {
        Some(u) => u,
        None => return false,
    };
    let user_id = sender_user.id.0;

    // ۰. بررسی قفل کلی گروه (قفل سراسری یا فقط کامنت آزاد)
    match settings.group_lock_mode.as_str() {
        "total" => {
            let _ = bot.delete_message(chat_id, msg.id).await;
            return true;
        }
        "comments_only" => {
            if !is_channel_comment(msg) {
                let _ = bot.delete_message(chat_id, msg.id).await;
                return true;
            }
        }
        _ => {}
    }

    // ۰.۱ بررسی رهگیری کانال و فیلتر نوع محتوای کامنت‌ها بر اساس پست متصل‌شده
    if let Some(post_id) = get_comment_post_id(msg) {
        let allowed_opt = cache.get_post_rule(chat_id.0, post_id);
        let allowed_type = match allowed_opt {
            Some(a) => Some(a),
            None => {
                if let Ok(Some(a)) = crate::db::groups::get_channel_post_rule(pool, chat_id.0, post_id).await {
                    cache.set_post_rule(chat_id.0, post_id, a.clone());
                    Some(a)
                } else {
                    None
                }
            }
        };

        if let Some(allowed) = allowed_type {
            let matches_allowed = crate::engine::channel_tracker::is_content_allowed_by_rule(&allowed, msg);

            if !matches_allowed {
                let _ = bot.delete_message(chat_id, msg.id).await;
                return true;
            }
        }
    }

    // ۱. بررسی قفل شبانه (Night Mode)
    if cache.is_night_mode_on(chat_id.0) {
        let _ = bot.delete_message(chat_id, msg.id).await;
        return true;
    }

    // ۲. بررسی عضویت اجباری در کانال (Forced Channel Membership با کشینگ دوگانه برگرفته از force-join)
    if let Some(ref channel_username) = settings.forced_channel {
        let is_cached = cache.get_channel_membership(chat_id.0, user_id);
        let must_block = match is_cached {
            Some(true) => false,
            Some(false) => true,
            None => {
                let clean_ch = channel_username.trim()
                    .trim_start_matches("https://t.me/")
                    .trim_start_matches("http://t.me/")
                    .trim_start_matches("t.me/")
                    .trim_start_matches('@');
                let formatted = format!("@{}", clean_ch);

                match bot.get_chat_member(Recipient::ChannelUsername(formatted.clone()), UserId(user_id)).await {
                    Ok(member) => {
                        let joined = !matches!(member.status(), ChatMemberStatus::Left | ChatMemberStatus::Banned);
                        cache.set_channel_membership(chat_id.0, user_id, joined);
                        !joined
                    }
                    Err(e) => {
                        tracing::warn!("عدم امکان استعلام عضویت در کانال {}: {}", formatted, e);
                        false
                    }
                }
            }
        };

        if must_block {
            let _ = bot.delete_message(chat_id, msg.id).await;
            let channel_url = if channel_username.starts_with("http://") || channel_username.starts_with("https://") {
                channel_username.clone()
            } else {
                format!("https://t.me/{}", channel_username.trim_start_matches('@'))
            };
            let kb = crate::ui::keyboards::force_join_styled(&channel_url);
            let notice = format!(
                "{RLM}⚠️ <a href=\"tg://user?id={}\">{}</a> عزیز، برای ارسال پیام در گروه ابتدا باید در کانال <b>{}</b> عضو شوید.",
                user_id,
                html_escape(&sender_user.first_name),
                html_escape(channel_username)
            );
            let _ = send_styled_message(bot, chat_id, &notice, Some(&kb)).await;
            return true;
        }
    }

    // ۳. بررسی ضداسپم / ارسال رگباری (Anti-Flood)
    if settings.anti_flood_enabled {
        let is_flooding = cache.check_flood(
            chat_id.0,
            user_id,
            settings.anti_flood_rate as usize,
            settings.anti_flood_window_secs as u64,
        );

        if is_flooding {
            let _ = bot.delete_message(chat_id, msg.id).await;
            match settings.anti_flood_action.as_str() {
                "ban" => {
                    let _ = bot.ban_chat_member(chat_id, sender_user.id).await;
                    if !settings.silent_punishment {
                        let notice = format!(
                            "{RLM}⛔ کاربر <a href=\"tg://user?id={}\">{}</a> به دلیل ارسال پیام رگباری (اسپم) مسدود شد.",
                            user_id,
                            html_escape(&sender_user.first_name)
                        );
                        let _ = send_styled_message(bot, chat_id, &notice, None).await;
                    }
                }
                "kick" => {
                    let _ = bot.ban_chat_member(chat_id, sender_user.id).await;
                    let _ = bot.unban_chat_member(chat_id, sender_user.id).await;
                    if !settings.silent_punishment {
                        let notice = format!(
                            "{RLM}🚪 کاربر <a href=\"tg://user?id={}\">{}</a> به دلیل ارسال پیام رگباری اخراج شد.",
                            user_id,
                            html_escape(&sender_user.first_name)
                        );
                        let _ = send_styled_message(bot, chat_id, &notice, None).await;
                    }
                }
                "warn" => {
                    let count = crate::db::moderation::add_warning(pool, chat_id.0, user_id as i64, "ارسال پیام رگباری (اسپم)", 0).await.unwrap_or(1);
                    if !settings.silent_punishment {
                        let notice = format!(
                            "{RLM}⚠️ اخطار به <a href=\"tg://user?id={}\">{}</a>:\nارسال پیام رگباری (اخطار {} از {})",
                            user_id,
                            html_escape(&sender_user.first_name),
                            to_persian_num(count),
                            to_persian_num(settings.warn_limit)
                        );
                        let _ = send_styled_message(bot, chat_id, &notice, None).await;
                    }
                }
                "mute_1h" => {
                    let empty_perms = ChatPermissions::empty();
                    let _ = bot.restrict_chat_member(chat_id, sender_user.id, empty_perms).await;
                    let expires = chrono::Utc::now() + chrono::Duration::hours(1);
                    let _ = crate::db::moderation::add_temp_punishment(pool, chat_id.0, user_id as i64, "mute", expires, Some("اسپم"), 0).await;
                    if !settings.silent_punishment {
                        let notice = format!(
                            "{RLM}🔇 کاربر <a href=\"tg://user?id={}\">{}</a> به دلیل ارسال رگباری پیام به مدت ۱ ساعت در وضعیت سکوت قرار گرفت.",
                            user_id,
                            html_escape(&sender_user.first_name)
                        );
                        let _ = send_styled_message(bot, chat_id, &notice, None).await;
                    }
                }
                _ => {
                    let empty_perms = ChatPermissions::empty();
                    let _ = bot.restrict_chat_member(chat_id, sender_user.id, empty_perms).await;
                    let expires = chrono::Utc::now() + chrono::Duration::minutes(5);
                    let _ = crate::db::moderation::add_temp_punishment(pool, chat_id.0, user_id as i64, "mute", expires, Some("اسپم"), 0).await;
                    if !settings.silent_punishment {
                        let notice = format!(
                            "{RLM}⚠️ کاربر <a href=\"tg://user?id={}\">{}</a> به دلیل ارسال رگباری پیام به مدت ۵ دقیقه در وضعیت سکوت قرار گرفت.",
                            user_id,
                            html_escape(&sender_user.first_name)
                        );
                        let _ = send_styled_message(bot, chat_id, &notice, None).await;
                    }
                }
            }
            info!("کاربر {} به دلیل فلاد در گروه {} مجازات شد (اقدام: {}).", user_id, chat_id.0, settings.anti_flood_action);
            return true;
        }
    }

    // ۴. بررسی فوروارد
    if settings.forward_filter_enabled && (msg.forward_date().is_some() || msg.forward_origin().is_some()) {
        let _ = bot.delete_message(chat_id, msg.id).await;
        return true;
    }

    // ۵. بررسی استیکر
    if settings.sticker_filter_enabled && msg.sticker().is_some() {
        let _ = bot.delete_message(chat_id, msg.id).await;
        return true;
    }

    // ۶. بررسی عکس
    if settings.photo_filter_enabled && msg.photo().is_some() {
        let _ = bot.delete_message(chat_id, msg.id).await;
        return true;
    }

    // ۷. بررسی ویدیو و ویدیو مسیج
    if settings.video_filter_enabled && (msg.video().is_some() || msg.video_note().is_some()) {
        let _ = bot.delete_message(chat_id, msg.id).await;
        return true;
    }

    // ۸. بررسی وویس و فایل صوتی
    if settings.voice_filter_enabled && (msg.voice().is_some() || msg.audio().is_some()) {
        let _ = bot.delete_message(chat_id, msg.id).await;
        return true;
    }

    // ۹. بررسی فایل و اسناد
    if settings.document_filter_enabled && msg.document().is_some() {
        let _ = bot.delete_message(chat_id, msg.id).await;
        return true;
    }

    // ۱۰. بررسی گیف و انیمیشن
    if settings.gif_filter_enabled && msg.animation().is_some() {
        let _ = bot.delete_message(chat_id, msg.id).await;
        return true;
    }

    // ۱۱. بررسی رسانه‌ها (عکس، ویدیو، وویس، فایل، گیف)
    if settings.media_filter_enabled {
        let is_media = msg.photo().is_some()
            || msg.video().is_some()
            || msg.voice().is_some()
            || msg.audio().is_some()
            || msg.document().is_some()
            || msg.animation().is_some()
            || msg.video_note().is_some();

        if is_media {
            let _ = bot.delete_message(chat_id, msg.id).await;
            return true;
        }
    }

    // ۶.۵ بررسی هوشمند ریسک لینک‌ها، کلاهبرداری و کوتاه‌کننده‌ها (Spam Gate Scorer)
    let raw_msg_text = msg.text().unwrap_or_else(|| msg.caption().unwrap_or(""));
    if !raw_msg_text.is_empty() {
        let spam_eval = crate::engine::spam_scorer::evaluate_message_spam_risk(raw_msg_text);
        if spam_eval.is_spam {
            let _ = bot.delete_message(chat_id, msg.id).await;
            if !settings.silent_punishment {
                let reason_str = spam_eval.reasons.join("، ");
                let notice = format!(
                    "{RLM}⛔ پیام <a href=\"tg://user?id={}\">{}</a> به دلیل شناسایی محتوای پرخطر یا مشکوک به کلاهبرداری حذف شد.\n🔍 علت: {}",
                    user_id,
                    html_escape(&sender_user.first_name),
                    html_escape(&reason_str)
                );
                let _ = send_styled_message(bot, chat_id, &notice, None).await;
            }
            return true;
        }
    }

    // ۷. بررسی لینک و تبلیغات
    if settings.link_filter_enabled {
        let mut has_link = false;
        let all_entities = msg.entities().or_else(|| msg.caption_entities());
        if let Some(entities) = all_entities {
            for entity in entities {
                match entity.kind {
                    MessageEntityKind::Url | MessageEntityKind::TextLink { .. } => {
                        has_link = true;
                        break;
                    }
                    _ => {}
                }
            }
        }

        let full_text = msg.text().unwrap_or_else(|| msg.caption().unwrap_or(""));
        if full_text.contains("t.me/") || full_text.contains("http://") || full_text.contains("https://") {
            has_link = true;
        }

        if has_link {
            let allowed = get_allowed_domains(pool, chat_id.0).await.unwrap_or_default();
            let mut is_allowed = false;
            for domain in &allowed {
                if full_text.contains(domain) {
                    is_allowed = true;
                    break;
                }
            }

            if !is_allowed {
                let _ = bot.delete_message(chat_id, msg.id).await;
                match settings.link_filter_action.as_str() {
                    "warn" => {
                        let count = crate::db::moderation::add_warning(pool, chat_id.0, user_id as i64, "ارسال لینک غیرمجاز", 0).await.unwrap_or(1);
                        if !settings.silent_punishment {
                            let notice = format!(
                                "{RLM}⚠️ <a href=\"tg://user?id={}\">{}</a>، ارسال لینک در این گروه مجاز نیست! (اخطار {} از {})",
                                user_id,
                                html_escape(&sender_user.first_name),
                                to_persian_num(count),
                                to_persian_num(settings.warn_limit)
                            );
                            let _ = send_styled_message(bot, chat_id, &notice, None).await;
                        }
                    }
                    "mute" => {
                        let empty_perms = ChatPermissions::empty();
                        let _ = bot.restrict_chat_member(chat_id, sender_user.id, empty_perms).await;
                        let expires = chrono::Utc::now() + chrono::Duration::hours(1);
                        let _ = crate::db::moderation::add_temp_punishment(pool, chat_id.0, user_id as i64, "mute", expires, Some("ارسال لینک غیرمجاز"), 0).await;
                        if !settings.silent_punishment {
                            let notice = format!(
                                "{RLM}🔇 کاربر <a href=\"tg://user?id={}\">{}</a> به دلیل ارسال لینک به مدت ۱ ساعت در وضعیت سکوت قرار گرفت.",
                                user_id,
                                html_escape(&sender_user.first_name)
                            );
                            let _ = send_styled_message(bot, chat_id, &notice, None).await;
                        }
                    }
                    _ => {}
                }
                return true;
            }
        }
    }

    // ۸. بررسی کلمات ممنوعه
    let text_to_check = msg.text().or_else(|| msg.caption());
    if let Some(text) = text_to_check {
        let normalized = normalize_persian_text(text);
        if let Ok(banned_list) = get_banned_words(pool, chat_id.0).await {
            for (word, list_type) in banned_list {
                let norm_word = normalize_persian_text(&word);
                if normalized.contains(&norm_word) {
                    let _ = bot.delete_message(chat_id, msg.id).await;
                    if list_type == 2 {
                        let notice = format!("{RLM}⚠️ ارسال پیام حاوی عبارات غیرمجاز ممنوع است.");
                        let _ = send_styled_message(bot, chat_id, &notice, None).await;
                    }
                    return true;
                }
            }
        }
    }

    false
}
