pub mod admin_permissions;
pub mod audit;
pub mod custom_commands;
pub mod filters;
pub mod groups;
pub mod moderation;
pub mod schedules;
pub mod users;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::info;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("در حال اجرای مهاجرت‌های پایگاه داده...");
    let schema_sql = include_str!("../../migrations/0001_initial_schema.sql");
    sqlx::raw_sql(schema_sql).execute(pool).await?;

    let schema_0002 = include_str!("../../migrations/0002_admin_permissions.sql");
    sqlx::raw_sql(schema_0002).execute(pool).await?;

    info!("مهاجرت‌های پایگاه داده با موفقیت اعمال شدند.");
    Ok(())
}
