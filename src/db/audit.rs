use sqlx::PgPool;

pub async fn record_audit(
    pool: &PgPool,
    chat_id: i64,
    admin_id: i64,
    target_id: Option<i64>,
    action: &str,
    details: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO audit_logs (chat_id, admin_id, target_id, action, details)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        chat_id,
        admin_id,
        target_id,
        action,
        details
    )
    .execute(pool)
    .await?;
    Ok(())
}
