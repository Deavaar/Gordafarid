use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, MessageId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StyledButton {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>, // "success" (green), "danger" (red), "primary" (blue)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_custom_emoji_id: Option<String>,
}

impl StyledButton {
    pub fn new<T: Into<String>, C: Into<String>>(text: T, callback_data: C, style: Option<&str>) -> Self {
        let raw_text = text.into();
        let cb = callback_data.into();
        let (clean_text, inferred_icon) = infer_icon_and_clean_text(&raw_text, Some(&cb), style);
        Self {
            text: clean_text,
            callback_data: Some(cb),
            url: None,
            style: style.map(|s| s.to_string()),
            icon_custom_emoji_id: inferred_icon,
        }
    }

    /// افزودن یا جایگزینی آیکون متحرک پریمیوم اختصاصی به دکمه شیشه‌ای (Bot API 9.4+)
    pub fn with_icon<I: Into<String>>(mut self, icon_id: I) -> Self {
        self.icon_custom_emoji_id = Some(icon_id.into());
        self
    }

    /// حذف آیکون از روی دکمه (مثلاً برای اعداد در کپچا)
    pub fn without_icon(mut self) -> Self {
        self.icon_custom_emoji_id = None;
        self
    }

    /// دکمه شیشه‌ای سبز (مخصوص قفل‌های فعال و موارد تایید)
    pub fn success<T: Into<String>, C: Into<String>>(text: T, callback_data: C) -> Self {
        Self::new(text, callback_data, Some("success"))
    }

    /// دکمه شیشه‌ای قرمز (مخصوص قفل‌های خاموش و اقدامات خطرناک مثل بن یا لغو)
    pub fn danger<T: Into<String>, C: Into<String>>(text: T, callback_data: C) -> Self {
        Self::new(text, callback_data, Some("danger"))
    }

    /// دکمه شیشه‌ای آبی (مخصوص ناوبری، راهنما و منوهای فرعی)
    pub fn primary<T: Into<String>, C: Into<String>>(text: T, callback_data: C) -> Self {
        Self::new(text, callback_data, Some("primary"))
    }

    /// دکمه شیشه‌ای پیش‌فرض تلگرام
    pub fn default<T: Into<String>, C: Into<String>>(text: T, callback_data: C) -> Self {
        Self::new(text, callback_data, None)
    }

    pub fn url<T: Into<String>, U: Into<String>>(text: T, url: U, style: Option<&str>) -> Self {
        let raw_text = text.into();
        let u = url.into();
        let (clean_text, mut inferred_icon) = infer_icon_and_clean_text(&raw_text, None, style);
        if inferred_icon.is_none() && (u.contains("startgroup") || u.contains("startchannel")) {
            inferred_icon = Some("5877219383691972108".to_string()); // plus icon
        }
        Self {
            text: clean_text,
            callback_data: None,
            url: Some(u),
            style: style.map(|s| s.to_string()),
            icon_custom_emoji_id: inferred_icon,
        }
    }

    pub fn to_teloxide(&self) -> InlineKeyboardButton {
        if let Some(ref cb) = self.callback_data {
            InlineKeyboardButton::callback(self.text.clone(), cb.clone())
        } else if let Some(ref u) = self.url {
            InlineKeyboardButton::url(
                self.text.clone(),
                u.parse().unwrap_or_else(|_| "https://t.me".parse().unwrap()),
            )
        } else {
            InlineKeyboardButton::callback(self.text.clone(), "")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StyledKeyboard {
    pub inline_keyboard: Vec<Vec<StyledButton>>,
}

impl StyledKeyboard {
    pub fn new(rows: Vec<Vec<StyledButton>>) -> Self {
        Self { inline_keyboard: rows }
    }

    pub fn to_teloxide(&self) -> InlineKeyboardMarkup {
        let rows: Vec<Vec<InlineKeyboardButton>> = self.inline_keyboard
            .iter()
            .map(|row| row.iter().map(|b| b.to_teloxide()).collect())
            .collect();
        InlineKeyboardMarkup::new(rows)
    }
}

/// نگاشت کامل ایموجی‌های پرکاربرد گردآفرید به شناسه‌های پریمیوم پک TgAndroidIcons (برگرفته از متدولوژی رستم)
pub static EMOJI_MAP: &[(&str, &str)] = &[
    // --- قفل‌ها، سپر و امنیت ---
    ("🛡️", "5926783847453692661"),
    ("🛡", "5926783847453692661"),
    ("🔒", "5832546462478635761"),
    ("🔓", "6034962180875490251"),
    ("🚫", "5872829476143894491"),
    ("⛔", "5872829476143894491"),
    ("⚠️", "5881702736843511327"),
    ("⚠", "5881702736843511327"),
    ("🔇", "5890838600433536921"),
    ("🔈", "5891206318353551398"),
    ("🔊", "5890997763331591703"),
    ("🚨", "5909201569898827582"),
    ("🔔", "5909201569898827582"),
    ("🔕", "5909123362839335003"),
    ("📌", "5908961403917570106"),
    ("🗑️", "5879896690210639947"),
    ("🗑", "5879896690210639947"),
    ("🧹", "5845947563601041174"),
    ("🔨", "5875450995332353523"),
    ("🛠", "5875450995332353523"),
    ("👢", "5877341274863832725"),
    ("🚪", "5877341274863832725"),
    ("🔑", "6005570495603282482"),

    // --- وضعیت‌ها و چک‌لیست‌ها ---
    ("🟢", "5776375003280838798"),
    ("🔴", "5967333011652350314"),
    ("⚪️", "5994324703559290598"),
    ("⚪", "5994324703559290598"),
    ("🔘", "5994324703559290598"),
    ("☑️", "5776375003280838798"),
    ("☑", "5776375003280838798"),
    ("⬜", "5994324703559290598"),
    ("✅️", "5776375003280838798"),
    ("✅", "5776375003280838798"),
    ("✔️", "5825794181183836432"),
    ("✔", "5825794181183836432"),
    ("❌", "5778527486270770928"),

    // --- ناوبری و منوها ---
    ("🌸", "5967574255670399788"),
    ("👋", "5994750571041525522"),
    ("🌙", "5886366951067883092"),
    ("☀️", "6005924903419646424"),
    ("🤖", "5819078828017849357"),
    ("📢", "5771695636411847302"),
    ("💬", "5886666250158870040"),
    ("📋", "5839323457015256759"),
    ("📜", "5839323457015256759"),
    ("📖", "5897850551156084824"),
    ("📝", "5886330010054168711"),
    ("📄", "5839323457015256759"),
    ("🌐", "5879585266426973039"),
    ("✨", "5843843420468024653"),
    ("⭐️", "5843843420468024653"),
    ("⭐", "5843843420468024653"),
    ("⚡️", "5843553939672274145"),
    ("⚡", "5843553939672274145"),
    ("📡", "5874986954180791957"),
    ("📶", "5874986954180791957"),
    ("⚙️", "5877260593903177342"),
    ("⚙", "5877260593903177342"),
    ("📊", "5877485980901971030"),
    ("📈", "5776219138917668486"),
    ("📅", "5967412305338568701"),
    ("👑", "5807461353799030682"),
    ("👤", "5771887475421090729"),
    ("👥", "5915556996215476302"),
    ("🔙", "5875082500023258804"),
    ("⬅️", "5875082500023258804"),
    ("⬅", "5875082500023258804"),
    ("◀️", "5877629862306385808"),
    ("◀", "5877629862306385808"),
    ("➔", "5877468380125990242"),
    ("➡️", "5877468380125990242"),
    ("➡", "5877468380125990242"),
    ("➕", "5877219383691972108"),
    ("🔍", "5874960879434338403"),
    ("🔎", "5874960879434338403"),
    ("🔄", "5839200986022812209"),
    ("🔁", "5839200986022812209"),
    ("ℹ️", "5877477876298681283"),
    ("ℹ", "5877477876298681283"),

    // --- رسانه‌ها ---
    ("📷", "5846024087033353251"),
    ("📸", "5846024087033353251"),
    ("🎥", "5882002216323125435"),
    ("🎬", "5775981206319402773"),
    ("🎞️", "5775981206319402773"),
    ("🎞", "5775981206319402773"),
    ("🎙️", "5897853622057700958"),
    ("🎙", "5897853622057700958"),
    ("📁", "5875206779196935950"),
    ("📂", "5875206779196935950"),
    ("📦", "5924720918826848520"),
    ("🎭", "5891093751555694829"),
    ("🏷️", "5854776233950188167"),
    ("🏷", "5854776233950188167"),
    ("🔗", "5877465816030515018"),
    ("⏱️", "5985616167740379273"),
    ("⏱", "5985616167740379273"),
    ("⏰", "5985616167740379273"),
    ("⏳", "5985616167740379273"),
    ("⌛", "5985616167740379273"),
    ("👁️", "5960714428394507968"),
    ("👁", "5960714428394507968"),
    ("🚀", "5843553939672274145"),
    ("⏩", "5938555239333367807"),
];

static CALLBACK_ICONS: &[(&str, &str)] = &[
    ("nav_locks", "5832546462478635761"),
    ("nav_main", "5875082500023258804"),
    ("close_settings", "5778527486270770928"),
    ("nav_words", "5839323457015256759"),
    ("nav_domains", "5879585266426973039"),
    ("nav_custom_cmds", "5843843420468024653"),
    ("nav_channel_track", "5874986954180791957"),
    ("nav_actions", "5843553939672274145"),
    ("nav_welcome", "5877260593903177342"),
    ("show_rules", "5839323457015256759"),
    ("toggle_welcome", "5967574255670399788"),
    ("toggle_goodbye", "5994750571041525522"),
    ("toggle_flood", "5926783847453692661"),
    ("toggle_night", "5886366951067883092"),
    ("toggle_captcha", "5819078828017849357"),
    ("nav_warn", "5881702736843511327"),
    ("nav_channel", "5771695636411847302"),
    ("toggle_silent", "5890838600433536921"),
    ("toggle_link", "5877465816030515018"),
    ("toggle_forward", "5938555239333367807"),
    ("toggle_photo", "5846024087033353251"),
    ("toggle_video", "5882002216323125435"),
    ("toggle_voice", "5897853622057700958"),
    ("toggle_gif", "5775981206319402773"),
    ("toggle_sticker", "5891093751555694829"),
    ("toggle_document", "5875206779196935950"),
    ("toggle_service_msg", "5909201569898827582"),
    ("toggle_service_msg_main", "5909201569898827582"),
    ("toggle_media", "5924720918826848520"),
    ("toggle_track_photo", "5846024087033353251"),
    ("toggle_track_video", "5882002216323125435"),
    ("toggle_track_voice", "5897853622057700958"),
    ("toggle_track_gif", "5775981206319402773"),
    ("toggle_track_sticker", "5891093751555694829"),
    ("toggle_track_document", "5875206779196935950"),
    ("toggle_track_text", "5886330010054168711"),
];

/// استخراج خودکار آیکون از متن یا کال‌بک دکمه و پیراستن متن دکمه جهت نمایش شیک و یکپارچه
pub fn infer_icon_and_clean_text(
    raw_text: &str,
    callback_data: Option<&str>,
    style: Option<&str>,
) -> (String, Option<String>) {
    let mut clean = raw_text.trim().to_string();
    let mut icon_id: Option<String> = None;

    // ۱. بررسی شروع متن با ایموجی شناخته شده
    for (em, cid) in EMOJI_MAP {
        if clean.starts_with(em) {
            icon_id = Some(cid.to_string());
            let trimmed = clean[em.len()..].trim().to_string();
            if !trimmed.is_empty() {
                clean = trimmed;
            }
            break;
        }
    }

    // ۲. بررسی انتهای متن با ایموجی شناخته شده (مانند "سقف اخطار (۳) ⚠️" یا "بستن منو ❌")
    if icon_id.is_none() {
        for (em, cid) in EMOJI_MAP {
            if clean.ends_with(em) {
                icon_id = Some(cid.to_string());
                let trimmed = clean[..clean.len() - em.len()].trim().to_string();
                if !trimmed.is_empty() {
                    clean = trimmed;
                }
                break;
            }
        }
    }

    // ۳. بررسی وجود ایموجی در هر کجای متن (مانند "من انسان هستم ✅ (تایید)")
    if icon_id.is_none() {
        for (em, cid) in EMOJI_MAP {
            if clean.contains(em) {
                icon_id = Some(cid.to_string());
                let trimmed = clean.replace(em, "").trim().to_string();
                if !trimmed.is_empty() {
                    clean = trimmed;
                }
                break;
            }
        }
    }

    // ۴. در صورت نیافتن ایموجی در متن، تحلیل شناسه کال‌بک و استایل
    if icon_id.is_none() {
        if let Some(cb) = callback_data {
            for (prefix, cid) in CALLBACK_ICONS {
                if cb == *prefix || cb.starts_with(prefix) {
                    icon_id = Some(cid.to_string());
                    break;
                }
            }

            if icon_id.is_none() {
                if cb.starts_with("confirm") || cb.starts_with("verify_captcha") {
                    icon_id = Some("5776375003280838798".to_string()); // check
                } else if cb.starts_with("reject") {
                    icon_id = Some("5778527486270770928".to_string()); // cross
                } else if cb.starts_with("clear_") {
                    icon_id = Some("5879896690210639947".to_string()); // trash
                } else if cb.starts_with("preview_") {
                    icon_id = Some("5960714428394507968".to_string()); // eye
                } else if cb.starts_with("check_") {
                    icon_id = Some("5874960879434338403".to_string()); // search
                } else if cb.starts_with("guide_") || cb.starts_with("help_") {
                    icon_id = Some("5877477876298681283".to_string()); // info
                } else if cb.starts_with("list_") {
                    icon_id = Some("5839323457015256759".to_string()); // list
                } else if cb == "toggle_channel_track" {
                    icon_id = if style == Some("success") {
                        Some("5776375003280838798".to_string())
                    } else {
                        Some("5778527486270770928".to_string())
                    };
                }
            }
        }
    }

    (clean, icon_id)
}

/// اعمال خودکار تگ‌های <tg-emoji> بر روی تمامی ایموجی‌های متن بدون تغییر تگ‌های HTML (الگوی رستم)
pub fn apply_premium_to_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + 256);
    let mut rest = text;
    let mut in_tag = false;
    let mut in_tg_emoji = false;

    while !rest.is_empty() {
        if rest.starts_with("</tg-emoji>") {
            result.push_str("</tg-emoji>");
            rest = &rest["</tg-emoji>".len()..];
            in_tg_emoji = false;
            continue;
        }

        if rest.starts_with("<tg-emoji") {
            if let Some(end_tag) = rest.find('>') {
                result.push_str(&rest[..=end_tag]);
                rest = &rest[end_tag + 1..];
                in_tg_emoji = true;
                continue;
            }
        }

        let Some(c) = rest.chars().next() else {
            break;
        };

        if in_tag {
            result.push(c);
            if c == '>' {
                in_tag = false;
            }
            rest = &rest[c.len_utf8()..];
            continue;
        }

        if c == '<' {
            in_tag = true;
            result.push(c);
            rest = &rest[c.len_utf8()..];
            continue;
        }

        if !in_tg_emoji {
            let mut matched = false;
            for (emoji_str, icon_id) in EMOJI_MAP {
                if rest.starts_with(emoji_str) {
                    result.push_str(&format!("<tg-emoji emoji-id=\"{icon_id}\">{emoji_str}</tg-emoji>"));
                    rest = &rest[emoji_str.len()..];
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
        }

        result.push(c);
        rest = &rest[c.len_utf8()..];
    }
    result
}

/// پاکسازی و تبدیل برچسب‌های ویژه (جدول‌ها، چک‌لیست‌ها و آکاردئون‌ها) به HTML استاندارد تلگرام
/// توجه: تگ‌های <tg-emoji> حفظ می‌شوند چون تلگرام بومی از آن‌ها در HTML پشتیبانی می‌کند.
pub fn sanitize_for_legacy_telegram(html: &str) -> String {
    html.replace("<details open>", "")
        .replace("<details>", "")
        .replace("</details>", "\n")
        .replace("<summary>", "<b>")
        .replace("</summary>", "</b>\n")
        .replace("<input type=\"checkbox\" checked>", "☑️ ")
        .replace("<input type=\"checkbox\">", "⬜ ")
        .replace("<mark>", "<b>")
        .replace("</mark>", "</b>")
        .replace("<table bordered striped compact>", "<blockquote>")
        .replace("<table bordered striped>", "<blockquote>")
        .replace("<table bordered compact>", "<blockquote>")
        .replace("<table compact>", "<blockquote>")
        .replace("<table>", "<blockquote>")
        .replace("</table>", "</blockquote>")
        .replace("<caption align=\"center\">", "<b>")
        .replace("<caption>", "<b>")
        .replace("</caption>", "</b>\n")
        .replace("<tr>", "")
        .replace("</tr>", "\n")
        .replace("<th align=\"center\">", " • <b>")
        .replace("<th>", " • <b>")
        .replace("</th>", "</b> ")
        .replace("<td align=\"center\">", " | ")
        .replace("<td>", " | ")
        .replace("</td>", "")
        .replace("<p>", "")
        .replace("</p>", "\n")
        .replace("<ul>", "<blockquote>")
        .replace("</ul>", "</blockquote>")
        .replace("<li>", "")
        .replace("</li>", "\n")
}

/// پاکسازی اضطراری تگ‌های <tg-emoji> تنها در صورت وقوع خطای پارس در تلگرام
pub fn strip_all_custom_emojis(html: &str) -> String {
    let mut s = html.to_string();
    while let Some(start) = s.find("<tg-emoji") {
        if let Some(end) = s[start..].find('>') {
            s.replace_range(start..start + end + 1, "");
        } else {
            break;
        }
    }
    s.replace("</tg-emoji>", "")
}

/// ارسال پیام با قالب‌بندی Rich Message بومی تلگرام (Bot API 10.1 / 10.2) و دکمه‌های رنگی
pub async fn send_styled_message(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
    keyboard: Option<&StyledKeyboard>,
) -> Result<(), anyhow::Error> {
    send_styled_message_with_topic(bot, chat_id, None, text, keyboard).await
}

/// ارسال پیام Rich Message به همراه پشتیبانی از ارسال به تاپیک اختصاصی در فروم‌ها (Topic Router)
pub async fn send_styled_message_with_topic(
    bot: &Bot,
    chat_id: ChatId,
    topic_id: Option<i64>,
    text: &str,
    keyboard: Option<&StyledKeyboard>,
) -> Result<(), anyhow::Error> {
    send_styled_message_full(bot, chat_id, topic_id, None, text, keyboard, None).await
}

/// ارسال پیام Rich Message موقت و محرمانه (Ephemeral Message - Bot API 10.2)
pub async fn send_styled_ephemeral_message(
    bot: &Bot,
    chat_id: ChatId,
    receiver_user_id: u64,
    text: &str,
    keyboard: Option<&StyledKeyboard>,
) -> Result<(), anyhow::Error> {
    send_styled_message_full(bot, chat_id, None, None, text, keyboard, Some(receiver_user_id)).await
}

/// ارسال کامل پیام با اولویت متد نوین sendRichMessage و فال‌بک شفاف با پشتیبانی کامل از تگ‌های <tg-emoji>
pub async fn send_styled_message_full(
    bot: &Bot,
    chat_id: ChatId,
    topic_id: Option<i64>,
    reply_to_msg_id: Option<MessageId>,
    text: &str,
    keyboard: Option<&StyledKeyboard>,
    ephemeral_user_id: Option<u64>,
) -> Result<(), anyhow::Error> {
    let client = bot.client();
    let token = bot.token();

    // ۱. اعمال ایموجی‌های پریمیوم متحرک بر روی متن خام اولیه
    let premium_rich_text = apply_premium_to_html(text);

    let rich_url = format!("https://api.telegram.org/bot{}/sendRichMessage", token);
    let mut rich_payload = serde_json::json!({
        "chat_id": chat_id.0,
        "rich_message": {
            "html": premium_rich_text,
            "is_rtl": true,
        }
    });

    if let Some(t_id) = topic_id {
        if t_id > 0 {
            rich_payload["message_thread_id"] = serde_json::json!(t_id);
        }
    }
    if let Some(rep_id) = reply_to_msg_id {
        rich_payload["reply_parameters"] = serde_json::json!({
            "message_id": rep_id.0
        });
    }
    if let Some(e_uid) = ephemeral_user_id {
        rich_payload["ephemeral_message_parameters"] = serde_json::json!({
            "receiver_user_id": e_uid
        });
    }
    if let Some(kb) = keyboard {
        rich_payload["reply_markup"] = serde_json::json!(kb);
    }

    let resp = client.post(&rich_url).json(&rich_payload).send().await;
    if let Ok(res) = resp {
        if res.status().is_success() {
            return Ok(());
        }
        let err_text = res.text().await.unwrap_or_default();
        tracing::debug!("متد sendRichMessage موفق نبود ({})، فال‌بک به sendMessage...", err_text);
    }

    // ۲. فال‌بک: تبدیل تگ‌های اختصاصی جدول/آکاردئون به HTML معتبر تلگرام به همراه حفظ کامل <tg-emoji>
    let legacy_text = sanitize_for_legacy_telegram(text);
    let final_html = apply_premium_to_html(&legacy_text);

    let send_url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let mut legacy_payload = serde_json::json!({
        "chat_id": chat_id.0,
        "text": final_html,
        "parse_mode": "HTML",
    });

    if let Some(t_id) = topic_id {
        if t_id > 0 {
            legacy_payload["message_thread_id"] = serde_json::json!(t_id);
        }
    }
    if let Some(rep_id) = reply_to_msg_id {
        legacy_payload["reply_parameters"] = serde_json::json!({
            "message_id": rep_id.0
        });
    }
    if let Some(kb) = keyboard {
        legacy_payload["reply_markup"] = serde_json::json!(kb);
    }

    let legacy_resp = client.post(&send_url).json(&legacy_payload).send().await?;
    if !legacy_resp.status().is_success() {
        let err_text = legacy_resp.text().await.unwrap_or_default();
        tracing::warn!("ارسال با sendMessage ناموفق بود ({})، تلاش با حذف اضطراری تگ‌های ایموجی...", err_text);

        let safe_text = strip_all_custom_emojis(&final_html);
        let mut rescue_payload = serde_json::json!({
            "chat_id": chat_id.0,
            "text": safe_text,
            "parse_mode": "HTML",
        });
        if let Some(t_id) = topic_id {
            if t_id > 0 {
                rescue_payload["message_thread_id"] = serde_json::json!(t_id);
            }
        }
        if let Some(rep_id) = reply_to_msg_id {
            rescue_payload["reply_parameters"] = serde_json::json!({
                "message_id": rep_id.0
            });
        }
        if let Some(kb) = keyboard {
            rescue_payload["reply_markup"] = serde_json::json!(kb);
        }

        let rescue_resp = client.post(&send_url).json(&rescue_payload).send().await?;
        if !rescue_resp.status().is_success() {
            let res_err = rescue_resp.text().await.unwrap_or_default();
            tracing::error!("فال‌بک اضطراری به کلاینت پیش‌فرض Teloxide: {}", res_err);
            let mut req = bot.send_message(chat_id, safe_text).parse_mode(teloxide::types::ParseMode::Html);
            if let Some(t_id) = topic_id {
                if t_id > 0 {
                    req = req.message_thread_id(teloxide::types::ThreadId(teloxide::types::MessageId(t_id as i32)));
                }
            }
            if let Some(rep_id) = reply_to_msg_id {
                req = req.reply_parameters(teloxide::types::ReplyParameters::new(rep_id));
            }
            if let Some(kb) = keyboard {
                req = req.reply_markup(kb.to_teloxide());
            }
            let _ = req.await;
        }
    }
    Ok(())
}

/// ویرایش کیبورد شیشه‌ای با استایل رنگی و آیکون‌های متحرک بومی تلگرام
pub async fn edit_styled_keyboard(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    keyboard: &StyledKeyboard,
) -> Result<(), anyhow::Error> {
    let client = bot.client();
    let token = bot.token();
    let url = format!("https://api.telegram.org/bot{}/editMessageReplyMarkup", token);

    let payload = serde_json::json!({
        "chat_id": chat_id.0,
        "message_id": message_id.0,
        "reply_markup": keyboard,
    });

    let resp = client.post(&url).json(&payload).send().await?;
    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        tracing::warn!("ویرایش کیبورد رنگی ناموفق بود، فال‌بک به کلاینت پیش‌فرض: {}", err_text);
        let _ = bot.edit_message_reply_markup(chat_id, message_id)
            .reply_markup(keyboard.to_teloxide())
            .await;
    }
    Ok(())
}

/// ویرایش همزمان متن و کیبورد با استایل Rich Message، ایموجی‌های پریمیوم و فال‌بک ایمن
pub async fn edit_styled_message_text(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    text: &str,
    keyboard: Option<&StyledKeyboard>,
) -> Result<(), anyhow::Error> {
    let client = bot.client();
    let token = bot.token();
    let url = format!("https://api.telegram.org/bot{}/editMessageText", token);

    let legacy_text = sanitize_for_legacy_telegram(text);
    let final_html = apply_premium_to_html(&legacy_text);

    let mut payload = serde_json::json!({
        "chat_id": chat_id.0,
        "message_id": message_id.0,
        "text": final_html,
        "parse_mode": "HTML",
    });

    if let Some(kb) = keyboard {
        payload["reply_markup"] = serde_json::json!(kb);
    }

    let resp = client.post(&url).json(&payload).send().await?;
    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        tracing::warn!("ویرایش متن پیام ناموفق بود ({})، تلاش با حذف اضطراری تگ‌های ایموجی...", err_text);

        let safe_text = strip_all_custom_emojis(&final_html);
        let mut rescue_payload = serde_json::json!({
            "chat_id": chat_id.0,
            "message_id": message_id.0,
            "text": safe_text,
            "parse_mode": "HTML",
        });
        if let Some(kb) = keyboard {
            rescue_payload["reply_markup"] = serde_json::json!(kb);
        }

        let rescue_resp = client.post(&url).json(&rescue_payload).send().await?;
        if !rescue_resp.status().is_success() {
            let res_err = rescue_resp.text().await.unwrap_or_default();
            tracing::error!("فال‌بک نهایی ویرایش به کلاینت Teloxide: {}", res_err);
            let mut req = bot.edit_message_text(chat_id, message_id, safe_text).parse_mode(teloxide::types::ParseMode::Html);
            if let Some(kb) = keyboard {
                req = req.reply_markup(kb.to_teloxide());
            }
            let _ = req.await;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_preserves_custom_emoji_and_converts_rich() {
        let rich = "<details open><summary>عنوان</summary><table compact><tr><th>ردیف</th><th>نام</th></tr><tr><td>1</td><td>علی</td></tr></table></details><mark>هشدار</mark><ul><li><input type=\"checkbox\" checked>انجام شد</li><li><input type=\"checkbox\">در انتظار</li></ul>";
        let legacy = sanitize_for_legacy_telegram(rich);
        assert!(!legacy.contains("<details"));
        assert!(!legacy.contains("<table"));
        assert!(!legacy.contains("<mark>"));
        assert!(!legacy.contains("<input"));
        assert!(!legacy.contains("<ul"));
        assert!(!legacy.contains("<li"));
        assert!(legacy.contains("<blockquote>"));
        assert!(legacy.contains("☑️ انجام شد"));
        assert!(legacy.contains("⬜ در انتظار"));
        assert!(legacy.contains("<b>هشدار</b>"));
    }

    #[test]
    fn test_infer_icon_and_clean_text() {
        let (txt1, icon1) = infer_icon_and_clean_text("🔒 مدیریت قفل‌ها", Some("nav_locks"), Some("primary"));
        assert_eq!(txt1, "مدیریت قفل‌ها");
        assert_eq!(icon1.as_deref(), Some("5832546462478635761"));

        let (txt2, icon2) = infer_icon_and_clean_text("🌸 پیام خوشامد", Some("toggle_welcome"), Some("success"));
        assert_eq!(txt2, "پیام خوشامد");
        assert_eq!(icon2.as_deref(), Some("5967574255670399788"));

        let (txt3, icon3) = infer_icon_and_clean_text("بستن منو ❌", Some("close_settings"), Some("danger"));
        assert_eq!(txt3, "بستن منو");
        assert_eq!(icon3.as_deref(), Some("5778527486270770928"));

        let (txt4, icon4) = infer_icon_and_clean_text("روشن", Some("toggle_channel_track"), Some("success"));
        assert_eq!(txt4, "روشن");
        assert_eq!(icon4.as_deref(), Some("5776375003280838798"));

        let (txt5, icon5) = infer_icon_and_clean_text("🔔 اعلان‌های تلگرام", Some("toggle_service_msg_main"), Some("success"));
        assert_eq!(txt5, "اعلان‌های تلگرام");
        assert_eq!(icon5.as_deref(), Some("5909201569898827582"));
    }

    #[test]
    fn test_apply_premium_to_html_full() {
        let input = "وضعیت: فعال 🟢 | غیرفعال 🔴 | بدون تنظیم ⚪ | سپر: 🛡️";
        let out = apply_premium_to_html(input);
        assert!(out.contains("<tg-emoji emoji-id=\"5776375003280838798\">🟢</tg-emoji>"));
        assert!(out.contains("<tg-emoji emoji-id=\"5967333011652350314\">🔴</tg-emoji>"));
        assert!(out.contains("<tg-emoji emoji-id=\"5994324703559290598\">⚪</tg-emoji>"));
        assert!(out.contains("<tg-emoji emoji-id=\"5926783847453692661\">🛡️</tg-emoji>"));
    }
}
