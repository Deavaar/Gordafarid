use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct DbUser {
    pub user_id: i64,
    pub first_name: String,
    pub username: Option<String>,
}

pub async fn upsert_user(
    pool: &PgPool,
    user_id: i64,
    first_name: &str,
    username: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO known_users (user_id, first_name, username, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (user_id) DO UPDATE
        SET first_name = EXCLUDED.first_name,
            username = EXCLUDED.username,
            updated_at = NOW()
        "#,
        user_id,
        first_name,
        username
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_user_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<DbUser>, sqlx::Error> {
    let clean = username.trim_start_matches('@').to_lowercase();
    let rec = sqlx::query!(
        "SELECT user_id, first_name, username FROM known_users WHERE LOWER(username) = $1 LIMIT 1",
        clean
    )
    .fetch_optional(pool)
    .await?;

    Ok(rec.map(|r| DbUser {
        user_id: r.user_id,
        first_name: r.first_name,
        username: r.username,
    }))
}

pub async fn find_user_by_id(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<DbUser>, sqlx::Error> {
    let rec = sqlx::query!(
        "SELECT user_id, first_name, username FROM known_users WHERE user_id = $1 LIMIT 1",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(rec.map(|r| DbUser {
        user_id: r.user_id,
        first_name: r.first_name,
        username: r.username,
    }))
}
