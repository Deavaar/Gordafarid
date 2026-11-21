use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use teloxide::types::UserId;
use crate::db::groups::GroupSettings;
use crate::engine::target::TargetUser;

#[derive(Clone, Default)]
pub struct AppCache {
    // شناسه خود ربات در حافظه
    pub bot_id: Arc<AtomicU64>,
    // کش تنظیمات هر گروه
    pub settings: Arc<DashMap<i64, GroupSettings>>,
    // کش کلمات ممنوعه
    pub banned_words: Arc<DashMap<i64, Vec<(String, i32)>>>,
    // کش دامنه‌های مجاز
    pub allowed_domains: Arc<DashMap<i64, Vec<String>>>,
    // پنجره لغزان ضداسپم: (chat_id, user_id) -> timestamps
    pub flood_tracker: Arc<DashMap<(i64, u64), Vec<Instant>>>,
    // وضعیت فعال بودن حالت شب در لحظه: chat_id -> bool
    pub is_night_active: Arc<DashMap<i64, bool>>,
    // کش نگاشت نام‌کاربری به اطلاعات کاربر
    pub user_cache: Arc<DashMap<String, TargetUser>>,
    // کش نگاشت شناسه عددی به اطلاعات کاربر
    pub id_to_user: Arc<DashMap<u64, TargetUser>>,
    // درخواست‌های معلق به‌روزرسانی حکم سکوت: token -> PendingMuteUpdate
    pub pending_mute_updates: Arc<DashMap<String, PendingMuteUpdate>>,
    // قوانین پست‌های کانال متصل (chat_id, post_id) -> allowed_type
    pub post_rules: Arc<DashMap<(i64, i32), String>>,
    // کش ادمین‌های گروه برای پاسخ‌دهی آنی (RBAC Cache): chat_id -> (admin_user_ids, cached_at)
    pub admin_cache: Arc<DashMap<i64, (Vec<u64>, Instant)>>,
    // کش وضعیت عضویت کاربران در کانال (Force-Join Cache): (chat_id, user_id) -> (is_member, cached_at)
    pub channel_membership_cache: Arc<DashMap<(i64, u64), (bool, Instant)>>,
    // گروه فعال در پیوی برای مدیریت از راه دور: user_id -> chat_id
    pub user_active_pv_group: Arc<DashMap<u64, i64>>,
    // پیش‌نویس موقت تغییر دسترسی‌های ادمین: owner_user_id -> AdminPermDraft
    pub admin_perm_drafts: Arc<DashMap<u64, AdminPermDraft>>,
}

#[derive(Clone, Debug)]
pub struct AdminPermDraft {
    pub chat_id: i64,
    pub target_admin_id: i64,
    pub target_admin_name: String,
    pub permissions: std::collections::HashMap<String, bool>,
}

#[derive(Clone, Debug)]
pub struct PendingMuteUpdate {
    pub target_id: u64,
    pub target_name: String,
    pub target_username: Option<String>,
    pub duration: Option<chrono::Duration>,
    pub reason: Option<String>,
    pub requested_by_admin: u64,
    pub created_at: Instant,
}

impl AppCache {
    pub fn new() -> Self {
        Self {
            bot_id: Arc::new(AtomicU64::new(0)),
            settings: Arc::new(DashMap::new()),
            banned_words: Arc::new(DashMap::new()),
            allowed_domains: Arc::new(DashMap::new()),
            flood_tracker: Arc::new(DashMap::new()),
            is_night_active: Arc::new(DashMap::new()),
            user_cache: Arc::new(DashMap::new()),
            id_to_user: Arc::new(DashMap::new()),
            pending_mute_updates: Arc::new(DashMap::new()),
            post_rules: Arc::new(DashMap::new()),
            admin_cache: Arc::new(DashMap::new()),
            channel_membership_cache: Arc::new(DashMap::new()),
            user_active_pv_group: Arc::new(DashMap::new()),
            admin_perm_drafts: Arc::new(DashMap::new()),
        }
    }

    pub fn get_user_active_pv_group(&self, user_id: u64) -> Option<i64> {
        self.user_active_pv_group.get(&user_id).map(|v| *v)
    }

    pub fn set_user_active_pv_group(&self, user_id: u64, chat_id: i64) {
        self.user_active_pv_group.insert(user_id, chat_id);
    }

    pub fn get_admin_perm_draft(&self, user_id: u64) -> Option<AdminPermDraft> {
        self.admin_perm_drafts.get(&user_id).map(|d| d.clone())
    }

    pub fn set_admin_perm_draft(&self, user_id: u64, draft: AdminPermDraft) {
        self.admin_perm_drafts.insert(user_id, draft);
    }

    pub fn remove_admin_perm_draft(&self, user_id: u64) -> Option<AdminPermDraft> {
        self.admin_perm_drafts.remove(&user_id).map(|(_, d)| d)
    }

    pub fn toggle_admin_perm_draft(&self, user_id: u64, key: &str) -> Option<bool> {
        if let Some(mut entry) = self.admin_perm_drafts.get_mut(&user_id) {
            let current = entry.permissions.get(key).copied().unwrap_or(true);
            let new_val = !current;
            entry.permissions.insert(key.to_string(), new_val);
            Some(new_val)
        } else {
            None
        }
    }

    pub fn set_bot_id(&self, id: u64) {
        self.bot_id.store(id, Ordering::Relaxed);
    }

    pub fn get_bot_id(&self) -> Option<UserId> {
        let val = self.bot_id.load(Ordering::Relaxed);
        if val > 0 {
            Some(UserId(val))
        } else {
            None
        }
    }

    pub fn record_user(&self, target: &TargetUser) {
        self.id_to_user.insert(target.id.0, target.clone());
        if let Some(ref uname) = target.username {
            let clean = uname.trim_start_matches('@').to_lowercase();
            self.user_cache.insert(clean, target.clone());
        }
    }

    pub fn get_user_by_username(&self, username: &str) -> Option<TargetUser> {
        let clean = username.trim_start_matches('@').to_lowercase();
        self.user_cache.get(&clean).map(|u| u.clone())
    }

    pub fn get_user_by_id(&self, user_id: u64) -> Option<TargetUser> {
        self.id_to_user.get(&user_id).map(|u| u.clone())
    }

    /// ثبت پیام جدید و بررسی نقض حد مجاز فلاد (Anti-Flood)
    pub fn check_flood(
        &self,
        chat_id: i64,
        user_id: u64,
        rate: usize,
        window_secs: u64,
    ) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(window_secs);
        let key = (chat_id, user_id);

        let mut entry = self.flood_tracker.entry(key).or_default();
        entry.retain(|&time| now.duration_since(time) <= window);
        entry.push(now);

        entry.len() > rate
    }

    pub fn set_night_active(&self, chat_id: i64, active: bool) {
        self.is_night_active.insert(chat_id, active);
    }

    pub fn is_night_mode_on(&self, chat_id: i64) -> bool {
        self.is_night_active.get(&chat_id).map(|r| *r).unwrap_or(false)
    }

    pub fn invalidate_settings(&self, chat_id: i64) {
        self.settings.remove(&chat_id);
    }

    pub fn invalidate_banned_words(&self, chat_id: i64) {
        self.banned_words.remove(&chat_id);
    }

    pub fn invalidate_allowed_domains(&self, chat_id: i64) {
        self.allowed_domains.remove(&chat_id);
    }

    pub fn store_pending_mute(
        &self,
        target_id: u64,
        target_name: String,
        target_username: Option<String>,
        duration: Option<chrono::Duration>,
        reason: Option<String>,
        admin_id: u64,
    ) -> String {
        // حذف توکن‌های منقضی‌شده (> ۱۰ دقیقه / ۶۰۰ ثانیه)
        let now = Instant::now();
        self.pending_mute_updates.retain(|_, v| now.duration_since(v.created_at) < Duration::from_secs(600));

        let token = format!("{:x}", rand_u64());
        self.pending_mute_updates.insert(
            token.clone(),
            PendingMuteUpdate {
                target_id,
                target_name,
                target_username,
                duration,
                reason,
                requested_by_admin: admin_id,
                created_at: now,
            },
        );
        token
    }

    pub fn get_pending_mute(&self, token: &str) -> Option<PendingMuteUpdate> {
        let entry = self.pending_mute_updates.get(token)?;
        if entry.created_at.elapsed() < Duration::from_secs(600) {
            Some(entry.clone())
        } else {
            None
        }
    }

    pub fn remove_pending_mute(&self, token: &str) -> Option<PendingMuteUpdate> {
        self.pending_mute_updates.remove(token).map(|(_, v)| v)
    }

    pub fn set_post_rule(&self, chat_id: i64, post_id: i32, allowed: String) {
        self.post_rules.insert((chat_id, post_id), allowed);
    }

    pub fn get_post_rule(&self, chat_id: i64, post_id: i32) -> Option<String> {
        self.post_rules.get(&(chat_id, post_id)).map(|r| r.clone())
    }

    // RBAC Admin Caching (3 minutes TTL)
    pub fn get_group_admins(&self, chat_id: i64) -> Option<Vec<u64>> {
        let entry = self.admin_cache.get(&chat_id)?;
        if entry.1.elapsed() < Duration::from_secs(180) {
            Some(entry.0.clone())
        } else {
            None
        }
    }

    pub fn set_group_admins(&self, chat_id: i64, admins: Vec<u64>) {
        self.admin_cache.insert(chat_id, (admins, Instant::now()));
    }

    pub fn invalidate_group_admins(&self, chat_id: i64) {
        self.admin_cache.remove(&chat_id);
    }

    // Force-Join Membership Caching (30m for members, 1m for non-members)
    pub fn get_channel_membership(&self, chat_id: i64, user_id: u64) -> Option<bool> {
        let entry = self.channel_membership_cache.get(&(chat_id, user_id))?;
        let ttl = if entry.0 {
            Duration::from_secs(1800)
        } else {
            Duration::from_secs(60)
        };
        if entry.1.elapsed() < ttl {
            Some(entry.0)
        } else {
            None
        }
    }

    pub fn set_channel_membership(&self, chat_id: i64, user_id: u64, is_member: bool) {
        self.channel_membership_cache.insert((chat_id, user_id), (is_member, Instant::now()));
    }

    pub fn invalidate_channel_membership(&self, chat_id: i64, user_id: u64) {
        self.channel_membership_cache.remove(&(chat_id, user_id));
    }
}

fn rand_u64() -> u64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    (nanos ^ (nanos >> 32)) as u64
}
