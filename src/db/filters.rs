use sqlx::PgPool;

pub async fn add_banned_word(
    pool: &PgPool,
    chat_id: i64,
    word: &str,
    list_type: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO banned_words (chat_id, word, list_type)
        VALUES ($1, $2, $3)
        ON CONFLICT (chat_id, word, list_type) DO NOTHING
        "#,
        chat_id,
        word,
        list_type
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_banned_word(
    pool: &PgPool,
    chat_id: i64,
    word: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM banned_words WHERE chat_id = $1 AND word = $2",
        chat_id,
        word
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_banned_words(
    pool: &PgPool,
    chat_id: i64,
) -> Result<Vec<(String, i32)>, sqlx::Error> {
    let records = sqlx::query!(
        "SELECT word, list_type FROM banned_words WHERE chat_id = $1",
        chat_id
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| (r.word, r.list_type)).collect())
}

pub async fn add_allowed_domain(
    pool: &PgPool,
    chat_id: i64,
    domain: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO allowed_domains (chat_id, domain)
        VALUES ($1, $2)
        ON CONFLICT (chat_id, domain) DO NOTHING
        "#,
        chat_id,
        domain
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_allowed_domain(
    pool: &PgPool,
    chat_id: i64,
    domain: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM allowed_domains WHERE chat_id = $1 AND domain = $2",
        chat_id,
        domain
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_allowed_domains(
    pool: &PgPool,
    chat_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    let records = sqlx::query!(
        "SELECT domain FROM allowed_domains WHERE chat_id = $1",
        chat_id
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| r.domain).collect())
}

pub async fn clear_all_banned_words(
    pool: &PgPool,
    chat_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM banned_words WHERE chat_id = $1",
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn clear_all_allowed_domains(
    pool: &PgPool,
    chat_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM allowed_domains WHERE chat_id = $1",
        chat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
