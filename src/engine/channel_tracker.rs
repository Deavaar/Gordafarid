use crate::db::groups::GroupSettings;
use crate::engine::normalizer::normalize_persian_text;

/// فهرست واژگان کلیدی انواع محتوا
pub const PHOTO_KEYWORDS: &[&str] = &[
    "عکس", "تصویر", "فوتو", "photo", "اسکرین", "اسکرین شات", "اسکرین‌شات", "شات",
    "عکساتون", "عکساتونو", "عکسا تون", "عکسهاتون", "عکسهاتونو", "تصاویر", "والپیپر",
    "بک گراند", "بک‌گراند", "عکسات", "عکس‌هاتون", "عکستون",
];

pub const VIDEO_KEYWORDS: &[&str] = &[
    "ویدیو", "ویدئو", "فیلم", "کلیپ", "مووی", "video", "ویدیوهاتون", "ویدیوهاتو",
    "ویدیو هاتون", "فیلماتون", "فیلماتونو", "فیلم‌هاتون", "کلیپاتون", "کلیپ‌هاتون",
    "ویدیوهات", "فیلمات",
];

pub const VOICE_KEYWORDS: &[&str] = &[
    "ویس", "صدا", "وویس", "صوت", "voice", "ویساتون", "ویساتونو", "صداتون", "صداتونو",
    "ویس‌هاتون", "وویساتون", "وویس‌هاتون", "پادکست", "صوتتون", "صوت‌تون",
];

pub const GIF_KEYWORDS: &[&str] = &[
    "گیف", "انیمیشن", "gif", "گیفاتون", "گیف‌هاتون", "گیف هاتون", "گیفاتونو", "متحرک",
];

pub const STICKER_KEYWORDS: &[&str] = &[
    "استیکر", "sticker", "استیکراتون", "استیکرهاتون", "استیکر هاتون", "استیکراتونو",
];

pub const DOCUMENT_KEYWORDS: &[&str] = &[
    "فایل", "سند", "داکیومنت", "پی دی اف", "پی‌دی‌اف", "pdf", "file", "فایل‌هاتون",
    "فایل هاتون", "فایلاتون", "فایلاتونو", "سورس", "سورس کد", "سورس‌کد", "آرشیو", "زیپ", "zip",
];

pub const TEXT_KEYWORDS: &[&str] = &[
    "متن", "تکست", "پیام", "نظر", "نظرات", "کامنت", "text", "متناتون", "تکستاتون",
    "پیاماتون", "پیام‌هاتون", "نظراتتون", "دیدگاه", "نقد", "تجربه", "تجربه‌هاتون",
];

/// بیش از ۱۰۰ نوع الگوی درخواستی، امری، ضمیری و چالشی به زبان فارسی
pub const CTA_PATTERNS: &[&str] = &[
    // ۱. افعال امری مستقیم و محاوره‌ای
    "بدید", "بدین", "بده", "بدهید", "بدیدش", "بدینش", "میدید", "میدین", "بدن",
    "بفرستید", "بفرستین", "بفرست", "بفرستیدش", "بفرستینش", "میفرستید", "میفرستین", "بفرستن",
    "ارسال کنید", "ارسال کنین", "ارسال کن", "ارسال فرمایید", "ارسال بنمایید", "ارسال نمایید", "ارسال شود",
    "بذارید", "بذارین", "بذار", "بزارید", "بزارین", "بزار", "بگذارید", "بگذارین", "بذاریدش", "بزاریدش", "بذارن", "بزارن",
    "قرار بدید", "قرار بدین", "قرار دهید", "قرار بدهید",
    "کامنت کنید", "کامنت کنین", "کامنت کن", "کامنت بذارید", "کامنت بزارید", "کامنت بذارین", "کامنت بزارین", "کامنت بگذارید",
    "به اشتراک بذارید", "به اشتراک بگذارید", "به اشتراک بذارین", "به اشتراک بزارید", "به اشتراک بزارین", "اشتراک بذارید", "اشتراک بگذارید",
    "شیر کنید", "شیر کنین", "شر کنید", "شر کنین",
    "رو کنید", "رو کنین", "رو کن",
    "آپلود کنید", "آپلود کنین", "آپلود نمایید",
    "ثبت کنید", "ثبت کنین",
    "پیوست کنید", "اتچ کنید",
    "در میان بذارید", "در میون بذارید", "در میان بگذارید",
    "نشون بدید", "نشون بدین", "نشون بده", "نمایش بدید",

    // ۲. ترکیب ضمایر متصل (-هاتون، -هاتونو، -تون، -تونو، -اتون) با افعال امری
    "هاتون رو بدید", "هاتونو بدید", "هاتون بدید", "هاتون رو بدین", "هاتونو بدین", "هاتون بدین",
    "هاتون رو بفرستید", "هاتونو بفرستید", "هاتون بفرستید", "هاتون رو بفرستین", "هاتونو بفرستین", "هاتون بفرستین",
    "هاتون رو بذارید", "هاتونو بذارید", "هاتون بذارید", "هاتون رو بزارید", "هاتونو بزارید", "هاتون بزارید", "هاتون رو بذارین", "هاتونو بذارین",
    "هاتون رو کامنت کنید", "هاتونو کامنت کنید", "هاتون کامنت کنید", "هاتون رو کامنت کنین", "هاتونو کامنت کنین", "هاتون کامنت کنین",
    "تون رو بدید", "تونو بدید", "تون بدید", "تون رو بدین", "تونو بدین", "تون بدین",
    "تون رو بفرستید", "تونو بفرستید", "تون بفرستید", "تون رو بفرستین", "تونو بفرستین", "تون بفرستین",
    "تون رو بذارید", "تونو بذارید", "تون بذارید", "تون رو بزارید", "تونو بزارید", "تون بزارید", "تون رو بذارین", "تونو بذارین",
    "تون رو کامنت کنید", "تونو کامنت کنید", "تون کامنت کنید", "تون رو کامنت کنین", "تونو کامنت کنین",
    "اتون رو بفرستید", "اتونو بفرستید", "اتون بفرستید", "اتون رو بفرستین", "اتونو بفرستین",
    "اتون رو بدید", "اتونو بدید", "اتون بدید", "اتون رو بدین", "اتونو بدین",
    "اتون رو بذارید", "اتونو بذارید", "اتون بزارید", "اتون بذارید",
    "اتون رو کامنت کنید", "اتونو کامنت کنید", "اتون کامنت کنید",
    "هاتون رو رو کنید", "هاتونو رو کنید", "تون رو رو کنید", "تونو رو کنید",
    "هاتون رو شیر کنید", "هاتونو شیر کنید", "تون رو شیر کنید", "تونو شیر کنید",
    "هاتون رو شر کنید", "هاتونو شر کنید",

    // ۳. عبارات اسمی، چالشی و تاکیدی
    "فقط ارسال", "ارسال فقط", "فقط بفرستید", "فقط بدید", "فقط بذارید", "فقط بزارید", "فقط کامنت",
    "چالش", "مسابقه",
    "نوبت دادنه", "وقت دادنه", "نوبت فرستادنه", "وقت فرستادنه",
    "نوبت ارسال", "وقت ارسال", "نوبت به اشتراک", "وقت به اشتراک",
    "مشتاق دیدن", "منتظر دیدن", "منتظر", "مشتاقیم ببینیم", "میخوایم ببینیم", "می‌خوایم ببینیم",
    "بیارید بالا", "بالا بیارید",
    "کی داره", "کیا دارن", "هرکی داره", "هر کی داره",
    "در کامنت ها", "توی کامنت ها", "تو کامنت ها", "در کامنت‌ها", "توی کامنت‌ها", "تو کامنت‌ها", "توی کامنتا", "تو کامنتا",
    "زیر این پست", "زیر همین پست", "توی ریپلای", "در ریپلای",
    "بهترین رو بفرستید", "بهترینشو بفرستید", "بهترینتون رو بفرستید",
];

/// عبارات اخباری که باید مانع فعال‌سازی اشتباه شوند (در صورتی که هیچ فعل امری در متن نباشد)
pub const EXCLUSION_PATTERNS: &[&str] = &[
    "به روزرسانی شد", "به‌روزرسانی شد", "بروزرسانی شد", "تغییر کرد", "عوض شد",
    "منتشر شد", "قرار گرفت", "بارگذاری شد", "دانلود کنید", "تماشا کنید", "مشاهده کنید",
    "بخوانید", "بخونید",
];

/// تشخیص هوشمند درخواست ارسال محتوا در پست کانال
pub fn detect_channel_post_cta(text: &str, settings: &GroupSettings) -> Option<&'static str> {
    if !settings.channel_track_enabled {
        return None;
    }

    let norm = normalize_persian_text(text);

    // ۱. بررسی وجود حداقل یک الگوی امری یا درخواستی (CTA)
    let has_cta = CTA_PATTERNS.iter().any(|&cta| {
        let norm_cta = normalize_persian_text(cta);
        norm.contains(&norm_cta)
    });

    if !has_cta {
        return None;
    }

    // ۲. بررسی دسته‌بندی‌های فعال به ترتیب اولویت
    let categories: [(&str, bool, &[&str]); 7] = [
        ("photo", settings.track_photo, PHOTO_KEYWORDS),
        ("video", settings.track_video, VIDEO_KEYWORDS),
        ("voice", settings.track_voice, VOICE_KEYWORDS),
        ("gif", settings.track_gif, GIF_KEYWORDS),
        ("sticker", settings.track_sticker, STICKER_KEYWORDS),
        ("document", settings.track_document, DOCUMENT_KEYWORDS),
        ("text", settings.track_text, TEXT_KEYWORDS),
    ];

    for (cat_name, is_enabled, keywords) in categories {
        if !is_enabled {
            continue;
        }

        let has_keyword = keywords.iter().any(|&kw| {
            let norm_kw = normalize_persian_text(kw);
            norm.contains(&norm_kw)
        });

        if has_keyword {
            return Some(cat_name);
        }
    }

    None
}

/// ساختار خروجی استخراج دستور مستقیم تنظیم محتوا در پست کانال
#[derive(Debug, PartialEq, Clone)]
pub struct ExplicitDirective {
    pub rule: &'static str,
    pub raw_matched: String,
    pub cleaned_text: String,
}

/// متن پیش‌فرض تمیز در صورتی که تمام متن پیام صرفاً دستور بوده باشد
pub fn directive_fallback_notice(rule: &str) -> String {
    match rule {
        "gif" => "💬 ارسال فقط گیف در کامنت‌ها مجاز است".to_string(),
        "photo" => "💬 ارسال فقط عکس در کامنت‌ها مجاز است".to_string(),
        "video" => "💬 ارسال فقط ویدیو در کامنت‌ها مجاز است".to_string(),
        "voice" => "💬 ارسال فقط ویس در کامنت‌ها مجاز است".to_string(),
        "sticker" => "💬 ارسال فقط استیکر در کامنت‌ها مجاز است".to_string(),
        "document" => "💬 ارسال فقط فایل در کامنت‌ها مجاز است".to_string(),
        "text" => "💬 ارسال فقط پیام متنی در کامنت‌ها مجاز است".to_string(),
        "except_video" => "💬 ارسال همه نوع محتوا بجز فیلم در کامنت‌ها مجاز است".to_string(),
        "except_photo" => "💬 ارسال همه نوع محتوا بجز عکس در کامنت‌ها مجاز است".to_string(),
        "except_gif" => "💬 ارسال همه نوع محتوا بجز گیف در کامنت‌ها مجاز است".to_string(),
        "except_voice" => "💬 ارسال همه نوع محتوا بجز ویس در کامنت‌ها مجاز است".to_string(),
        "except_sticker" => "💬 ارسال همه نوع محتوا بجز استیکر در کامنت‌ها مجاز است".to_string(),
        "except_document" => "💬 ارسال همه نوع محتوا بجز فایل در کامنت‌ها مجاز است".to_string(),
        "except_text" => "💬 ارسال همه نوع محتوا بجز پیام متنی در کامنت‌ها مجاز است".to_string(),
        "except_link" => "💬 ارسال همه نوع محتوا بجز لینک در کامنت‌ها مجاز است".to_string(),
        _ => "💬 ارسال نظر در کامنت‌ها آزاد است".to_string(),
    }
}

/// استخراج دستور مستقیم ادمین در پست کانال و تفکیک متن اصلی از دستور
pub fn extract_explicit_directive(text: &str) -> Option<ExplicitDirective> {
    if text.trim().is_empty() {
        return None;
    }

    let lines: Vec<&str> = text.lines().collect();
    let mut matched_line_idx: Option<usize> = None;
    let mut matched_rule: Option<&'static str> = None;
    let mut matched_raw: String = String::new();
    let mut inline_cleaned_line: Option<String> = None;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        let norm_line = normalize_persian_text(trimmed_line);
        if !norm_line.contains("تنظیم") {
            continue;
        }

        if let Some(rule) = parse_directive_tokens(&norm_line) {
            matched_line_idx = Some(idx);
            matched_rule = Some(rule);

            if let Some(pos) = line.find("تنظیم") {
                let prefix = line[..pos].trim_end_matches(|c: char| {
                    c == '.' || c == '-' || c == ':' || c == '|' || c == '/' || c == '\\' || c.is_whitespace()
                });
                if prefix.is_empty() {
                    matched_raw = line.to_string();
                    inline_cleaned_line = None;
                } else {
                    matched_raw = line[pos..].to_string();
                    inline_cleaned_line = Some(prefix.to_string());
                }
            } else {
                matched_raw = line.to_string();
                inline_cleaned_line = None;
            }
            break;
        }
    }

    let rule = matched_rule?;
    let line_idx = matched_line_idx?;

    let mut cleaned_lines = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i == line_idx {
            if let Some(ref rem) = inline_cleaned_line {
                if !rem.trim().is_empty() {
                    cleaned_lines.push(rem.as_str());
                }
            }
        } else {
            cleaned_lines.push(*line);
        }
    }

    let cleaned_text = cleaned_lines.join("\n").trim().to_string();

    Some(ExplicitDirective {
        rule,
        raw_matched: matched_raw,
        cleaned_text,
    })
}

fn parse_directive_tokens(norm: &str) -> Option<&'static str> {
    let words: Vec<&str> = norm.split_whitespace().collect();
    let tanzim_idx = words.iter().position(|&w| w == "تنظیم" || w.starts_with("تنظیم"))?;

    let after_tanzim = &words[tanzim_idx + 1..];
    if after_tanzim.is_empty() {
        return None;
    }

    // ۱. بررسی الگوی منفی: "تنظیم همه بجز فیلم" یا "تنظیم بجز عکس" یا "تنظیم همه به جز فیلم"
    let mut neg_idx: Option<usize> = None;
    for (i, &w) in after_tanzim.iter().enumerate() {
        if w == "بجز" || w == "بدون" || w == "غیر" || w == "الا" || w == "الّا" {
            neg_idx = Some(i);
            break;
        }
        if w == "به" && i + 1 < after_tanzim.len() && after_tanzim[i + 1] == "جز" {
            neg_idx = Some(i + 1);
            break;
        }
    }

    if let Some(n_idx) = neg_idx {
        let targets = &after_tanzim[n_idx + 1..];
        for &t in targets {
            if matches_category(t, VIDEO_KEYWORDS) {
                return Some("except_video");
            }
            if matches_category(t, PHOTO_KEYWORDS) {
                return Some("except_photo");
            }
            if matches_category(t, GIF_KEYWORDS) {
                return Some("except_gif");
            }
            if matches_category(t, VOICE_KEYWORDS) {
                return Some("except_voice");
            }
            if matches_category(t, STICKER_KEYWORDS) {
                return Some("except_sticker");
            }
            if matches_category(t, DOCUMENT_KEYWORDS) {
                return Some("except_document");
            }
            if matches_category(t, TEXT_KEYWORDS) {
                return Some("except_text");
            }
            if t == "لینک" || t == "تبلیغ" || t == "link" {
                return Some("except_link");
            }
        }
        return None;
    }

    // ۲. بررسی الگوی مثبت: "تنظیم گیف" یا "تنظیم فقط عکس"
    for &w in after_tanzim {
        if w == "فقط" || w == "تنها" || w == ":" || w == "-" {
            continue;
        }
        if matches_category(w, GIF_KEYWORDS) {
            return Some("gif");
        }
        if matches_category(w, PHOTO_KEYWORDS) {
            return Some("photo");
        }
        if matches_category(w, VIDEO_KEYWORDS) {
            return Some("video");
        }
        if matches_category(w, VOICE_KEYWORDS) {
            return Some("voice");
        }
        if matches_category(w, STICKER_KEYWORDS) {
            return Some("sticker");
        }
        if matches_category(w, DOCUMENT_KEYWORDS) {
            return Some("document");
        }
        if matches_category(w, TEXT_KEYWORDS) {
            return Some("text");
        }
        if w == "همه" || w == "آزاد" || w == "ازاد" {
            return Some("all");
        }
    }

    None
}

fn matches_category(word: &str, keywords: &[&str]) -> bool {
    let clean = word.trim_matches(|c: char| c == ':' || c == '.' || c == '،' || c == '!' || c == '؟');
    keywords.iter().any(|&k| clean == k || clean.contains(k))
}

/// ارزیابی مجاز بودن محتوای پیام بر مبنای قانون پست (قوانین مثبت و منفی)
pub fn is_content_allowed_by_rule(rule: &str, msg: &teloxide::types::Message) -> bool {
    let has_media = msg.photo().is_some()
        || msg.video().is_some()
        || msg.voice().is_some()
        || msg.audio().is_some()
        || msg.document().is_some()
        || msg.animation().is_some()
        || msg.sticker().is_some()
        || msg.video_note().is_some();

    match rule {
        // قوانین مثبت (فقط محتوای مشخص شده)
        "gif" => msg.animation().is_some(),
        "photo" => msg.photo().is_some(),
        "video" => msg.video().is_some() || msg.video_note().is_some(),
        "voice" => msg.voice().is_some() || msg.audio().is_some(),
        "sticker" => msg.sticker().is_some(),
        "document" => msg.document().is_some() && msg.animation().is_none(),
        "text" => !has_media && msg.text().is_some(),
        "all" => true,

        // قوانین منفی (همه چیز بجز محتوای مشخص شده)
        "except_video" => !(msg.video().is_some() || msg.video_note().is_some()),
        "except_photo" => msg.photo().is_none(),
        "except_gif" => msg.animation().is_none(),
        "except_voice" => !(msg.voice().is_some() || msg.audio().is_some()),
        "except_sticker" => msg.sticker().is_none(),
        "except_document" => !(msg.document().is_some() && msg.animation().is_none()),
        "except_text" => has_media || msg.text().is_none(),
        "except_link" => !message_has_link(msg),

        _ => true,
    }
}

pub fn message_has_link(msg: &teloxide::types::Message) -> bool {
    if let Some(entities) = msg.entities().or_else(|| msg.caption_entities()) {
        for entity in entities {
            match entity.kind {
                teloxide::types::MessageEntityKind::Url
                | teloxide::types::MessageEntityKind::TextLink { .. } => return true,
                _ => {}
            }
        }
    }
    let full_text = msg.text().unwrap_or_else(|| msg.caption().unwrap_or(""));
    full_text.contains("t.me/") || full_text.contains("http://") || full_text.contains("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_settings() -> GroupSettings {
        GroupSettings {
            chat_id: 100,
            welcome_enabled: false,
            welcome_text: "".into(),
            goodbye_enabled: false,
            goodbye_text: "".into(),
            rules_text: "".into(),
            anti_flood_enabled: false,
            anti_flood_rate: 5,
            anti_flood_window_secs: 5,
            anti_flood_action: "".into(),
            link_filter_enabled: false,
            link_filter_action: "".into(),
            forward_filter_enabled: false,
            media_filter_enabled: false,
            photo_filter_enabled: false,
            video_filter_enabled: false,
            voice_filter_enabled: false,
            document_filter_enabled: false,
            gif_filter_enabled: false,
            sticker_filter_enabled: false,
            service_msg_filter_enabled: false,
            night_mode_enabled: false,
            night_mode_start: chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            night_mode_end: chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            night_mode_timezone: "Asia/Tehran".into(),
            captcha_enabled: false,
            captcha_timeout_secs: 60,
            captcha_fail_action: "".into(),
            warn_limit: 3,
            warn_escalation_action: "".into(),
            forced_channel: None,
            silent_punishment: false,
            group_lock_mode: "unlocked".into(),
            channel_track_enabled: true,
            track_photo: true,
            track_video: true,
            track_voice: true,
            track_gif: true,
            track_sticker: true,
            track_document: true,
            track_text: true,
            log_topic_id: None,
        }
    }

    #[test]
    fn test_detects_photo_cta() {
        let settings = sample_settings();
        assert_eq!(
            detect_channel_post_cta("از fastfetch هاتون عکس بدید!", &settings),
            Some("photo")
        );
        assert_eq!(
            detect_channel_post_cta("اسکرین شات بفرستید ببینیم", &settings),
            Some("photo")
        );
    }

    #[test]
    fn test_excludes_photo_announcement_without_cta() {
        let settings = sample_settings();
        assert_eq!(
            detect_channel_post_cta("عکس کانال ما به روزرسانی شد!", &settings),
            None
        );
        assert_eq!(
            detect_channel_post_cta("تصویر پس‌زمینه جدید قرار گرفت.", &settings),
            None
        );
    }

    #[test]
    fn test_detects_gif_cta() {
        let settings = sample_settings();
        assert_eq!(
            detect_channel_post_cta("گیف‌های خنده‌دارتونو بفرستید زیر این پست", &settings),
            Some("gif")
        );
    }

    #[test]
    fn test_detects_voice_cta() {
        let settings = sample_settings();
        assert_eq!(
            detect_channel_post_cta("چالش وویس: صداتونو بفرستید", &settings),
            Some("voice")
        );
    }

    #[test]
    fn test_respects_disabled_category() {
        let mut settings = sample_settings();
        settings.track_photo = false;
        assert_eq!(
            detect_channel_post_cta("از fastfetch هاتون عکس بدید!", &settings),
            None
        );
    }

    #[test]
    fn test_respects_master_switch() {
        let mut settings = sample_settings();
        settings.channel_track_enabled = false;
        assert_eq!(
            detect_channel_post_cta("از fastfetch هاتون عکس بدید!", &settings),
            None
        );
    }

    #[test]
    fn test_extract_explicit_directive_gif() {
        let text = "سلام به همه\nتنظیم گیف";
        let res = extract_explicit_directive(text).expect("باید پیدا شود");
        assert_eq!(res.rule, "gif");
        assert_eq!(res.cleaned_text, "سلام به همه");
    }

    #[test]
    fn test_extract_explicit_directive_except_video() {
        let text = "پست جدید کانال\nتنظیم همه بجز فیلم";
        let res = extract_explicit_directive(text).expect("باید پیدا شود");
        assert_eq!(res.rule, "except_video");
        assert_eq!(res.cleaned_text, "پست جدید کانال");
    }

    #[test]
    fn test_extract_explicit_directive_except_photo() {
        let text = "تنظیم همه بجز عکس";
        let res = extract_explicit_directive(text).expect("باید پیدا شود");
        assert_eq!(res.rule, "except_photo");
        assert_eq!(res.cleaned_text, "");
    }

    #[test]
    fn test_extract_explicit_directive_variations() {
        assert_eq!(
            extract_explicit_directive("تنظیم همه به جز فیلم").unwrap().rule,
            "except_video"
        );
        assert_eq!(
            extract_explicit_directive("تنظیم همه چیز بجز ویس").unwrap().rule,
            "except_voice"
        );
        assert_eq!(
            extract_explicit_directive("تنظیم همه چی به جز استیکر").unwrap().rule,
            "except_sticker"
        );
        assert_eq!(
            extract_explicit_directive("تنظیم آزاد بجز فایل").unwrap().rule,
            "except_document"
        );
        assert_eq!(
            extract_explicit_directive("تنظیم بجز ویدیو").unwrap().rule,
            "except_video"
        );
        assert_eq!(
            extract_explicit_directive("تنظیم فقط گیف").unwrap().rule,
            "gif"
        );
        assert_eq!(
            extract_explicit_directive("تنظیم: عکس").unwrap().rule,
            "photo"
        );
    }

    #[test]
    fn test_extract_explicit_directive_inline_end() {
        let text = "امروز چالش گیف داریم. تنظیم گیف";
        let res = extract_explicit_directive(text).expect("باید پیدا شود");
        assert_eq!(res.rule, "gif");
        assert_eq!(res.cleaned_text, "امروز چالش گیف داریم");
    }
}
