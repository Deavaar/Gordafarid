use sqlx::PgPool;
use std::collections::HashMap;

pub const ALL_PERMISSION_KEYS: &[(&str, &str)] = &[
    ("locks", "مدیریت قفل‌های گروه"),
    ("welcome", "پیام خوشامد و خروج"),
    ("flood", "سیستم ضداسپم"),
    ("night", "قفل شبانه گروه"),
    ("captcha", "تایید هویت (کپچا)"),
    ("warn", "سقف اخطار و تنبیهات"),
    ("channel", "کانال عضویت اجباری"),
    ("service_msg", "اعلان‌های تلگرام"),
    ("words", "فیلتر کلمات ممنوعه"),
    ("domains", "دامنه‌های مجاز"),
    ("custom_cmds", "دستورات دلخواه"),
    ("channel_track", "رهگیری کانال"),
];

pub fn default_permissions_map() -> HashMap<String, bool> {
    let mut map = HashMap::new();
    for (k, _) in ALL_PERMISSION_KEYS {
        map.insert(k.to_string(), true);
    }
    map
}

pub async fn get_admin_permissions(
    pool: &PgPool,
    chat_id: i64,
    admin_id: i64,
) -> Result<HashMap<String, bool>, sqlx::Error> {
    let mut perms = default_permissions_map();

    let rows = sqlx::query!(
        r#"
        SELECT permission_key, allowed
        FROM admin_permissions
        WHERE chat_id = $1 AND admin_id = $2
        "#,
        chat_id,
        admin_id
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        perms.insert(row.permission_key, row.allowed);
    }

    Ok(perms)
}

pub async fn save_admin_permissions_batch(
    pool: &PgPool,
    chat_id: i64,
    admin_id: i64,
    perms: &HashMap<String, bool>,
) -> Result<(), sqlx::Error> {
    for (key, allowed) in perms {
        sqlx::query!(
            r#"
            INSERT INTO admin_permissions (chat_id, admin_id, permission_key, allowed, updated_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (chat_id, admin_id, permission_key)
            DO UPDATE SET allowed = EXCLUDED.allowed, updated_at = NOW()
            "#,
            chat_id,
            admin_id,
            key,
            allowed
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn is_admin_feature_allowed(
    pool: &PgPool,
    chat_id: i64,
    user_id: u64,
    feature_key: &str,
    is_owner: bool,
) -> bool {
    // مالک گروه همیشه دسترسی کامل دارد و هیچ وقت دسترسی‌اش کم نمی‌شود
    if is_owner {
        return true;
    }

    // ۱. بررسی دسترسی اختصاصی ادمین
    let specific = sqlx::query_scalar!(
        r#"
        SELECT allowed
        FROM admin_permissions
        WHERE chat_id = $1 AND admin_id = $2 AND permission_key = $3
        "#,
        chat_id,
        user_id as i64,
        feature_key
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some(allowed) = specific {
        return allowed;
    }

    // ۲. بررسی دسترسی سراسری (Global - admin_id = 0)
    let global = sqlx::query_scalar!(
        r#"
        SELECT allowed
        FROM admin_permissions
        WHERE chat_id = $1 AND admin_id = 0 AND permission_key = $2
        "#,
        chat_id,
        feature_key
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some(allowed) = global {
        return allowed;
    }

    // پیش‌فرض: دسترسی مجاز است
    true
}
