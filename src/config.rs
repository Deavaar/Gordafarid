use std::env;
use crate::errors::AppError;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bot_token: String,
    pub database_url: String,
    pub owner_id: i64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        let bot_token = env::var("TELOXIDE_TOKEN")
            .map_err(|_| AppError::Config("متغیر TELOXIDE_TOKEN در فایل .env یافت نشد.".to_string()))?;

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("متغیر DATABASE_URL در فایل .env یافت نشد.".to_string()))?;

        let owner_id = env::var("OWNER_ID")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<i64>()
            .map_err(|_| AppError::Config("شناسه OWNER_ID باید یک عدد صحیح باشد.".to_string()))?;

        Ok(Self {
            bot_token,
            database_url,
            owner_id,
        })
    }
}
