use teloxide::types::InlineKeyboardMarkup;
use crate::db::groups::GroupSettings;
use crate::ui::styled_keyboard::{StyledButton, StyledKeyboard};

#[derive(Debug, Clone, Default)]
pub struct SettingsCounts {
    pub words: usize,
    pub domains: usize,
    pub custom_cmds: usize,
}

pub fn toggle_icon(enabled: bool) -> &'static str {
    if enabled { "🟢" } else { "🔴" }
}

pub fn toggle_style(enabled: bool) -> (&'static str, &'static str) {
    if enabled {
        ("فعال 🟢", "success")
    } else {
        ("غیرفعال 🔴", "danger")
    }
}

/// دکمه شیشه‌ای افزودن به گروه در پی‌وی ربات
pub fn start_private_styled(bot_username: &str) -> StyledKeyboard {
    let clean_name = bot_username.trim_start_matches('@');
    let add_url = format!(
        "https://t.me/{}?startgroup=true&admin=change_info+delete_messages+restrict_members+pin_messages",
        clean_name
    );

    StyledKeyboard::new(vec![
        vec![
            StyledButton::url("➕ افزودن به گروه", add_url, Some("success")),
        ],
    ])
}

/// کیبورد گیت قفل عضویت اجباری کانال (همراه با لینک مستقیم و دکمه بررسی عضویت)
pub fn force_join_styled(channel_url: &str) -> StyledKeyboard {
    StyledKeyboard::new(vec![
        vec![
            StyledButton::url("📢 عضویت در کانال", channel_url, Some("primary")),
        ],
        vec![
            StyledButton::new("🔄 بررسی عضویت", "check_force_join", Some("success")),
        ],
    ])
}

/// منوی اصلی جامع تنظیمات و امنیت گروه (نسخه رنگی بومی تلگرام)
pub fn settings_main_styled(settings: &GroupSettings, counts: &SettingsCounts) -> StyledKeyboard {
    let (_, w_st) = toggle_style(settings.welcome_enabled);
    let (_, g_st) = toggle_style(settings.goodbye_enabled);
    let (_, fl_st) = toggle_style(settings.anti_flood_enabled);
    let (_, n_st) = toggle_style(settings.night_mode_enabled);
    let (_, c_st) = toggle_style(settings.captcha_enabled);
    let (_, ch_st) = if settings.forced_channel.is_some() { ("فعال 🟢", "success") } else { ("تنظیم نشده 🔴", "danger") };
    let (_, s_st) = toggle_style(settings.silent_punishment);
    let (_, srv_st) = toggle_style(settings.service_msg_filter_enabled);

    StyledKeyboard::new(vec![
        vec![
            StyledButton::primary("🔒 مدیریت قفل‌های گروه (رسانه، محتوا، قفل گروه)", "nav_locks"),
        ],
        vec![
            StyledButton::new("🌸 پیام خوشامد", "toggle_welcome_main", Some(w_st)),
            StyledButton::new("👋 پیام خروج", "toggle_goodbye_main", Some(g_st)),
        ],
        vec![
            StyledButton::new("🛡️ ضداسپم", "toggle_flood_main", Some(fl_st)),
            StyledButton::new("🌙 قفل شب", "toggle_night_main", Some(n_st)),
        ],
        vec![
            StyledButton::new("🤖 تایید هویت", "toggle_captcha_main", Some(c_st)),
            StyledButton::primary(format!("⚠️ سقف اخطار ({})", settings.warn_limit), "nav_warn"),
        ],
        vec![
            StyledButton::new("📢 کانال اجباری", "nav_channel", Some(ch_st)),
            StyledButton::new("🔇 مجازات بی‌صدا", "toggle_silent", Some(s_st)),
        ],
        vec![
            StyledButton::new("🔔 اعلان‌های تلگرام", "toggle_service_msg_main", Some(srv_st)),
        ],
        vec![
            StyledButton::primary(format!("📋 کلمات ممنوعه ({})", counts.words), "nav_words"),
            StyledButton::primary(format!("🌐 دامنه‌های مجاز ({})", counts.domains), "nav_domains"),
        ],
        vec![
            StyledButton::primary(format!("✨ دستورات دلخواه ({})", counts.custom_cmds), "nav_custom_cmds"),
            StyledButton::primary("📡 رهگیری کانال", "nav_channel_track"),
        ],
        vec![
            StyledButton::primary("⚡ اقدامات تنبیهی", "nav_actions"),
            StyledButton::primary("⚙️ تنظیمات پیام خوشامد و خروج", "nav_welcome"),
        ],
        vec![
            StyledButton::primary("📜 مشاهده قوانین گروه", "show_rules"),
            StyledButton::danger("❌ بستن منو", "close_settings"),
        ],
    ])
}

pub fn settings_main_keyboard(settings: &GroupSettings, counts: &SettingsCounts) -> InlineKeyboardMarkup {
    settings_main_styled(settings, counts).to_teloxide()
}

/// پنل جامع قفل‌های امنیتی، رسانه‌ای و محتوایی گروه (نسخه رنگی)
pub fn settings_locks_styled(settings: &GroupSettings) -> StyledKeyboard {
    let (_, l_st) = toggle_style(settings.link_filter_enabled);
    let (_, f_st) = toggle_style(settings.forward_filter_enabled);
    let (_, p_st) = toggle_style(settings.photo_filter_enabled);
    let (_, v_st) = toggle_style(settings.video_filter_enabled);
    let (_, vo_st) = toggle_style(settings.voice_filter_enabled);
    let (_, gi_st) = toggle_style(settings.gif_filter_enabled);
    let (_, st_st) = toggle_style(settings.sticker_filter_enabled);
    let (_, d_st) = toggle_style(settings.document_filter_enabled);
    let (_, s_st) = toggle_style(settings.service_msg_filter_enabled);
    let (_, m_st) = toggle_style(settings.media_filter_enabled);
    let (_, fl_st) = toggle_style(settings.anti_flood_enabled);
    let (_, n_st) = toggle_style(settings.night_mode_enabled);

    let (gl_txt, gl_st) = match settings.group_lock_mode.as_str() {
        "total" => ("🔒 وضعیت قفل گروه: قفل سراسری", "danger"),
        "comments_only" => ("💬 وضعیت قفل گروه: کامنت آزاد", "success"),
        _ => ("🔓 وضعیت قفل گروه: باز (عادی)", "primary"),
    };

    StyledKeyboard::new(vec![
        vec![
            StyledButton::new(gl_txt, "nav_group_lock", Some(gl_st)),
        ],
        vec![
            StyledButton::new("🔗 لینک", "toggle_link", Some(l_st)),
            StyledButton::new("⏩ فوروارد", "toggle_forward", Some(f_st)),
        ],
        vec![
            StyledButton::new("📷 عکس", "toggle_photo", Some(p_st)),
            StyledButton::new("🎥 ویدیو", "toggle_video", Some(v_st)),
        ],
        vec![
            StyledButton::new("🎙️ ویس", "toggle_voice", Some(vo_st)),
            StyledButton::new("🎬 گیف", "toggle_gif", Some(gi_st)),
        ],
        vec![
            StyledButton::new("🎭 استیکر", "toggle_sticker", Some(st_st)),
            StyledButton::new("📁 فایل", "toggle_document", Some(d_st)),
        ],
        vec![
            StyledButton::new("🔔 اعلان‌های تلگرام", "toggle_service_msg", Some(s_st)),
            StyledButton::new("📦 قفل رسانه", "toggle_media", Some(m_st)),
        ],
        vec![
            StyledButton::new("🛡️ ضداسپم", "toggle_flood_lock", Some(fl_st)),
            StyledButton::new("🌙 قفل شب", "toggle_night_lock", Some(n_st)),
        ],
        vec![
            StyledButton::primary("🌐 دامنه‌های مجاز", "nav_domains"),
            StyledButton::primary("📋 کلمات ممنوعه", "nav_words"),
        ],
        vec![
            StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main"),
        ],
    ])
}

pub fn settings_locks_keyboard(settings: &GroupSettings) -> InlineKeyboardMarkup {
    settings_locks_styled(settings).to_teloxide()
}

/// منوی اختصاصی قفل گروه (قفل سراسری / کامنت آزاد)
pub fn group_lock_styled(settings: &GroupSettings) -> StyledKeyboard {
    let is_comments = settings.group_lock_mode == "comments_only";
    let is_total = settings.group_lock_mode == "total";
    let is_unlocked = !is_comments && !is_total;

    StyledKeyboard::new(vec![
        vec![
            StyledButton::new("💬 کامنت آزاد", "set_lock_comments", Some(if is_comments { "success" } else { "primary" })),
            StyledButton::new("🔒 قفل سراسری", "set_lock_total", Some(if is_total { "danger" } else { "primary" })),
        ],
        vec![
            StyledButton::new("🔓 باز کردن گروه (عادی)", "set_lock_unlocked", Some(if is_unlocked { "success" } else { "primary" })),
        ],
        vec![
            StyledButton::danger("❌ بستن", "close_settings"),
        ],
    ])
}

/// منوی قفل گروه هنگام باز شدن از درون پنل تنظیمات
pub fn group_lock_from_panel_styled(settings: &GroupSettings) -> StyledKeyboard {
    let is_comments = settings.group_lock_mode == "comments_only";
    let is_total = settings.group_lock_mode == "total";
    let is_unlocked = !is_comments && !is_total;

    StyledKeyboard::new(vec![
        vec![
            StyledButton::new("💬 کامنت آزاد", "set_lock_comments", Some(if is_comments { "success" } else { "primary" })),
            StyledButton::new("🔒 قفل سراسری", "set_lock_total", Some(if is_total { "danger" } else { "primary" })),
        ],
        vec![
            StyledButton::new("🔓 باز کردن گروه (عادی)", "set_lock_unlocked", Some(if is_unlocked { "success" } else { "primary" })),
        ],
        vec![
            StyledButton::primary("🔙 بازگشت به منوی قفل‌ها", "nav_locks"),
        ],
    ])
}

/// زیرمنوی خوشامد و خداحافظی
pub fn settings_welcome_styled(settings: &GroupSettings) -> StyledKeyboard {
    let (_, w_st) = toggle_style(settings.welcome_enabled);
    let (_, g_st) = toggle_style(settings.goodbye_enabled);

    StyledKeyboard::new(vec![
        vec![
            StyledButton::new("🌸 پیام خوشامد", "toggle_welcome", Some(w_st)),
            StyledButton::new("👋 پیام خروج", "toggle_goodbye", Some(g_st)),
        ],
        vec![
            StyledButton::primary("👁️ پیش‌نمایش خوشامد", "preview_welcome"),
            StyledButton::primary("👁️ پیش‌نمایش خروج", "preview_goodbye"),
        ],
        vec![
            StyledButton::default("🔄 بازنشانی متن خوشامد", "reset_welcome"),
            StyledButton::primary("ℹ️ راهنمای متغیرها", "help_welcome_text"),
        ],
        vec![StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main")],
    ])
}

pub fn settings_welcome_keyboard(settings: &GroupSettings) -> InlineKeyboardMarkup {
    settings_welcome_styled(settings).to_teloxide()
}

/// منوی مدیریت رهگیری کانال (کلید روشن/خاموش + تنظیمات کلمات)
pub fn settings_channel_track_styled(settings: &GroupSettings) -> StyledKeyboard {
    let (status_text, status_style) = if settings.channel_track_enabled {
        ("روشن", "success")
    } else {
        ("خاموش", "danger")
    };

    StyledKeyboard::new(vec![
        vec![
            StyledButton::new(status_text, "toggle_channel_track", Some(status_style)),
            StyledButton::primary("⚙️ تنظیمات کلمات", "nav_track_words"),
        ],
        vec![
            StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main"),
        ],
    ])
}

/// زیرمنوی کلمات و دسته‌بندی‌های فعال رهگیری کانال
pub fn settings_channel_track_words_styled(settings: &GroupSettings) -> StyledKeyboard {
    let photo_st = if settings.track_photo { "success" } else { "danger" };
    let video_st = if settings.track_video { "success" } else { "danger" };
    let voice_st = if settings.track_voice { "success" } else { "danger" };
    let gif_st = if settings.track_gif { "success" } else { "danger" };
    let sticker_st = if settings.track_sticker { "success" } else { "danger" };
    let doc_st = if settings.track_document { "success" } else { "danger" };
    let text_st = if settings.track_text { "success" } else { "danger" };

    StyledKeyboard::new(vec![
        vec![
            StyledButton::new("📷 عکس و تصویر", "toggle_track_photo", Some(photo_st)),
            StyledButton::new("🎥 ویدیو و فیلم", "toggle_track_video", Some(video_st)),
        ],
        vec![
            StyledButton::new("🎙️ ویس و صدا", "toggle_track_voice", Some(voice_st)),
            StyledButton::new("🎬 گیف و انیمیشن", "toggle_track_gif", Some(gif_st)),
        ],
        vec![
            StyledButton::new("🎭 استیکر", "toggle_track_sticker", Some(sticker_st)),
            StyledButton::new("📁 فایل و اسناد", "toggle_track_document", Some(doc_st)),
        ],
        vec![
            StyledButton::new("📝 متن و پیام", "toggle_track_text", Some(text_st)),
        ],
        vec![
            StyledButton::primary("🔙 بازگشت به منوی رهگیری", "nav_channel_track"),
        ],
    ])
}

/// دکمه‌های تایید یا رد به‌روزرسانی حکم کاربر تبعید‌شده
pub fn confirm_mute_update_styled(token: &str) -> StyledKeyboard {
    StyledKeyboard::new(vec![
        vec![
            StyledButton::new("✅ تایید", format!("confirm_mute:{}", token), Some("success")),
            StyledButton::new("❌ رد", format!("reject_mute:{}", token), Some("danger")),
        ],
    ])
}


/// زیرمنوی ضداسپم و پیام‌های رگباری
pub fn settings_flood_styled(settings: &GroupSettings) -> StyledKeyboard {
    let (_, fl_st) = toggle_style(settings.anti_flood_enabled);

    StyledKeyboard::new(vec![
        vec![StyledButton::new("🛡️ سیستم ضداسپم", "toggle_flood", Some(fl_st))],
        vec![
            StyledButton::new(
                if settings.anti_flood_rate == 3 { "• ۳ پیام •" } else { "۳ پیام" },
                "set_rate_3",
                if settings.anti_flood_rate == 3 { Some("success") } else { None },
            ),
            StyledButton::new(
                if settings.anti_flood_rate == 5 { "• ۵ پیام •" } else { "۵ پیام" },
                "set_rate_5",
                if settings.anti_flood_rate == 5 { Some("success") } else { None },
            ),
            StyledButton::new(
                if settings.anti_flood_rate == 7 { "• ۷ پیام •" } else { "۷ پیام" },
                "set_rate_7",
                if settings.anti_flood_rate == 7 { Some("success") } else { None },
            ),
            StyledButton::new(
                if settings.anti_flood_rate == 10 { "• ۱۰ پیام •" } else { "۱۰ پیام" },
                "set_rate_10",
                if settings.anti_flood_rate == 10 { Some("success") } else { None },
            ),
        ],
        vec![
            StyledButton::new(
                if settings.anti_flood_window_secs == 3 { "• ۳ ثانیه •" } else { "۳ ثانیه" },
                "set_win_3",
                if settings.anti_flood_window_secs == 3 { Some("success") } else { None },
            ),
            StyledButton::new(
                if settings.anti_flood_window_secs == 5 { "• ۵ ثانیه •" } else { "۵ ثانیه" },
                "set_win_5",
                if settings.anti_flood_window_secs == 5 { Some("success") } else { None },
            ),
            StyledButton::new(
                if settings.anti_flood_window_secs == 10 { "• ۱۰ ثانیه •" } else { "۱۰ ثانیه" },
                "set_win_10",
                if settings.anti_flood_window_secs == 10 { Some("success") } else { None },
            ),
            StyledButton::new(
                if settings.anti_flood_window_secs == 15 { "• ۱۵ ثانیه •" } else { "۱۵ ثانیه" },
                "set_win_15",
                if settings.anti_flood_window_secs == 15 { Some("success") } else { None },
            ),
        ],
        vec![
            StyledButton::primary(format!("تنبیه فلاد: {}", flood_action_display(&settings.anti_flood_action)), "cycle_flood_action"),
        ],
        vec![StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main")],
    ])
}

pub fn settings_flood_keyboard(settings: &GroupSettings) -> InlineKeyboardMarkup {
    settings_flood_styled(settings).to_teloxide()
}

/// زیرمنوی حالت شب
pub fn settings_night_styled(settings: &GroupSettings) -> StyledKeyboard {
    let (_, n_st) = toggle_style(settings.night_mode_enabled);
    let hours_display = format!("⏰ ساعات فعلی: {} تا {}", settings.night_mode_start.format("%H:%M"), settings.night_mode_end.format("%H:%M"));

    StyledKeyboard::new(vec![
        vec![StyledButton::new("🌙 قفل شبانه", "toggle_night", Some(n_st))],
        vec![StyledButton::default(hours_display, "night_status_toast")],
        vec![
            StyledButton::primary("۲۲:۰۰ تا ۰۶:۰۰", "set_night_22_06"),
            StyledButton::primary("۲۳:۰۰ تا ۰۷:۰۰", "set_night_23_07"),
        ],
        vec![
            StyledButton::primary("۰۰:۰۰ تا ۰۸:۰۰", "set_night_00_08"),
            StyledButton::primary("۰۱:۰۰ تا ۰۹:۰۰", "set_night_01_09"),
        ],
        vec![StyledButton::default("🇮🇷 منطقه زمانی: ایران (Asia/Tehran)", "night_tz_toast")],
        vec![StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main")],
    ])
}

pub fn settings_night_keyboard(settings: &GroupSettings) -> InlineKeyboardMarkup {
    settings_night_styled(settings).to_teloxide()
}

/// زیرمنوی تایید هویت (کپچا)
pub fn settings_captcha_styled(settings: &GroupSettings) -> StyledKeyboard {
    let (_, c_st) = toggle_style(settings.captcha_enabled);

    StyledKeyboard::new(vec![
        vec![StyledButton::new("🤖 تایید عضویت (کپچا)", "toggle_captcha", Some(c_st))],
        vec![
            StyledButton::new(
                if settings.captcha_timeout_secs == 30 { "• ۳۰ ثانیه •" } else { "۳۰ ثانیه" },
                "set_cap_30",
                if settings.captcha_timeout_secs == 30 { Some("success") } else { None },
            ),
            StyledButton::new(
                if settings.captcha_timeout_secs == 60 { "• ۶۰ ثانیه •" } else { "۶۰ ثانیه" },
                "set_cap_60",
                if settings.captcha_timeout_secs == 60 { Some("success") } else { None },
            ),
            StyledButton::new(
                if settings.captcha_timeout_secs == 120 { "• ۱۲۰ ثانیه •" } else { "۱۲۰ ثانیه" },
                "set_cap_120",
                if settings.captcha_timeout_secs == 120 { Some("success") } else { None },
            ),
            StyledButton::new(
                if settings.captcha_timeout_secs == 180 { "• ۱۸۰ ثانیه •" } else { "۱۸۰ ثانیه" },
                "set_cap_180",
                if settings.captcha_timeout_secs == 180 { Some("success") } else { None },
            ),
        ],
        vec![
            StyledButton::primary(format!("تنبیه عدم حل: {}", captcha_action_display(&settings.captcha_fail_action)), "cycle_captcha_action"),
        ],
        vec![StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main")],
    ])
}

pub fn settings_captcha_keyboard(settings: &GroupSettings) -> InlineKeyboardMarkup {
    settings_captcha_styled(settings).to_teloxide()
}

/// زیرمنوی سیستم اخطارها
pub fn settings_warn_styled(settings: &GroupSettings) -> StyledKeyboard {
    StyledKeyboard::new(vec![
        vec![
            StyledButton::new(
                if settings.warn_limit == 3 { "• ۳ اخطار •" } else { "۳ اخطار" },
                "set_warn_3",
                if settings.warn_limit == 3 { Some("success") } else { None },
            ),
            StyledButton::new(
                if settings.warn_limit == 5 { "• ۵ اخطار •" } else { "۵ اخطار" },
                "set_warn_5",
                if settings.warn_limit == 5 { Some("success") } else { None },
            ),
            StyledButton::new(
                if settings.warn_limit == 10 { "• ۱۰ اخطار •" } else { "۱۰ اخطار" },
                "set_warn_10",
                if settings.warn_limit == 10 { Some("success") } else { None },
            ),
        ],
        vec![
            StyledButton::primary(format!("تنبیه سقف اخطار: {}", warn_action_display(&settings.warn_escalation_action)), "cycle_warn_action"),
        ],
        vec![StyledButton::danger("🧹 پاکسازی اخطارهای همه کاربران گروه", "clear_all_warnings")],
        vec![StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main")],
    ])
}

pub fn settings_warn_keyboard(settings: &GroupSettings) -> InlineKeyboardMarkup {
    settings_warn_styled(settings).to_teloxide()
}

/// زیرمنوی عضویت اجباری
pub fn settings_channel_styled(settings: &GroupSettings) -> StyledKeyboard {
    let ch_text = match &settings.forced_channel {
        Some(c) => format!("📢 کانال فعلی: {}", c),
        None => "📢 کانال اجباری: تنظیم نشده 🔴".to_string(),
    };

    let mut rows = vec![
        vec![StyledButton::default(ch_text, "channel_status_toast")],
    ];

    if settings.forced_channel.is_some() {
        rows.push(vec![
            StyledButton::primary("🔍 بررسی دسترسی ربات در کانال", "check_channel"),
            StyledButton::danger("🗑️ حذف کانال اجباری", "clear_forced_channel"),
        ]);
    } else {
        rows.push(vec![
            StyledButton::primary("ℹ️ راهنمای تنظیم کانال", "guide_channel"),
        ]);
    }

    rows.push(vec![StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main")]);
    StyledKeyboard::new(rows)
}

pub fn settings_channel_keyboard(settings: &GroupSettings) -> InlineKeyboardMarkup {
    settings_channel_styled(settings).to_teloxide()
}

/// زیرمنوی کلمات ممنوعه
pub fn settings_words_styled(word_count: usize) -> StyledKeyboard {
    StyledKeyboard::new(vec![
        vec![StyledButton::default(format!("تعداد کلمات ممنوعه: {}", word_count), "words_status_toast")],
        vec![
            StyledButton::primary("📋 مشاهده لیست کلمات", "list_banned_words"),
            StyledButton::danger("🗑️ پاک کردن همه کلمات", "clear_all_banned_words"),
        ],
        vec![StyledButton::primary("ℹ️ نحوه افزودن/حذف کلمه", "guide_banned_words")],
        vec![StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main")],
    ])
}

pub fn settings_words_keyboard(word_count: usize) -> InlineKeyboardMarkup {
    settings_words_styled(word_count).to_teloxide()
}

/// زیرمنوی دامنه‌های مجاز
pub fn settings_domains_styled(domain_count: usize) -> StyledKeyboard {
    StyledKeyboard::new(vec![
        vec![StyledButton::default(format!("تعداد دامنه‌های مجاز: {}", domain_count), "domains_status_toast")],
        vec![
            StyledButton::primary("🌐 مشاهده لیست دامنه‌ها", "list_allowed_domains"),
            StyledButton::danger("🗑️ پاک کردن همه دامنه‌ها", "clear_all_domains"),
        ],
        vec![StyledButton::primary("ℹ️ نحوه افزودن/حذف دامنه", "guide_allowed_domains")],
        vec![StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main")],
    ])
}

pub fn settings_domains_keyboard(domain_count: usize) -> InlineKeyboardMarkup {
    settings_domains_styled(domain_count).to_teloxide()
}

/// زیرمنوی دستورات دلخواه
pub fn settings_custom_cmds_styled(cmd_count: usize) -> StyledKeyboard {
    StyledKeyboard::new(vec![
        vec![StyledButton::default(format!("تعداد دستورات دلخواه: {}", cmd_count), "cmds_status_toast")],
        vec![
            StyledButton::primary("✨ مشاهده لیست دستورات", "list_custom_cmds_btn"),
            StyledButton::danger("🗑️ پاک کردن همه دستورات", "clear_all_custom_cmds"),
        ],
        vec![StyledButton::primary("ℹ️ نحوه تعریف دستور دلخواه", "guide_custom_cmds")],
        vec![StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main")],
    ])
}

pub fn settings_custom_cmds_keyboard(cmd_count: usize) -> InlineKeyboardMarkup {
    settings_custom_cmds_styled(cmd_count).to_teloxide()
}

/// زیرمنوی تنظیم اقدامات تنبیهی خودکار
pub fn settings_actions_styled(settings: &GroupSettings) -> StyledKeyboard {
    StyledKeyboard::new(vec![
        vec![StyledButton::primary(format!("ضداسپم ➔ {}", flood_action_display(&settings.anti_flood_action)), "cycle_flood_action")],
        vec![StyledButton::primary(format!("تکمیل اخطار ➔ {}", warn_action_display(&settings.warn_escalation_action)), "cycle_warn_action")],
        vec![StyledButton::primary(format!("شکست کپچا ➔ {}", captcha_action_display(&settings.captcha_fail_action)), "cycle_captcha_action")],
        vec![StyledButton::primary(format!("فیلتر لینک ➔ {}", link_action_display(&settings.link_filter_action)), "cycle_link_action")],
        vec![StyledButton::primary("🔙 بازگشت به منوی اصلی", "nav_main")],
    ])
}

pub fn settings_actions_keyboard(settings: &GroupSettings) -> InlineKeyboardMarkup {
    settings_actions_styled(settings).to_teloxide()
}

/// کیبورد تایید انسان بودن (کپچا برای اعضای جدید با دکمه سبز رنگ)
pub fn captcha_styled(user_id: u64) -> StyledKeyboard {
    StyledKeyboard::new(vec![vec![
        StyledButton::success("من انسان هستم ✅ (تایید عضویت)", format!("verify_captcha_{}", user_id)),
    ]])
}

#[allow(dead_code)]
pub fn captcha_keyboard(user_id: u64) -> InlineKeyboardMarkup {
    captcha_styled(user_id).to_teloxide()
}

/// کیبورد محاسباتی کپچا (سوال ریاضی ساده)
pub fn captcha_math_styled(user_id: u64, correct_answer: i32) -> StyledKeyboard {
    let mut options = vec![
        correct_answer,
        correct_answer + 1,
        (correct_answer - 2).max(1),
        correct_answer + 3,
    ];
    options.sort_by(|a, b| (a * 7 % 5).cmp(&(b * 7 % 5)));

    let buttons: Vec<StyledButton> = options
        .into_iter()
        .map(|ans| {
            let data = if ans == correct_answer {
                format!("math_correct_{}", user_id)
            } else {
                format!("math_wrong_{}", user_id)
            };
            StyledButton::primary(format!("{}", ans), data)
        })
        .collect();

    StyledKeyboard::new(vec![buttons])
}

pub fn captcha_math_keyboard(user_id: u64, correct_answer: i32) -> InlineKeyboardMarkup {
    captcha_math_styled(user_id, correct_answer).to_teloxide()
}

pub fn flood_action_display(action: &str) -> &'static str {
    match action {
        "mute_1h" => "سکوت ۱ ساعت 🔇",
        "kick" => "اخراج از گروه 🚪",
        "ban" => "مسدودسازی دائم ⛔",
        "warn" => "ثبت اخطار ⚠️",
        _ => "سکوت ۵ دقیقه 🔇",
    }
}

pub fn warn_action_display(action: &str) -> &'static str {
    match action {
        "ban" => "مسدودسازی دائم ⛔",
        "kick" => "اخراج از گروه 🚪",
        "mute_1h" => "سکوت ۱ ساعت 🔇",
        _ => "سکوت ۲۴ ساعت 🔇",
    }
}

pub fn captcha_action_display(action: &str) -> &'static str {
    match action {
        "ban" => "مسدودسازی دائم ⛔",
        "mute" => "سکوت ۲۴ ساعت 🔇",
        _ => "اخراج از گروه 🚪",
    }
}

pub fn link_action_display(action: &str) -> &'static str {
    match action {
        "warn" => "اخطار + حذف پیام ⚠️",
        "mute" => "سکوت ۱ ساعت 🔇",
        _ => "فقط حذف پیام 🗑️",
    }
}

/// دکمه‌های زیر ربات (Reply Keyboard) در پیوی برای مالک ربات و مالکان گروه‌ها
pub fn pv_owner_reply_keyboard() -> teloxide::types::KeyboardMarkup {
    use teloxide::types::{KeyboardButton, KeyboardMarkup};
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new("دسترسی ادمین‌ها"),
            KeyboardButton::new("پنل"),
        ],
        vec![
            KeyboardButton::new("راهنما"),
        ],
    ])
    .resize_keyboard()
    .persistent()
}

/// کیبورد شیشه‌ای انتخاب گروه برای مدیریت در پیوی ربات
pub fn pv_group_selection_styled(groups: &[(i64, String)], prefix: &str) -> StyledKeyboard {
    let mut rows: Vec<Vec<StyledButton>> = Vec::new();

    for (chat_id, title) in groups {
        rows.push(vec![
            StyledButton::primary(format!("👥 {}", title), format!("{}:{}", prefix, chat_id)),
        ]);
    }

    rows.push(vec![
        StyledButton::danger("❌ بستن", "close_settings"),
    ]);

    StyledKeyboard::new(rows)
}

/// کیبورد شیشه‌ای لیست ادمین‌های گروه به همراه گزینه سراسری (Global)
pub fn pv_admin_list_styled(chat_id: i64, admins: &[teloxide::types::ChatMember]) -> StyledKeyboard {
    let mut rows: Vec<Vec<StyledButton>> = Vec::new();

    // دکمه دسترسی سراسری (Global - تمام ادمین‌ها) در ابتدای لیست
    rows.push(vec![
        StyledButton::primary("🌐 سراسری (تمام ادمین‌های گروه)", format!("ap_admin:{}:0", chat_id)),
    ]);

    for admin in admins {
        // حذف ربات‌ها و مالک اصلی گروه از لیست تغییر دسترسی
        if admin.user.is_bot || matches!(admin.kind, teloxide::types::ChatMemberKind::Owner(_)) {
            continue;
        }

        let label = if let Some(ref uname) = admin.user.username {
            format!("👤 @{} ({})", uname, admin.user.id.0)
        } else {
            format!("👤 {} ({})", admin.user.first_name, admin.user.id.0)
        };

        rows.push(vec![
            StyledButton::primary(label, format!("ap_admin:{}:{}", chat_id, admin.user.id.0)),
        ]);
    }

    rows.push(vec![
        StyledButton::danger("🔙 بازگشت به لیست گروه‌ها", "pv_ap_back"),
    ]);

    StyledKeyboard::new(rows)
}

/// کیبورد شیشه‌ای مدیریت و جابجایی دسترسی‌های ادمین (Toggle Buttons)
pub fn pv_admin_permissions_styled(
    chat_id: i64,
    perms: &std::collections::HashMap<String, bool>,
) -> StyledKeyboard {
    use crate::db::admin_permissions::ALL_PERMISSION_KEYS;
    let mut rows: Vec<Vec<StyledButton>> = Vec::new();

    // ایجاد دکمه‌های دو به دو برای گزینه‌های پنل
    let mut current_row: Vec<StyledButton> = Vec::new();
    for (key, title) in ALL_PERMISSION_KEYS {
        let allowed = perms.get(*key).copied().unwrap_or(true);
        let status = if allowed { "مجاز 🟢" } else { "ممنوع 🔴" };
        let style = if allowed { "success" } else { "danger" };

        current_row.push(StyledButton::new(
            format!("{}: {}", title, status),
            format!("ap_tog:{}", key),
            Some(style),
        ));

        if current_row.len() == 2 {
            rows.push(current_row);
            current_row = Vec::new();
        }
    }
    if !current_row.is_empty() {
        rows.push(current_row);
    }

    // دکمه‌های تایید و اعمال یا لغو
    rows.push(vec![
        StyledButton::success("💾 تایید و اعمال", "ap_save"),
        StyledButton::danger("❌ لغو", "ap_cancel"),
    ]);

    // بازگشت به لیست ادمین‌ها
    rows.push(vec![
        StyledButton::primary("🔙 بازگشت به لیست ادمین‌ها", format!("ap_back_list:{}", chat_id)),
    ]);

    StyledKeyboard::new(rows)
}

