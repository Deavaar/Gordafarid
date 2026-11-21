use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::{ChatPermissions, Message, User};
use crate::db::groups::GroupSettings;
use crate::engine::normalizer::{html_escape, to_persian_num, RLM};
use crate::engine::target::TargetUser;
use crate::services::cache::AppCache;
use crate::ui::keyboards::captcha_math_styled;
use crate::ui::messages::format_template;
use crate::ui::styled_keyboard::send_styled_message;

pub async fn handle_new_chat_members(
    bot: &Bot,
    msg: &Message,
    pool: &PgPool,
    cache: &AppCache,
    settings: &GroupSettings,
    new_members: &[User],
) {
    let chat_id = msg.chat.id;
    let chat_title = msg.chat.title().unwrap_or("گروه");

    // حذف بلادرنگ پیام اعلان پیوستن به گروه تلگرام (مانند Ali joined the group)
    if settings.service_msg_filter_enabled {
        let _ = bot.delete_message(chat_id, msg.id).await;
    }

    for member in new_members {
        let target = TargetUser {
            id: member.id,
            first_name: member.first_name.clone(),
            username: member.username.clone(),
        };
        cache.record_user(&target);
        let _ = crate::db::users::upsert_user(
            pool,
            member.id.0 as i64,
            &member.first_name,
            member.username.as_deref(),
        )
        .await;

        if member.is_bot {
            continue;
        }

        if settings.captcha_enabled {
            // سلب موقت دسترسی تا زمان حل کپچا
            let empty_perms = ChatPermissions::empty();
            let _ = bot.restrict_chat_member(chat_id, member.id, empty_perms).await;

            // ایجاد کپچای محاسباتی ساده یا دکمه‌ای
            let a = (member.id.0 % 5 + 2) as i32;
            let b = (member.id.0 % 4 + 1) as i32;
            let sum = a + b;

            let kb = captcha_math_styled(member.id.0, sum);
            let prompt = crate::i18n::get_i18n().captcha.prompt
                .replace("{user_id}", &member.id.0.to_string())
                .replace("{name}", &html_escape(&member.first_name))
                .replace("{group}", &html_escape(chat_title))
                .replace("{a}", &to_persian_num(a))
                .replace("{b}", &to_persian_num(b))
                .replace("{timeout}", &to_persian_num(settings.captcha_timeout_secs))
                .replace("{RLM}", RLM);

            let _ = send_styled_message(bot, chat_id, &prompt, Some(&kb)).await;
        } else if settings.welcome_enabled {
            let welcome = format_template(
                &settings.welcome_text,
                &member.first_name,
                member.username.as_deref(),
                member.id.0,
                chat_title,
            );
            let _ = send_styled_message(bot, chat_id, &welcome, None).await;
        }
    }
}

pub async fn handle_left_chat_member(
    bot: &Bot,
    msg: &Message,
    settings: &GroupSettings,
    left_user: &User,
) {
    if settings.service_msg_filter_enabled {
        let _ = bot.delete_message(msg.chat.id, msg.id).await;
    }

    if settings.goodbye_enabled {
        let chat_title = msg.chat.title().unwrap_or("گروه");
        let goodbye = format_template(
            &settings.goodbye_text,
            &left_user.first_name,
            left_user.username.as_deref(),
            left_user.id.0,
            chat_title,
        );
        let _ = send_styled_message(bot, msg.chat.id, &goodbye, None).await;
    }
}
