/// ماژول نرمال‌سازی متون فارسی، حروف عربی و ارقام
pub fn normalize_persian_text(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            // تبدیل حروف عربی به فارسی
            'ي' => result.push('ی'),
            'ك' => result.push('ک'),
            'ة' => result.push('ه'),
            'ؤ' => result.push('و'),
            'إ' | 'أ' | 'آ' => result.push('ا'),

            // تبدیل ارقام فارسی و عربی به ارقام استاندارد لاتین
            '۰' | '٠' => result.push('0'),
            '۱' | '١' => result.push('1'),
            '۲' | '٢' => result.push('2'),
            '۳' | '٣' => result.push('3'),
            '۴' | '٤' => result.push('4'),
            '۵' | '٥' => result.push('5'),
            '۶' | '٦' => result.push('6'),
            '۷' | '٧' => result.push('7'),
            '۸' | '٨' => result.push('8'),
            '۹' | '٩' => result.push('9'),

            // کشیده / تطویل (حذف کشیدگی متن)
            'ـ' => {}

            // نیم‌فاصله و نویسه‌های نامرئی به فاصله تبدیل می‌شوند تا کلمات ترکیبی تفکیک شوند
            '\u{200c}' | '\u{200b}' | '\u{200e}' | '\u{200f}' => result.push(' '),

            // سایر کاراکترها
            other => result.push(other),
        }
    }

    // نرمال‌سازی فاصله‌ها و حذف فضاهای خالی مازاد
    let collapsed: Vec<&str> = result.split_whitespace().collect();
    collapsed.join(" ")
}

/// تبدیل اعداد متن به ارقام انگلیسی
pub fn normalize_digits(input: &str) -> String {
    input
        .chars()
        .map(|ch| match ch {
            '۰' | '٠' => '0',
            '۱' | '١' => '1',
            '۲' | '٢' => '2',
            '۳' | '٣' => '3',
            '۴' | '٤' => '4',
            '۵' | '٥' => '5',
            '۶' | '٦' => '6',
            '۷' | '٧' => '7',
            '۸' | '٨' => '8',
            '۹' | '٩' => '9',
            c => c,
        })
        .collect()
}

/// کاراکترهای استاندارد جهت‌دهی و ایزولاسیون یونیکد
pub const RLM: &str = "\u{200F}"; // Right-to-Left Mark
pub const LRI: &str = "\u{2066}"; // Left-to-Right Isolate
pub const PDI: &str = "\u{2069}"; // Pop Directional Isolate

/// تبدیل ارقام انگلیسی به ارقام فارسی
pub fn to_persian_digits(input: &str) -> String {
    input
        .chars()
        .map(|ch| match ch {
            '0' => '۰',
            '1' => '۱',
            '2' => '۲',
            '3' => '۳',
            '4' => '۴',
            '5' => '۵',
            '6' => '۶',
            '7' => '۷',
            '8' => '۸',
            '9' => '۹',
            c => c,
        })
        .collect()
}

/// تبدیل مقدار عددی به ارقام فارسی
pub fn to_persian_num<T: std::fmt::Display>(val: T) -> String {
    to_persian_digits(&val.to_string())
}

/// ایزوله‌سازی توکن‌های انگلیسی، آیدی‌ها و اعداد برای عدم به‌هم‌ریختگی متن RTL
pub fn isolate_ltr(text: &str) -> String {
    format!("{}{}{}{}{}", RLM, LRI, text, PDI, RLM)
}

/// اعمال جهت‌گیری راست‌به‌چپ بر روی خطوط متنی
pub fn rtl_line(text: &str) -> String {
    format!("{}{}", RLM, text)
}

/// گریز دادن نویسه‌های HTML برای جلوگیری از شکستن تگ‌ها
pub fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_persian_digits() {
        assert_eq!(to_persian_digits("1234567890"), "۱۲۳۴۵۶۷۸۹۰");
        assert_eq!(to_persian_num(42), "۴۲");
    }

    #[test]
    fn test_isolate_ltr() {
        let isolated = isolate_ltr("@username");
        assert!(isolated.starts_with(RLM));
        assert!(isolated.contains("@username"));
        assert!(isolated.ends_with(RLM));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<b>Test & Fun</b>"), "&lt;b&gt;Test &amp; Fun&lt;/b&gt;");
    }

    #[test]
    fn test_persian_char_normalization() {
        assert_eq!(normalize_persian_text("يك پيام آزمايشي"), "یک پیام ازمایشی");
        assert_eq!(normalize_persian_text("مـتـن کـشـیـده"), "متن کشیده");
    }

    #[test]
    fn test_digit_normalization() {
        assert_eq!(normalize_digits("۱۲۳۴۵۶۷۸۹۰"), "1234567890");
        assert_eq!(normalize_digits("١٢٣٤٥٦٧٨٩٠"), "1234567890");
    }

    #[test]
    fn test_whitespace_normalization() {
        assert_eq!(
            normalize_persian_text("  بن   ۲    ساعت  اسپم  "),
            "بن 2 ساعت اسپم"
        );
    }

    #[test]
    fn test_zwnj_normalization() {
        assert_eq!(normalize_persian_text("بی‌صدا"), "بی صدا");
        assert_eq!(normalize_persian_text("خفه‌شو"), "خفه شو");
        assert_eq!(normalize_persian_text("پاک‌سازی"), "پاک سازی");
    }
}
