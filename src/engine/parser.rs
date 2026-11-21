use chrono::Duration;
use crate::engine::normalizer::{normalize_digits, normalize_persian_text};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LockType {
    Link,
    Forward,
    Photo,
    Video,
    Voice,
    Gif,
    Sticker,
    Document,
    ServiceMsg,
    Media,
    AntiFlood,
    NightMode,
}

impl LockType {
    pub fn display_name(&self) -> &'static str {
        match self {
            LockType::Link => "لینک",
            LockType::Forward => "فوروارد",
            LockType::Photo => "عکس",
            LockType::Video => "ویدیو",
            LockType::Voice => "ویس",
            LockType::Gif => "گیف",
            LockType::Sticker => "استیکر",
            LockType::Document => "فایل",
            LockType::ServiceMsg => "اعلان‌های تلگرام",
            LockType::Media => "کلی رسانه",
            LockType::AntiFlood => "ضداسپم",
            LockType::NightMode => "شبانه",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum ParsedCommand {
    // دستورات نیازمند ریپلای یا هدف
    Ban { duration: Option<Duration>, reason: Option<String> },
    Kick { reason: Option<String> },
    Mute { duration: Option<Duration>, reason: Option<String> },
    Unban,
    Unmute,
    Warn { reason: Option<String> },
    Unwarn,
    Warnings,
    DeleteMessage,
    Purge { count: usize },
    Pin { silent: bool },
    Unpin,
    UserInfo,

    // دستورات عمومی بدون ریپلای
    Start,
    Help,
    Rules,
    Settings,
    Stats,
    Status,

    // مدیریت قفل‌های گروه
    LockToggle { lock_type: LockType, enable: bool },
    GroupLockMenu,
    SetGroupLock(String),
    SetLogTopic,
    ClearLogTopic,

    // تنظیمات متنی مستقیم
    SetWelcome(bool),
    SetGoodbye(bool),
    SetAntiFlood(bool),
    SetLinkFilter(bool),
    SetChannel(String),
    SetRules(String),
    AddCustomCmd { trigger: String, response: String },
    DeleteCustomCmd { trigger: String },
    ListCustomCmds,
    AddBannedWord { word: String, list_type: i32 },
    RemoveBannedWord { word: String },
    AddAllowedDomain { domain: String },
    RemoveAllowedDomain { domain: String },

    // پیام عادی یا دستور سفارشی
    CustomOrUnknown(String),
}

/// پارس کردن مدت زمان از متن
pub fn parse_duration(text: &str) -> Option<(Duration, usize)> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    // حالت ۱: عدد + واحد (مثل: «۲ ساعت» یا «۳۰ دقیقه»)
    if parts.len() >= 2 {
        let num_str = normalize_digits(parts[0]);
        if let Ok(num) = num_str.parse::<i64>() {
            let unit = parts[1];
            let dur = match unit {
                "ثانیه" | "ثانیه‌ها" | "ثانیه_ها" | "s" | "sec" | "secs" => Some(Duration::seconds(num)),
                "دقیقه" | "دقیقه‌ها" | "دقیقه_ها" | "m" | "min" | "mins" => Some(Duration::minutes(num)),
                "ساعت" | "ساعت‌ها" | "ساعت_ها" | "h" | "hour" | "hours" => Some(Duration::hours(num)),
                "روز" | "روزها" | "d" | "day" | "days" => Some(Duration::days(num)),
                "هفته" | "هفته‌ها" | "هفته_ها" | "w" | "week" | "weeks" => Some(Duration::weeks(num)),
                "ماه" | "ماه‌ها" | "month" | "months" => Some(Duration::days(num * 30)),
                _ => None,
            };

            if let Some(d) = dur {
                return Some((d, 2));
            }
        }
    }

    // حالت ۲: عدد متصل به واحد (مثل «30m» یا «2h» یا «1d»)
    let first = normalize_digits(parts[0]);
    if let Some(ch) = first.chars().last() {
        if ch == 's' || ch == 'm' || ch == 'h' || ch == 'd' || ch == 'w' {
            let num_part = &first[..first.len() - 1];
            if let Ok(num) = num_part.parse::<i64>() {
                let dur = match ch {
                    's' => Duration::seconds(num),
                    'm' => Duration::minutes(num),
                    'h' => Duration::hours(num),
                    'd' => Duration::days(num),
                    'w' => Duration::weeks(num),
                    _ => unreachable!(),
                };
                return Some((dur, 1));
            }
        }
    }

    None
}

/// استخراج هوشمند مدت زمان و دلیل از باقیمانده دستور (با نادیده گرفتن توکن‌های هدف مانند @username یا آیدی عددی)
pub fn extract_duration_and_reason(parts: &[&str]) -> (Option<Duration>, Option<String>) {
    if parts.is_empty() {
        return (None, None);
    }

    // ۱. بررسی مدت زمان در ابتدای باقیمانده متن
    let full_remaining = parts.join(" ");
    if let Some((dur, consumed)) = parse_duration(&full_remaining) {
        let reason_parts: Vec<&str> = parts[consumed..]
            .iter()
            .filter(|&&p| !is_target_token(p))
            .copied()
            .collect();
        let reason = if !reason_parts.is_empty() {
            Some(reason_parts.join(" "))
        } else {
            None
        };
        return (Some(dur), reason);
    }

    // ۲. اگر توکن اول شناسه یا منشن هدف بود، مدت‌زمان را از توکن بعدی بررسی کن
    if is_target_token(parts[0]) && parts.len() > 1 {
        let after_target = parts[1..].join(" ");
        if let Some((dur, consumed)) = parse_duration(&after_target) {
            let reason_parts: Vec<&str> = parts[1 + consumed..]
                .iter()
                .filter(|&&p| !is_target_token(p))
                .copied()
                .collect();
            let reason = if !reason_parts.is_empty() {
                Some(reason_parts.join(" "))
            } else {
                None
            };
            return (Some(dur), reason);
        }
    }

    // ۳. جستجوی مدت‌زمان در هر جای توکن‌ها
    for i in 0..parts.len() {
        let sub = parts[i..].join(" ");
        if let Some((dur, consumed)) = parse_duration(&sub) {
            let mut reason_parts = Vec::new();
            for (idx, &part) in parts.iter().enumerate() {
                if idx >= i && idx < i + consumed {
                    continue;
                }
                if is_target_token(part) {
                    continue;
                }
                reason_parts.push(part);
            }
            let reason = if !reason_parts.is_empty() {
                Some(reason_parts.join(" "))
            } else {
                None
            };
            return (Some(dur), reason);
        }
    }

    // ۴. بدون مدت زمان — فقط دلیل (با حذف توکن‌های هدف)
    let reason_parts: Vec<&str> = parts
        .iter()
        .filter(|&&p| !is_target_token(p))
        .copied()
        .collect();
    let reason = if !reason_parts.is_empty() {
        Some(reason_parts.join(" "))
    } else {
        None
    };

    (None, reason)
}

fn is_target_token(token: &str) -> bool {
    if token.starts_with('@') && token.len() > 1 {
        return true;
    }
    let norm = normalize_digits(token);
    if let Ok(num) = norm.parse::<u64>() {
        if num > 1000 {
            return true;
        }
    }
    false
}

/// موتور شناسایی و پارس دستورات فارسی و انگلیسی تلگرام
fn is_conversational_verb(word: &str) -> bool {
    let clean = word.trim_matches(|c: char| c.is_ascii_punctuation() || "،؛؟!«»()[]{}.".contains(c));
    let stem = clean.trim_end_matches('ا'); // برای کلماتی مثل میشیا یا نکنیا
    matches!(
        clean,
        // افعال مشتق از «شدن»
        "میشی" | "نمیشی" | "میشه" | "نمیشه" | "میشن" | "نمیشن" | "میشم" | "نمیشم" | "میشید" | "نمیشید" | "میشیم" | "نمیشیم" | "میشین" | "نمیشین"
        | "بشی" | "نبشی" | "بشه" | "نبشه" | "بشن" | "نبشن" | "بشم" | "نبشم" | "بشید" | "نبشید" | "بشیم" | "نبشیم" | "بشین" | "نبشین"
        | "شدی" | "نشدی" | "شد" | "نشد" | "شدیم" | "نشدیم" | "شدید" | "نشدید" | "شدن" | "نشدن" | "شدم" | "نشدم" | "شده" | "نشده" | "شدند" | "شدین" | "نشدین"
        | "نشی" | "نشید" | "نشه" | "نشن" | "نشم" | "نشیم" | "نشین"
        
        // افعال منفی یا غیرامری مشتق از «کردن»
        | "نکن" | "نکنید" | "نکنی" | "نکنه" | "نکنن" | "نکنم" | "نکنیم" | "نکنیا" | "نکنین" | "نکنند"
        | "میکنی" | "نمیکنی" | "میکنه" | "نمیکنه" | "میکنن" | "نمیکنن" | "میکنم" | "نمیکنم" | "میکنیم" | "نمیکنیم" | "میکنید" | "نمیکنید" | "میکنین" | "نمیکنین"
        | "کردی" | "نکردی" | "کرد" | "نکرد" | "کردید" | "نکردید" | "کردند" | "نکردند" | "کردم" | "نکردم" | "کرده" | "نکرده" | "کردیم" | "نکردیم" | "کردین" | "نکردین"
        | "کنم" | "کنیم" | "کنه" | "کنن"
        
        // افعال مشتق از «گرفتن»
        | "میگیری" | "نمیگیری" | "میگیره" | "نمیگیره" | "میگیرن" | "نمیگیرن" | "میگیرم" | "نمیگیرم" | "میگیرین" | "میگیرید"
        | "گرفتی" | "نگرفتی" | "گرفت" | "نگرفت" | "گرفتن" | "نگرفتن" | "گرفته" | "نگرفته" | "گرفتیم" | "گرفتید" | "گرفتین"
        | "نگیری" | "نگیره" | "نگیر" | "نگیرین" | "نگیرند" | "نگیرید"
        
        // افعال داشتن، دادن، زدن
        | "داره" | "نداره" | "دارن" | "ندارن" | "داری" | "نداری" | "دارم" | "ندارم" | "داشت" | "نداشت" | "داشته" | "دارید" | "دارین"
        | "میدن" | "نمیدن" | "میده" | "نمیده" | "دادن" | "ندادن" | "دادی" | "ندادی" | "داد" | "نداد" | "بدهند" | "ندید" | "نده" | "ندین"
        | "میزنه" | "نمیزنه" | "میزنن" | "نمیزنن" | "زدی" | "نزدی" | "زد" | "نزد" | "نزن" | "نزنید" | "نزنین"
        
        // افعال بودن و هستن
        | "هستی" | "نیستی" | "هست" | "نیست" | "بود" | "نبود" | "بودی" | "نبودی" | "بودن" | "نبودن" | "بودند" | "باشی" | "نباشی" | "باشه" | "نباشه"
    ) || matches!(stem, "میشی" | "نشی" | "نکن" | "بشی")
}

pub fn parse_command(raw_text: &str) -> ParsedCommand {
    let normalized = normalize_persian_text(raw_text);
    let parts: Vec<&str> = normalized.split_whitespace().collect();

    if parts.is_empty() {
        return ParsedCommand::CustomOrUnknown(String::new());
    }

    let has_prefix = raw_text.starts_with('/') || raw_text.starts_with('!') || raw_text.starts_with('.');
    let is_question = raw_text.contains('?') || raw_text.contains('؟');

    // اگر جمله با علامت سوال خاتمه یافته باشد و پیشوند رسمی کامند نداشته باشد، صرفاً یک سوال است
    if !has_prefix && is_question {
        return ParsedCommand::CustomOrUnknown(normalized);
    }

    // بررسی دستورات قفل کلی گروه
    let clean_norm = normalized.trim_start_matches('/').trim_start_matches('!').trim_start_matches('.');
    if clean_norm == "قفل گروه" || clean_norm == "قفل چت" || clean_norm == "lockgroup" || clean_norm == "lock group" || clean_norm == "قفل_گروه" {
        return ParsedCommand::GroupLockMenu;
    }
    if clean_norm == "باز کردن گروه" || clean_norm == "بازگشایی گروه" || clean_norm == "رفع قفل گروه" || clean_norm == "لغو قفل گروه" || clean_norm == "انلاک گروه" || clean_norm == "آنلاک گروه" || clean_norm == "unlock group" || clean_norm == "unlockgroup" {
        return ParsedCommand::SetGroupLock("unlocked".to_string());
    }
    if clean_norm == "قفل سراسری" || clean_norm == "قفل کامل" {
        return ParsedCommand::SetGroupLock("total".to_string());
    }
    if clean_norm == "کامنت ازاد" || clean_norm == "کامنت آزاد" || clean_norm == "فقط کامنت" {
        return ParsedCommand::SetGroupLock("comments_only".to_string());
    }

    // دستورات مسیریاب تاپیک‌های فروم (Topic Router)
    if clean_norm == "تنظیم تاپیک لاگ" || clean_norm == "تنظیم تاپیک" || clean_norm == "set_log_topic" || clean_norm == "setlogtopic" {
        return ParsedCommand::SetLogTopic;
    }
    if clean_norm == "حذف تاپیک لاگ" || clean_norm == "لغو تاپیک لاگ" || clean_norm == "clear_log_topic" || clean_norm == "clearlogtopic" {
        return ParsedCommand::ClearLogTopic;
    }

    // ۱. بررسی دستورات قفل و بازکردن رسانه‌ها و فیلترها
    if let Some(lock_cmd) = parse_lock_command(&parts) {
        return lock_cmd;
    }

    // ۲. دستورات لغو / رفع مجازات‌ها (حتی در صورت همراه بودن با یوزرنیم یا شناسه عددی)
    if is_unban_command(&normalized) {
        return ParsedCommand::Unban;
    }
    if is_unmute_command(&normalized) {
        return ParsedCommand::Unmute;
    }
    if is_unwarn_command(&normalized) {
        return ParsedCommand::Unwarn;
    }
    if is_unpin_command(&normalized) {
        return ParsedCommand::Unpin;
    }
    if normalized == "لیست دستورات" || normalized == "دستورات گروه" {
        return ParsedCommand::ListCustomCmds;
    }
    if parts.len() >= 3 && parts[0] == "جوین" && parts[1] == "اجباری" {
        return ParsedCommand::SetChannel(parts[2].to_string());
    }

    // تفکیک دستور و پارامترها با پشتیبانی از دستورات چندکلمه‌ای و پیشوندهای تلگرامی (/، !، .)
    let (cmd, remaining_parts): (String, &[&str]) = if parts.len() >= 2 && parts[0] == "بی" && parts[1] == "صدا" {
        ("سکوت".to_string(), &parts[2..])
    } else if parts.len() >= 2 && (parts[0] == "خفه" || parts[0] == "ساکت") && parts[1] == "شو" {
        ("سکوت".to_string(), &parts[2..])
    } else if parts.len() >= 2 && (parts[1] == "کن" || parts[1] == "بده" || parts[1] == "بکن" || parts[1] == "بزن") {
        let base = match parts[0] {
            "بنش" | "بن" | "مسدودش" | "مسدود" => "بن",
            "سکوتش" | "سکوت" | "میوتش" | "میوت" | "سایلنتش" | "سایلنت" | "ساکتش" | "ساکت" | "خفه‌ش" | "خفش" => "سکوت",
            "اخراجش" | "اخراج" | "کیکش" | "کیک" => "اخراج",
            "اخطارش" | "اخطار" | "وارنش" | "وارن" => "اخطار",
            "پینش" | "پین" | "سنجاقش" | "سنجاق" => "سنجاق",
            "حذفش" | "حذف" | "پاکش" | "پاک" => "حذف",
            _ => parts[0],
        };
        (base.to_string(), &parts[2..])
    } else {
        let clean = parts[0]
            .split('@')
            .next()
            .unwrap_or(parts[0])
            .trim_start_matches('/')
            .trim_start_matches('!')
            .trim_start_matches('.')
            .to_lowercase();
        (clean, &parts[1..])
    };

    let is_conversational = !has_prefix && (
        is_question || remaining_parts.iter().any(|&part| is_conversational_verb(part))
    );

    let remaining = remaining_parts.join(" ");

    match cmd.as_str() {
        // شروع و راهنما
        "start" | "شروع" => ParsedCommand::Start,
        "help" | "راهنما" | "دستورات" => ParsedCommand::Help,
        "rules" | "قوانین" => ParsedCommand::Rules,
        "settings" | "panel" | "تنظیمات" | "پنل" => ParsedCommand::Settings,
        "stats" | "status" | "امار" | "آمار" | "وضعیت" => ParsedCommand::Stats,

        // مسدودسازی (بن)
        "ban" | "بن" | "مسدود" | "بنش" | "مسدودش" => {
            if is_conversational {
                return ParsedCommand::CustomOrUnknown(normalized);
            }
            let (duration, reason) = extract_duration_and_reason(remaining_parts);
            ParsedCommand::Ban { duration, reason }
        }

        // اخراج (کیک)
        "kick" | "اخراج" | "کیک" | "اخراجش" | "کیکش" => {
            if is_conversational {
                return ParsedCommand::CustomOrUnknown(normalized);
            }
            let (_, reason) = extract_duration_and_reason(remaining_parts);
            ParsedCommand::Kick { reason }
        }

        // سکوت (میوت / سایلنت / بی‌صدا)
        "mute" | "میوت" | "سکوت" | "سایلنت" | "بی صدا" | "بی‌صدا" | "میوتش" | "سکوتش" | "سایلنتش" | "خفه" | "خفه شو" | "خفه‌شو" => {
            if is_conversational {
                return ParsedCommand::CustomOrUnknown(normalized);
            }
            let (duration, reason) = extract_duration_and_reason(remaining_parts);
            ParsedCommand::Mute { duration, reason }
        }

        // اخطارها
        "warn" | "اخطار" | "وارن" | "اخطارش" | "وارنش" => {
            if is_conversational {
                return ParsedCommand::CustomOrUnknown(normalized);
            }
            let (_, reason) = extract_duration_and_reason(remaining_parts);
            ParsedCommand::Warn { reason }
        }
        "warnings" | "warns" | "اخطارها" | "اخطارهای من" | "لیست اخطارها" => ParsedCommand::Warnings,

        // حذف پیام و پاکسازی
        "del" | "delete" => ParsedCommand::DeleteMessage,
        "حذف" | "پاک" => {
            if is_conversational {
                return ParsedCommand::CustomOrUnknown(normalized);
            }
            if remaining_parts.is_empty() || (remaining_parts.len() == 1 && remaining_parts[0] == "پیام") {
                ParsedCommand::DeleteMessage
            } else if remaining_parts.len() >= 3 && remaining_parts[0] == "کلمه" && remaining_parts[1] == "ممنوعه" {
                ParsedCommand::RemoveBannedWord { word: remaining_parts[2..].join(" ") }
            } else if remaining_parts.len() >= 3 && remaining_parts[0] == "لینک" && remaining_parts[1] == "مجاز" {
                ParsedCommand::RemoveAllowedDomain { domain: remaining_parts[2].to_string() }
            } else if remaining_parts.len() >= 2 && remaining_parts[0] == "دستور" {
                ParsedCommand::DeleteCustomCmd { trigger: remaining_parts[1..].join(" ") }
            } else {
                ParsedCommand::DeleteMessage
            }
        }
        "purge" | "پاکسازی" => {
            if is_conversational {
                return ParsedCommand::CustomOrUnknown(normalized);
            }
            let count = if !remaining.is_empty() {
                normalize_digits(&remaining).parse::<usize>().unwrap_or(20)
            } else {
                20
            };
            ParsedCommand::Purge { count: count.clamp(1, 100) }
        }

        // پین
        "pin" | "سنجاق" | "پین" | "سنجاقش" | "پینش" => {
            if is_conversational {
                return ParsedCommand::CustomOrUnknown(normalized);
            }
            ParsedCommand::Pin { silent: false }
        }

        // اطلاعات کاربر
        "info" | "id" | "whois" | "اطلاعات" | "شناسه" | "ایدی" | "آیدی" => {
            if is_conversational {
                return ParsedCommand::CustomOrUnknown(normalized);
            }
            ParsedCommand::UserInfo
        }

        // تنظیمات متنی فارسی
        "تنظیم" => {
            if parts.len() >= 3 && parts[1] == "خوشامد" {
                ParsedCommand::SetWelcome(parts[2] == "روشن" || parts[2] == "فعال")
            } else if parts.len() >= 3 && parts[1] == "خداحافظی" {
                ParsedCommand::SetGoodbye(parts[2] == "روشن" || parts[2] == "فعال")
            } else if parts.len() >= 3 && parts[1] == "ضداسپم" {
                ParsedCommand::SetAntiFlood(parts[2] == "روشن" || parts[2] == "فعال")
            } else if parts.len() >= 4 && parts[1] == "فیلتر" && parts[2] == "لینک" {
                ParsedCommand::SetLinkFilter(parts.get(3).map_or(false, |&s| s == "روشن" || s == "فعال"))
            } else if parts.len() >= 3 && (parts[1] == "کانال" || parts[1] == "چنل") {
                ParsedCommand::SetChannel(parts[2].to_string())
            } else if parts.len() >= 4 && parts[1] == "جوین" && parts[2] == "اجباری" {
                ParsedCommand::SetChannel(parts[3].to_string())
            } else if parts.len() >= 3 && parts[1] == "قوانین" {
                ParsedCommand::SetRules(parts[2..].join(" "))
            } else {
                ParsedCommand::CustomOrUnknown(normalized)
            }
        }

        // افزودن و حذف کلمات ممنوعه، دستورات، لینک‌ها
        "افزودن" => {
            if parts.len() >= 4 && parts[1] == "کلمه" && parts[2] == "ممنوعه" {
                ParsedCommand::AddBannedWord { word: parts[3..].join(" "), list_type: 1 }
            } else if parts.len() >= 4 && parts[1] == "لینک" && parts[2] == "مجاز" {
                ParsedCommand::AddAllowedDomain { domain: parts[3].to_string() }
            } else if parts.len() >= 3 && parts[1] == "دستور" {
                let rest = parts[2..].join(" ");
                if let Some((trigger, response)) = rest.split_once('|') {
                    ParsedCommand::AddCustomCmd {
                        trigger: trigger.trim().to_string(),
                        response: response.trim().to_string(),
                    }
                } else {
                    ParsedCommand::CustomOrUnknown(normalized)
                }
            } else {
                ParsedCommand::CustomOrUnknown(normalized)
            }
        }

        _ => ParsedCommand::CustomOrUnknown(normalized),
    }
}

fn parse_lock_command(parts: &[&str]) -> Option<ParsedCommand> {
    if parts.is_empty() {
        return None;
    }

    let first = parts[0].trim_start_matches('/').trim_start_matches('!').trim_start_matches('.');

    let (enable, target_parts) = if first == "قفل" || first == "lock" {
        if parts.len() < 2 {
            return None;
        }
        (true, &parts[1..])
    } else if first == "بازکردن" || first == "آنلاک" || first == "انلاک" || first == "unlock" {
        if parts.len() < 2 {
            return None;
        }
        (false, &parts[1..])
    } else if parts.len() >= 3 && parts[0] == "باز" && parts[1] == "کردن" {
        (false, &parts[2..])
    } else if parts.len() >= 3 && parts[0] == "لغو" && parts[1] == "قفل" {
        (false, &parts[2..])
    } else {
        return None;
    };

    let lock_target = target_parts.join(" ");
    let lock_type = match lock_target.as_str() {
        "لینک" | "link" | "links" => LockType::Link,
        "فوروارد" | "فروارد" | "forward" | "forwards" => LockType::Forward,
        "عکس" | "تصویر" | "photo" | "photos" | "pic" | "pics" => LockType::Photo,
        "ویدیو" | "فیلم" | "video" | "videos" => LockType::Video,
        "ویس" | "وویس" | "صدا" | "voice" | "audio" => LockType::Voice,
        "گیف" | "انیمیشن" | "gif" | "gifs" | "animation" => LockType::Gif,
        "استیکر" | "sticker" | "stickers" => LockType::Sticker,
        "فایل" | "سند" | "داکیومنت" | "file" | "doc" | "document" => LockType::Document,
        "سرویس" | "پیام ورود" | "ورود" | "خروج" | "پیامهای ورود" | "service" | "اعلان" | "اعلانها" | "اعلان ها" | "اعلانهای تلگرام" | "اعلان های تلگرام" => LockType::ServiceMsg,
        "رسانه" | "مدیا" | "کل رسانه" | "همه رسانه ها" | "media" => LockType::Media,
        "اسپم" | "ضداسپم" | "فلاد" | "flood" | "antiflood" | "spam" => LockType::AntiFlood,
        "شب" | "حالت شب" | "قفل شب" | "night" => LockType::NightMode,
        _ => {
            if let Some(&first_word) = target_parts.first() {
                match first_word {
                    "لینک" | "link" => LockType::Link,
                    "فوروارد" | "فروارد" | "forward" => LockType::Forward,
                    "عکس" | "تصویر" | "photo" => LockType::Photo,
                    "ویدیو" | "فیلم" | "video" => LockType::Video,
                    "ویس" | "وویس" | "صدا" | "voice" => LockType::Voice,
                    "گیف" | "انیمیشن" | "gif" => LockType::Gif,
                    "استیکر" | "sticker" => LockType::Sticker,
                    "فایل" | "سند" | "file" | "doc" => LockType::Document,
                    "سرویس" | "ورود" | "خروج" | "اعلان" | "اعلانها" | "اعلان ها" | "اعلانهای تلگرام" | "اعلان های تلگرام" => LockType::ServiceMsg,
                    "رسانه" | "مدیا" => LockType::Media,
                    "اسپم" | "ضداسپم" | "فلاد" => LockType::AntiFlood,
                    "شب" => LockType::NightMode,
                    _ => return None,
                }
            } else {
                return None;
            }
        }
    };

    Some(ParsedCommand::LockToggle { lock_type, enable })
}

fn is_unban_command(text: &str) -> bool {
    let clean = text.trim_start_matches('/').trim_start_matches('!').trim_start_matches('.');
    clean.starts_with("unban")
        || clean.starts_with("رفع بن")
        || clean.starts_with("لغو بن")
        || clean.starts_with("حذف بن")
        || clean.starts_with("برداشتن بن")
        || clean.starts_with("رفع مسدودیت")
        || clean.starts_with("لغو مسدودیت")
        || clean.starts_with("رفع مسدود")
        || clean.starts_with("لغو مسدود")
        || clean.starts_with("ازاد کن")
        || clean.starts_with("آزاد کن")
        || clean.starts_with("ازادش کن")
        || clean.starts_with("آزادش کن")
        || clean.starts_with("انبن")
        || clean.starts_with("آنبن")
}

fn is_unmute_command(text: &str) -> bool {
    let clean = text.trim_start_matches('/').trim_start_matches('!').trim_start_matches('.');
    clean.starts_with("unmute")
        || clean.starts_with("رفع میوت")
        || clean.starts_with("لغو میوت")
        || clean.starts_with("حذف میوت")
        || clean.starts_with("برداشتن میوت")
        || clean.starts_with("رفع سکوت")
        || clean.starts_with("لغو سکوت")
        || clean.starts_with("حذف سکوت")
        || clean.starts_with("برداشتن سکوت")
        || clean.starts_with("رفع سایلنت")
        || clean.starts_with("لغو سایلنت")
        || clean.starts_with("رفع بی صدا")
        || clean.starts_with("لغو بی صدا")
        || clean.starts_with("رفع بی‌صدا")
        || clean.starts_with("لغو بی‌صدا")
        || clean.starts_with("باصدا")
        || clean.starts_with("با صدا")
        || clean.starts_with("رفع صدا")
        || clean.starts_with("انمیوت")
        || clean.starts_with("آنمیوت")
}

fn is_unwarn_command(text: &str) -> bool {
    let clean = text.trim_start_matches('/').trim_start_matches('!').trim_start_matches('.');
    clean.starts_with("unwarn")
        || clean.starts_with("حذف اخطار")
        || clean.starts_with("لغو اخطار")
        || clean.starts_with("رفع اخطار")
        || clean.starts_with("پاک کردن اخطار")
        || clean.starts_with("پاکسازی اخطار")
        || clean.starts_with("پاک اخطار")
        || clean.starts_with("حذف وارن")
        || clean.starts_with("لغو وارن")
        || clean.starts_with("رفع وارن")
        || clean.starts_with("پاک وارن")
}

fn is_unpin_command(text: &str) -> bool {
    let clean = text.trim_start_matches('/').trim_start_matches('!').trim_start_matches('.');
    clean.starts_with("unpin")
        || clean.starts_with("لغو سنجاق")
        || clean.starts_with("حذف سنجاق")
        || clean.starts_with("برداشتن سنجاق")
        || clean.starts_with("آنپین")
        || clean.starts_with("انپین")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        let (d1, _) = parse_duration("۲ ساعت اسپم").unwrap();
        assert_eq!(d1, Duration::hours(2));

        let (d2, _) = parse_duration("۳۰ دقیقه").unwrap();
        assert_eq!(d2, Duration::minutes(30));

        let (d3, _) = parse_duration("1d").unwrap();
        assert_eq!(d3, Duration::days(1));
    }

    #[test]
    fn test_parse_commands() {
        assert_eq!(parse_command("/start"), ParsedCommand::Start);
        assert_eq!(parse_command("/start@Gordafarid_Bot"), ParsedCommand::Start);
        assert_eq!(parse_command("قوانین"), ParsedCommand::Rules);
        assert_eq!(parse_command("راهنما"), ParsedCommand::Help);
        assert_eq!(parse_command("پاکسازی ۲۰"), ParsedCommand::Purge { count: 20 });
        assert_eq!(parse_command("رفع بن"), ParsedCommand::Unban);
        assert_eq!(parse_command("رفع بن @GlitchySupport"), ParsedCommand::Unban);
        assert_eq!(parse_command("رفع میوت"), ParsedCommand::Unmute);
        assert_eq!(parse_command("رفع میوت @GlitchySupport"), ParsedCommand::Unmute);
        assert_eq!(parse_command("حذف میوت"), ParsedCommand::Unmute);
        assert_eq!(parse_command("لغو سکوت"), ParsedCommand::Unmute);
        assert_eq!(parse_command("سایلنت"), ParsedCommand::Mute { duration: None, reason: None });
        assert_eq!(parse_command("حذف اخطار"), ParsedCommand::Unwarn);
        assert_eq!(parse_command("حذف اخطار @GlitchySupport"), ParsedCommand::Unwarn);
        assert_eq!(parse_command("اطلاعات"), ParsedCommand::UserInfo);
        assert_eq!(parse_command("اطلاعات @GlitchySupport"), ParsedCommand::UserInfo);
        assert_eq!(parse_command("آمار"), ParsedCommand::Stats);
        assert_eq!(parse_command("امار"), ParsedCommand::Stats);
        
        match parse_command("بن ۲ ساعت تبلیغات مکرر") {
            ParsedCommand::Ban { duration, reason } => {
                assert_eq!(duration, Some(Duration::hours(2)));
                assert_eq!(reason, Some("تبلیغات مکرر".to_string()));
            }
            _ => panic!("Expected Ban"),
        }

        match parse_command("میوت ۳۰ دقیقه") {
            ParsedCommand::Mute { duration, reason } => {
                assert_eq!(duration, Some(Duration::minutes(30)));
                assert_eq!(reason, None);
            }
            _ => panic!("Expected Mute"),
        }

        match parse_command("میوت @GlitchySupport ۳۰ دقیقه اسپم") {
            ParsedCommand::Mute { duration, reason } => {
                assert_eq!(duration, Some(Duration::minutes(30)));
                assert_eq!(reason, Some("اسپم".to_string()));
            }
            _ => panic!("Expected Mute with username and duration"),
        }

        assert_eq!(parse_command("سکوت"), ParsedCommand::Mute { duration: None, reason: None });
        assert_eq!(parse_command("سکوت کن"), ParsedCommand::Mute { duration: None, reason: None });
        assert_eq!(parse_command("بی صدا"), ParsedCommand::Mute { duration: None, reason: None });
        assert_eq!(parse_command("!mute"), ParsedCommand::Mute { duration: None, reason: None });
        assert_eq!(parse_command("/سکوت"), ParsedCommand::Mute { duration: None, reason: None });
        assert_eq!(parse_command("!سکوت"), ParsedCommand::Mute { duration: None, reason: None });
        assert_eq!(parse_command("بنش کن"), ParsedCommand::Ban { duration: None, reason: None });
        assert_eq!(parse_command("اخراجش کن"), ParsedCommand::Kick { reason: None });
        assert_eq!(parse_command("اخطارش بده"), ParsedCommand::Warn { reason: None });
        assert_eq!(parse_command("!ban"), ParsedCommand::Ban { duration: None, reason: None });
        assert_eq!(parse_command("!unban"), ParsedCommand::Unban);
        assert_eq!(parse_command("!unmute"), ParsedCommand::Unmute);
        assert_eq!(parse_command("!warn"), ParsedCommand::Warn { reason: None });
        assert_eq!(parse_command("!unwarn"), ParsedCommand::Unwarn);

        assert_eq!(parse_command("تنظیم خوشامد روشن"), ParsedCommand::SetWelcome(true));
        assert_eq!(parse_command("تنظیم کانال @test"), ParsedCommand::SetChannel("@test".to_string()));

        // تست‌های قفل و بازکردن
        assert_eq!(parse_command("قفل عکس"), ParsedCommand::LockToggle { lock_type: LockType::Photo, enable: true });
        assert_eq!(parse_command("بازکردن عکس"), ParsedCommand::LockToggle { lock_type: LockType::Photo, enable: false });
        assert_eq!(parse_command("باز کردن عکس"), ParsedCommand::LockToggle { lock_type: LockType::Photo, enable: false });
        assert_eq!(parse_command("قفل لینک"), ParsedCommand::LockToggle { lock_type: LockType::Link, enable: true });
        assert_eq!(parse_command("بازکردن لینک"), ParsedCommand::LockToggle { lock_type: LockType::Link, enable: false });
        assert_eq!(parse_command("قفل فوروارد"), ParsedCommand::LockToggle { lock_type: LockType::Forward, enable: true });
        assert_eq!(parse_command("قفل استیکر"), ParsedCommand::LockToggle { lock_type: LockType::Sticker, enable: true });
        assert_eq!(parse_command("قفل گیف"), ParsedCommand::LockToggle { lock_type: LockType::Gif, enable: true });
        assert_eq!(parse_command("قفل ویس"), ParsedCommand::LockToggle { lock_type: LockType::Voice, enable: true });
        assert_eq!(parse_command("قفل ویدیو"), ParsedCommand::LockToggle { lock_type: LockType::Video, enable: true });
        assert_eq!(parse_command("قفل فایل"), ParsedCommand::LockToggle { lock_type: LockType::Document, enable: true });
        assert_eq!(parse_command("قفل پیام ورود"), ParsedCommand::LockToggle { lock_type: LockType::ServiceMsg, enable: true });
        assert_eq!(parse_command("قفل اعلان های تلگرام"), ParsedCommand::LockToggle { lock_type: LockType::ServiceMsg, enable: true });
        assert_eq!(parse_command("بازکردن اعلان های تلگرام"), ParsedCommand::LockToggle { lock_type: LockType::ServiceMsg, enable: false });
        assert_eq!(parse_command("قفل رسانه"), ParsedCommand::LockToggle { lock_type: LockType::Media, enable: true });
        assert_eq!(parse_command("قفل اسپم"), ParsedCommand::LockToggle { lock_type: LockType::AntiFlood, enable: true });
        assert_eq!(parse_command("قفل شب"), ParsedCommand::LockToggle { lock_type: LockType::NightMode, enable: true });
        assert_eq!(parse_command("بازکردن شب"), ParsedCommand::LockToggle { lock_type: LockType::NightMode, enable: false });

        // تست‌های نام‌های محاوره‌ای
        assert_eq!(parse_command("خفه شو"), ParsedCommand::Mute { duration: None, reason: None });
        assert_eq!(parse_command("آزاد کن"), ParsedCommand::Unban);
        assert_eq!(parse_command("رفع مسدود"), ParsedCommand::Unban);
        assert_eq!(parse_command("باصدا"), ParsedCommand::Unmute);
        assert_eq!(parse_command("حذف وارن"), ParsedCommand::Unwarn);

        // تست‌های قفل کلی گروه
        assert_eq!(parse_command("قفل گروه"), ParsedCommand::GroupLockMenu);
        assert_eq!(parse_command("/lockgroup"), ParsedCommand::GroupLockMenu);

        // تست‌های جامع جلوگیری از تریگر اشتباه در جملات محاوره‌ای
        assert_eq!(parse_command("بن میشی"), ParsedCommand::CustomOrUnknown("بن میشی".to_string()));
        assert_eq!(parse_command("بن نشی"), ParsedCommand::CustomOrUnknown("بن نشی".to_string()));
        assert_eq!(parse_command("بن نکن"), ParsedCommand::CustomOrUnknown("بن نکن".to_string()));
        assert_eq!(parse_command("بن شدی"), ParsedCommand::CustomOrUnknown("بن شدی".to_string()));
        assert_eq!(parse_command("بن کنم؟"), ParsedCommand::CustomOrUnknown("بن کنم؟".to_string()));
        assert_eq!(parse_command("سکوت میشی"), ParsedCommand::CustomOrUnknown("سکوت میشی".to_string()));
        assert_eq!(parse_command("اخراج میشی"), ParsedCommand::CustomOrUnknown("اخراج میشی".to_string()));
        assert_eq!(parse_command("اخطار میگیری"), ParsedCommand::CustomOrUnknown("اخطار میگیری".to_string()));
        assert_eq!(parse_command("پین نکن"), ParsedCommand::CustomOrUnknown("پین نکن".to_string()));
        assert_eq!(parse_command("حذف شد"), ParsedCommand::CustomOrUnknown("حذف شد".to_string()));
        assert_eq!(parse_command("بن میشی داداش"), ParsedCommand::CustomOrUnknown("بن میشی داداش".to_string()));
        assert_eq!(parse_command("باز کردن گروه"), ParsedCommand::SetGroupLock("unlocked".to_string()));
        assert_eq!(parse_command("قفل سراسری"), ParsedCommand::SetGroupLock("total".to_string()));
        assert_eq!(parse_command("کامنت آزاد"), ParsedCommand::SetGroupLock("comments_only".to_string()));
    }
}
