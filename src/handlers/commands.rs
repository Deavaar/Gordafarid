use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::{ChatMemberStatus, Message, MessageKind, Recipient};
use tracing::{info, warn};

use crate::db::custom_commands::*;
use crate::db::filters::*;
use crate::db::groups::*;
use crate::db::moderation::{clear_warnings, deactivate_user_punishments, get_warnings_count};
use crate::engine::parser::{parse_command, ParsedCommand};
use crate::engine::permissions::check_sender_is_admin_cached;
use crate::engine::target::{resolve_target, TargetUser};
use crate::errors::AppError;
use crate::handlers::moderation::*;
use crate::engine::normalizer::{html_escape, to_persian_digits, to_persian_num, RLM};
use crate::services::cache::AppCache;
use crate::ui::keyboards::{group_lock_styled, pv_admin_list_styled, pv_group_selection_styled, pv_owner_reply_keyboard, settings_main_styled, start_private_styled, SettingsCounts};
use crate::ui::messages::{help_message, start_private_message};
use crate::ui::styled_keyboard::{send_styled_message, StyledButton, StyledKeyboard};

pub async fn handle_message(
    bot: Bot,
    msg: Message,
    pool: PgPool,
    cache: AppCache,
    owner_id: i64,
) -> Result<(), teloxide::RequestError> {
    let chat = &msg.chat;

    // ۱. پیام‌های خصوصی (PV)
    if chat.is_private() {
        let sender = match msg.from.as_ref() {
            Some(s) => s,
            None => return Ok(()),
        };

        if let Some(text) = msg.text() {
            let clean = text.trim();
            if clean.starts_with("/start") || clean == "شروع" {
                let me = bot.get_me().await.ok();
                let username = me.as_ref().and_then(|u| u.username.clone()).unwrap_or_else(|| "Gordafarid_robot".to_string());
                let kb = start_private_styled(&username);
                let _ = send_styled_message(&bot, chat.id, &start_private_message(), Some(&kb)).await;
            } else if clean.starts_with("/help") || clean == "راهنما" {
                let _ = send_styled_message(&bot, chat.id, &help_message(), None).await;

            } else if clean == "پنل" || clean == "/panel" || clean == "!پنل" || clean == "تنظیمات" || clean == "🛡️ پنل مدیریت" {
                let groups = resolve_user_admin_groups(&bot, &pool, sender.id.0).await.unwrap_or_default();
                if groups.is_empty() {
                    let text = crate::i18n::get_i18n().pv.no_admin_groups.replace("{RLM}", RLM);
                    let _ = send_styled_message(&bot, chat.id, &text, None).await;
                } else {
                    let prompt = crate::i18n::get_i18n().pv.select_group_panel.replace("{RLM}", RLM);
                    let kb = pv_group_selection_styled(&groups, "pv_grp_sel");
                    let _ = send_styled_message(&bot, chat.id, &prompt, Some(&kb)).await;
                }
            } else if clean == "دسترسی ادمین‌ها" || clean == "👑 دسترسی ادمین‌ها" || clean == "/admin_perms" {
                let groups = resolve_user_strictly_owned_groups(&bot, &pool, sender.id.0).await.unwrap_or_default();
                if groups.is_empty() {
                    let text = crate::i18n::get_i18n().pv.no_owned_groups.replace("{RLM}", RLM);
                    let _ = send_styled_message(&bot, chat.id, &text, None).await;
                } else if groups.len() == 1 {
                    let (target_chat_id, target_title) = &groups[0];
                    match bot.get_chat_administrators(teloxide::types::ChatId(*target_chat_id)).await {
                        Ok(admins) => {
                            let kb = pv_admin_list_styled(*target_chat_id, &admins);
                            let prompt = format!("{RLM}👑 <b>تنظیم دسترسی ادمین‌های گروه {}:</b>\nادمین مورد نظر یا گزینه سراسری را انتخاب کنید:", html_escape(target_title));
                            let _ = send_styled_message(&bot, chat.id, &prompt, Some(&kb)).await;
                        }
                        Err(e) => {
                            let err_msg = format!("{RLM}❌ خطا در دریافت لیست ادمین‌های گروه: {}", e);
                            let _ = send_styled_message(&bot, chat.id, &err_msg, None).await;
                        }
                    }
                } else {
                    let prompt = crate::i18n::get_i18n().pv.select_group_admin_perms.replace("{RLM}", RLM);
                    let kb = pv_group_selection_styled(&groups, "pv_ap_grp");
                    let _ = send_styled_message(&bot, chat.id, &prompt, Some(&kb)).await;
                }
            }
        }
        return Ok(());
    }

    // ۲. گروه‌ها و سوپرگروه‌ها
    if !chat.is_group() && !chat.is_supergroup() {
        return Ok(());
    }

    let chat_title = chat.title().unwrap_or("گروه");
    let settings = match get_or_create_group(&pool, chat.id.0, chat_title).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("خطا در دریافت اطلاعات گروه: {}", e);
            return Ok(());
        }
    };

    // ۳. بررسی ورود و خروج اعضا
    if let Some(new_members) = msg.new_chat_members() {
        if let Ok(me) = bot.get_me().await {
            if new_members.iter().any(|m| m.id == me.id) {
                if let Some(sender) = msg.from.as_ref() {
                    let _ = update_group_added_by(&pool, chat.id.0, sender.id.0 as i64).await;
                    info!("🤖 ربات توسط کاربر {} ({}) به گروه {} افزوده شد.", sender.id.0, sender.first_name, chat.id.0);
                }
            }
        }
        crate::handlers::join_gate::handle_new_chat_members(&bot, &msg, &pool, &cache, &settings, new_members).await;
        return Ok(());
    }
    if let Some(left_user) = msg.left_chat_member() {
        crate::handlers::join_gate::handle_left_chat_member(&bot, &msg, &settings, left_user).await;
        return Ok(());
    }

    // ۳.۱. بررسی و حذف بلادرنگ کلیه اعلان‌های سیستمی تلگرام و نظرسنجی‌ها در صورت فعال بودن فیلتر
    if settings.service_msg_filter_enabled {
        let is_service_or_poll = match &msg.kind {
            MessageKind::NewChatTitle(_)
            | MessageKind::NewChatPhoto(_)
            | MessageKind::DeleteChatPhoto(_)
            | MessageKind::GroupChatCreated(_)
            | MessageKind::SupergroupChatCreated(_)
            | MessageKind::ChannelChatCreated(_)
            | MessageKind::MessageAutoDeleteTimerChanged(_)
            | MessageKind::Pinned(_)
            | MessageKind::ChatShared(_)
            | MessageKind::UsersShared(_)
            | MessageKind::ConnectedWebsite(_)
            | MessageKind::WriteAccessAllowed(_)
            | MessageKind::PassportData(_)
            | MessageKind::ProximityAlertTriggered(_)
            | MessageKind::ForumTopicCreated(_)
            | MessageKind::ForumTopicEdited(_)
            | MessageKind::ForumTopicClosed(_)
            | MessageKind::ForumTopicReopened(_)
            | MessageKind::GeneralForumTopicHidden(_)
            | MessageKind::GeneralForumTopicUnhidden(_)
            | MessageKind::Giveaway(_)
            | MessageKind::GiveawayCompleted(_)
            | MessageKind::GiveawayCreated(_)
            | MessageKind::GiveawayWinners(_)
            | MessageKind::VideoChatScheduled(_)
            | MessageKind::VideoChatStarted(_)
            | MessageKind::VideoChatEnded(_)
            | MessageKind::VideoChatParticipantsInvited(_)
            | MessageKind::WebAppData(_)
            | MessageKind::Dice(_) => true,
            _ => msg.poll().is_some(),
        };

        if is_service_or_poll {
            info!("🗑️ حذف بلادرنگ اعلان تلگرام یا نظرسنجی (شناسه {}) در گروه {}", msg.id.0, chat.id.0);
            let _ = bot.delete_message(chat.id, msg.id).await;
            return Ok(());
        }
    }

    // بررسی پیام‌های ارسالی از کانال متصل‌شده و رهگیری دستورات مستقیم و هوشمند کامنت‌ها
    let is_channel_post = msg.is_automatic_forward()
        || msg.sender_chat.as_ref().map(|c| c.is_channel()).unwrap_or(false)
        || msg.forward_from_chat().map(|c| c.is_channel()).unwrap_or(false)
        || msg.from.as_ref().map(|u| u.id.0 == 777000 || u.id.0 == 136817688).unwrap_or(false);

    if is_channel_post {
        let post_text = msg.text().or_else(|| msg.caption()).unwrap_or("");
        let mut resolved_rule: Option<String> = None;

        // ۱. بررسی کش قانون ذخیره‌شده از رویداد channel_post کانال
        if let (Some(orig_ch), Some(orig_mid)) = (msg.forward_from_chat(), msg.forward_from_message_id()) {
            if let Some(r) = cache.get_post_rule(orig_ch.id.0, orig_mid.0) {
                resolved_rule = Some(r);
            } else if let Ok(Some(r)) = crate::db::groups::get_channel_post_rule(&pool, orig_ch.id.0, orig_mid.0).await {
                resolved_rule = Some(r);
            }
        }

        // ۲. بررسی وجود دستور مستقیم ادمین در متن پیام فورواردشده (مانند: "تنظیم گیف" یا "تنظیم همه بجز فیلم")
        if resolved_rule.is_none() {
            if let Some(directive) = crate::engine::channel_tracker::extract_explicit_directive(post_text) {
                resolved_rule = Some(directive.rule.to_string());

                // تلاش برای ویرایش و حذف عبارت دستور از پست کانال در صورتی که ربات در کانال ادمین باشد
                if let (Some(orig_ch), Some(orig_mid)) = (msg.forward_from_chat(), msg.forward_from_message_id()) {
                    let cleaned = directive.cleaned_text.trim();
                    if msg.text().is_some() {
                        let new_text = if cleaned.is_empty() {
                            crate::engine::channel_tracker::directive_fallback_notice(directive.rule)
                        } else {
                            cleaned.to_string()
                        };
                        let _ = bot.edit_message_text(orig_ch.id, orig_mid, new_text).await;
                    } else if msg.caption().is_some() {
                        let _ = bot.edit_message_caption(orig_ch.id, orig_mid).caption(cleaned).await;
                    }
                }
            }
        }

        // ۳. در صورت نبود دستور مستقیم، ارزیابی سیستم هوشمند رهگیری محتوا (NLP CTA)
        if resolved_rule.is_none() && settings.channel_track_enabled {
            if let Some(allowed_type) = crate::engine::channel_tracker::detect_channel_post_cta(post_text, &settings) {
                resolved_rule = Some(allowed_type.to_string());
            }
        }

        // ذخیره قانون برای پیام پست در گروه جهت اعمال بر روی کامنت‌ها
        if let Some(rule) = resolved_rule {
            cache.set_post_rule(chat.id.0, msg.id.0, rule.clone());
            let _ = crate::db::groups::save_channel_post_rule(&pool, chat.id.0, msg.id.0, &rule).await;
            info!("📡 قانون کامنت برای پست {} در گروه {} ثبت شد: '{}'", msg.id.0, chat.id.0, rule);
        }

        return Ok(());
    }

    let sender = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    // ذخیره فرستنده در کش و دیتابیس برای امکان منشن با یوزرنیم (@username) در دستورات بعدی
    let sender_target = TargetUser {
        id: sender.id,
        first_name: sender.first_name.clone(),
        username: sender.username.clone(),
    };
    cache.record_user(&sender_target);
    let _ = crate::db::users::upsert_user(
        &pool,
        sender.id.0 as i64,
        &sender.first_name,
        sender.username.as_deref(),
    )
    .await;

    let is_owner = crate::db::groups::check_is_group_owner(&bot, &pool, chat.id.0, sender.id.0).await;
    let is_admin = if is_owner {
        true
    } else {
        check_sender_is_admin_cached(&bot, &cache, chat.id, sender.id, owner_id)
            .await
            .unwrap_or(false)
    };

    // ثبت لاگ جامع دریافت پیام در گروه
    info!(
        "📨 [گروه {}] [فرستنده {} (ادمین: {})] متن: {:?}, کپشن: {:?}, ریپلای: {:?}",
        chat.id,
        sender.id,
        is_admin,
        msg.text(),
        msg.caption(),
        msg.reply_to_message().map(|r| r.id.0)
    );

    // ۴. بررسی فیلترهای پیام (قفل شب، ضداسپم، جوین اجباری، لینک، کلمات ممنوعه)
    let filtered = crate::handlers::filters::inspect_message_filters(
        &bot,
        &msg,
        &pool,
        &cache,
        &settings,
        is_admin,
    )
    .await;

    if filtered {
        return Ok(());
    }

    let raw_text = match msg.text().or_else(|| msg.caption()) {
        Some(t) => t,
        None => return Ok(()),
    };

    let cmd = parse_command(raw_text);

    // ۵. اجرای دستورات شناسایی‌شده
    let result: Result<(), AppError> = match cmd {
        ParsedCommand::Start => {
            let _ = send_styled_message(&bot, chat.id, &format!("{RLM}🌸 ربات گردآفرید در این گروه فعال و آماده به کار است."), None).await;
            Ok(())
        }
        ParsedCommand::Help => {
            let _ = send_styled_message(&bot, chat.id, &help_message(), None).await;
            Ok(())
        }
        ParsedCommand::Rules => {
            let rules = if settings.rules_text.is_empty() {
                "قوانین برای این گروه هنوز ثبت نشده است."
            } else {
                &settings.rules_text
            };
            let text = format!("{RLM}📜 <b>قوانین گروه:</b>\n\n<details open><summary>متن رسمی قوانین</summary><p>{}</p></details>", html_escape(rules));
            let _ = send_styled_message(&bot, chat.id, &text, None).await;
            Ok(())
        }
        ParsedCommand::Settings => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let words = get_banned_words(&pool, chat.id.0).await.map(|w| w.len()).unwrap_or(0);
                let domains = get_allowed_domains(&pool, chat.id.0).await.map(|d| d.len()).unwrap_or(0);
                let custom_cmds = list_custom_commands(&pool, chat.id.0).await.map(|c| c.len()).unwrap_or(0);
                let counts = SettingsCounts { words, domains, custom_cmds };

                let kb = settings_main_styled(&settings, &counts);
                let text = crate::i18n::get_i18n().settings.prompt.replace("{RLM}", RLM);
                let _ = send_styled_message(&bot, chat.id, &text, Some(&kb)).await;
                Ok(())
            }
        }
        ParsedCommand::Stats | ParsedCommand::Status => {
            let jalali_time = crate::engine::jalali::jalali_persian_stamp();
            let log_topic_status = match settings.log_topic_id {
                Some(tid) => format!("فعال 🟢 (شناسه تاپیک: {})", to_persian_num(tid)),
                None => "تنظیم نشده ⚪".to_string(),
            };
            let lock_mode_display = match settings.group_lock_mode.as_str() {
                "total" => "قفل سراسری 🔒",
                "comments_only" => "فقط کامنت آزاد 💬",
                _ => "آزاد (عادی) 🔓",
            };

            let status_text = format!(
                "{RLM}📊 <b>داشبورد وضعیت حفاظتی گروه:</b> {}\n\
                📅 <b>زمان خورشیدی:</b> <code>{}</code>\n\n\
                <table bordered striped compact>\n\
                  <caption align=\"center\">🛡️ <b>وضعیت قفل‌ها و فیلترهای امنیتی</b></caption>\n\
                  <tr><th align=\"center\">قفل و فیلتر</th><th align=\"center\">وضعیت</th></tr>\n\
                  <tr><td align=\"center\">سیستم ضداسپم</td><td align=\"center\">{} ({} در {} ثانیه)</td></tr>\n\
                  <tr><td align=\"center\">فیلتر لینک و تبلیغات</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">فیلتر فوروارد پیام</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">فیلتر استیکر</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">فیلتر کلی رسانه</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">قفل شبانه</td><td align=\"center\">{} ({} تا {})</td></tr>\n\
                  <tr><td align=\"center\">تایید عضویت (کپچا)</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">سقف اخطارهای مجاز</td><td align=\"center\">{} اخطار</td></tr>\n\
                  <tr><td align=\"center\">کانال عضویت اجباری</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">تاپیک ثبت لاگ‌ها</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">وضعیت قفل گروه</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">پیام خوشامدگویی</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">پیام خروج اعضا</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">مجازات بی‌صدا</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">اعلان‌های تلگرام</td><td align=\"center\">{}</td></tr>\n\
                </table>",
                html_escape(&chat_title),
                jalali_time,
                if settings.anti_flood_enabled { "فعال 🟢" } else { "غیرفعال 🔴" },
                to_persian_num(settings.anti_flood_rate),
                to_persian_num(settings.anti_flood_window_secs),
                if settings.link_filter_enabled { "فعال 🟢" } else { "غیرفعال 🔴" },
                if settings.forward_filter_enabled { "فعال 🟢" } else { "غیرفعال 🔴" },
                if settings.sticker_filter_enabled { "فعال 🟢" } else { "غیرفعال 🔴" },
                if settings.media_filter_enabled { "فعال 🟢" } else { "غیرفعال 🔴" },
                if settings.night_mode_enabled { "فعال 🟢" } else { "غیرفعال 🔴" },
                to_persian_digits(&settings.night_mode_start.format("%H:%M").to_string()),
                to_persian_digits(&settings.night_mode_end.format("%H:%M").to_string()),
                if settings.captcha_enabled { "فعال 🟢" } else { "غیرفعال 🔴" },
                to_persian_num(settings.warn_limit),
                settings.forced_channel.as_deref().map(|c| html_escape(c)).unwrap_or_else(|| "تنظیم نشده ⚪".to_string()),
                log_topic_status,
                lock_mode_display,
                if settings.welcome_enabled { "فعال 🟢" } else { "غیرفعال 🔴" },
                if settings.goodbye_enabled { "فعال 🟢" } else { "غیرفعال 🔴" },
                if settings.silent_punishment { "فعال 🟢" } else { "غیرفعال 🔴" },
                if settings.service_msg_filter_enabled { "حذف خودکار 🟢" } else { "غیرفعال 🔴" },
            );
            let _ = send_styled_message(&bot, chat.id, &status_text, None).await;
            Ok(())
        }
        ParsedCommand::Ban { duration, reason } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                handle_ban(&bot, &msg, &pool, &cache, &settings, duration, reason, owner_id).await
            }
        }
        ParsedCommand::Kick { reason } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                handle_kick(&bot, &msg, &pool, &cache, &settings, reason, owner_id).await
            }
        }
        ParsedCommand::Mute { duration, reason } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                handle_mute(&bot, &msg, &pool, &cache, &settings, duration, reason, owner_id).await
            }
        }
        ParsedCommand::Unban => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                handle_unban(&bot, &msg, &pool, &cache).await
            }
        }
        ParsedCommand::Unmute => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                handle_unmute(&bot, &msg, &pool, &cache).await
            }
        }
        ParsedCommand::Warn { reason } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                handle_warn(&bot, &msg, &pool, &cache, &settings, reason, owner_id).await
            }
        }
        ParsedCommand::Unwarn => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else if let Some(target) = resolve_target(&bot, &msg, &pool, &cache).await {
                let _ = clear_warnings(&pool, chat.id.0, target.id.0 as i64).await;
                let _ = deactivate_user_punishments(&pool, chat.id.0, target.id.0 as i64, "warn").await;
                let notice = format!("{RLM}✅ اخطارهای کاربر {} پاکسازی شد.", target.mention_display());
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            } else {
                Err(AppError::TargetNotFound)
            }
        }
        ParsedCommand::Warnings => {
            if let Some(target) = resolve_target(&bot, &msg, &pool, &cache).await {
                let count = get_warnings_count(&pool, chat.id.0, target.id.0 as i64).await.unwrap_or(0);
                let notice = format!(
                    "{RLM}⚠️ کاربر {} دارای <b>{}</b> اخطار از مجموع <b>{}</b> اخطار مجاز است.",
                    target.mention_display(),
                    to_persian_num(count),
                    to_persian_num(settings.warn_limit)
                );
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            } else {
                Err(AppError::TargetNotFound)
            }
        }
        ParsedCommand::Purge { count } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                handle_purge(&bot, &msg, count).await
            }
        }
        ParsedCommand::DeleteMessage => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else if let Some(reply) = msg.reply_to_message() {
                let _ = bot.delete_message(chat.id, reply.id).await;
                let _ = bot.delete_message(chat.id, msg.id).await;
                Ok(())
            } else {
                Err(AppError::TargetNotFound)
            }
        }
        ParsedCommand::Pin { silent } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else if let Some(reply) = msg.reply_to_message() {
                let mut req = bot.pin_chat_message(chat.id, reply.id);
                if silent {
                    req = req.disable_notification(true);
                }
                let _ = req.await;
                let notice = format!("{RLM}📌 پیام با موفقیت سنجاق شد.");
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            } else {
                Err(AppError::TargetNotFound)
            }
        }
        ParsedCommand::Unpin => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else if let Some(reply) = msg.reply_to_message() {
                let _ = bot.unpin_chat_message(chat.id).message_id(reply.id).await;
                let notice = format!("{RLM}📌 سنجاق پیام برداشته شد.");
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            } else {
                let _ = bot.unpin_all_chat_messages(chat.id).await;
                let notice = format!("{RLM}📌 تمامی پیام‌های سنجاق‌شده آزاد شدند.");
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::UserInfo => {
            let target = resolve_target(&bot, &msg, &pool, &cache).await.unwrap_or_else(|| crate::engine::target::TargetUser {
                id: sender.id,
                first_name: sender.first_name.clone(),
                username: sender.username.clone(),
            });
            let warns = get_warnings_count(&pool, chat.id.0, target.id.0 as i64).await.unwrap_or(0);
            let uname_display = match &target.username {
                Some(u) => format!("@{}", html_escape(u)),
                None => "ندارد".to_string(),
            };
            let text = format!(
                "{RLM}👤 <b>شناسنامه امنیتی کاربر:</b> {}\n\n\
                <table bordered compact>\n\
                  <caption align=\"center\">اطلاعات و سوابق کاربری</caption>\n\
                  <tr><td align=\"center\">شناسه عددی</td><td align=\"center\"><code>{}</code></td></tr>\n\
                  <tr><td align=\"center\">نام کاربری</td><td align=\"center\">{}</td></tr>\n\
                  <tr><td align=\"center\">اخطارهای انضباطی</td><td align=\"center\">{} از {}</td></tr>\n\
                </table>",
                target.mention_display(),
                target.id.0,
                uname_display,
                to_persian_num(warns),
                to_persian_num(settings.warn_limit)
            );
            let _ = send_styled_message(&bot, chat.id, &text, None).await;
            Ok(())
        }
        ParsedCommand::SetChannel(channel) => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let clean_ch = channel.trim()
                    .trim_start_matches("https://t.me/")
                    .trim_start_matches("http://t.me/")
                    .trim_start_matches("t.me/")
                    .trim_start_matches('@');
                let formatted_ch = format!("@{}", clean_ch);

                let me = bot.get_me().await.ok();
                let bot_id = me.as_ref().map(|u| u.id).unwrap_or(teloxide::types::UserId(0));
                let bot_username = me.as_ref().and_then(|u| u.username.clone()).unwrap_or_else(|| "Gordafarid_robot".to_string());

                // بررسی اینکه آیا ربات در کانال ادمین هست یا خیر
                let is_channel_admin = match bot.get_chat_member(Recipient::ChannelUsername(formatted_ch.clone()), bot_id).await {
                    Ok(member) => matches!(
                        member.status(),
                        ChatMemberStatus::Administrator | ChatMemberStatus::Owner
                    ),
                    Err(_) => false,
                };

                if !is_channel_admin {
                    let warn_text = format!(
                        "{RLM}⚠️ گردآفرید بدون داشتن کنترل بر روی کانال، نمیتونه بفهمه چه کسانی عضو هستن!\n\
                        لطفا اول از طریق دکمه‌ی زیر گردآفرید رو ادمین کانالت کن"
                    );
                    let add_url = format!(
                        "https://t.me/{}?startchannel=true&admin=invite_users+manage_chat",
                        bot_username
                    );
                    let kb = StyledKeyboard::new(vec![
                        vec![
                            StyledButton::url("➕ افزودن به کانال", add_url, Some("success")),
                        ]
                    ]);
                    let _ = send_styled_message(&bot, chat.id, &warn_text, Some(&kb)).await;
                    Ok(())
                } else {
                    let _ = update_forced_channel(&pool, chat.id.0, Some(&formatted_ch)).await;
                    cache.invalidate_settings(chat.id.0);
                    let notice = format!("{RLM}📢 کانال عضویت اجباری با موفقیت به <b>{}</b> تنظیم شد.", html_escape(&formatted_ch));
                    let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                    Ok(())
                }
            }
        }
        ParsedCommand::SetRules(rules) => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = set_rules_text(&pool, chat.id.0, &rules).await;
                cache.invalidate_settings(chat.id.0);
                let notice = format!("{RLM}📜 قوانین گروه با موفقیت به‌روزرسانی شد.");
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::AddCustomCmd { trigger, response } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = add_custom_command(&pool, chat.id.0, &trigger, &response, "exact").await;
                let notice = format!("{RLM}✨ دستور جدید «<b>{}</b>» با موفقیت افزوده شد.", html_escape(&trigger));
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::DeleteCustomCmd { trigger } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = delete_custom_command(&pool, chat.id.0, &trigger).await;
                let notice = format!("{RLM}🗑️ دستور «<b>{}</b>» حذف شد.", html_escape(&trigger));
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::ListCustomCmds => {
            let list = list_custom_commands(&pool, chat.id.0).await.unwrap_or_default();
            if list.is_empty() {
                let notice = format!("{RLM}ℹ️ دستور دلخواهی برای این گروه ثبت نشده است.");
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
            } else {
                let mut out = format!("{RLM}📋 <b>لیست دستورات دلخواه گروه:</b>\n");
                for (trig, resp) in list {
                    out.push_str(&format!("• <code>{}</code> ➔ {}\n", html_escape(&trig), html_escape(&resp)));
                }
                let _ = send_styled_message(&bot, chat.id, &out, None).await;
            }
            Ok(())
        }
        ParsedCommand::AddBannedWord { word, list_type } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = add_banned_word(&pool, chat.id.0, &word, list_type).await;
                cache.invalidate_banned_words(chat.id.0);
                let notice = format!("{RLM}⛔ کلمه «<b>{}</b>» به لیست کلمات ممنوعه اضافه شد.", html_escape(&word));
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::RemoveBannedWord { word } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = remove_banned_word(&pool, chat.id.0, &word).await;
                cache.invalidate_banned_words(chat.id.0);
                let notice = format!("{RLM}✅ کلمه «<b>{}</b>» از لیست ممنوعه حذف شد.", html_escape(&word));
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::AddAllowedDomain { domain } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = add_allowed_domain(&pool, chat.id.0, &domain).await;
                cache.invalidate_allowed_domains(chat.id.0);
                let notice = format!("{RLM}🌐 دامنه «<b>{}</b>» به لیست مجاز افزوده شد.", html_escape(&domain));
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::RemoveAllowedDomain { domain } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = remove_allowed_domain(&pool, chat.id.0, &domain).await;
                cache.invalidate_allowed_domains(chat.id.0);
                let notice = format!("{RLM}✅ دامنه «<b>{}</b>» از لیست مجاز حذف شد.", html_escape(&domain));
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::SetWelcome(enabled) => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = update_welcome_toggle(&pool, chat.id.0, enabled).await;
                cache.invalidate_settings(chat.id.0);
                let status_text = if enabled { "روشن ✅" } else { "خاموش ❌" };
                let notice = format!("{RLM}🌸 پیام خوشامد: <b>{}</b>", status_text);
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::SetGoodbye(enabled) => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = update_goodbye_toggle(&pool, chat.id.0, enabled).await;
                cache.invalidate_settings(chat.id.0);
                let status_text = if enabled { "روشن ✅" } else { "خاموش ❌" };
                let notice = format!("{RLM}👋 پیام خروج: <b>{}</b>", status_text);
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::SetAntiFlood(enabled) => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = update_anti_flood_toggle(&pool, chat.id.0, enabled).await;
                cache.invalidate_settings(chat.id.0);
                let status_text = if enabled { "روشن ✅" } else { "خاموش ❌" };
                let notice = format!("{RLM}⚡ سیستم ضداسپم: <b>{}</b>", status_text);
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::SetLinkFilter(enabled) => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = update_link_filter_toggle(&pool, chat.id.0, enabled).await;
                cache.invalidate_settings(chat.id.0);
                let status_text = if enabled { "روشن ✅" } else { "خاموش ❌" };
                let notice = format!("{RLM}🔗 فیلتر لینک: <b>{}</b>", status_text);
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::LockToggle { lock_type, enable } => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let name = lock_type.display_name();
                match lock_type {
                    crate::engine::parser::LockType::Link => {
                        let _ = update_link_filter_toggle(&pool, chat.id.0, enable).await;
                    }
                    crate::engine::parser::LockType::Forward => {
                        let _ = update_forward_filter_toggle(&pool, chat.id.0, enable).await;
                    }
                    crate::engine::parser::LockType::Photo => {
                        let _ = update_photo_filter_toggle(&pool, chat.id.0, enable).await;
                    }
                    crate::engine::parser::LockType::Video => {
                        let _ = update_video_filter_toggle(&pool, chat.id.0, enable).await;
                    }
                    crate::engine::parser::LockType::Voice => {
                        let _ = update_voice_filter_toggle(&pool, chat.id.0, enable).await;
                    }
                    crate::engine::parser::LockType::Gif => {
                        let _ = update_gif_filter_toggle(&pool, chat.id.0, enable).await;
                    }
                    crate::engine::parser::LockType::Sticker => {
                        let _ = update_sticker_filter_toggle(&pool, chat.id.0, enable).await;
                    }
                    crate::engine::parser::LockType::Document => {
                        let _ = update_document_filter_toggle(&pool, chat.id.0, enable).await;
                    }
                    crate::engine::parser::LockType::ServiceMsg => {
                        let _ = update_service_msg_filter_toggle(&pool, chat.id.0, enable).await;
                    }
                    crate::engine::parser::LockType::Media => {
                        let _ = update_media_filter_toggle(&pool, chat.id.0, enable).await;
                    }
                    crate::engine::parser::LockType::AntiFlood => {
                        let _ = update_anti_flood_toggle(&pool, chat.id.0, enable).await;
                    }
                    crate::engine::parser::LockType::NightMode => {
                        let _ = update_night_mode_toggle(&pool, chat.id.0, enable).await;
                    }
                }
                cache.invalidate_settings(chat.id.0);
                let status_text = if enable { "قفل شد 🟢" } else { "باز شد 🔴" };
                let notice = format!("{RLM}🔒 قفل <b>{}</b> {}", name, status_text);
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::GroupLockMenu => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let kb = group_lock_styled(&settings);
                let text = format!("{RLM}🔒 <b>مدیریت قفل گروه:</b>\nنوع قفل مورد نظر خود را برای اعمال بر روی گروه انتخاب نمایید:");
                let _ = send_styled_message(&bot, chat.id, &text, Some(&kb)).await;
                Ok(())
            }
        }
        ParsedCommand::SetGroupLock(mode) => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = update_group_lock_mode(&pool, chat.id.0, &mode).await;
                cache.invalidate_settings(chat.id.0);
                let notice = match mode.as_str() {
                    "total" => format!("{RLM}🔒 <b>«قفل سراسری» بر روی گروه فعال شد.</b>\nارسال هرگونه پیام در گروه تا اطلاع ثانوی مسدود است."),
                    "comments_only" => format!("{RLM}💬 <b>قفل گروه در حالت «کامنت آزاد» قرار گرفت.</b>\nکاربران تنها می‌توانند بر روی پست‌های کانال متصل نظر (کامنت) ثبت کنند و ارسال پیام عادی در گروه مسدود است."),
                    _ => format!("{RLM}🔓 <b>قفل گروه برداشته شد.</b>\nارسال پیام برای تمامی اعضا آزاد است."),
                };
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::SetLogTopic => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else if let Some(thread_id) = msg.thread_id {
                let topic_id = thread_id.0 .0 as i64;
                let _ = update_log_topic(&pool, chat.id.0, Some(topic_id)).await;
                cache.invalidate_settings(chat.id.0);
                let notice = format!("{RLM}✅ این تاپیک با شناسه {} به عنوان تاپیک لاگ‌های مدیریتی گروه تنظیم گردید.", to_persian_num(topic_id));
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            } else {
                let notice = format!("{RLM}⚠️ این دستور باید درون یکی از تاپیک‌های گروه فروم ارسال شود تا آن تاپیک به عنوان لاگ تعیین گردد.");
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::ClearLogTopic => {
            if !is_admin {
                Err(AppError::PermissionDenied)
            } else {
                let _ = update_log_topic(&pool, chat.id.0, None).await;
                cache.invalidate_settings(chat.id.0);
                let notice = format!("{RLM}✅ تاپیک لاگ لغو شد و اعلان‌ها به چت اصلی ارسال خواهند شد.");
                let _ = send_styled_message(&bot, chat.id, &notice, None).await;
                Ok(())
            }
        }
        ParsedCommand::CustomOrUnknown(trigger) => {
            // بررسی دستورات سفارشی ثبت‌شده توسط ادمین در گروه
            if let Ok(Some(response)) = get_custom_command(&pool, chat.id.0, &trigger).await {
                let _ = send_styled_message(&bot, chat.id, &response, None).await;
            }
            Ok(())
        }
    };

    if let Err(err) = result {
        // عدم ارسال پیام خطا به گروه در صورتی که کاربر دسترسی ادمین نداشته باشد (سکوت کامل برای اعضای عادی)
        if matches!(err, AppError::PermissionDenied) {
            return Ok(());
        }
        warn!("خطا در اجرای دستور در گروه {}: {:?}", chat.id, err);
        let notice = format!("{RLM}⚠️ {}", err.to_persian_message());
        let _ = send_styled_message(&bot, chat.id, &notice, None).await;
    }

    Ok(())
}
