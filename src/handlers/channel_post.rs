use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::Message;
use tracing::{info, warn};

use crate::engine::channel_tracker::{directive_fallback_notice, extract_explicit_directive};
use crate::services::cache::AppCache;

pub async fn handle_channel_post(
    bot: Bot,
    msg: Message,
    pool: PgPool,
    cache: AppCache,
) -> Result<(), teloxide::RequestError> {
    let post_text = msg.text().or_else(|| msg.caption()).unwrap_or("");

    // بررسی دستور مستقیم ادمین در کانال (مانند: "تنظیم گیف" یا "تنظیم همه بجز فیلم")
    if let Some(directive) = extract_explicit_directive(post_text) {
        info!(
            "📢 دستور مستقیم کانال در پست {} چنل {} شناسایی شد: نوع '{}'",
            msg.id.0,
            msg.chat.id.0,
            directive.rule
        );

        // ذخیره در کش برای استفاده هنگام فوروارد خودکار به گروه متصل
        cache.set_post_rule(msg.chat.id.0, msg.id.0, directive.rule.to_string());
        let _ = crate::db::groups::save_channel_post_rule(&pool, msg.chat.id.0, msg.id.0, directive.rule).await;

        // ویرایش پیام کانال و حذف عبارت دستور
        let cleaned = directive.cleaned_text.trim();
        if msg.text().is_some() {
            let new_text = if cleaned.is_empty() {
                directive_fallback_notice(directive.rule)
            } else {
                cleaned.to_string()
            };
            if let Err(e) = bot.edit_message_text(msg.chat.id, msg.id, new_text).await {
                warn!("عدم امکان ویرایش متن پست کانال {}: {}", msg.id.0, e);
            }
        } else if msg.caption().is_some() {
            if let Err(e) = bot.edit_message_caption(msg.chat.id, msg.id).caption(cleaned).await {
                warn!("عدم امکان ویرایش کپشن پست کانال {}: {}", msg.id.0, e);
            }
        }
    }

    Ok(())
}
