//! Smart Spam & Link Risk Scorer (برگرفته از ماژول spam-gate-scorer)
//!
//! موتور ارزیابی ریسک پیام‌ها و لینک‌ها برای مقابله با کوتاه‌کننده‌های لینک،
//! کلاهبرداری‌های کریپتو، سایت‌های شرط‌بندی/قمار و فیشینگ.

use regex::Regex;
use std::sync::OnceLock;

/// الگوهای کوتاه‌کننده‌های لینک پرخطر
static SHORTENER_REGEX: OnceLock<Regex> = OnceLock::new();
/// الگوهای ایردراپ و کلاهبرداری کریپتو
static CRYPTO_SCAM_REGEX: OnceLock<Regex> = OnceLock::new();
/// الگوهای شرط‌بندی، قمار و کازینو
static GAMBLING_REGEX: OnceLock<Regex> = OnceLock::new();
/// الگوهای فیشینگ و مهندسی اجتماعی فارسی
static PHISHING_REGEX: OnceLock<Regex> = OnceLock::new();
/// لینک‌های عضویت مخفی تلگرام
static TG_INVITE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_shortener_regex() -> &'static Regex {
    SHORTENER_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\b(bit\.ly|tinyurl\.com|shorturl\.at|cutt\.ly|is\.gd|t\.co|ow\.ly|goo\.gl|rebrand\.ly|soo\.gd|1url\.cz|v\.gd)\b").unwrap()
    })
}

fn get_crypto_scam_regex() -> &'static Regex {
    CRYPTO_SCAM_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(airdrop|free\s*crypto|claim\s*token|earn\s*usdt|free\s*bitcoin|cloud\s*mining|ایردراپ|ترون\s*رایگان|استخراج\s*ابری|سود\s*روزانه|سیگنال\s*رایگان)").unwrap()
    })
}

fn get_gambling_regex() -> &'static Regex {
    GAMBLING_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(casino|betting|gambling|lottery|crash\s*game|شرط\s*بندی|پیش\s*بینی|انفجار|کازینو|پوکر\s*آنلاین|سایت\s*بت)").unwrap()
    })
}

fn get_phishing_regex() -> &'static Regex {
    PHISHING_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(شارژ\s*رایگان|اینترنت\s*رایگان|هک\s*تلگرام|مچ\s*گیری|صیغه\s*یابی|دوربین\s*مخفی|واریز\s*فوری|سهام\s*عدالت|سبد\s*معیشتی)").unwrap()
    })
}

fn get_tg_invite_regex() -> &'static Regex {
    TG_INVITE_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(t\.me/(joinchat/|\+[\w-]+))").unwrap()
    })
}

#[derive(Debug, Clone)]
pub struct SpamRiskEvaluation {
    pub is_spam: bool,
    pub risk_score: u32,
    pub reasons: Vec<&'static str>,
}

/// ارزیابی جامع ریسک اسپم و فیشینگ یک پیام (امتیازدهی ۰ تا ۱۰۰)
pub fn evaluate_message_spam_risk(text: &str) -> SpamRiskEvaluation {
    let mut score = 0u32;
    let mut reasons = Vec::new();

    // ۱. بررسی کوتاه‌کننده‌های لینک ناشناس (+۴۵ ریسک)
    if get_shortener_regex().is_match(text) {
        score += 45;
        reasons.push("کوتاه‌کننده لینک ناشناس و مشکوک");
    }

    // ۲. بررسی تبلیغات ایردراپ و کلاهبرداری رمزارز (+۴۰ ریسک)
    if get_crypto_scam_regex().is_match(text) {
        score += 40;
        reasons.push("کلاهبرداری رمزارز یا ایردراپ فیک");
    }

    // ۳. بررسی سایت‌های شرط‌بندی و قمار (+۴۵ ریسک)
    if get_gambling_regex().is_match(text) {
        score += 45;
        reasons.push("تبلیغ سایت‌های قمار، بت و شرط‌بندی");
    }

    // ۴. بررسی فیشینگ، اینترنت رایگان و مهندسی اجتماعی (+۴۰ ریسک)
    if get_phishing_regex().is_match(text) {
        score += 40;
        reasons.push("الگوهای فیشینگ و مهندسی اجتماعی");
    }

    // ۵. لینک‌های جوین تلگرام (+۳۰ ریسک)
    if get_tg_invite_regex().is_match(text) {
        score += 30;
        reasons.push("لینک دعوت خصوصی کانال/گروه تلگرام");
    }

    SpamRiskEvaluation {
        is_spam: score >= 50,
        risk_score: score,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortener_and_crypto_flagged() {
        let text = "از این لینک بیت‌لی ثبت‌نام کنید و ایردراپ ترون رایگان بگیرید: bit.ly/test";
        let res = evaluate_message_spam_risk(text);
        assert!(res.is_spam);
        assert!(res.risk_score >= 50);
        assert!(res.reasons.contains(&"کوتاه‌کننده لینک ناشناس و مشکوک"));
        assert!(res.reasons.contains(&"کلاهبرداری رمزارز یا ایردراپ فیک"));
    }

    #[test]
    fn test_gambling_flagged() {
        let text = "بهترین سایت شرط بندی و کازینو آنلاین با واریز فوری";
        let res = evaluate_message_spam_risk(text);
        assert!(res.is_spam);
        assert!(res.risk_score >= 45);
    }

    #[test]
    fn test_normal_message_safe() {
        let text = "سلام دوستان، کسی می‌داند چطور باید در لینوکس کرنل آرچ را آپدیت کرد؟";
        let res = evaluate_message_spam_risk(text);
        assert!(!res.is_spam);
        assert_eq!(res.risk_score, 0);
    }
}
