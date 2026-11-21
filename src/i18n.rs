use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nConfig {
    pub start: StartMessages,
    pub help: String,
    pub poems: PoemsConfig,
    pub notices: NoticesConfig,
    pub moderation: ModerationMessages,
    pub captcha: CaptchaMessages,
    pub settings: SettingsMessages,
    pub pv: PvMessages,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvMessages {
    pub select_group_panel: String,
    pub select_group_admin_perms: String,
    pub no_owned_groups: String,
    #[serde(default = "default_no_admin_groups")]
    pub no_admin_groups: String,
    pub permission_denied_admin: String,
    pub admin_perms_saved: String,
    pub admin_perms_cancelled: String,
}

fn default_no_admin_groups() -> String {
    "<blockquote>{RLM}⚠️ شما در هیچ‌یک از گروه‌های فعال ربات دسترسی مدیریت ندارید.</blockquote>".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartMessages {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoemsConfig {
    pub mute: Vec<String>,
    pub ban: Vec<String>,
    pub warn: Vec<String>,
    pub kick: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticesConfig {
    pub ban: NoticeBanConfig,
    pub mute: NoticeMuteConfig,
    pub warn: NoticeWarnConfig,
    pub kick: NoticeKickConfig,
    pub unban: String,
    pub unmute: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeBanConfig {
    pub title: String,
    pub outcome_timed: String,
    pub outcome_permanent: String,
    pub reason_template: String,
    pub quote_author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeMuteConfig {
    pub title: String,
    pub duration_timed: String,
    pub duration_permanent: String,
    pub reason_template: String,
    pub quote_author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeWarnConfig {
    pub title: String,
    pub count_template: String,
    pub reason_template: String,
    pub quote_author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeKickConfig {
    pub title: String,
    pub outcome: String,
    pub reason_template: String,
    pub quote_author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationMessages {
    pub mute_update_prompt: String,
    pub mute_update_success: String,
    pub mute_update_rejected: String,
    pub mute_update_expired: String,
    pub cannot_punish_admin: String,
    pub cannot_kick_admin: String,
    pub cannot_mute_admin: String,
    pub cannot_warn_admin: String,
    pub user_not_member: String,
    pub warn_escalation_ban: String,
    pub warn_escalation_kick: String,
    pub warn_escalation_mute_1h: String,
    pub warn_escalation_mute_24h: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaMessages {
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsMessages {
    pub prompt: String,
}

struct RamCacheEntry {
    config: Arc<I18nConfig>,
    mtime: Option<SystemTime>,
}

static RAM_CACHE: RwLock<Option<RamCacheEntry>> = RwLock::new(None);

fn get_redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
}

/// لایه ۲: دریافت متن خام کانفیگ از ردیس
fn fetch_from_redis() -> Option<String> {
    let url = get_redis_url();
    let client = redis::Client::open(url).ok()?;
    let mut conn = client.get_connection().ok()?;
    redis::Commands::get(&mut conn, "gordafarid:i18n:config").ok()
}

/// لایه ۲: ذخیره متن خام کانفیگ در ردیس
fn save_to_redis(json_str: &str) {
    let url = get_redis_url();
    if let Ok(client) = redis::Client::open(url) {
        if let Ok(mut conn) = client.get_connection() {
            let _: Result<(), _> = redis::Commands::set(&mut conn, "gordafarid:i18n:config", json_str);
        }
    }
}

/// لایه ۳: خواندن مستقیم فایل دیسک
fn read_disk_file() -> (String, Option<SystemTime>) {
    let mtime = std::fs::metadata("i18n.json")
        .and_then(|m| m.modified())
        .ok();
    let content = std::fs::read_to_string("i18n.json")
        .unwrap_or_else(|_| include_str!("../i18n.json").to_string());
    let normalized = content.replace("{RLM}", "\u{200F}");
    (normalized, mtime)
}

/// بازخوانی و اعمال مجدد متون i18n در تمام لایه‌ها (RAM -> Redis -> Disk)
pub fn reload_i18n() -> Result<Arc<I18nConfig>, String> {
    let (content, mtime) = read_disk_file();
    let config: I18nConfig = serde_json::from_str(&content)
        .map_err(|e| format!("خطا در تحلیل ساختار JSON فایل i18n.json: {}", e))?;
    let arc_config = Arc::new(config);

    // به‌روزرسانی ردیس (Layer 2)
    save_to_redis(&content);

    // به‌روزرسانی رم (Layer 1)
    if let Ok(mut lock) = RAM_CACHE.write() {
        *lock = Some(RamCacheEntry {
            config: arc_config.clone(),
            mtime,
        });
    }

    info!("✅ متون i18n با موفقیت در لایه‌های RAM و Redis به‌روزرسانی شدند.");
    Ok(arc_config)
}

/// دریافت پیکربندی متون با معماری ۳ لایه:
/// ۱. حافظه موقت (RAM)
/// ۲. پایگاه داده ردیس (Redis)
/// ۳. فایل دیسک (i18n.json)
pub fn get_i18n() -> Arc<I18nConfig> {
    let current_mtime = std::fs::metadata("i18n.json")
        .and_then(|m| m.modified())
        .ok();

    // ۱. بررسی لایه رم (RAM)
    if let Ok(lock) = RAM_CACHE.read() {
        if let Some(entry) = lock.as_ref() {
            // اگر فایل روی دیسک تغییر نکرده باشد، مستقیماً از حافظه رم برگردان
            if entry.mtime == current_mtime && current_mtime.is_some() {
                return entry.config.clone();
            }
        }
    }

    // اگر رم خالی بود:
    let is_empty = RAM_CACHE.read().map(|l| l.is_none()).unwrap_or(true);
    if is_empty {
        // ۲. بررسی لایه ردیس (Redis)
        if let Some(cached_json) = fetch_from_redis() {
            if let Ok(config) = serde_json::from_str::<I18nConfig>(&cached_json) {
                let arc_config = Arc::new(config);
                if let Ok(mut lock) = RAM_CACHE.write() {
                    *lock = Some(RamCacheEntry {
                        config: arc_config.clone(),
                        mtime: current_mtime,
                    });
                }
                return arc_config;
            }
        }
    }

    // ۳. بارگذاری از فایل دیسک و سنکرون با ردیس و رم
    match reload_i18n() {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("⚠️ خطا در بارگذاری متون i18n از دیسک، استفاده از مقادیر تعبیه‌شده: {}", e);
            let fallback_content = include_str!("../i18n.json").replace("{RLM}", "\u{200F}");
            let fallback_cfg: I18nConfig = serde_json::from_str(&fallback_content)
                .expect("فایل تعبیه‌شده i18n.json نامعتبر است");
            let arc = Arc::new(fallback_cfg);
            if let Ok(mut lock) = RAM_CACHE.write() {
                *lock = Some(RamCacheEntry {
                    config: arc.clone(),
                    mtime: None,
                });
            }
            arc
        }
    }
}
