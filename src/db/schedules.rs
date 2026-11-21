use sqlx::PgPool;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct ScheduledMessage {
    pub id: i32,
    pub chat_id: i64,
    pub message_text: String,
    pub interval_secs: Option<i32>,
    pub next_run_at: DateTime<Utc>,
    pub is_active: bool,
}

#[allow(dead_code)]
pub async fn add_scheduled_message(
    pool: &PgPool,
    chat_id: i64,
    message_text: &str,
    interval_secs: Option<i32>,
    next_run_at: DateTime<Utc>,
) -> Result<i32, sqlx::Error> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO scheduled_messages (chat_id, message_text, interval_secs, next_run_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        chat_id,
        message_text,
        interval_secs,
        next_run_at
    )
    .fetch_one(pool)
    .await?;

    Ok(id)
}

pub async fn get_due_schedules(pool: &PgPool) -> Result<Vec<ScheduledMessage>, sqlx::Error> {
    sqlx::query_as!(
        ScheduledMessage,
        r#"
        SELECT id, chat_id, message_text, interval_secs, next_run_at, is_active
        FROM scheduled_messages
        WHERE is_active = TRUE AND next_run_at <= NOW()
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn update_schedule_after_run(
    pool: &PgPool,
    id: i32,
    interval_secs: Option<i32>,
) -> Result<(), sqlx::Error> {
    if let Some(secs) = interval_secs {
        sqlx::query!(
            r#"
            UPDATE scheduled_messages
            SET next_run_at = NOW() + ($1 || ' seconds')::INTERVAL
            WHERE id = $2
            "#,
            secs.to_string(),
            id
        )
        .execute(pool)
        .await?;
    } else {
        sqlx::query!(
            "UPDATE scheduled_messages SET is_active = FALSE WHERE id = $1",
            id
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}
