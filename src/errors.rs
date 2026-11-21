use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AppError {
    #[error("خطای پایگاه داده: {0}")]
    Database(#[from] sqlx::Error),

    #[error("خطای ارتباط با تلگرام: {0}")]
    Telegram(String),

    #[error("شما دسترسی لازم برای اجرای این دستور را ندارید.")]
    PermissionDenied,

    #[error("ربات دسترسی لازم در گروه را ندارد: {0}")]
    BotLacksPermission(&'static str),

    #[error("کاربر هدف مشخص نشده است. لطفاً روی پیام کاربر ریپلای کنید.")]
    TargetNotFound,

    #[error("مدت زمان نامعتبر است. نمونه: «۳۰ دقیقه» یا «۲ ساعت»")]
    InvalidDuration,

    #[error("دستور نامعتبر یا پارامترها ناقص هستند.")]
    InvalidCommand,

    #[error("تنظیمات گروه یافت نشد.")]
    GroupNotFound,

    #[error("خطای پیکربندی: {0}")]
    Config(String),
}

impl AppError {
    pub fn to_persian_message(&self) -> String {
        match self {
            AppError::Database(_) => "❌ خطای داخلی در پایگاه داده رخ داده است.".to_string(),
            AppError::Telegram(msg) => format!("❌ خطای تلگرام: {}", msg),
            AppError::PermissionDenied => "⛔ شما دسترسی لازم برای اجرای این دستور را ندارید.".to_string(),
            AppError::BotLacksPermission(perm) => format!("⚠️ ربات برای این عملیات به دسترسی «{}» نیاز دارد.", perm),
            AppError::TargetNotFound => "⚠️ کاربر هدف مشخص نشده است. لطفاً روی پیام کاربر ریپلای کنید یا شناسه عددی او را در دستور وارد نمایید (مثال: «میوت ۱۲۳۴۵۶۷۸ ۳۰ دقیقه»).".to_string(),
            AppError::InvalidDuration => "⚠️ مدت زمان مشخص‌شده معتبر نیست. مثال: «۳۰ دقیقه»، «۱ ساعت»، «۲ روز»".to_string(),
            AppError::InvalidCommand => "⚠️ دستور وارد شده نامعتبر است یا پارامترهای کافی ندارد.".to_string(),
            AppError::GroupNotFound => "⚠️ اطلاعات این گروه در سامانه ثبت نشده است.".to_string(),
            AppError::Config(msg) => format!("⚠️ خطای پیکربندی: {}", msg),
        }
    }
}
