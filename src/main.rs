mod config;
mod db;
mod engine;
mod errors;
mod handlers;
pub mod i18n;
mod services;
mod ui;

use teloxide::prelude::*;
use tracing::{error, info};

use crate::config::AppConfig;
use crate::handlers::callbacks::handle_callback;
use crate::handlers::commands::handle_message;
use crate::services::cache::AppCache;
use crate::services::scheduler::start_background_scheduler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. مقداردهی لاگر ساخت‌یافته
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,gordafarid=debug")),
        )
        .init();
    info!("در حال راه‌اندازی ربات مدیریت گروه گردآفرید...");

    // 2. بارگذاری پیکربندی
    let config = match AppConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!("خطا در بارگذاری پیکربندی: {}", e);
            std::process::exit(1);
        }
    };

    // 3. اتصال به پایگاه داده محلی PostgreSQL
    info!("در حال اتصال به پایگاه داده PostgreSQL...");
    let pool = match db::create_pool(&config.database_url).await {
        Ok(p) => p,
        Err(e) => {
            error!("عدم امکان اتصال به پایگاه داده: {}", e);
            std::process::exit(1);
        }
    };
    info!("اتصال به پایگاه داده برقرار شد.");

    // 4. اجرای مهاجرت‌های پایگاه داده
    if let Err(e) = db::run_migrations(&pool).await {
        error!("خطا در اعمال مهاجرت‌های پایگاه داده: {}", e);
        std::process::exit(1);
    }

    // 5. مقداردهی لایه کش در حافظه (In-Memory Cache)
    let cache = AppCache::new();

    // 6. ایجاد کلاینت ربات تلگرام
    let bot = Bot::new(&config.bot_token);
    let owner_id = config.owner_id;
    if let Ok(me) = bot.get_me().await {
        info!("شناسه ربات در تلگرام: {} (@{})", me.id.0, me.username());
        cache.set_bot_id(me.id.0);
    }

    // 7. راه‌اندازی سرویس پس‌زمینه رفع مجازات‌های زمان‌دار و پیام‌های زمان‌بندی‌شده
    let _scheduler_handle = start_background_scheduler(bot.clone(), pool.clone(), cache.clone());

    // 7.5. راه‌اندازی سرور پایش سلامت و متریک‌ها (Zero-Overhead TCP Health Server)
    let health_server = crate::services::health::HealthServer::new();
    if let Err(e) = health_server.start(8089).await {
        tracing::warn!("راه‌اندازی سرور پایش سلامت با خطا مواجه شد: {}", e);
    }

    // 8. ثبت هندلرهای پیام و دکمه‌های شیشه‌ای با DPTree
    info!("گردآفرید آماده دریافت پیام‌ها از تلگرام است.");

    let handler = dptree::entry()
        .branch(
            Update::filter_message().endpoint(
                move |b: Bot, m: Message, p: sqlx::PgPool, c: AppCache| {
                    handle_message(b, m, p, c, owner_id)
                },
            ),
        )
        .branch(
            Update::filter_callback_query().endpoint(
                move |b: Bot, q: CallbackQuery, p: sqlx::PgPool, c: AppCache| {
                    handle_callback(b, q, p, c, owner_id)
                },
            ),
        )
        .branch(
            Update::filter_channel_post().endpoint(
                move |b: Bot, m: Message, p: sqlx::PgPool, c: AppCache| {
                    crate::handlers::channel_post::handle_channel_post(b, m, p, c)
                },
            ),
        )
        .branch(
            Update::filter_edited_channel_post().endpoint(
                move |b: Bot, m: Message, p: sqlx::PgPool, c: AppCache| {
                    crate::handlers::channel_post::handle_channel_post(b, m, p, c)
                },
            ),
        );

    // 9. شروع شنود پیام‌ها با Long Polling و پشتیبانی از توقف تمیز (Graceful Shutdown)
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![pool, cache])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    info!("ربات گردآفرید با موفقیت متوقف شد.");
    Ok(())
}
