use chrono::Duration;
use crate::engine::normalizer::{html_escape, to_persian_num, RLM};
use crate::engine::target::TargetUser;
use crate::i18n::get_i18n;

/// متن راهنمای جامع ربات با جدول‌های تفکیک‌شده درون آکاردئون‌های تعاملی و چینش وسط (Bot API 10.1 / 10.2)
pub fn help_message() -> String {
    get_i18n().help.clone()
}

/// پیام آغازین در پیوی ربات با استایل Rich Message
pub fn start_private_message() -> String {
    get_i18n().start.text.clone()
}

/// فرمت‌بندی فارسی مدت زمان به ساعت و روز
pub fn format_duration_persian(d: Duration) -> String {
    let mins = d.num_minutes();
    if mins >= 1440 && mins % 1440 == 0 {
        format!("{} روز", to_persian_num(mins / 1440))
    } else if mins >= 60 {
        let hours = mins / 60;
        let rem_mins = mins % 60;
        if rem_mins == 0 {
            format!("{} ساعت", to_persian_num(hours))
        } else {
            format!("{} ساعت و {} دقیقه", to_persian_num(hours), to_persian_num(rem_mins))
        }
    } else {
        format!("{} دقیقه", to_persian_num(mins))
    }
}

fn pick_random_poem(pool: &[String]) -> &str {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let time_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0);
    let idx = (count ^ time_seed) % pool.len();
    &pool[idx]
}

/// اعلان مسدودسازی حماسی (Ban) همراه با بیت تصادفی از شاهنامه فردوسی
pub fn ban_notice(target: &TargetUser, duration: Option<Duration>, reason: Option<&str>) -> String {
    let clean_reason = reason.map(|r| r.trim()).filter(|r| !r.is_empty() && *r != "ذکر نشده");
    let dur_str = duration.map(format_duration_persian);
    let target_mention = target.mention_display();
    let i18n = get_i18n();
    let poem = pick_random_poem(&i18n.poems.ban);

    let outcome_line = match dur_str {
        Some(dur) => i18n.notices.ban.outcome_timed.replace("{duration}", &dur),
        None => i18n.notices.ban.outcome_permanent.clone(),
    };

    let reason_part = match clean_reason {
        Some(r) => i18n.notices.ban.reason_template.replace("{reason}", &html_escape(r)),
        None => String::new(),
    };

    let title = &i18n.notices.ban.title;
    let author = &i18n.notices.ban.quote_author;

    format!(
        "{title}\n\n\
        👤 <b>کاربر:</b> {target_mention}\n\
        {outcome_line}\
        {reason_part}\n\n\
        {RLM}📖 <i>«{poem}»</i>\n\
        — <b>{author}</b>"
    )
}

/// اعلان سکوت حماسی (Mute) همراه با بیت تصادفی از شاهنامه فردوسی
pub fn mute_notice(target: &TargetUser, duration: Option<Duration>, reason: Option<&str>) -> String {
    let clean_reason = reason.map(|r| r.trim()).filter(|r| !r.is_empty() && *r != "ذکر نشده");
    let dur_str = duration.map(format_duration_persian);
    let target_mention = target.mention_display();
    let i18n = get_i18n();
    let poem = pick_random_poem(&i18n.poems.mute);

    let dur_part = match dur_str {
        Some(dur) => i18n.notices.mute.duration_timed.replace("{duration}", &dur),
        None => i18n.notices.mute.duration_permanent.clone(),
    };

    let reason_part = match clean_reason {
        Some(r) => i18n.notices.mute.reason_template.replace("{reason}", &html_escape(r)),
        None => String::new(),
    };

    let title = &i18n.notices.mute.title;
    let author = &i18n.notices.mute.quote_author;

    format!(
        "{title}\n\n\
        👤 <b>کاربر:</b> {target_mention}\
        {dur_part}\
        {reason_part}\n\n\
        {RLM}📖 <i>«{poem}»</i>\n\
        — <b>{author}</b>"
    )
}

/// اعلان ثبت اخطار حماسی (Warn) همراه با بیت تصادفی از شاهنامه فردوسی
pub fn warn_notice(target: &TargetUser, current: i64, limit: i32, reason: Option<&str>) -> String {
    let clean_reason = reason.map(|r| r.trim()).filter(|r| !r.is_empty() && *r != "ذکر نشده" && *r != "نقض قوانین");
    let target_mention = target.mention_display();
    let count_display = format!("{} از {}", to_persian_num(current), to_persian_num(limit));
    let i18n = get_i18n();
    let poem = pick_random_poem(&i18n.poems.warn);

    let count_line = i18n.notices.warn.count_template.replace("{count}", &count_display);

    let reason_part = match clean_reason {
        Some(r) => i18n.notices.warn.reason_template.replace("{reason}", &html_escape(r)),
        None => String::new(),
    };

    let title = &i18n.notices.warn.title;
    let author = &i18n.notices.warn.quote_author;

    format!(
        "{title}\n\n\
        👤 <b>کاربر:</b> {target_mention}\n\
        {count_line}\
        {reason_part}\n\n\
        {RLM}📖 <i>«{poem}»</i>\n\
        — <b>{author}</b>"
    )
}

/// اعلان اخراج حماسی (Kick) همراه با بیت تصادفی از شاهنامه فردوسی
pub fn kick_notice(target: &TargetUser, reason: Option<&str>) -> String {
    let clean_reason = reason.map(|r| r.trim()).filter(|r| !r.is_empty() && *r != "ذکر نشده");
    let target_mention = target.mention_display();
    let i18n = get_i18n();
    let poem = pick_random_poem(&i18n.poems.kick);

    let outcome_line = &i18n.notices.kick.outcome;

    let reason_part = match clean_reason {
        Some(r) => i18n.notices.kick.reason_template.replace("{reason}", &html_escape(r)),
        None => String::new(),
    };

    let title = &i18n.notices.kick.title;
    let author = &i18n.notices.kick.quote_author;

    format!(
        "{title}\n\n\
        👤 <b>کاربر:</b> {target_mention}\n\
        {outcome_line}\
        {reason_part}\n\n\
        {RLM}📖 <i>«{poem}»</i>\n\
        — <b>{author}</b>"
    )
}

pub fn unban_notice(target_mention: &str) -> String {
    get_i18n().notices.unban.replace("{mention}", target_mention)
}

pub fn unmute_notice(target_mention: &str) -> String {
    get_i18n().notices.unmute.replace("{mention}", target_mention)
}

/// قالب‌بندی متغیرهای خوشامدگویی با پشتیبانی از RTL
pub fn format_template(
    template: &str,
    user_name: &str,
    username: Option<&str>,
    user_id: u64,
    chat_title: &str,
) -> String {
    let escaped_name = html_escape(user_name);
    let escaped_group = html_escape(chat_title);
    let uname = username.map_or("".to_string(), |u| format!("@{}", html_escape(u)));
    let mention = if !uname.is_empty() {
        format!("<b>{}</b>", uname)
    } else {
        format!("<a href=\"tg://user?id={}\">{}</a>", user_id, escaped_name)
    };

    let formatted = template
        .replace("{name}", &escaped_name)
        .replace("{username}", &uname)
        .replace("{id}", &to_persian_num(user_id))
        .replace("{group}", &escaped_group)
        .replace("{mention}", &mention);

    format!("{RLM}{formatted}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use teloxide::types::UserId;

    #[test]
    fn test_mute_notice_cases() {
        let target = TargetUser {
            id: UserId(12345),
            first_name: "علی".to_string(),
            username: Some("ali".to_string()),
        };

        // حالت ۱: بدون علت و زمان
        let n1 = mute_notice(&target, None, None);
        assert!(n1.contains("کاربر:</b> <b>@ali</b>"));
        assert!(n1.contains("مُهر خاموشی گردآفرید"));
        assert!(n1.contains("درنگ و تامل:</b> نامحدود"));
        assert!(!n1.contains("انگیزه حکم:</b>"));
        assert!(!n1.contains("<blockquote>"));
        assert!(n1.contains("فردوسی بزرگ"));

        // حالت ۲: فقط زمان
        let n2 = mute_notice(&target, Some(Duration::hours(2)), None);
        assert!(n2.contains("کاربر:</b> <b>@ali</b>"));
        assert!(n2.contains("درنگ و تامل:</b> ۲ ساعت"));
        assert!(!n2.contains("انگیزه حکم:</b>"));

        // حالت ۳: فقط علت
        let n3 = mute_notice(&target, None, Some("اسپم"));
        assert!(n3.contains("کاربر:</b> <b>@ali</b>"));
        assert!(n3.contains("انگیزه حکم:</b> اسپم"));

        // حالت ۴: علت و زمان
        let n4 = mute_notice(&target, Some(Duration::days(1)), Some("تبلیغات"));
        assert!(n4.contains("کاربر:</b> <b>@ali</b>"));
        assert!(n4.contains("درنگ و تامل:</b> ۱ روز"));
        assert!(n4.contains("انگیزه حکم:</b> تبلیغات"));
    }

    #[test]
    fn test_ban_notice_cases() {
        let target = TargetUser {
            id: UserId(12345),
            first_name: "علی".to_string(),
            username: Some("ali".to_string()),
        };

        // حالت ۱: بدون علت و زمان
        let b1 = ban_notice(&target, None, None);
        assert!(b1.contains("کاربر:</b> <b>@ali</b>"));
        assert!(b1.contains("فرمان راندن از دژ گردآفرید"));
        assert!(b1.contains("اخراج و مسدودسازی همیشگی"));
        assert!(!b1.contains("انگیزه حکم:</b>"));
        assert!(!b1.contains("<blockquote>"));
        assert!(b1.contains("فردوسی بزرگ"));

        // حالت ۲: فقط زمان
        let b2 = ban_notice(&target, Some(Duration::hours(2)), None);
        assert!(b2.contains("کاربر:</b> <b>@ali</b>"));
        assert!(b2.contains("مسدودسازی موقت به مدت ۲ ساعت"));
        assert!(!b2.contains("انگیزه حکم:</b>"));

        // حالت ۳: فقط علت
        let b3 = ban_notice(&target, None, Some("اسپم"));
        assert!(b3.contains("کاربر:</b> <b>@ali</b>"));
        assert!(b3.contains("انگیزه حکم:</b> اسپم"));

        // حالت ۴: علت و زمان
        let b4 = ban_notice(&target, Some(Duration::days(1)), Some("تبلیغات"));
        assert!(b4.contains("کاربر:</b> <b>@ali</b>"));
        assert!(b4.contains("مسدودسازی موقت به مدت ۱ روز"));
        assert!(b4.contains("انگیزه حکم:</b> تبلیغات"));
    }

    #[test]
    fn test_kick_and_warn_notice_cases() {
        let target = TargetUser {
            id: UserId(12345),
            first_name: "علی".to_string(),
            username: Some("ali".to_string()),
        };

        let k1 = kick_notice(&target, None);
        assert!(k1.contains("کاربر:</b> <b>@ali</b>"));
        assert!(k1.contains("اخراج از دژ گردآفرید"));
        assert!(k1.contains("اخراج از انجمن"));
        assert!(!k1.contains("انگیزه حکم:</b>"));
        assert!(!k1.contains("<blockquote>"));
        assert!(k1.contains("فردوسی بزرگ"));

        let k2 = kick_notice(&target, Some("اسپم"));
        assert!(k2.contains("کاربر:</b> <b>@ali</b>"));
        assert!(k2.contains("انگیزه حکم:</b> اسپم"));

        let w1 = warn_notice(&target, 2, 3, None);
        assert!(w1.contains("کاربر:</b> <b>@ali</b>"));
        assert!(w1.contains("هشدار انضباطی گردآفرید"));
        assert!(w1.contains("شمار اخطارها:</b> ۲ از ۳"));
        assert!(!w1.contains("انگیزه حکم:</b>"));
        assert!(!w1.contains("<blockquote>"));
        assert!(w1.contains("فردوسی بزرگ"));

        let w2 = warn_notice(&target, 1, 3, Some("توهین"));
        assert!(w2.contains("کاربر:</b> <b>@ali</b>"));
        assert!(w2.contains("انگیزه حکم:</b> توهین"));
    }
}
