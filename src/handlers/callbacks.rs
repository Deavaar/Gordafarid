use chrono::NaiveTime;
use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::{CallbackQuery, ChatMemberStatus, Recipient};
use tracing::info;

use crate::db::custom_commands::*;
use crate::db::filters::*;
use crate::db::groups::*;
use crate::db::moderation::*;
use crate::engine::normalizer::{html_escape, RLM};
use crate::engine::permissions::check_sender_is_admin_cached;
use crate::services::cache::AppCache;
use crate::ui::keyboards::*;
use crate::i18n::get_i18n;
use crate::ui::messages::format_template;
use crate::ui::styled_keyboard::{edit_styled_keyboard, edit_styled_message_text, send_styled_message};


fn map_callback_to_feature(data: &str) -> Option<&'static str> {
    if data.starts_with("nav_locks")
        || data.starts_with("nav_filters")
        || data.starts_with("nav_group_lock")
        || data.starts_with("set_lock_")
        || data.starts_with("toggle_link")
        || data.starts_with("toggle_forward")
        || data.starts_with("toggle_photo")
        || data.starts_with("toggle_video")
        || data.starts_with("toggle_voice")
        || data.starts_with("toggle_document")
        || data.starts_with("toggle_gif")
        || data.starts_with("toggle_sticker")
        || data.starts_with("toggle_media")
        || data.starts_with("cycle_link_action")
    {
        return Some("locks");
    }

    if data.starts_with("nav_welcome")
        || data.starts_with("toggle_welcome")
        || data.starts_with("toggle_goodbye")
        || data.starts_with("preview_welcome")
        || data.starts_with("preview_goodbye")
        || data.starts_with("reset_welcome")
        || data.starts_with("help_welcome_text")
    {
        return Some("welcome");
    }

    if data.starts_with("nav_flood")
        || data.starts_with("toggle_flood")
        || data.starts_with("set_rate_")
        || data.starts_with("set_win_")
        || data.starts_with("cycle_flood_action")
    {
        return Some("flood");
    }

    if data.starts_with("nav_night")
        || data.starts_with("toggle_night")
        || data.starts_with("set_night_")
    {
        return Some("night");
    }

    if data.starts_with("nav_captcha")
        || data.starts_with("toggle_captcha")
        || data.starts_with("set_cap_")
        || data.starts_with("cycle_captcha_action")
    {
        return Some("captcha");
    }

    if data.starts_with("nav_warn")
        || data.starts_with("nav_actions")
        || data.starts_with("set_warn_")
        || data.starts_with("cycle_warn_action")
        || data.starts_with("toggle_silent")
        || data.starts_with("clear_all_warnings")
    {
        return Some("warn");
    }

    if data.starts_with("nav_channel")
        || data.starts_with("clear_forced_channel")
        || data.starts_with("guide_channel")
        || data.starts_with("check_channel")
    {
        return Some("channel");
    }

    if data.starts_with("toggle_service_msg") {
        return Some("service_msg");
    }

    if data.starts_with("nav_words")
        || data.starts_with("list_banned_words")
        || data.starts_with("clear_all_banned_words")
        || data.starts_with("guide_banned_words")
    {
        return Some("words");
    }

    if data.starts_with("nav_domains")
        || data.starts_with("list_allowed_domains")
        || data.starts_with("clear_all_domains")
        || data.starts_with("guide_allowed_domains")
    {
        return Some("domains");
    }

    if data.starts_with("nav_custom_cmds")
        || data.starts_with("list_custom_cmds_btn")
        || data.starts_with("clear_all_custom_cmds")
        || data.starts_with("guide_custom_cmds")
    {
        return Some("custom_cmds");
    }

    if data.starts_with("nav_channel_track")
        || data.starts_with("toggle_channel_track")
        || data.starts_with("toggle_track_")
    {
        return Some("channel_track");
    }

    None
}

pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    pool: PgPool,
    cache: AppCache,
    owner_id: i64,
) -> Result<(), teloxide::RequestError> {
    let data = match q.data {
        Some(ref d) => d.as_str(),
        None => return Ok(()),
    };

    let msg = match q.message {
        Some(ref m) => m,
        None => return Ok(()),
    };

    let msg_chat_id = msg.chat().id;
    let user = &q.from;

    info!("کال‌بک کوئری دریافت شد: '{}' از کاربر {} در چت {}", data, user.id.0, msg_chat_id);

    // بررسی کال‌بک‌های انتخاب گروه و مدیریت دسترسی ادمین‌ها در پیوی
    if data.starts_with("pv_grp_sel:") {
        let target_chat_id: i64 = match data.trim_start_matches("pv_grp_sel:").parse() {
            Ok(id) => id,
            Err(_) => return Ok(()),
        };
        let admin_groups = crate::db::groups::resolve_user_admin_groups(&bot, &pool, user.id.0).await.unwrap_or_default();
        if !admin_groups.iter().any(|(cid, _)| *cid == target_chat_id) {
            let _ = bot.answer_callback_query(q.id).text("شما ادمین این گروه نیستید.").show_alert(true).await;
            return Ok(());
        }

        cache.set_user_active_pv_group(user.id.0, target_chat_id);
        let settings = match get_settings(&pool, target_chat_id).await {
            Ok(s) => s,
            Err(_) => {
                let _ = bot.answer_callback_query(q.id).text("خطا در بازیابی تنظیمات گروه.").await;
                return Ok(());
            }
        };
        let counts = get_settings_counts(&pool, target_chat_id).await;
        let kb = settings_main_styled(&settings, &counts);
        let group_title = admin_groups.iter().find(|(cid, _)| *cid == target_chat_id).map(|(_, t)| t.as_str()).unwrap_or("گروه");
        let text = format!("{RLM}🛡️ <b>پنل مدیریت گروه:</b> {}\nگردآفرید چگونه نظم را برپا کند؟ از اینجا مشخص کنید:", html_escape(group_title));
        let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
        let _ = bot.answer_callback_query(q.id).await;
        return Ok(());
    }

    if data.starts_with("pv_ap_grp:") {
        let target_chat_id: i64 = match data.trim_start_matches("pv_ap_grp:").parse() {
            Ok(id) => id,
            Err(_) => return Ok(()),
        };
        let owned = crate::db::groups::resolve_user_strictly_owned_groups(&bot, &pool, user.id.0).await.unwrap_or_default();
        if !owned.iter().any(|(cid, _)| *cid == target_chat_id) {
            let _ = bot.answer_callback_query(q.id).text("شما مالک این گروه نیستید.").show_alert(true).await;
            return Ok(());
        }

        match bot.get_chat_administrators(teloxide::types::ChatId(target_chat_id)).await {
            Ok(admins) => {
                let kb = pv_admin_list_styled(target_chat_id, &admins);
                let group_title = owned.iter().find(|(cid, _)| *cid == target_chat_id).map(|(_, t)| t.as_str()).unwrap_or("گروه");
                let text = format!("{RLM}👑 <b>تنظیم دسترسی ادمین‌های گروه {}:</b>\nادمین مورد نظر یا گزینه سراسری را انتخاب کنید:", html_escape(group_title));
                let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
                let _ = bot.answer_callback_query(q.id).await;
            }
            Err(e) => {
                let _ = bot.answer_callback_query(q.id).text(format!("خطا در دریافت لیست ادمین‌ها: {}", e)).show_alert(true).await;
            }
        }
        return Ok(());
    }

    if data == "pv_ap_back" {
        let groups = crate::db::groups::resolve_user_strictly_owned_groups(&bot, &pool, user.id.0).await.unwrap_or_default();
        let prompt = crate::i18n::get_i18n().pv.select_group_admin_perms.replace("{RLM}", RLM);
        let kb = pv_group_selection_styled(&groups, "pv_ap_grp");
        let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &prompt, Some(&kb)).await;
        let _ = bot.answer_callback_query(q.id).await;
        return Ok(());
    }

    if data.starts_with("ap_admin:") {
        let parts: Vec<&str> = data.trim_start_matches("ap_admin:").split(':').collect();
        if parts.len() == 2 {
            let target_chat_id: i64 = parts[0].parse().unwrap_or(0);
            let target_admin_id: i64 = parts[1].parse().unwrap_or(0);

            let perms = crate::db::admin_permissions::get_admin_permissions(&pool, target_chat_id, target_admin_id).await.unwrap_or_default();
            let target_name = if target_admin_id == 0 {
                "سراسری (تمام ادمین‌ها)".to_string()
            } else {
                format!("ادمین {}", target_admin_id)
            };

            cache.set_admin_perm_draft(user.id.0, crate::services::cache::AdminPermDraft {
                chat_id: target_chat_id,
                target_admin_id,
                target_admin_name: target_name.clone(),
                permissions: perms.clone(),
            });

            let kb = pv_admin_permissions_styled(target_chat_id, &perms);
            let text = format!("{RLM}⚙️ <b>مدیریت دسترسی‌های {}:</b>\nبا کلیک روی هر دکمه، وضعیت دسترسی را مجاز یا ممنوع کنید و سپس دکمه «تایید و اعمال» را بزنید:", html_escape(&target_name));
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
        }
        let _ = bot.answer_callback_query(q.id).await;
        return Ok(());
    }

    if data.starts_with("ap_tog:") {
        let key = data.trim_start_matches("ap_tog:");
        if let Some(new_state) = cache.toggle_admin_perm_draft(user.id.0, key) {
            if let Some(draft) = cache.get_admin_perm_draft(user.id.0) {
                let kb = pv_admin_permissions_styled(draft.chat_id, &draft.permissions);
                let text = format!("{RLM}⚙️ <b>مدیریت دسترسی‌های {}:</b>\nبا کلیک روی هر دکمه، وضعیت دسترسی را مجاز یا ممنوع کنید و سپس دکمه «تایید و اعمال» را بزنید:", html_escape(&draft.target_admin_name));
                let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
                let toast = if new_state { "مجاز شد 🟢" } else { "ممنوع شد 🔴" };
                let _ = bot.answer_callback_query(q.id).text(toast).await;
                return Ok(());
            }
        }
        let _ = bot.answer_callback_query(q.id).await;
        return Ok(());
    }

    if data == "ap_save" {
        if let Some(draft) = cache.remove_admin_perm_draft(user.id.0) {
            let _ = crate::db::admin_permissions::save_admin_permissions_batch(&pool, draft.chat_id, draft.target_admin_id, &draft.permissions).await;
            let _ = bot.answer_callback_query(q.id).text(&crate::i18n::get_i18n().pv.admin_perms_saved).show_alert(true).await;
            if let Ok(admins) = bot.get_chat_administrators(teloxide::types::ChatId(draft.chat_id)).await {
                let kb = pv_admin_list_styled(draft.chat_id, &admins);
                let text = format!("{RLM}👑 <b>تنظیم دسترسی ادمین‌های گروه:</b>\nادمین مورد نظر یا گزینه سراسری را انتخاب کنید:");
                let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            }
        }
        return Ok(());
    }

    if data == "ap_cancel" {
        if let Some(draft) = cache.remove_admin_perm_draft(user.id.0) {
            let _ = bot.answer_callback_query(q.id).text(&crate::i18n::get_i18n().pv.admin_perms_cancelled).show_alert(true).await;
            if let Ok(admins) = bot.get_chat_administrators(teloxide::types::ChatId(draft.chat_id)).await {
                let kb = pv_admin_list_styled(draft.chat_id, &admins);
                let text = format!("{RLM}👑 <b>تنظیم دسترسی ادمین‌های گروه:</b>\nادمین مورد نظر یا گزینه سراسری را انتخاب کنید:");
                let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            }
        }
        return Ok(());
    }

    if data.starts_with("ap_back_list:") {
        let target_chat_id: i64 = data.trim_start_matches("ap_back_list:").parse().unwrap_or(0);
        let _ = cache.remove_admin_perm_draft(user.id.0);
        if let Ok(admins) = bot.get_chat_administrators(teloxide::types::ChatId(target_chat_id)).await {
            let kb = pv_admin_list_styled(target_chat_id, &admins);
            let text = format!("{RLM}👑 <b>تنظیم دسترسی ادمین‌های گروه:</b>\nادمین مورد نظر یا گزینه سراسری را انتخاب کنید:");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
        }
        let _ = bot.answer_callback_query(q.id).await;
        return Ok(());
    }

    // ۱. بررسی دکمه‌های حل کپچای اعضای جدید
    if data.starts_with("verify_captcha_") || data.starts_with("math_correct_") {
        let expected_user_id: u64 = if data.starts_with("verify_captcha_") {
            data.trim_start_matches("verify_captcha_").parse().unwrap_or(0)
        } else {
            data.trim_start_matches("math_correct_").parse().unwrap_or(0)
        };

        if user.id.0 == expected_user_id {
            let default_perms = crate::services::scheduler::default_member_permissions();
            let _ = bot.restrict_chat_member(msg_chat_id, user.id, default_perms).await;
            let _ = bot.answer_callback_query(q.id.clone()).text("عضویت شما با موفقیت تایید شد! خوش آمدید 🌸").await;
            let _ = bot.delete_message(msg_chat_id, msg.id()).await;
        } else {
            let _ = bot.answer_callback_query(q.id.clone()).text("این آزمون تایید هویت برای شما نیست!").await;
        }
        return Ok(());
    }

    if data.starts_with("math_wrong_") {
        let expected_user_id: u64 = data.trim_start_matches("math_wrong_").parse().unwrap_or(0);
        if user.id.0 == expected_user_id {
            let _ = bot.answer_callback_query(q.id.clone()).text("❌ پاسخ اشتباه بود! لطفاً مجدداً تلاش کنید.").await;
        } else {
            let _ = bot.answer_callback_query(q.id.clone()).text("این آزمون تایید هویت برای شما نیست!").await;
        }
        return Ok(());
    }

    // بررسی دکمه استعلام زنده عضویت اجباری کانال (Force-Join Live Check)
    if data == "check_force_join" {
        let settings = match get_settings(&pool, msg_chat_id.0).await {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };

        if let Some(ref channel_username) = settings.forced_channel {
            let formatted = if channel_username.starts_with('@') {
                channel_username.clone()
            } else {
                format!("@{}", channel_username)
            };

            match bot.get_chat_member(Recipient::ChannelUsername(formatted.clone()), user.id).await {
                Ok(member) => {
                    let is_joined = !matches!(member.status(), ChatMemberStatus::Left | ChatMemberStatus::Banned);
                    cache.set_channel_membership(msg_chat_id.0, user.id.0, is_joined);

                    if is_joined {
                        let _ = bot.answer_callback_query(q.id).text("عضویت شما با موفقیت تایید شد! سپاسگزاریم 🌸").await;
                        let _ = bot.delete_message(msg_chat_id, msg.id()).await;
                    } else {
                        let _ = bot.answer_callback_query(q.id)
                            .text("⚠️ شما هنوز در کانال عضو نشده‌اید! لطفاً ابتدا در کانال عضو شوید و سپس این دکمه را بزنید.")
                            .show_alert(true)
                            .await;
                    }
                }
                Err(e) => {
                    tracing::error!("خطا در بررسی زنده عضویت کانال: {}", e);
                    let _ = bot.answer_callback_query(q.id).text("خطا در بررسی وضعیت عضویت تلگرام.").await;
                }
            }
        } else {
            let _ = bot.answer_callback_query(q.id).text("کانال عضویت اجباری تنظیم نشده است.").await;
            let _ = bot.delete_message(msg_chat_id, msg.id()).await;
        }
        return Ok(());
    }

    // ۲. شناسایی چت مقصد (گروه عادی یا مدیریت از طریق پیوی)
    let is_pv = msg.chat().is_private();
    let target_group_id = if is_pv {
        match cache.get_user_active_pv_group(user.id.0) {
            Some(gid) => gid,
            None => {
                let _ = bot.answer_callback_query(q.id).text("لطفاً ابتدا گروه مورد نظر را انتخاب نمایید.").show_alert(true).await;
                return Ok(());
            }
        }
    } else {
        msg_chat_id.0
    };
    let chat_id = teloxide::types::ChatId(target_group_id);

    let is_owner = crate::db::groups::check_is_group_owner(&bot, &pool, target_group_id, user.id.0).await;

    let is_admin = if is_owner {
        true
    } else {
        check_sender_is_admin_cached(&bot, &cache, chat_id, user.id, owner_id)
            .await
            .unwrap_or(false)
    };

    if !is_admin {
        let _ = bot.answer_callback_query(q.id).text("⛔ شما دسترسی مدیریت برای انجام این عملیات را ندارید.").show_alert(true).await;
        return Ok(());
    }

    // بررسی دسترسی اختصاصی ادمین بر اساس تنظیمات مشخص‌شده توسط مالک گروه
    if let Some(feature_key) = map_callback_to_feature(data) {
        if !crate::db::admin_permissions::is_admin_feature_allowed(&pool, chat_id.0, user.id.0, feature_key, is_owner).await {
            let _ = bot.answer_callback_query(q.id).text(&crate::i18n::get_i18n().pv.permission_denied_admin).show_alert(true).await;
            return Ok(());
        }
    }

    // بررسی تایید یا رد به‌روزرسانی حکم سکوت
    if data.starts_with("confirm_mute:") {
        let token = data.trim_start_matches("confirm_mute:");
        let pending = cache.remove_pending_mute(token);
        match pending {
            Some(p) => {
                let empty_perms = teloxide::types::ChatPermissions::empty();
                let _ = bot.restrict_chat_member(chat_id, teloxide::types::UserId(p.target_id), empty_perms).await;
                let _ = deactivate_user_punishments(&pool, chat_id.0, p.target_id as i64, "mute").await;
                if let Some(dur) = p.duration {
                    let expires = chrono::Utc::now() + dur;
                    let _ = add_temp_punishment(
                        &pool,
                        chat_id.0,
                        p.target_id as i64,
                        "mute",
                        expires,
                        p.reason.as_deref(),
                        user.id.0 as i64,
                    )
                    .await;
                }
                let _ = crate::db::audit::record_audit(
                    &pool,
                    chat_id.0,
                    user.id.0 as i64,
                    Some(p.target_id as i64),
                    "mute_update",
                    p.reason.as_deref(),
                )
                .await;

                let target_disp = if let Some(ref uname) = p.target_username {
                    format!("@{}", uname)
                } else {
                    p.target_name.clone()
                };

                let updated_text = get_i18n().moderation.mute_update_success
                    .replace("{target_display}", &html_escape(&target_disp))
                    .replace("{RLM}", RLM);
                let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &updated_text, None).await;
                let _ = bot.answer_callback_query(q.id).text("مجازات کاربر به‌روزرسانی شد.").await;
            }
            None => {
                let _ = bot.answer_callback_query(q.id).text(&get_i18n().moderation.mute_update_expired).show_alert(true).await;
                let _ = bot.delete_message(msg_chat_id, msg.id()).await;
            }
        }
        return Ok(());
    }

    if data.starts_with("reject_mute:") {
        let token = data.trim_start_matches("reject_mute:");
        let _ = cache.remove_pending_mute(token);
        let reject_text = get_i18n().moderation.mute_update_rejected.replace("{RLM}", RLM);
        let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &reject_text, None).await;
        let _ = bot.answer_callback_query(q.id).text("به‌روزرسانی لغو گردید.").await;
        return Ok(());
    }

    let mut settings = match get_settings(&pool, chat_id.0).await {
        Ok(s) => s,
        Err(_) => {
            let _ = bot.answer_callback_query(q.id).text("خطا در بازیابی تنظیمات گروه.").await;
            return Ok(());
        }
    };

    match data {
        // ناوبری منوها
        "nav_main" => {
            let counts = get_settings_counts(&pool, chat_id.0).await;
            let kb = settings_main_styled(&settings, &counts);
            let text = format!("{RLM}🛡️ <b>گردآفرید چگونه نظم را برپا کند؟</b>\nاز اینجا مشخص کنید");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_group_lock" => {
            let kb = group_lock_from_panel_styled(&settings);
            let text = format!("{RLM}🔒 <b>مدیریت قفل گروه:</b>\nنوع قفل مورد نظر خود را برای اعمال بر روی گروه انتخاب نمایید:");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_welcome" => {
            let kb = settings_welcome_styled(&settings);
            let text = format!("{RLM}🌸 <b>تنظیمات پیام خوشامدگویی و خداحافظی:</b>");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_flood" => {
            let kb = settings_flood_styled(&settings);
            let text = format!("{RLM}🛡️ <b>تنظیمات سیستم ضداسپم و پیام‌های رگباری:</b>");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_locks" | "nav_filters" => {
            let kb = settings_locks_styled(&settings);
            let text = format!("{RLM}🔒 <b>مدیریت قفل‌های امنیتی، رسانه‌ای و محتوایی گروه:</b>\nبا لمس هر دکمه وضعیت قفل مربوطه را تغییر دهید:");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_night" => {
            let kb = settings_night_styled(&settings);
            let text = format!("{RLM}🌙 <b>تنظیمات قفل شبانه گروه (Night Mode):</b>");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_captcha" => {
            let kb = settings_captcha_styled(&settings);
            let text = format!("{RLM}🤖 <b>تنظیمات تایید هویت و کپچای عضویت اعضای جدید:</b>");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_warn" => {
            let kb = settings_warn_styled(&settings);
            let text = format!("{RLM}⚠️ <b>تنظیمات سیستم سقف اخطار و تشدید مجازات:</b>");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_channel" => {
            let kb = settings_channel_styled(&settings);
            let text = format!("{RLM}📢 <b>تنظیمات قفل عضویت اجباری در کانال:</b>\nبرای تنظیم کانال، دستور زیر را در گروه ارسال کنید:\n<code>تنظیم کانال @آیدی_کانال</code>");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_words" => {
            let words = get_banned_words(&pool, chat_id.0).await.unwrap_or_default();
            let kb = settings_words_styled(words.len());
            let text = format!("{RLM}📋 <b>مدیریت کلمات ممنوعه گروه:</b>\nپیام‌های حاوی کلمات این لیست به‌صورت خودکار حذف می‌شوند.");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_domains" => {
            let domains = get_allowed_domains(&pool, chat_id.0).await.unwrap_or_default();
            let kb = settings_domains_styled(domains.len());
            let text = format!("{RLM}🌐 <b>مدیریت دامنه‌های مجاز گروه:</b>\nهنگام فعال بودن فیلتر لینک، لینک‌های این دامنه‌ها مجاز خواهند بود.");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_custom_cmds" => {
            let cmds = list_custom_commands(&pool, chat_id.0).await.unwrap_or_default();
            let kb = settings_custom_cmds_styled(cmds.len());
            let text = format!("{RLM}✨ <b>مدیریت دستورات دلخواه و پاسخ‌های خودکار:</b>");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_actions" => {
            let kb = settings_actions_styled(&settings);
            let text = format!("{RLM}⚡ <b>تنظیم اقدامات تنبیهی خودکار در مواجهه با تخلفات:</b>");
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_channel_track" => {
            let kb = settings_channel_track_styled(&settings);
            let text = format!(
                "{RLM}📡 <b>مدیریت رهگیری کانال</b>\n\n\
                {RLM}رهگیری کانال به پست‌های کانال متصل‌شده وابسته است و بر اساس درخواست ادمین در پست کانال، نوع محتوای مجاز برای کامنت‌های همان پست را مشخص می‌کند."
            );
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "nav_track_words" => {
            let kb = settings_channel_track_words_styled(&settings);
            let text = format!(
                "{RLM}⚙️ <b>تنظیمات کلمات و دسته‌بندی‌های رهگیری کانال</b>\n\n\
                {RLM}دسته‌بندی‌های مورد نظر برای شناسایی در پست‌های کانال را فعال یا غیرفعال کنید:"
            );
            let _ = edit_styled_message_text(&bot, msg_chat_id, msg.id(), &text, Some(&kb)).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "close_settings" => {
            let _ = bot.delete_message(msg_chat_id, msg.id()).await;
        }
        "show_rules" => {
            let rules = if settings.rules_text.is_empty() {
                "قوانین برای این گروه هنوز ثبت نشده است."
            } else {
                &settings.rules_text
            };
            let text = format!("{RLM}📜 <b>قوانین گروه:</b>\n\n{RLM}{}", html_escape(rules));
            let _ = send_styled_message(&bot, chat_id, &text, None).await;
            let _ = bot.answer_callback_query(q.id).await;
        }

        // سوئیچ‌ها و تاگل‌ها
        "toggle_welcome_main" => {
            let new_val = !settings.welcome_enabled;
            let _ = update_welcome_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.welcome_enabled = new_val;
            let counts = get_settings_counts(&pool, chat_id.0).await;
            let kb = settings_main_styled(&settings, &counts);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "پیام خوشامد فعال شد ✅" } else { "پیام خوشامد غیرفعال شد ❌" }).await;
        }
        "toggle_goodbye_main" => {
            let new_val = !settings.goodbye_enabled;
            let _ = update_goodbye_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.goodbye_enabled = new_val;
            let counts = get_settings_counts(&pool, chat_id.0).await;
            let kb = settings_main_styled(&settings, &counts);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "پیام خروج فعال شد ✅" } else { "پیام خروج غیرفعال شد ❌" }).await;
        }
        "toggle_flood_main" => {
            let new_val = !settings.anti_flood_enabled;
            let _ = update_anti_flood_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.anti_flood_enabled = new_val;
            let counts = get_settings_counts(&pool, chat_id.0).await;
            let kb = settings_main_styled(&settings, &counts);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "ضداسپم فعال شد ✅" } else { "ضداسپم غیرفعال شد ❌" }).await;
        }
        "toggle_night_main" => {
            let new_val = !settings.night_mode_enabled;
            let _ = update_night_mode_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.night_mode_enabled = new_val;
            let counts = get_settings_counts(&pool, chat_id.0).await;
            let kb = settings_main_styled(&settings, &counts);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "قفل شبانه فعال شد ✅" } else { "قفل شبانه غیرفعال شد ❌" }).await;
        }
        "toggle_captcha_main" => {
            let new_val = !settings.captcha_enabled;
            let _ = update_captcha_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.captcha_enabled = new_val;
            let counts = get_settings_counts(&pool, chat_id.0).await;
            let kb = settings_main_styled(&settings, &counts);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "کپچای عضویت فعال شد ✅" } else { "کپچای عضویت غیرفعال شد ❌" }).await;
        }
        "toggle_flood_lock" => {
            let new_val = !settings.anti_flood_enabled;
            let _ = update_anti_flood_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.anti_flood_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "سیستم ضداسپم فعال شد ✅" } else { "سیستم ضداسپم غیرفعال شد ❌" }).await;
        }
        "toggle_night_lock" => {
            let new_val = !settings.night_mode_enabled;
            let _ = update_night_mode_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.night_mode_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "قفل شبانه فعال شد ✅" } else { "قفل شبانه غیرفعال شد ❌" }).await;
        }
        "toggle_silent" => {
            let new_val = !settings.silent_punishment;
            let _ = update_silent_punishment(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.silent_punishment = new_val;
            let counts = get_settings_counts(&pool, chat_id.0).await;
            let kb = settings_main_styled(&settings, &counts);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let status = if new_val { "روشن شد 🔇 (پیام مجازات در گروه ارسال نمی‌شود)" } else { "خاموش شد 📢 (پیام مجازات در گروه اعلام می‌شود)" };
            let _ = bot.answer_callback_query(q.id).text(format!("مجازات بی‌صدا {}", status)).await;
        }
        "toggle_service_msg_main" => {
            let new_val = !settings.service_msg_filter_enabled;
            let _ = update_service_msg_filter_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.service_msg_filter_enabled = new_val;
            let counts = get_settings_counts(&pool, chat_id.0).await;
            let kb = settings_main_styled(&settings, &counts);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "پاکسازی بلادرنگ اعلان‌های تلگرام فعال شد 🔔" } else { "پاکسازی اعلان‌های تلگرام غیرفعال شد 🔕" }).await;
        }
        "toggle_welcome" => {
            let new_val = !settings.welcome_enabled;
            let _ = update_welcome_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.welcome_enabled = new_val;
            let kb = settings_welcome_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "پیام خوشامد فعال شد ✅" } else { "پیام خوشامد غیرفعال شد ❌" }).await;
        }
        "toggle_goodbye" => {
            let new_val = !settings.goodbye_enabled;
            let _ = update_goodbye_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.goodbye_enabled = new_val;
            let kb = settings_welcome_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "پیام خروج فعال شد ✅" } else { "پیام خروج غیرفعال شد ❌" }).await;
        }
        "toggle_flood" => {
            let new_val = !settings.anti_flood_enabled;
            let _ = update_anti_flood_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.anti_flood_enabled = new_val;
            let kb = settings_flood_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "ضداسپم فعال شد ✅" } else { "ضداسپم غیرفعال شد ❌" }).await;
        }
        "toggle_link" => {
            let new_val = !settings.link_filter_enabled;
            let _ = update_link_filter_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.link_filter_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "فیلتر لینک فعال شد ✅" } else { "فیلتر لینک غیرفعال شد ❌" }).await;
        }
        "toggle_forward" => {
            let new_val = !settings.forward_filter_enabled;
            let _ = update_forward_filter_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.forward_filter_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "فیلتر فوروارد فعال شد ✅" } else { "فیلتر فوروارد غیرفعال شد ❌" }).await;
        }
        "toggle_photo" => {
            let new_val = !settings.photo_filter_enabled;
            let _ = update_photo_filter_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.photo_filter_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "فیلتر عکس فعال شد ✅" } else { "فیلتر عکس غیرفعال شد ❌" }).await;
        }
        "toggle_video" => {
            let new_val = !settings.video_filter_enabled;
            let _ = update_video_filter_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.video_filter_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "فیلتر ویدیو فعال شد ✅" } else { "فیلتر ویدیو غیرفعال شد ❌" }).await;
        }
        "toggle_voice" => {
            let new_val = !settings.voice_filter_enabled;
            let _ = update_voice_filter_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.voice_filter_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "فیلتر ویس فعال شد ✅" } else { "فیلتر ویس غیرفعال شد ❌" }).await;
        }
        "toggle_gif" => {
            let new_val = !settings.gif_filter_enabled;
            let _ = update_gif_filter_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.gif_filter_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "فیلتر گیف فعال شد ✅" } else { "فیلتر گیف غیرفعال شد ❌" }).await;
        }
        "toggle_sticker" => {
            let new_val = !settings.sticker_filter_enabled;
            let _ = update_sticker_filter_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.sticker_filter_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "فیلتر استیکر فعال شد ✅" } else { "فیلتر استیکر غیرفعال شد ❌" }).await;
        }
        "toggle_document" => {
            let new_val = !settings.document_filter_enabled;
            let _ = update_document_filter_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.document_filter_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "فیلتر فایل فعال شد ✅" } else { "فیلتر فایل غیرفعال شد ❌" }).await;
        }
        "toggle_service_msg" => {
            let new_val = !settings.service_msg_filter_enabled;
            let _ = update_service_msg_filter_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.service_msg_filter_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "پاکسازی بلادرنگ اعلان‌های تلگرام فعال شد 🔔" } else { "پاکسازی اعلان‌های تلگرام غیرفعال شد 🔕" }).await;
        }
        "toggle_media" => {
            let new_val = !settings.media_filter_enabled;
            let _ = update_media_filter_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.media_filter_enabled = new_val;
            let kb = settings_locks_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "فیلتر کلی رسانه فعال شد ✅" } else { "فیلتر کلی رسانه غیرفعال شد ❌" }).await;
        }

        // تاگل‌های رهگیری کانال
        "toggle_channel_track" => {
            let new_val = !settings.channel_track_enabled;
            let _ = update_channel_track_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.channel_track_enabled = new_val;
            let kb = settings_channel_track_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "رهگیری کانال روشن شد 🟢" } else { "رهگیری کانال خاموش شد 🔴" }).await;
        }
        "toggle_track_photo" => {
            let new_val = !settings.track_photo;
            let _ = update_track_category_toggle(&pool, chat_id.0, "photo", new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.track_photo = new_val;
            let kb = settings_channel_track_words_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "رهگیری عکس فعال شد 🟢" } else { "رهگیری عکس غیرفعال شد 🔴" }).await;
        }
        "toggle_track_video" => {
            let new_val = !settings.track_video;
            let _ = update_track_category_toggle(&pool, chat_id.0, "video", new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.track_video = new_val;
            let kb = settings_channel_track_words_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "رهگیری ویدیو فعال شد 🟢" } else { "رهگیری ویدیو غیرفعال شد 🔴" }).await;
        }
        "toggle_track_voice" => {
            let new_val = !settings.track_voice;
            let _ = update_track_category_toggle(&pool, chat_id.0, "voice", new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.track_voice = new_val;
            let kb = settings_channel_track_words_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "رهگیری ویس فعال شد 🟢" } else { "رهگیری ویس غیرفعال شد 🔴" }).await;
        }
        "toggle_track_gif" => {
            let new_val = !settings.track_gif;
            let _ = update_track_category_toggle(&pool, chat_id.0, "gif", new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.track_gif = new_val;
            let kb = settings_channel_track_words_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "رهگیری گیف فعال شد 🟢" } else { "رهگیری گیف غیرفعال شد 🔴" }).await;
        }
        "toggle_track_sticker" => {
            let new_val = !settings.track_sticker;
            let _ = update_track_category_toggle(&pool, chat_id.0, "sticker", new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.track_sticker = new_val;
            let kb = settings_channel_track_words_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "رهگیری استیکر فعال شد 🟢" } else { "رهگیری استیکر غیرفعال شد 🔴" }).await;
        }
        "toggle_track_document" => {
            let new_val = !settings.track_document;
            let _ = update_track_category_toggle(&pool, chat_id.0, "document", new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.track_document = new_val;
            let kb = settings_channel_track_words_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "رهگیری فایل فعال شد 🟢" } else { "رهگیری فایل غیرفعال شد 🔴" }).await;
        }
        "toggle_track_text" => {
            let new_val = !settings.track_text;
            let _ = update_track_category_toggle(&pool, chat_id.0, "text", new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.track_text = new_val;
            let kb = settings_channel_track_words_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "رهگیری متن فعال شد 🟢" } else { "رهگیری متن غیرفعال شد 🔴" }).await;
        }
        "toggle_night" => {
            let new_val = !settings.night_mode_enabled;
            let _ = update_night_mode_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.night_mode_enabled = new_val;
            let kb = settings_night_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "قفل شبانه فعال شد ✅" } else { "قفل شبانه غیرفعال شد ❌" }).await;
        }
        "toggle_captcha" => {
            let new_val = !settings.captcha_enabled;
            let _ = update_captcha_toggle(&pool, chat_id.0, new_val).await;
            cache.invalidate_settings(chat_id.0);
            settings.captcha_enabled = new_val;
            let kb = settings_captcha_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(if new_val { "کپچای عضویت فعال شد ✅" } else { "کپچای عضویت غیرفعال شد ❌" }).await;
        }

        // تنظیم مقادیر عددی فلاد
        "set_rate_3" | "set_rate_5" | "set_rate_7" | "set_rate_10" => {
            let rate = match data {
                "set_rate_3" => 3,
                "set_rate_7" => 7,
                "set_rate_10" => 10,
                _ => 5,
            };
            let _ = update_anti_flood_params(&pool, chat_id.0, rate, settings.anti_flood_window_secs).await;
            cache.invalidate_settings(chat_id.0);
            settings.anti_flood_rate = rate;
            let kb = settings_flood_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(format!("حساسیت به {} پیام تنظیم شد.", rate)).await;
        }
        "set_win_3" | "set_win_5" | "set_win_10" | "set_win_15" => {
            let win = match data {
                "set_win_3" => 3,
                "set_win_10" => 10,
                "set_win_15" => 15,
                _ => 5,
            };
            let _ = update_anti_flood_params(&pool, chat_id.0, settings.anti_flood_rate, win).await;
            cache.invalidate_settings(chat_id.0);
            settings.anti_flood_window_secs = win;
            let kb = settings_flood_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(format!("بازه زمانی ضداسپم به {} ثانیه تنظیم شد.", win)).await;
        }

        // تنظیم ساعات حالت شب
        "set_night_22_06" | "set_night_23_07" | "set_night_00_08" | "set_night_01_09" => {
            let (s, e, desc) = match data {
                "set_night_22_06" => (NaiveTime::from_hms_opt(22, 0, 0).unwrap(), NaiveTime::from_hms_opt(6, 0, 0).unwrap(), "۲۲:۰۰ تا ۰۶:۰۰"),
                "set_night_00_08" => (NaiveTime::from_hms_opt(0, 0, 0).unwrap(), NaiveTime::from_hms_opt(8, 0, 0).unwrap(), "۰۰:۰۰ تا ۰۸:۰۰"),
                "set_night_01_09" => (NaiveTime::from_hms_opt(1, 0, 0).unwrap(), NaiveTime::from_hms_opt(9, 0, 0).unwrap(), "۰۱:۰۰ تا ۰۹:۰۰"),
                _ => (NaiveTime::from_hms_opt(23, 0, 0).unwrap(), NaiveTime::from_hms_opt(7, 0, 0).unwrap(), "۲۳:۰۰ تا ۰۷:۰۰"),
            };
            let _ = update_night_mode_hours(&pool, chat_id.0, s, e).await;
            cache.invalidate_settings(chat_id.0);
            settings.night_mode_start = s;
            settings.night_mode_end = e;
            let kb = settings_night_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(format!("ساعات قفل شب به {} تغییر یافت.", desc)).await;
        }

        // تنظیمات قفل گروه (سراسری / کامنت آزاد)
        "set_lock_comments" => {
            let _ = update_group_lock_mode(&pool, chat_id.0, "comments_only").await;
            cache.invalidate_settings(chat_id.0);
            settings.group_lock_mode = "comments_only".to_string();
            let kb = group_lock_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text("💬 حالت «کامنت آزاد» فعال شد.").show_alert(true).await;
            let announce = format!("{RLM}💬 <b>قفل گروه در حالت «کامنت آزاد» قرار گرفت.</b>\nکاربران تنها می‌توانند بر روی پست‌های کانال متصل نظر (کامنت) ثبت کنند و ارسال پیام عادی در گروه مسدود است.");
            let _ = send_styled_message(&bot, chat_id, &announce, None).await;
        }
        "set_lock_total" => {
            let _ = update_group_lock_mode(&pool, chat_id.0, "total").await;
            cache.invalidate_settings(chat_id.0);
            settings.group_lock_mode = "total".to_string();
            let kb = group_lock_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text("🔒 «قفل سراسری» فعال شد.").show_alert(true).await;
            let announce = format!("{RLM}🔒 <b>«قفل سراسری» بر روی گروه فعال شد.</b>\nارسال هرگونه پیام در گروه تا اطلاع ثانوی مسدود است.");
            let _ = send_styled_message(&bot, chat_id, &announce, None).await;
        }
        "set_lock_unlocked" => {
            let _ = update_group_lock_mode(&pool, chat_id.0, "unlocked").await;
            cache.invalidate_settings(chat_id.0);
            settings.group_lock_mode = "unlocked".to_string();
            let kb = group_lock_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text("🔓 قفل گروه برداشته شد.").show_alert(true).await;
            let announce = format!("{RLM}🔓 <b>قفل گروه برداشته شد.</b>\nارسال پیام برای تمامی اعضا آزاد است.");
            let _ = send_styled_message(&bot, chat_id, &announce, None).await;
        }

        // تنظیم مهلت کپچا
        "set_cap_30" | "set_cap_60" | "set_cap_120" | "set_cap_180" => {
            let secs = match data {
                "set_cap_30" => 30,
                "set_cap_60" => 60,
                "set_cap_180" => 180,
                _ => 120,
            };
            let _ = update_captcha_timeout(&pool, chat_id.0, secs).await;
            cache.invalidate_settings(chat_id.0);
            settings.captcha_timeout_secs = secs;
            let kb = settings_captcha_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(format!("مهلت حل کپچا به {} ثانیه تغییر یافت.", secs)).await;
        }

        // تنظیم سقف اخطارها
        "set_warn_3" | "set_warn_5" | "set_warn_10" => {
            let limit = match data {
                "set_warn_5" => 5,
                "set_warn_10" => 10,
                _ => 3,
            };
            let _ = update_warn_limit(&pool, chat_id.0, limit).await;
            cache.invalidate_settings(chat_id.0);
            settings.warn_limit = limit;
            let kb = settings_warn_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(format!("سقف اخطار به {} تغییر یافت.", limit)).await;
        }

        // چرخش اقدامات تنبیهی (Action cycling)
        "cycle_flood_action" => {
            let next_act = match settings.anti_flood_action.as_str() {
                "mute_5m" => "mute_1h",
                "mute_1h" => "kick",
                "kick" => "ban",
                "ban" => "warn",
                _ => "mute_5m",
            };
            let _ = update_anti_flood_action(&pool, chat_id.0, next_act).await;
            cache.invalidate_settings(chat_id.0);
            settings.anti_flood_action = next_act.to_string();
            let kb = settings_flood_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(format!("تنبیه فلاد: {}", flood_action_display(next_act))).await;
        }
        "cycle_warn_action" => {
            let next_act = match settings.warn_escalation_action.as_str() {
                "mute_24h" => "ban",
                "ban" => "kick",
                "kick" => "mute_1h",
                _ => "mute_24h",
            };
            let _ = update_warn_escalation_action(&pool, chat_id.0, next_act).await;
            cache.invalidate_settings(chat_id.0);
            settings.warn_escalation_action = next_act.to_string();
            let kb = settings_warn_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(format!("تنبیه سقف اخطار: {}", warn_action_display(next_act))).await;
        }
        "cycle_captcha_action" => {
            let next_act = match settings.captcha_fail_action.as_str() {
                "kick" => "ban",
                "ban" => "mute",
                _ => "kick",
            };
            let _ = update_captcha_fail_action(&pool, chat_id.0, next_act).await;
            cache.invalidate_settings(chat_id.0);
            settings.captcha_fail_action = next_act.to_string();
            let kb = settings_captcha_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(format!("تنبیه عدم حل کپچا: {}", captcha_action_display(next_act))).await;
        }
        "cycle_link_action" => {
            let next_act = match settings.link_filter_action.as_str() {
                "delete" => "warn",
                "warn" => "mute",
                _ => "delete",
            };
            let _ = update_link_filter_action(&pool, chat_id.0, next_act).await;
            cache.invalidate_settings(chat_id.0);
            settings.link_filter_action = next_act.to_string();
            let kb = settings_actions_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text(format!("اقدام فیلتر لینک: {}", link_action_display(next_act))).await;
        }

        // پیش‌نمایش‌ها و بازنشانی
        "preview_welcome" => {
            let preview = format_template(
                &settings.welcome_text,
                &user.first_name,
                user.username.as_deref(),
                user.id.0,
                msg.chat().title().unwrap_or("گروه"),
            );
            let text = format!("{RLM}👁️ <b>پیش‌نمایش پیام خوشامدگویی:</b>\n{}", preview);
            let _ = send_styled_message(&bot, chat_id, &text, None).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "preview_goodbye" => {
            let preview = format_template(
                &settings.goodbye_text,
                &user.first_name,
                user.username.as_deref(),
                user.id.0,
                msg.chat().title().unwrap_or("گروه"),
            );
            let text = format!("{RLM}👁️ <b>پیش‌نمایش پیام خروج:</b>\n{}", preview);
            let _ = send_styled_message(&bot, chat_id, &text, None).await;
            let _ = bot.answer_callback_query(q.id).await;
        }
        "reset_welcome" => {
            let default_text = "سلام {mention}، به گروه {group} خوش آمدید! 🌸";
            let _ = set_welcome_text(&pool, chat_id.0, default_text).await;
            cache.invalidate_settings(chat_id.0);
            let _ = bot.answer_callback_query(q.id).text("متن خوشامدگویی به حالت پیش‌فرض بازنشانی شد.").show_alert(true).await;
        }
        "help_welcome_text" => {
            let guide = "ℹ️ برای تغییر متن خوشامد یا خروج، دستور زیر را در گروه بفرستید:\n\nتنظیم متن خوشامد [متن مورد نظر]\n\nمتغیرهای قابل استفاده:\n• {name} ➔ نام کاربر\n• {mention} ➔ منشن کاربر\n• {group} ➔ نام گروه\n• {user_id} ➔ آیدی عددی کاربر";
            let _ = bot.answer_callback_query(q.id).text(guide).show_alert(true).await;
        }

        // عضویت اجباری کانال
        "clear_forced_channel" => {
            let _ = update_forced_channel(&pool, chat_id.0, None).await;
            cache.invalidate_settings(chat_id.0);
            settings.forced_channel = None;
            let kb = settings_channel_styled(&settings);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text("عضویت اجباری کانال حذف شد.").await;
        }
        "check_channel" => {
            if let Some(ref ch) = settings.forced_channel {
                let formatted = if ch.starts_with('@') { ch.clone() } else { format!("@{}", ch) };
                let me = bot.get_me().await;
                match me {
                    Ok(bot_info) => {
                        let member = bot.get_chat_member(Recipient::ChannelUsername(formatted.clone()), bot_info.id).await;
                        match member {
                            Ok(m) => {
                                if m.status() == ChatMemberStatus::Administrator {
                                    let _ = bot.answer_callback_query(q.id).text("✅ ربات در کانال مدیر است و بررسی عضویت فعال است.").show_alert(true).await;
                                } else {
                                    let _ = bot.answer_callback_query(q.id).text("⚠️ ربات در کانال حضور دارد ولی مدیر نیست! لطفاً به ربات دسترسی ادمین بدهید.").show_alert(true).await;
                                }
                            }
                            Err(_) => {
                                let _ = bot.answer_callback_query(q.id).text("❌ ربات در کانال عضو نیست یا آیدی کانال اشتباه است.").show_alert(true).await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = bot.answer_callback_query(q.id).text(format!("خطا: {}", e)).await;
                    }
                }
            } else {
                let _ = bot.answer_callback_query(q.id).text("کانالی تنظیم نشده است.").await;
            }
        }
        "guide_channel" => {
            let guide = "📢 راهنمای عضویت اجباری کانال:\n۱. ربات را در کانال خود مدیر (ادمین) کنید.\n۲. دستور زیر را در گروه بفرستید:\nتنظیم کانال @آیدی_کانال";
            let _ = bot.answer_callback_query(q.id).text(guide).show_alert(true).await;
        }

        // لیست‌ها و پاکسازی‌های کلمات ممنوعه، دامنه‌ها و دستورات
        "list_banned_words" => {
            let words = get_banned_words(&pool, chat_id.0).await.unwrap_or_default();
            if words.is_empty() {
                let _ = bot.answer_callback_query(q.id).text("هیچ کلمه ممنوعه‌ای برای این گروه ثبت نشده است.").show_alert(true).await;
            } else {
                let mut out = format!("{RLM}📋 <b>لیست کلمات ممنوعه گروه:</b>\n");
                for (w, list_type) in words {
                    let action = if list_type == 2 { "اخطار + حذف" } else { "حذف پیام" };
                    out.push_str(&format!("• <code>{}</code> ({})\n", html_escape(&w), action));
                }
                let _ = send_styled_message(&bot, chat_id, &out, None).await;
                let _ = bot.answer_callback_query(q.id).await;
            }
        }
        "clear_all_banned_words" => {
            let _ = clear_all_banned_words(&pool, chat_id.0).await;
            cache.invalidate_banned_words(chat_id.0);
            let kb = settings_words_styled(0);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text("تمامی کلمات ممنوعه گروه پاکسازی شدند.").show_alert(true).await;
        }
        "guide_banned_words" => {
            let guide = "📋 نحوه افزودن/حذف کلمه ممنوعه:\n• افزودن: افزودن کلمه ممنوعه [کلمه]\n• حذف: حذف کلمه ممنوعه [کلمه]";
            let _ = bot.answer_callback_query(q.id).text(guide).show_alert(true).await;
        }

        "list_allowed_domains" => {
            let domains = get_allowed_domains(&pool, chat_id.0).await.unwrap_or_default();
            if domains.is_empty() {
                let _ = bot.answer_callback_query(q.id).text("هیچ دامنه مجازی ثبت نشده است.").show_alert(true).await;
            } else {
                let mut out = format!("{RLM}🌐 <b>لیست دامنه‌های مجاز گروه:</b>\n");
                for d in domains {
                    out.push_str(&format!("• <code>{}</code>\n", html_escape(&d)));
                }
                let _ = send_styled_message(&bot, chat_id, &out, None).await;
                let _ = bot.answer_callback_query(q.id).await;
            }
        }
        "clear_all_domains" => {
            let _ = clear_all_allowed_domains(&pool, chat_id.0).await;
            cache.invalidate_allowed_domains(chat_id.0);
            let kb = settings_domains_styled(0);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text("تمامی دامنه‌های مجاز پاکسازی شدند.").show_alert(true).await;
        }
        "guide_allowed_domains" => {
            let guide = "🌐 نحوه افزودن/حذف دامنه مجاز:\n• افزودن: افزودن لینک مجاز [example.com]\n• حذف: حذف لینک مجاز [example.com]";
            let _ = bot.answer_callback_query(q.id).text(guide).show_alert(true).await;
        }

        "list_custom_cmds_btn" => {
            let list = list_custom_commands(&pool, chat_id.0).await.unwrap_or_default();
            if list.is_empty() {
                let _ = bot.answer_callback_query(q.id).text("هیچ دستور دلخواهی تعریف نشده است.").show_alert(true).await;
            } else {
                let mut out = format!("{RLM}✨ <b>دستورات دلخواه تعریف‌شده برای این گروه:</b>\n");
                for (trig, resp) in list {
                    out.push_str(&format!("• <code>{}</code> ➔ {}\n", html_escape(&trig), html_escape(&resp)));
                }
                let _ = send_styled_message(&bot, chat_id, &out, None).await;
                let _ = bot.answer_callback_query(q.id).await;
            }
        }
        "clear_all_custom_cmds" => {
            let _ = clear_all_custom_commands(&pool, chat_id.0).await;
            let kb = settings_custom_cmds_styled(0);
            let _ = edit_styled_keyboard(&bot, msg_chat_id, msg.id(), &kb).await;
            let _ = bot.answer_callback_query(q.id).text("تمامی دستورات دلخواه حذف شدند.").show_alert(true).await;
        }
        "guide_custom_cmds" => {
            let guide = "✨ نحوه تعریف دستور جدید:\nافزودن دستور [دستور] | [پاسخ ربات]\nمثال:\nافزودن دستور پشتیبانی | برای ارتباط به @admin پیام دهید";
            let _ = bot.answer_callback_query(q.id).text(guide).show_alert(true).await;
        }

        "clear_all_warnings" => {
            let _ = clear_all_warnings_for_group(&pool, chat_id.0).await;
            let _ = bot.answer_callback_query(q.id).text("تمامی اخطارهای فعال کاربران در این گروه پاکسازی شدند.").show_alert(true).await;
        }

        // اعلان‌های توضیحی دکمه‌های آمار/وضعیت
        "channel_status_toast" => {
            let status = settings.forced_channel.as_deref().unwrap_or("تنظیم نشده است");
            let _ = bot.answer_callback_query(q.id).text(format!("کانال عضویت اجباری: {}", status)).await;
        }
        "night_status_toast" => {
            let active = cache.is_night_mode_on(chat_id.0);
            let txt = if active { "🌙 در حال حاضر قفل شبانه فعال است" } else { "☀️ در حال حاضر قفل شبانه غیرفعال است" };
            let _ = bot.answer_callback_query(q.id).text(txt).await;
        }
        "night_tz_toast" => {
            let _ = bot.answer_callback_query(q.id).text("منطقه زمانی سامانه بر اساس ساعت رسمی تهران محاسبه می‌شود.").await;
        }
        "words_status_toast" => {
            let words = get_banned_words(&pool, chat_id.0).await.unwrap_or_default();
            let _ = bot.answer_callback_query(q.id).text(format!("تعداد کلمات ممنوعه ثبت‌شده: {}", words.len())).await;
        }
        "domains_status_toast" => {
            let domains = get_allowed_domains(&pool, chat_id.0).await.unwrap_or_default();
            let _ = bot.answer_callback_query(q.id).text(format!("تعداد دامنه‌های مجاز: {}", domains.len())).await;
        }
        "cmds_status_toast" => {
            let cmds = list_custom_commands(&pool, chat_id.0).await.unwrap_or_default();
            let _ = bot.answer_callback_query(q.id).text(format!("تعداد دستورات دلخواه: {}", cmds.len())).await;
        }

        "noop" => {
            let _ = bot.answer_callback_query(q.id).await;
        }

        _ => {
            let _ = bot.answer_callback_query(q.id).text("گزینه انتخاب شد.").await;
        }
    }

    Ok(())
}

async fn get_settings_counts(pool: &PgPool, chat_id: i64) -> SettingsCounts {
    let words = get_banned_words(pool, chat_id).await.map(|w| w.len()).unwrap_or(0);
    let domains = get_allowed_domains(pool, chat_id).await.map(|d| d.len()).unwrap_or(0);
    let custom_cmds = list_custom_commands(pool, chat_id).await.map(|c| c.len()).unwrap_or(0);
    SettingsCounts { words, domains, custom_cmds }
}
