use sqlx::PgPool;

pub async fn add_custom_command(
    pool: &PgPool,
    chat_id: i64,
    trigger: &str,
    response: &str,
    match_type: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO custom_commands (chat_id, trigger, response, match_type)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (chat_id, trigger) DO UPDATE SET response = EXCLUDED.response, match_type = EXCLUDED.match_type
        "#,
        chat_id,
        trigger,
        response,
        match_type
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_custom_command(
    pool: &PgPool,
    chat_id: i64,
    trigger: &str,
) -> Result<Option<String>, sqlx::Error> {
    let record = sqlx::query!(
        "SELECT response FROM custom_commands WHERE chat_id = $1 AND trigger = $2",
        chat_id,
        trigger
    )
    .fetch_optional(pool)
    .await?;

    Ok(record.map(|r| r.response))
}

pub async fn delete_custom_command(
    pool: &PgPool,
    chat_id: i64,
    trigger: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM custom_commands WHERE chat_id = $1 AND trigger = $2",
        chat_id,
        trigger
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_custom_commands(
    pool: &PgPool,
    chat_id: i64,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    let records = sqlx::query!(
        "SELECT trigger, response FROM custom_commands WHERE chat_id = $1",
        chat_id
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| (r.trigger, r.response)).collect())
}

pub async fn clear_all_custom_commands(
    pool: &PgPool,
    chat_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM custom_commands WHERE chat_id = $1",
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
