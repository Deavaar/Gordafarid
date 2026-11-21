use sqlx::PgPool;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct TempPunishment {
    pub id: i32,
    pub chat_id: i64,
    pub user_id: i64,
    pub punishment_type: String, // 'mute', 'ban'
    pub expires_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub applied_by: i64,
    pub is_active: bool,
}

pub async fn add_warning(
    pool: &PgPool,
    chat_id: i64,
    user_id: i64,
    reason: &str,
    warned_by: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO warnings (chat_id, user_id, reason, warned_by)
        VALUES ($1, $2, $3, $4)
        "#,
        chat_id,
        user_id,
        reason,
        warned_by
    )
    .execute(pool)
    .await?;

    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM warnings WHERE chat_id = $1 AND user_id = $2",
        chat_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0))
}

pub async fn get_warnings_count(
    pool: &PgPool,
    chat_id: i64,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM warnings WHERE chat_id = $1 AND user_id = $2",
        chat_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0))
}

pub async fn clear_warnings(
    pool: &PgPool,
    chat_id: i64,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM warnings WHERE chat_id = $1 AND user_id = $2",
        chat_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn add_temp_punishment(
    pool: &PgPool,
    chat_id: i64,
    user_id: i64,
    punishment_type: &str,
    expires_at: DateTime<Utc>,
    reason: Option<&str>,
    applied_by: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO temporary_punishments (chat_id, user_id, punishment_type, expires_at, reason, applied_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        chat_id,
        user_id,
        punishment_type,
        expires_at,
        reason,
        applied_by
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_expired_punishments(pool: &PgPool) -> Result<Vec<TempPunishment>, sqlx::Error> {
    sqlx::query_as!(
        TempPunishment,
        r#"
        SELECT id, chat_id, user_id, punishment_type, expires_at, reason, applied_by, is_active
        FROM temporary_punishments
        WHERE is_active = TRUE AND expires_at <= NOW()
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn deactivate_punishment(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE temporary_punishments SET is_active = FALSE WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn deactivate_user_punishments(
    pool: &PgPool,
    chat_id: i64,
    user_id: i64,
    punishment_type: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE temporary_punishments SET is_active = FALSE WHERE chat_id = $1 AND user_id = $2 AND punishment_type = $3",
        chat_id,
        user_id,
        punishment_type
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn is_user_muted_active(
    pool: &PgPool,
    chat_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM temporary_punishments WHERE chat_id = $1 AND user_id = $2 AND punishment_type = 'mute' AND is_active = TRUE AND expires_at > NOW())",
        chat_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(exists.unwrap_or(false))
}

pub async fn clear_all_warnings_for_group(
    pool: &PgPool,
    chat_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM warnings WHERE chat_id = $1",
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
