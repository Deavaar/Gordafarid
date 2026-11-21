use std::time::Duration;
use chrono::Utc;
use chrono_tz::Tz;
use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::{ChatId, ChatPermissions, UserId};
use tracing::{error, info};

use crate::db::groups::get_all_night_mode_groups;
use crate::db::moderation::{deactivate_punishment, get_expired_punishments};
use crate::db::schedules::{get_due_schedules, update_schedule_after_run};
use crate::engine::normalizer::{to_persian_digits, RLM};
use crate::services::cache::AppCache;
use crate::ui::styled_keyboard::send_styled_message;

pub fn default_member_permissions() -> ChatPermissions {
    ChatPermissions::SEND_MESSAGES
        | ChatPermissions::SEND_AUDIOS
        | ChatPermissions::SEND_DOCUMENTS
        | ChatPermissions::SEND_PHOTOS
        | ChatPermissions::SEND_VIDEOS
        | ChatPermissions::SEND_VIDEO_NOTES
        | ChatPermissions::SEND_VOICE_NOTES
        | ChatPermissions::SEND_POLLS
        | ChatPermissions::SEND_OTHER_MESSAGES
        | ChatPermissions::ADD_WEB_PAGE_PREVIEWS
        | ChatPermissions::INVITE_USERS
}

pub fn start_background_scheduler(
    bot: Bot,
    pool: PgPool,
    cache: AppCache,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        info!("سرویس زمان‌بندی پس‌زمینه گردآفرید با موفقیت شروع به کار کرد.");
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            // 1. پردازش رفع مجازات‌های زمان‌دار منقضی‌شده (رفع میوت و رفع بن)
            if let Ok(expired) = get_expired_punishments(&pool).await {
                for pun in expired {
                    let chat = ChatId(pun.chat_id);
                    let user = UserId(pun.user_id as u64);

                    if pun.punishment_type == "mute" {
                        let perms = default_member_permissions();
                        if let Err(e) = bot.restrict_chat_member(chat, user, perms).await {
                            error!("خطا در رفع سکوت کاربر {}: {}", pun.user_id, e);
                        } else {
                            info!("سکوت کاربر {} در گروه {} به پایان رسید.", pun.user_id, pun.chat_id);
                        }
                    } else if pun.punishment_type == "ban" {
                        if let Err(e) = bot.unban_chat_member(chat, user).await {
                            error!("خطا در رفع مسدودیت کاربر {}: {}", pun.user_id, e);
                        } else {
                            info!("مسدودیت کاربر {} در گروه {} برطرف شد.", pun.user_id, pun.chat_id);
                        }
                    }

                    let _ = deactivate_punishment(&pool, pun.id).await;
                }
            }

            // 2. پردازش خودکار حالت شب برای گروه‌ها
            if let Ok(night_groups) = get_all_night_mode_groups(&pool).await {
                for g in night_groups {
                    let tz: Tz = g.night_mode_timezone.parse().unwrap_or(chrono_tz::Asia::Tehran);
                    let now_time = Utc::now().with_timezone(&tz).time();

                    let is_night = if g.night_mode_start <= g.night_mode_end {
                        now_time >= g.night_mode_start && now_time < g.night_mode_end
                    } else {
                        // عبور از نیمه‌شب (مثلاً ۲۳:۰۰ تا ۰۷:۰۰)
                        now_time >= g.night_mode_start || now_time < g.night_mode_end
                    };

                    let was_night = cache.is_night_mode_on(g.chat_id);

                    if is_night && !was_night {
                        // شروع حالت شب
                        cache.set_night_active(g.chat_id, true);
                        let notice = format!(
                            "{RLM}🌙 <b>قفل شبانه گروه فعال شد.</b>\nارسال پیام برای اعضای عادی تا ساعت <b>{}</b> قفل است.",
                            to_persian_digits(&g.night_mode_end.format("%H:%M").to_string())
                        );
                        let _ = send_styled_message(&bot, ChatId(g.chat_id), &notice, None).await;
                        info!("حالت شب در گروه {} فعال شد.", g.chat_id);
                    } else if !is_night && was_night {
                        // پایان حالت شب
                        cache.set_night_active(g.chat_id, false);
                        let notice = format!(
                            "{RLM}☀️ <b>صبح بخیر! قفل شبانه گروه پایان یافت.</b>\nارسال پیام برای تمام اعضا مجاز است."
                        );
                        let _ = send_styled_message(&bot, ChatId(g.chat_id), &notice, None).await;
                        info!("حالت شب در گروه {} به پایان رسید.", g.chat_id);
                    }
                }
            }

            // 3. ارسال پیام‌های زمان‌بندی‌شده
            if let Ok(due_schedules) = get_due_schedules(&pool).await {
                for sched in due_schedules {
                    let chat = ChatId(sched.chat_id);
                    if let Err(e) = send_styled_message(&bot, chat, &sched.message_text, None).await {
                        error!("خطا در ارسال پیام زمان‌بندی‌شده به گروه {}: {}", sched.chat_id, e);
                    } else {
                        info!("پیام زمان‌بندی‌شده شماره {} به گروه {} ارسال شد.", sched.id, sched.chat_id);
                    }

                    let _ = update_schedule_after_run(&pool, sched.id, sched.interval_secs).await;
                }
            }
        }
    })
}
