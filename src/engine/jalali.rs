//! Pure Rust Gregorian to Solar Hijri (Jalali) astronomical converter.

use chrono::{Datelike, Timelike, Utc};
use chrono_tz::Asia::Tehran;
use crate::engine::normalizer::to_persian_num;

/// تبدیل تاریخ میلادی به هجری شمسی بر اساس الگوریتم نجومی کاظم بورکوفسکی
pub fn gregorian_to_jalali(gy: i32, gm: i32, gd: i32) -> (i32, i32, i32) {
    let g_d_m = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let gy2 = if gm > 2 { gy + 1 } else { gy };
    let mut days = 355_666 + (365 * gy) + ((gy2 + 3) / 4) - ((gy2 + 99) / 100)
        + ((gy2 + 399) / 400)
        + gd
        + g_d_m[(gm - 1) as usize];
    let mut jy = -1595 + 33 * (days / 12053);
    days %= 12053;
    jy += 4 * (days / 1461);
    days %= 1461;
    if days > 365 {
        jy += (days - 1) / 365;
        days = (days - 1) % 365;
    }
    let (jm, jd) = if days < 186 {
        (1 + days / 31, 1 + days % 31)
    } else {
        (7 + (days - 186) / 30, 1 + (days - 186) % 30)
    };
    (jy, jm, jd)
}

/// دریافت تاریخ فعلی شمسی (سال، ماه، روز) به وقت تهران
pub fn jalali_now() -> (i32, i32, i32) {
    let now = Utc::now().with_timezone(&Tehran);
    gregorian_to_jalali(now.year(), now.month() as i32, now.day() as i32)
}

/// برچسب زمانی شمسی به وقت تهران (فرمت لاتین YYYY/MM/DD HH:MM)
pub fn jalali_stamp() -> String {
    let now = Utc::now().with_timezone(&Tehran);
    let (jy, jm, jd) = gregorian_to_jalali(now.year(), now.month() as i32, now.day() as i32);
    format!(
        "{:04}/{:02}/{:02} {:02}:{:02}",
        jy,
        jm,
        jd,
        now.hour(),
        now.minute()
    )
}

/// برچسب زمانی شمسی با ارقام زیبای فارسی
pub fn jalali_persian_stamp() -> String {
    let now = Utc::now().with_timezone(&Tehran);
    let (jy, jm, jd) = gregorian_to_jalali(now.year(), now.month() as i32, now.day() as i32);
    format!(
        "{}/{}/{} {}:{}",
        to_persian_num(jy),
        to_persian_num(jm),
        to_persian_num(jd),
        to_persian_num(now.hour()),
        to_persian_num(now.minute())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gregorian_to_jalali() {
        assert_eq!(gregorian_to_jalali(2024, 3, 20), (1403, 1, 1));
        assert_eq!(gregorian_to_jalali(2026, 9, 17), (1405, 6, 26));
    }
}
