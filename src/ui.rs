//! نظام الأيقونات والتصميمات وتطبيقات الهواتف الذكية والويب للغة لسان.
use std::path::{Path, PathBuf};

/// قائمة جميع الأيقونات المتجهة المتوفرة في لغة لسان
pub const ICON_NAMES: &[&str] = &[
    "رئيسية", "بحث", "إعدادات", "مستخدم", "مجموعة", "قلب", "نجمة", "سلة",
    "إشعار", "رسالة", "بريد", "كاميرا", "صورة", "هاتف", "موقع", "قفل",
    "مفتاح", "درع", "تحميل", "رفع", "تشغيل", "إيقاف", "تحديث", "مشاركة",
    "إضافة", "حذف", "تعديل", "صح", "خطأ", "تحذير", "معلومات", "ملف",
    "مجلد", "سحابة", "واي_فاي", "بلوتوث", "بطارية", "أردوينو", "شريحة",
    "روبوت", "حرارة", "شمس", "قمر", "كود", "قاعدة_بيانات", "قائمة",
    // الأيقونات الموسّعة للويب والتطبيقات الذكية 2.0
    "مخطط", "جدول", "متجر", "قلب_نبض", "خريطة", "كاميرا_فيديو", "صوت",
    "ميكروفون", "بطاقة_دفع", "طباعة", "تقويم", "ساعة", "مصباح", "جائزة",
    "صاروخ", "مغناطيس", "رابط", "قفل_مفتوح", "مستند", "بريد_وارد",
    "محفظة", "إرسال",
];

/// تهريب نص HTML بأمان لمنع حقن الشيفرات داخل الوسوم والسمات
pub fn escape_html_ui(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// تهريب نص للاستخدام الآمن داخل نص JavaScript محاط بعلامات اقتباس مفردة ومضمّن في سمة HTML
fn escape_js_attr(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(ch),
        }
    }
    out
}

/// تهريب نص للاستخدام داخل سلاسل JSON
#[cfg(not(target_arch = "wasm32"))]
fn escape_json_str(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// إرجاع كود SVG المتجه لأي أيقونة عربية مع الحجم واللون المطلوبين
pub fn get_icon_svg(name: &str, size: u32, color: &str) -> String {
    let norm = crate::diagnostics::normalize_arabic(name.trim());
    let safe_color = escape_html_ui(color);
    let path_data = match norm.as_str() {
        "رئيسيه" | "بيت" | "home" => {
            r#"<path d="M3 10.5L12 3l9 7.5V20a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1v-9.5z"/>"#
        }
        "بحث" | "search" => {
            r#"<circle cx="11" cy="11" r="7"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>"#
        }
        "اعدادات" | "ضبط" | "settings" => {
            r#"<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9c.26.6.77 1.02 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>"#
        }
        "مستخدم" | "حساب" | "user" => {
            r#"<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>"#
        }
        "مجموعه" | "اعضاء" | "users" => {
            r#"<path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>"#
        }
        "قلب" | "اعجاب" | "heart" => {
            r#"<path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>"#
        }
        "نجمه" | "تقييم" | "star" => {
            r#"<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>"#
        }
        "سله" | "cart" => {
            r#"<circle cx="9" cy="21" r="1"/><circle cx="20" cy="21" r="1"/><path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"/>"#
        }
        "اشعار" | "جرس" | "bell" => {
            r#"<path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/>"#
        }
        "رساله" | "محادثه" | "chat" => {
            r#"<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>"#
        }
        "بريد" | "ايميل" | "mail" => {
            r#"<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/>"#
        }
        "كاميرا" | "camera" => {
            r#"<path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="4"/>"#
        }
        "صوره" | "image" => {
            r#"<rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/>"#
        }
        "هاتف" | "جوال" | "موبايل" | "phone" => {
            r#"<rect x="5" y="2" width="14" height="20" rx="2" ry="2"/><line x1="12" y1="18" x2="12.01" y2="18"/>"#
        }
        "موقع" | "location" => {
            r#"<path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/><circle cx="12" cy="10" r="3"/>"#
        }
        "قفل" | "امان" | "lock" => {
            r#"<rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>"#
        }
        "مفتاح" | "key" => {
            r#"<path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/>"#
        }
        "درع" | "حمايه" | "shield" => {
            r#"<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>"#
        }
        "تحميل" | "تنزيل" | "download" => {
            r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>"#
        }
        "رفع" | "upload" => {
            r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/>"#
        }
        "تشغيل" | "ابدا" | "play" => {
            r#"<polygon points="5 3 19 12 5 21 5 3"/>"#
        }
        "ايقاف" | "stop" => {
            r#"<rect x="6" y="6" width="12" height="12" rx="2"/>"#
        }
        "تحديث" | "refresh" => {
            r#"<polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>"#
        }
        "مشاركه" | "share" => {
            r#"<circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>"#
        }
        "اضافه" | "زائد" | "plus" => {
            r#"<line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>"#
        }
        "حذف" | "مسح" | "trash" => {
            r#"<polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>"#
        }
        "تعديل" | "قلم" | "edit" => {
            r#"<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>"#
        }
        "صح" | "نجاح" | "check" => {
            r#"<polyline points="20 6 9 17 4 12"/>"#
        }
        "خطا" | "اغلاق" | "close" => {
            r#"<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>"#
        }
        "تحذير" | "تنبيه" | "warning" => {
            r#"<path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>"#
        }
        "معلومات" | "info" => {
            r#"<circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/>"#
        }
        "ملف" | "file" => {
            r#"<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/>"#
        }
        "مجلد" | "folder" => {
            r#"<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>"#
        }
        "سحابه" | "cloud" => {
            r#"<path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"/>"#
        }
        "واي_فاي" | "شبكه" | "wifi" => {
            r#"<path d="M5 12.55a11 11 0 0 1 14.08 0"/><path d="M1.42 9a16 16 0 0 1 21.16 0"/><path d="M8.53 16.11a6 6 0 0 1 6.95 0"/><line x1="12" y1="20" x2="12.01" y2="20"/>"#
        }
        "بلوتوث" | "bluetooth" => {
            r#"<polyline points="6.5 6.5 17.5 17.5 12 23 12 1 17.5 6.5 6.5 17.5"/>"#
        }
        "بطاريه" | "طاقه" | "battery" => {
            r#"<rect x="1" y="6" width="18" height="12" rx="2" ry="2"/><line x1="23" y1="13" x2="23" y2="11"/>"#
        }
        "اردوينو" | "شريحه" | "متحكم" | "معالج" | "cpu" => {
            r#"<rect x="4" y="4" width="16" height="16" rx="2" ry="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/>"#
        }
        "روبوت" | "ذكي" | "bot" => {
            r#"<rect x="3" y="11" width="18" height="10" rx="2"/><circle cx="12" cy="5" r="2"/><path d="M12 7v4"/><line x1="8" y1="16" x2="8" y2="16"/><line x1="16" y1="16" x2="16" y2="16"/>"#
        }
        "حراره" | "مقياس" | "thermometer" => {
            r#"<path d="M14 14.76V3.5a2.5 2.5 0 0 0-5 0v11.26a4.5 4.5 0 1 0 5 0z"/>"#
        }
        "شمس" | "اضاءه" | "sun" => {
            r#"<circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/>"#
        }
        "قمر" | "ليلي" | "moon" => {
            r#"<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>"#
        }
        "كود" | "برمجه" | "code" => {
            r#"<polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/>"#
        }
        "قاعده_بيانات" | "بيانات" | "database" => {
            r#"<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>"#
        }
        "قائمه" | "menu" => {
            r#"<line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/>"#
        }
        // الأيقونات الجديدة 2.0
        "مخطط" | "احصاء" | "chart" => {
            r#"<line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/><line x1="2" y1="20" x2="22" y2="20"/>"#
        }
        "جدول" | "table" => {
            r#"<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="3" y1="15" x2="21" y2="15"/><line x1="9" y1="3" x2="9" y2="21"/>"#
        }
        "متجر" | "دكان" | "store" | "shop" => {
            r#"<path d="M3 9l1-5h16l1 5"/><path d="M3 9a3 3 0 0 0 6 0 3 3 0 0 0 6 0 3 3 0 0 0 6 0"/><path d="M5 12v8h14v-8"/><path d="M10 20v-5h4v5"/>"#
        }
        "قلب_نبض" | "نبض" | "صحه" | "activity" | "pulse" => {
            r#"<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>"#
        }
        "خريطه" | "map" => {
            r#"<polygon points="1 6 1 22 8 18 16 22 23 18 23 2 16 6 8 2 1 6"/><line x1="8" y1="2" x2="8" y2="18"/><line x1="16" y1="6" x2="16" y2="22"/>"#
        }
        "كاميرا_فيديو" | "فيديو" | "video" => {
            r#"<polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>"#
        }
        "صوت" | "سماعه" | "volume" | "audio" => {
            r#"<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/>"#
        }
        "ميكروفون" | "مايك" | "mic" | "microphone" => {
            r#"<path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><line x1="12" y1="19" x2="12" y2="23"/><line x1="8" y1="23" x2="16" y2="23"/>"#
        }
        "بطاقه_دفع" | "دفع" | "بطاقه" | "credit_card" | "card" => {
            r#"<rect x="1" y="4" width="22" height="16" rx="2" ry="2"/><line x1="1" y1="10" x2="23" y2="10"/><line x1="6" y1="15" x2="10" y2="15"/>"#
        }
        "طباعه" | "طابعه" | "printer" | "print" => {
            r#"<polyline points="6 9 6 2 18 2 18 9"/><path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/><rect x="6" y="14" width="12" height="8"/>"#
        }
        "تقويم" | "تاريخ" | "calendar" => {
            r#"<rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>"#
        }
        "ساعه" | "وقت" | "clock" | "time" => {
            r#"<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>"#
        }
        "مصباح" | "فكره" | "bulb" | "lamp" => {
            r#"<path d="M9 18h6"/><path d="M10 22h4"/><path d="M12 2a7 7 0 0 0-4 12.74V17h8v-2.26A7 7 0 0 0 12 2z"/>"#
        }
        "جائزه" | "وسام" | "award" | "trophy" => {
            r#"<circle cx="12" cy="8" r="6"/><path d="M15.477 12.89L17 22l-5-3-5 3 1.523-9.11"/>"#
        }
        "صاروخ" | "اطلاق" | "rocket" => {
            r#"<path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/>"#
        }
        "مغناطيس" | "magnet" => {
            r#"<path d="M6 15V4h4v11a2 2 0 0 0 4 0V4h4v11a6 6 0 0 1-12 0z"/><line x1="6" y1="8" x2="10" y2="8"/><line x1="14" y1="8" x2="18" y2="8"/>"#
        }
        "رابط" | "وصله" | "link" => {
            r#"<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>"#
        }
        "قفل_مفتوح" | "مفتوح" | "unlock" => {
            r#"<rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 9.9-1"/>"#
        }
        "مستند" | "وثيقه" | "document" | "doc" => {
            r#"<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><line x1="10" y1="9" x2="8" y2="9"/>"#
        }
        "بريد_وارد" | "صندوق" | "inbox" => {
            r#"<polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/><path d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/>"#
        }
        "محفظه" | "مال" | "wallet" => {
            r#"<path d="M21 12V7H5a2 2 0 0 1 0-4h14v4"/><path d="M3 5v14a2 2 0 0 0 2 2h16v-5"/><path d="M18 12a2 2 0 0 0 0 4h4v-4h-4z"/>"#
        }
        "ارسال" | "send" => {
            r#"<line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/>"#
        }
        _ => {
            r#"<circle cx="12" cy="12" r="9"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>"#
        }
    };

    format!(
        r#"<svg class="lsn-icon" xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="{safe_color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{path_data}</svg>"#
    )
}

/// إرجاع متغيرات CSS للنسق اللوني المطلوب (داكن، فاتح، زمردي، ملكي، غروب، محيطي، لسان)
pub fn get_theme_css(theme: &str) -> &'static str {
    let norm = crate::diagnostics::normalize_arabic(theme.trim());
    match norm.as_str() {
        "فاتح" | "light" => {
            ":root { --bg: #f8fafc; --surface: #ffffff; --surface-2: #f1f5f9; --text: #0f172a; --muted: #64748b; --primary: #2563eb; --accent: #0ea5e9; --border: #e2e8f0; }"
        }
        "زمردي" | "اخضر" | "emerald" => {
            ":root { --bg: #041f1e; --surface: #083331; --surface-2: #0d4a47; --text: #ecfdf5; --muted: #6ee7b7; --primary: #10b981; --accent: #34d399; --border: #115e59; }"
        }
        "ملكي" | "بنفسجي" | "royal" => {
            ":root { --bg: #130b29; --surface: #1f1244; --surface-2: #2d1b63; --text: #f5f3ff; --muted: #c4b5fd; --primary: #8b5cf6; --accent: #f59e0b; --border: #4c1d95; }"
        }
        "غروب" | "برتقالي" | "sunset" => {
            ":root { --bg: #1c1018; --surface: #2d1925; --surface-2: #422336; --text: #fff1f2; --muted: #fda4af; --primary: #f43f5e; --accent: #fb923c; --border: #881337; }"
        }
        "محيطي" | "ازرق" | "ocean" => {
            ":root { --bg: #071926; --surface: #0c2b40; --surface-2: #123e5c; --text: #f0f9ff; --muted: #7dd3fc; --primary: #06b6d4; --accent: #38bdf8; --border: #155e75; }"
        }
        "لسان" | "هويه" | "lisan" => {
            ":root { --bg: #091d37; --surface: #102a4c; --surface-2: #173761; --text: #f8fafc; --muted: #b8c9e0; --primary: #e65c1a; --accent: #f59e0b; --border: #1f4474; }"
        }
        _ => {
            ":root { --bg: #0f172a; --surface: #1e293b; --surface-2: #334155; --text: #f8fafc; --muted: #94a3b8; --primary: #3b82f6; --accent: #10b981; --border: #334155; }"
        }
    }
}

/// بناء صفحة HTML5 متكاملة لتطبيق الهاتف الذكي (مع إطار هاتف تفاعلي وشاشات متعددة وزر عائم ونوافذ سفلية ودعم PWA)
pub fn build_mobile_app_html(
    app_name: &str,
    theme: &str,
    header_icon: &str,
    items_html: &[String],
    tabs: &[(String, String)],
) -> String {
    let theme_vars = get_theme_css(theme);
    let safe_app_name = escape_html_ui(app_name);
    let top_icon = get_icon_svg(header_icon, 22, "currentColor");
    let theme_icon = get_icon_svg("قمر", 16, "currentColor");

    // استخراج العلامات الخاصة للشاشات/التبويبات والتراكبات (الزر العائم والنافذة السفلية)
    let mut parsed_tabs: Vec<(String, String, String)> = Vec::new();
    let mut content_items: Vec<&str> = Vec::new();
    let mut overlay_items: Vec<&str> = Vec::new();

    for item in items_html {
        let trimmed = item.trim();
        if let Some(rest) = trimmed.strip_prefix("<!--LSN_TAB:") {
            if let Some(inner) = rest.strip_suffix("-->") {
                let mut parts = inner.splitn(3, '|');
                let screen_id = parts.next().unwrap_or("").trim().to_string();
                let title = parts.next().unwrap_or("").trim().to_string();
                let icon = parts.next().unwrap_or("رئيسية").trim().to_string();
                if !title.is_empty() {
                    parsed_tabs.push((screen_id, title, icon));
                }
                continue;
            }
        }
        if let Some(rest) = trimmed.strip_prefix("<!--LSN_FAB-->") {
            overlay_items.push(rest);
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("<!--LSN_SHEET-->") {
            overlay_items.push(rest);
            continue;
        }
        content_items.push(item.as_str());
    }

    let body_content = content_items.join("\n");
    let overlays_content = overlay_items.join("\n");

    let mut tabs_html = String::new();
    if !tabs.is_empty() {
        for (idx, (label, ic)) in tabs.iter().enumerate() {
            let active = if idx == 0 { "active" } else { "" };
            let svg = get_icon_svg(ic, 20, "currentColor");
            let safe_label = escape_html_ui(label);
            let js_label = escape_js_attr(label);
            tabs_html.push_str(&format!(
                r#"<button class="lsn-tab {active}" type="button" onclick="selectTab(this, '{js_label}')">{svg}<span>{safe_label}</span></button>"#
            ));
        }
    } else if !parsed_tabs.is_empty() {
        for (idx, (screen_id, label, ic)) in parsed_tabs.iter().enumerate() {
            let active = if idx == 0 { "active" } else { "" };
            let svg = get_icon_svg(ic, 20, "currentColor");
            let safe_label = escape_html_ui(label);
            let js_label = escape_js_attr(label);
            if screen_id.is_empty() {
                tabs_html.push_str(&format!(
                    r#"<button class="lsn-tab {active}" type="button" onclick="selectTab(this, '{js_label}')">{svg}<span>{safe_label}</span></button>"#
                ));
            } else {
                let safe_id = escape_html_ui(screen_id);
                let js_id = escape_js_attr(screen_id);
                tabs_html.push_str(&format!(
                    r#"<button class="lsn-tab {active}" type="button" data-target="{safe_id}" onclick="selectScreen(this, '{js_id}', '{js_label}')">{svg}<span>{safe_label}</span></button>"#
                ));
            }
        }
    } else {
        for (idx, (label, ic)) in [
            ("الرئيسية", "رئيسية"),
            ("الأجهزة", "أردوينو"),
            ("الإشعارات", "إشعار"),
            ("الإعدادات", "إعدادات"),
        ]
        .iter()
        .enumerate()
        {
            let active = if idx == 0 { "active" } else { "" };
            let svg = get_icon_svg(ic, 20, "currentColor");
            tabs_html.push_str(&format!(
                r#"<button class="lsn-tab {active}" type="button" onclick="selectTab(this, '{label}')">{svg}<span>{label}</span></button>"#
            ));
        }
    }

    format!(
        r##"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
<meta name="theme-color" content="#0f172a">
<link rel="manifest" href="manifest.json">
<title>{safe_app_name}</title>
<style>
{theme_vars}
* {{ box-sizing: border-box; margin: 0; padding: 0; font-family: 'Segoe UI', Tahoma, system-ui, sans-serif; }}
body {{ background: var(--bg); color: var(--text); display: flex; justify-content: center; align-items: center; min-height: 100vh; padding: 16px; transition: background 0.25s, color 0.25s; }}
.phone-frame {{ width: 100%; max-width: 390px; height: 780px; background: var(--bg); border: 8px solid var(--surface-2); border-radius: 42px; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 25px 60px rgba(0,0,0,0.45); position: relative; }}
.status-bar {{ display: flex; justify-content: space-between; align-items: center; padding: 10px 22px 6px; font-size: 12px; color: var(--muted); background: var(--surface); }}
.app-header {{ display: flex; align-items: center; justify-content: space-between; padding: 12px 18px; background: var(--surface); border-bottom: 1px solid var(--border); gap: 8px; }}
.app-header h1 {{ font-size: 17px; font-weight: 700; display: flex; align-items: center; gap: 10px; color: var(--text); }}
.mobile-theme-btn {{ background: var(--surface-2); border: 1px solid var(--border); color: var(--text); width: 34px; height: 34px; border-radius: 10px; display: inline-flex; align-items: center; justify-content: center; cursor: pointer; flex-shrink: 0; transition: border-color 0.2s, transform 0.15s; }}
.mobile-theme-btn:hover {{ border-color: var(--primary); color: var(--primary); }}
.mobile-theme-btn:active {{ transform: scale(0.94); }}
.app-content {{ flex: 1; overflow-y: auto; padding: 16px; display: flex; flex-direction: column; gap: 12px; padding-bottom: 84px; }}
.lsn-screen {{ display: none; flex-direction: column; gap: 12px; }}
.lsn-screen.active {{ display: flex; animation: lsnFadeIn 0.2s ease; }}
.lsn-screen-title {{ font-size: 14px; font-weight: 800; color: var(--muted); display: flex; align-items: center; gap: 6px; padding-bottom: 4px; border-bottom: 1px dashed var(--border); }}
@keyframes lsnFadeIn {{ from {{ opacity: 0; transform: translateY(6px); }} to {{ opacity: 1; transform: translateY(0); }} }}
.lsn-card {{ background: var(--surface); border: 1px solid var(--border); border-radius: 16px; padding: 14px 16px; display: flex; align-items: center; gap: 14px; transition: transform 0.15s; }}
.lsn-card:hover {{ transform: translateY(-2px); }}
.lsn-card-icon {{ width: 44px; height: 44px; border-radius: 12px; background: var(--surface-2); color: var(--primary); display: flex; align-items: center; justify-content: center; flex-shrink: 0; }}
.lsn-card-body h3 {{ font-size: 15px; margin-bottom: 4px; color: var(--text); }}
.lsn-card-body p {{ font-size: 13px; color: var(--muted); line-height: 1.4; }}
.lsn-btn {{ width: 100%; padding: 13px 18px; border: none; border-radius: 14px; background: linear-gradient(135deg, var(--primary), var(--accent)); color: #fff; font-size: 15px; font-weight: 700; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 8px; box-shadow: 0 6px 18px rgba(0,0,0,0.2); }}
.lsn-btn:active {{ transform: scale(0.98); }}
.lsn-stats-grid {{ display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }}
.lsn-stat {{ background: var(--surface); border: 1px solid var(--border); border-radius: 16px; padding: 14px; display: flex; flex-direction: column; gap: 6px; }}
.lsn-stat-top {{ display: flex; justify-content: space-between; align-items: center; color: var(--muted); font-size: 12px; }}
.lsn-stat-val {{ font-size: 22px; font-weight: 800; color: var(--primary); }}
.lsn-switch-row {{ background: var(--surface); border: 1px solid var(--border); border-radius: 16px; padding: 14px 16px; display: flex; justify-content: space-between; align-items: center; }}
.lsn-switch-label {{ display: flex; align-items: center; gap: 10px; font-size: 14px; font-weight: 600; }}
.lsn-toggle {{ width: 48px; height: 26px; border-radius: 13px; background: var(--surface-2); position: relative; cursor: pointer; border: none; transition: 0.2s; }}
.lsn-toggle.on {{ background: var(--primary); }}
.lsn-toggle::after {{ content: ''; width: 20px; height: 20px; border-radius: 50%; background: #fff; position: absolute; top: 3px; right: 3px; transition: 0.2s; }}
.lsn-toggle.on::after {{ right: 25px; }}
.lsn-mobile-form {{ background: var(--surface); border: 1px solid var(--border); border-radius: 16px; padding: 16px; display: flex; flex-direction: column; gap: 12px; }}
.lsn-mobile-form h3 {{ font-size: 15px; font-weight: 700; color: var(--text); }}
.lsn-field {{ display: flex; flex-direction: column; gap: 5px; }}
.lsn-field label {{ font-size: 12px; font-weight: 600; color: var(--muted); }}
.lsn-field input, .lsn-field textarea {{ width: 100%; padding: 10px 12px; border-radius: 10px; border: 1px solid var(--border); background: var(--surface-2); color: var(--text); font-size: 13px; outline: none; }}
.lsn-field input:focus, .lsn-field textarea:focus {{ border-color: var(--primary); }}
.lsn-fab {{ position: absolute; left: 18px; bottom: 82px; width: 52px; height: 52px; border-radius: 50%; border: none; background: linear-gradient(135deg, var(--primary), var(--accent)); color: #fff; display: flex; align-items: center; justify-content: center; cursor: pointer; box-shadow: 0 10px 24px rgba(0,0,0,0.35); z-index: 18; transition: transform 0.18s; }}
.lsn-fab:hover {{ transform: scale(1.06); }}
.lsn-fab:active {{ transform: scale(0.95); }}
.lsn-sheet-backdrop {{ position: absolute; inset: 0; background: rgba(0,0,0,0.52); display: none; align-items: flex-end; justify-content: center; z-index: 30; backdrop-filter: blur(2px); }}
.lsn-sheet-backdrop.open {{ display: flex; }}
.lsn-sheet {{ width: 100%; max-height: 75%; background: var(--surface); border-top: 1px solid var(--border); border-radius: 24px 24px 0 0; padding: 14px 18px 22px; display: flex; flex-direction: column; gap: 12px; overflow-y: auto; animation: lsnSheetUp 0.22s ease; }}
@keyframes lsnSheetUp {{ from {{ transform: translateY(100%); }} to {{ transform: translateY(0); }} }}
.lsn-sheet-handle {{ width: 42px; height: 4px; border-radius: 999px; background: var(--border); margin: 0 auto 4px; }}
.lsn-sheet-head {{ display: flex; justify-content: space-between; align-items: center; }}
.lsn-sheet-head h3 {{ font-size: 16px; font-weight: 800; color: var(--text); }}
.lsn-sheet-close {{ background: var(--surface-2); border: none; color: var(--text); width: 30px; height: 30px; border-radius: 50%; cursor: pointer; display: inline-flex; align-items: center; justify-content: center; font-size: 15px; }}
.lsn-sheet-body {{ display: flex; flex-direction: column; gap: 10px; font-size: 13px; color: var(--muted); line-height: 1.6; }}
.lsn-toast {{ position: absolute; bottom: 76px; right: 20px; left: 20px; background: var(--primary); color: #fff; padding: 10px 14px; border-radius: 12px; font-size: 13px; text-align: center; display: none; z-index: 40; box-shadow: 0 8px 20px rgba(0,0,0,0.3); }}
.bottom-nav {{ display: flex; justify-content: space-around; background: var(--surface); border-top: 1px solid var(--border); padding: 8px 6px 14px; z-index: 15; }}
.lsn-tab {{ background: none; border: none; color: var(--muted); display: flex; flex-direction: column; align-items: center; gap: 4px; font-size: 11px; cursor: pointer; padding: 4px 10px; border-radius: 10px; transition: color 0.15s; }}
.lsn-tab.active {{ color: var(--primary); font-weight: 700; }}
</style>
</head>
<body>
<div class="phone-frame">
  <div class="status-bar">
    <span>9:41</span>
    <span>لسان موبايل 5G 100%</span>
  </div>
  <div class="app-header">
    <h1>{top_icon} <span>{safe_app_name}</span></h1>
    <button class="mobile-theme-btn" type="button" onclick="toggleMobileTheme()" aria-label="تبديل الوضع الليلي والنهاري" title="تبديل النسق">{theme_icon}</button>
  </div>
  <div class="app-content" id="appContent">
    {body_content}
  </div>
  {overlays_content}
  <div class="lsn-toast" id="toastMsg" role="status" aria-live="polite"></div>
  <nav class="bottom-nav" aria-label="التنقل السفلي">
    {tabs_html}
  </nav>
</div>
<script>
const LSN_STORAGE_PREFIX = 'lisan_mobile_v2:';
function lsnSaveState(key, val) {{
  try {{ localStorage.setItem(LSN_STORAGE_PREFIX + key, JSON.stringify(val)); }} catch (_) {{}}
}}
function lsnLoadState(key, fallback) {{
  try {{
    const raw = localStorage.getItem(LSN_STORAGE_PREFIX + key);
    return raw === null ? fallback : JSON.parse(raw);
  }} catch (_) {{ return fallback; }}
}}
function showToast(msg) {{
  const t = document.getElementById('toastMsg');
  if (!t) return;
  t.textContent = msg;
  t.style.display = 'block';
  clearTimeout(window._lsnToastTimer);
  window._lsnToastTimer = setTimeout(() => {{ t.style.display = 'none'; }}, 2200);
}}
function toggleSwitch(btn, title) {{
  btn.classList.toggle('on');
  const isOn = btn.classList.contains('on');
  lsnSaveState('switch:' + title, isOn);
  showToast(title + ': ' + (isOn ? 'مُفعّل' : 'مُعطّل'));
}}
function selectTab(btn, name) {{
  document.querySelectorAll('.lsn-tab').forEach(b => b.classList.remove('active'));
  if (btn) btn.classList.add('active');
  showToast('الانتقال إلى: ' + name);
}}
function selectScreen(btn, screenId, name) {{
  document.querySelectorAll('.lsn-tab').forEach(b => b.classList.remove('active'));
  if (btn) btn.classList.add('active');
  const screens = document.querySelectorAll('.lsn-screen');
  if (screens.length > 0) {{
    screens.forEach(s => s.classList.remove('active'));
    const target = document.getElementById('screen-' + screenId);
    if (target) target.classList.add('active');
  }}
  lsnSaveState('activeScreen', screenId);
  showToast('الشاشة: ' + name);
}}
function openBottomSheet(sheetId) {{
  const el = document.getElementById('sheet-' + sheetId);
  if (el) el.classList.add('open');
}}
function closeBottomSheet(sheetId) {{
  const el = document.getElementById('sheet-' + sheetId);
  if (el) el.classList.remove('open');
}}
function applyMobileTheme(mode) {{
  const root = document.documentElement;
  if (mode === 'light') {{
    root.style.setProperty('--bg', '#f8fafc');
    root.style.setProperty('--surface', '#ffffff');
    root.style.setProperty('--surface-2', '#f1f5f9');
    root.style.setProperty('--text', '#0f172a');
    root.style.setProperty('--muted', '#64748b');
    root.style.setProperty('--border', '#e2e8f0');
  }} else if (mode === 'dark') {{
    root.style.setProperty('--bg', '#0f172a');
    root.style.setProperty('--surface', '#1e293b');
    root.style.setProperty('--surface-2', '#334155');
    root.style.setProperty('--text', '#f8fafc');
    root.style.setProperty('--muted', '#94a3b8');
    root.style.setProperty('--border', '#334155');
  }}
}}
function toggleMobileTheme() {{
  const root = document.documentElement;
  const currentBg = getComputedStyle(root).getPropertyValue('--bg').trim();
  const nextMode = (currentBg === '#f8fafc' || currentBg.includes('248')) ? 'dark' : 'light';
  applyMobileTheme(nextMode);
  lsnSaveState('themeMode', nextMode);
  showToast(nextMode === 'dark' ? 'الوضع الليلي 🌙' : 'الوضع النهاري ☀️');
}}
function handleMobileForm(e, formId, successMsg) {{
  e.preventDefault();
  const data = {{}};
  new FormData(e.target).forEach((v, k) => {{ data[k] = v; }});
  lsnSaveState('form:' + formId, data);
  showToast(successMsg || 'تم الحفظ بنجاح');
}}
document.addEventListener('keydown', (e) => {{
  if (e.key === 'Escape') {{
    document.querySelectorAll('.lsn-sheet-backdrop.open').forEach(el => el.classList.remove('open'));
  }}
}});
document.addEventListener('DOMContentLoaded', () => {{
  const savedMode = lsnLoadState('themeMode', null);
  if (savedMode) applyMobileTheme(savedMode);
  if ('serviceWorker' in navigator && location.protocol.startsWith('http')) {{
    navigator.serviceWorker.register('./sw.js').catch(() => {{}});
  }}
}});
</script>
</body>
</html>"##
    )
}

/// تصدير تطبيق الهاتف الذكي كحزمة PWA قابلة للتثبيت والعمل دون اتصال (Android / iOS)
#[cfg(not(target_arch = "wasm32"))]
pub fn export_mobile_pwa(app_name: &str, html: &str, out_dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("تعذر إنشاء مجلد تطبيق الهاتف '{}': {}", out_dir.display(), e))?;

    let index_path = out_dir.join("index.html");
    std::fs::write(&index_path, html)
        .map_err(|e| format!("تعذر حفظ ملف الواجهة '{}': {}", index_path.display(), e))?;

    let safe_name = escape_json_str(app_name);
    let manifest = format!(
        r##"{{
  "name": "{safe_name}",
  "short_name": "{safe_name}",
  "description": "تطبيق هاتف ذكي عربي مبني بلغة لسان",
  "lang": "ar",
  "dir": "rtl",
  "start_url": "./index.html",
  "scope": "./",
  "display": "standalone",
  "orientation": "portrait",
  "background_color": "#0f172a",
  "theme_color": "#3b82f6"
}}"##
    );
    std::fs::write(out_dir.join("manifest.json"), manifest)
        .map_err(|e| format!("تعذر حفظ manifest.json: {}", e))?;

    let sw = r##"const CACHE_NAME = 'lisan-pwa-cache-v2';
const PRECACHE_URLS = ['./', './index.html', './manifest.json'];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => cache.addAll(PRECACHE_URLS))
      .then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k)))
    ).then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') return;
  event.respondWith(
    fetch(event.request)
      .then((response) => {
        if (response && response.status === 200) {
          const clone = response.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(event.request, clone));
        }
        return response;
      })
      .catch(() => caches.match(event.request).then((hit) => hit || caches.match('./index.html')))
  );
});
"##;
    std::fs::write(out_dir.join("sw.js"), sw)
        .map_err(|e| format!("تعذر حفظ sw.js: {}", e))?;

    Ok(index_path)
}

#[cfg(target_arch = "wasm32")]
pub fn export_mobile_pwa(_app_name: &str, _html: &str, _out_dir: &Path) -> Result<PathBuf, String> {
    Err("تصدير تطبيق الهاتف إلى نظام الملفات غير مدعوم داخل WebAssembly".into())
}

/// تصدير صفحة موقع ويب كملف index.html جاهز داخل مجلد
#[cfg(not(target_arch = "wasm32"))]
pub fn export_web_page(_site_name: &str, html: &str, out_dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("تعذر إنشاء مجلد الموقع '{}': {}", out_dir.display(), e))?;

    let index_path = out_dir.join("index.html");
    std::fs::write(&index_path, html)
        .map_err(|e| format!("تعذر حفظ ملف الموقع '{}': {}", index_path.display(), e))?;

    Ok(index_path)
}

#[cfg(target_arch = "wasm32")]
pub fn export_web_page(_site_name: &str, _html: &str, _out_dir: &Path) -> Result<PathBuf, String> {
    Err("تصدير صفحة الويب إلى نظام الملفات غير مدعوم داخل WebAssembly".into())
}

/// بناء صفحة موقع ويب متكاملة ومتجاوبة بالعربية
pub fn build_web_page_html(title: &str, theme: &str, sections_html: &[String]) -> String {
    let theme_vars = get_theme_css(theme);
    let safe_title = escape_html_ui(title);
    let body = sections_html.join("\n");
    format!(
        r##"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{safe_title}</title>
<style>
{theme_vars}
* {{ box-sizing: border-box; margin: 0; padding: 0; font-family: 'Segoe UI', 'Cairo', Tahoma, system-ui, sans-serif; }}
body {{ background: var(--bg); color: var(--text); line-height: 1.6; transition: background 0.3s, color 0.3s; }}
.container {{ max-width: 1100px; margin: 0 auto; padding: 24px; display: flex; flex-direction: column; gap: 32px; }}

/* شريط التنبيه العلوي */
.web-banner {{ display: flex; align-items: center; justify-content: center; gap: 10px; padding: 12px 20px; border-radius: 14px; font-size: 14px; font-weight: 700; border: 1px solid var(--border); background: var(--surface); color: var(--text); }}
.web-banner-info {{ border-color: var(--primary); background: var(--surface); color: var(--text); }}
.web-banner-success {{ border-color: #10b981; background: rgba(16, 185, 129, 0.14); color: var(--text); }}
.web-banner-warning {{ border-color: #f59e0b; background: rgba(245, 158, 11, 0.14); color: var(--text); }}
.web-banner-danger {{ border-color: #ef4444; background: rgba(239, 68, 68, 0.14); color: var(--text); }}

/* شريط التنقل العلوي */
.web-nav {{ display: flex; justify-content: space-between; align-items: center; background: var(--surface); border: 1px solid var(--border); border-radius: 18px; padding: 14px 24px; position: sticky; top: 16px; z-index: 100; backdrop-filter: blur(10px); box-shadow: 0 4px 20px rgba(0,0,0,0.06); flex-wrap: wrap; gap: 12px; }}
.web-brand {{ display: flex; align-items: center; gap: 10px; font-size: 19px; font-weight: 800; color: var(--text); }}
.web-links {{ display: flex; align-items: center; gap: 20px; flex-wrap: wrap; }}
.web-links a {{ color: var(--muted); text-decoration: none; font-size: 15px; font-weight: 600; transition: color 0.2s; }}
.web-links a:hover {{ color: var(--primary); }}
.theme-toggle-btn {{ background: var(--surface-2); border: 1px solid var(--border); color: var(--text); padding: 8px 14px; border-radius: 10px; cursor: pointer; display: inline-flex; align-items: center; gap: 6px; font-size: 13px; font-weight: 600; transition: all 0.2s; }}
.theme-toggle-btn:hover {{ border-color: var(--primary); color: var(--primary); }}

/* القسم الترحيبي الرئيسي */
.web-hero {{ background: linear-gradient(135deg, var(--surface), var(--surface-2)); border: 1px solid var(--border); border-radius: 24px; padding: 56px 32px; text-align: center; display: flex; flex-direction: column; align-items: center; gap: 18px; box-shadow: 0 10px 30px rgba(0,0,0,0.08); position: relative; overflow: hidden; }}
.web-hero h1 {{ font-size: 38px; font-weight: 900; }}
.web-hero p {{ color: var(--muted); max-width: 680px; font-size: 18px; line-height: 1.7; }}

/* البطاقات والشبكة */
.web-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 20px; }}
.lsn-card {{ background: var(--surface); border: 1px solid var(--border); border-radius: 18px; padding: 22px; display: flex; align-items: flex-start; gap: 16px; transition: transform 0.2s, box-shadow 0.2s, border-color 0.2s; }}
.lsn-card:hover {{ transform: translateY(-4px); box-shadow: 0 8px 24px rgba(0,0,0,0.12); border-color: var(--primary); }}
.lsn-card-icon {{ width: 50px; height: 50px; border-radius: 14px; background: var(--surface-2); color: var(--primary); display: flex; align-items: center; justify-content: center; flex-shrink: 0; }}
.lsn-card-body h3 {{ font-size: 17px; margin-bottom: 6px; color: var(--text); font-weight: 700; }}
.lsn-card-body p {{ color: var(--muted); font-size: 14px; line-height: 1.5; }}

/* الأزرار الفصيحة */
.lsn-btn {{ padding: 13px 26px; border: none; border-radius: 14px; background: linear-gradient(135deg, var(--primary), var(--accent)); color: #fff; font-size: 15px; font-weight: 700; cursor: pointer; display: inline-flex; align-items: center; justify-content: center; gap: 10px; transition: transform 0.15s, opacity 0.2s; box-shadow: 0 4px 16px rgba(0,0,0,0.15); }}
.lsn-btn:hover {{ opacity: 0.95; transform: scale(1.02); }}
.lsn-btn:active {{ transform: scale(0.98); }}
.full-btn {{ width: 100%; }}

/* الإحصائيات */
.web-stats {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 16px; }}
.lsn-stat {{ background: var(--surface); border: 1px solid var(--border); border-radius: 18px; padding: 18px 20px; display: flex; flex-direction: column; gap: 8px; text-align: center; }}
.lsn-stat-top {{ display: flex; justify-content: space-between; align-items: center; color: var(--muted); font-size: 13px; font-weight: 600; }}
.lsn-stat-val {{ font-size: 28px; font-weight: 900; color: var(--primary); }}

/* الأقسام العامة */
.web-section {{ background: var(--surface); border: 1px solid var(--border); border-radius: 20px; padding: 32px; display: flex; flex-direction: column; gap: 14px; }}
.web-section h2 {{ font-size: 24px; font-weight: 800; display: flex; align-items: center; gap: 10px; }}
.web-section p {{ color: var(--muted); font-size: 16px; }}

/* باقات الأسعار */
.web-pricing {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 22px; align-items: stretch; }}
.web-price-card {{ background: var(--surface); border: 1px solid var(--border); border-radius: 22px; padding: 28px 24px; display: flex; flex-direction: column; gap: 14px; transition: transform 0.2s, border-color 0.2s; }}
.web-price-card:hover {{ transform: translateY(-4px); border-color: var(--primary); }}
.web-price-card.featured {{ border: 2px solid var(--primary); box-shadow: 0 12px 32px rgba(0,0,0,0.18); position: relative; }}
.web-price-badge {{ display: inline-block; align-self: flex-start; padding: 3px 12px; border-radius: 999px; background: var(--primary); color: #fff; font-size: 12px; font-weight: 700; }}
.web-price-card h3 {{ font-size: 20px; font-weight: 800; }}
.web-price-val {{ font-size: 34px; font-weight: 900; color: var(--primary); }}
.web-price-desc {{ font-size: 14px; color: var(--muted); }}
.web-price-list {{ list-style: none; display: flex; flex-direction: column; gap: 8px; flex: 1; margin: 4px 0 8px; }}
.web-price-list li {{ display: flex; align-items: center; gap: 8px; font-size: 14px; color: var(--text); }}

/* الأسئلة الشائعة التفاعلية */
.web-faq {{ display: flex; flex-direction: column; gap: 12px; }}
.web-faq-item {{ background: var(--surface); border: 1px solid var(--border); border-radius: 16px; padding: 16px 20px; transition: border-color 0.2s; }}
.web-faq-item[open] {{ border-color: var(--primary); }}
.web-faq-item summary {{ font-size: 16px; font-weight: 700; cursor: pointer; list-style: none; display: flex; justify-content: space-between; align-items: center; gap: 12px; }}
.web-faq-item summary::after {{ content: '+'; font-size: 20px; font-weight: 800; color: var(--primary); }}
.web-faq-item[open] summary::after {{ content: '−'; }}
.web-faq-item p {{ margin-top: 12px; color: var(--muted); font-size: 15px; line-height: 1.7; }}

/* الجداول المتجاوبة */
.web-table-wrap {{ background: var(--surface); border: 1px solid var(--border); border-radius: 18px; overflow-x: auto; }}
.web-table {{ width: 100%; border-collapse: collapse; text-align: right; font-size: 14px; }}
.web-table thead {{ background: var(--surface-2); }}
.web-table th {{ padding: 14px 18px; font-weight: 800; color: var(--text); border-bottom: 1px solid var(--border); white-space: nowrap; }}
.web-table td {{ padding: 13px 18px; color: var(--muted); border-bottom: 1px solid var(--border); }}
.web-table tbody tr:last-child td {{ border-bottom: none; }}
.web-table tbody tr:hover {{ background: var(--surface-2); }}

/* شهادات وآراء العملاء */
.web-testimonials {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 20px; }}
.web-quote-card {{ background: var(--surface); border: 1px solid var(--border); border-radius: 20px; padding: 24px; display: flex; flex-direction: column; gap: 14px; }}
.web-quote-stars {{ color: #f59e0b; font-size: 15px; letter-spacing: 2px; }}
.web-quote-text {{ font-size: 15px; color: var(--text); line-height: 1.7; flex: 1; }}
.web-quote-author {{ display: flex; flex-direction: column; gap: 2px; border-top: 1px solid var(--border); padding-top: 12px; }}
.web-quote-author strong {{ font-size: 15px; color: var(--text); }}
.web-quote-author span {{ font-size: 13px; color: var(--muted); }}

/* معرض الأعمال */
.web-gallery {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 20px; }}
.web-gallery-card {{ background: var(--surface); border: 1px solid var(--border); border-radius: 20px; overflow: hidden; display: flex; flex-direction: column; transition: transform 0.2s, border-color 0.2s; }}
.web-gallery-card:hover {{ transform: translateY(-4px); border-color: var(--primary); }}
.web-gallery-visual {{ height: 120px; background: linear-gradient(135deg, var(--surface-2), var(--bg)); display: flex; align-items: center; justify-content: center; color: var(--primary); }}
.web-gallery-body {{ padding: 18px 20px; display: flex; flex-direction: column; gap: 8px; }}
.web-gallery-tag {{ display: inline-block; align-self: flex-start; padding: 3px 10px; border-radius: 999px; background: var(--surface-2); color: var(--primary); font-size: 12px; font-weight: 700; }}
.web-gallery-body h3 {{ font-size: 17px; font-weight: 700; color: var(--text); }}
.web-gallery-body p {{ font-size: 14px; color: var(--muted); }}

/* نموذج التواصل التفاعلي */
.web-form-box {{ background: var(--surface); border: 1px solid var(--border); border-radius: 24px; padding: 36px 32px; display: flex; flex-direction: column; gap: 20px; max-width: 650px; margin: 0 auto; width: 100%; box-shadow: 0 6px 24px rgba(0,0,0,0.06); }}
.web-form-box h2 {{ font-size: 26px; font-weight: 800; text-align: center; }}
.web-form-box p {{ color: var(--muted); text-align: center; font-size: 15px; }}
.web-form-box form {{ display: flex; flex-direction: column; gap: 16px; }}
.form-group {{ display: flex; flex-direction: column; gap: 6px; text-align: right; }}
.form-group label {{ font-size: 13px; font-weight: 700; color: var(--muted); }}
.lsn-input {{ width: 100%; padding: 12px 16px; border-radius: 12px; border: 1px solid var(--border); background: var(--surface-2); color: var(--text); font-size: 14px; outline: none; transition: border-color 0.2s; }}
.lsn-input:focus {{ border-color: var(--primary); }}

/* التذييل */
.web-footer {{ text-align: center; padding: 28px 20px; border-top: 1px solid var(--border); color: var(--muted); font-size: 14px; display: flex; flex-direction: column; align-items: center; gap: 8px; }}
.footer-badge {{ display: inline-flex; align-items: center; gap: 6px; font-size: 12px; color: var(--primary); font-weight: 700; }}

/* إشعار عائم */
.lsn-toast {{ position: fixed; bottom: 28px; left: 50%; transform: translateX(-50%); background: var(--primary); color: #fff; padding: 12px 24px; border-radius: 14px; font-size: 14px; font-weight: 700; box-shadow: 0 8px 30px rgba(0,0,0,0.3); display: none; z-index: 1000; transition: opacity 0.3s; }}
</style>
</head>
<body>
<div class="container">
{body}
</div>
<div class="lsn-toast" id="toastMsg"></div>
<script>
function showToast(msg) {{
  const t = document.getElementById('toastMsg');
  if (!t) return;
  t.textContent = msg;
  t.style.display = 'block';
  t.style.opacity = '1';
  setTimeout(() => {{
    t.style.opacity = '0';
    setTimeout(() => {{ t.style.display = 'none'; }}, 300);
  }}, 2500);
}}
function toggleTheme() {{
  const root = document.documentElement;
  const currentBg = getComputedStyle(root).getPropertyValue('--bg').trim();
  if (currentBg === '#f8fafc' || currentBg.includes('248')) {{
    root.style.setProperty('--bg', '#0f172a');
    root.style.setProperty('--surface', '#1e293b');
    root.style.setProperty('--surface-2', '#334155');
    root.style.setProperty('--text', '#f8fafc');
    root.style.setProperty('--muted', '#94a3b8');
    root.style.setProperty('--border', '#334155');
    showToast('تم تفعيل الوضع الليلي 🌙');
  }} else {{
    root.style.setProperty('--bg', '#f8fafc');
    root.style.setProperty('--surface', '#ffffff');
    root.style.setProperty('--surface-2', '#f1f5f9');
    root.style.setProperty('--text', '#0f172a');
    root.style.setProperty('--muted', '#64748b');
    root.style.setProperty('--border', '#e2e8f0');
    showToast('تم تفعيل الوضع النهاري ☀️');
  }}
}}
function handleContact(e) {{
  e.preventDefault();
  showToast('شكراً لتواصلك! تم استلام رسالتك بنجاح ✅');
  e.target.reset();
}}
</script>
</body>
</html>"##
    )
}

/// المكتبة القياسية المدمجة للأيقونات والتصميمات وتطبيقات الهواتف والويب بلغة لسان
pub const STDLIB_UI_AND_MOBILE: &str = r##"
# مكتبة الأيقونات والتصميمات وتطبيقات الهواتف الذكية والويب في لغة لسان 2.0

دالة _تنظيف_نص_واجهة(قيمة) {
    دع س = استبدل(نص(قيمة)، "&"، "&amp;")
    س = استبدل(س، "<"، "&lt;")
    س = استبدل(س، ">"، "&gt;")
    س = استبدل(س، "\""، "&quot;")
    أرجع استبدل(س، "'"، "&#39;")
}

دالة _تنظيف_نص_js(قيمة) {
    دع س = استبدل(نص(قيمة)، "\\"، "\\\\")
    س = استبدل(س، "'"، "\\'")
    س = استبدل(س، "\""، "&quot;")
    س = استبدل(س، "<"، "&lt;")
    س = استبدل(س، ">"، "&gt;")
    أرجع استبدل(س، "\n"، " ")
}

صنف تطبيق_هاتف {
    دالة بناء(اسم_التطبيق، النسق) {
        هذا.اسم_التطبيق = اسم_التطبيق
        هذا.النسق = النسق
        هذا.أيقونة_العنوان = "هاتف"
        هذا.العناصر = []
        هذا.التبويبات = []
        هذا.الشاشات = []
        هذا.التراكبات = []
    }

    دالة اختر_نسق(نسق_جديد) {
        هذا.النسق = نسق_جديد
        أرجع هذا.النسق
    }

    دالة أضف_بطاقة(عنوان، وصف، اسم_الأيقونة) {
        دع كود = عنصر_بطاقة(عنوان، وصف، اسم_الأيقونة)
        هذا.العناصر = أضف(هذا.العناصر، كود)
        أرجع كود
    }

    دالة أضف_إحصائية(عنوان، قيمة، اسم_الأيقونة) {
        دع كود = عنصر_إحصائية(عنوان، نص(قيمة)، اسم_الأيقونة)
        هذا.العناصر = أضف(هذا.العناصر، كود)
        أرجع كود
    }

    دالة أضف_مفتاح_تحكم(عنوان، اسم_الأيقونة، مفعّل) {
        دع كود = عنصر_مفتاح(عنوان، اسم_الأيقونة، مفعّل)
        هذا.العناصر = أضف(هذا.العناصر، كود)
        أرجع كود
    }

    دالة أضف_زر(نص_الزر، اسم_الأيقونة، رسالة_التفاعل) {
        دع كود = عنصر_زر(نص_الزر، اسم_الأيقونة، رسالة_التفاعل)
        هذا.العناصر = أضف(هذا.العناصر، كود)
        أرجع كود
    }

    دالة أضف_عنصر(كود_html) {
        هذا.العناصر = أضف(هذا.العناصر، كود_html)
        أرجع كود_html
    }

    دالة أضف_تبويب(عنوان، اسم_الأيقونة) {
        هذا.التبويبات = أضف(هذا.التبويبات، [عنوان، اسم_الأيقونة])
        أرجع طول(هذا.التبويبات)
    }

    دالة أضف_شاشة(معرف، عنوان، اسم_الأيقونة، قائمة_عناصر) {
        دع معرف_آمن = _تنظيف_نص_واجهة(معرف)
        دع عنوان_آمن = _تنظيف_نص_واجهة(عنوان)
        دع رمز = أيقونة(اسم_الأيقونة، 16، "var(--primary)")
        دع محتوى_الشاشة = "<div class=\"lsn-screen-title\">" + رمز + "<span>" + عنوان_آمن + "</span></div>"
        لكل عنصر_شاشة في قائمة_عناصر {
            محتوى_الشاشة = محتوى_الشاشة + نص(عنصر_شاشة)
        }
        هذا.الشاشات = أضف(هذا.الشاشات، [معرف_آمن، عنوان، اسم_الأيقونة، محتوى_الشاشة])
        أرجع محتوى_الشاشة
    }

    دالة أضف_زر_عائم(اسم_الأيقونة، رسالة، معرف_نافذة = "") {
        دع رمز = أيقونة(اسم_الأيقونة، 22، "#ffffff")
        دع رسالة_js = _تنظيف_نص_js(رسالة)
        دع حدث = "showToast('" + رسالة_js + "')"
        إذا (معرف_نافذة != "") {
            دع معرف_js = _تنظيف_نص_js(معرف_نافذة)
            حدث = "openBottomSheet('" + معرف_js + "')"
            إذا (رسالة != "") {
                حدث = حدث + ";showToast('" + رسالة_js + "')"
            }
        }
        دع كود = "<!--LSN_FAB--><button class=\"lsn-fab\" type=\"button\" aria-label=\"" + _تنظيف_نص_واجهة(رسالة) + "\" onclick=\"" + حدث + "\">" + رمز + "</button>"
        هذا.التراكبات = أضف(هذا.التراكبات، كود)
        أرجع كود
    }

    دالة أضف_نافذة_سفلية(معرف، عنوان، محتوى) {
        دع معرف_آمن = _تنظيف_نص_واجهة(معرف)
        دع معرف_js = _تنظيف_نص_js(معرف)
        دع عنوان_آمن = _تنظيف_نص_واجهة(عنوان)
        دع كود = "<!--LSN_SHEET--><div class=\"lsn-sheet-backdrop\" id=\"sheet-" + معرف_آمن + "\" onclick=\"if(event.target===this)closeBottomSheet('" + معرف_js + "')\"><div class=\"lsn-sheet\" role=\"dialog\" aria-modal=\"true\" aria-label=\"" + عنوان_آمن + "\"><div class=\"lsn-sheet-handle\"></div><div class=\"lsn-sheet-head\"><h3>" + عنوان_آمن + "</h3><button class=\"lsn-sheet-close\" type=\"button\" aria-label=\"إغلاق\" onclick=\"closeBottomSheet('" + معرف_js + "')\">×</button></div><div class=\"lsn-sheet-body\">" + نص(محتوى) + "</div></div></div>"
        هذا.التراكبات = أضف(هذا.التراكبات، كود)
        أرجع كود
    }

    دالة أضف_نموذج_هاتف(معرف، عنوان، حقول، نص_الزر = "حفظ"، رسالة_النجاح = "تم الحفظ بنجاح ✅") {
        دع معرف_js = _تنظيف_نص_js(معرف)
        دع نجاح_js = _تنظيف_نص_js(رسالة_النجاح)
        دع كود_الحقول = ""
        لكل حقل في حقول {
            دع اسم_حقل = _تنظيف_نص_واجهة(حقل[0])
            دع تسمية_حقل = اسم_حقل
            دع نوع_ادخال = "text"
            دع تلميح = ""
            إذا (طول(حقل) > 1) {
                تسمية_حقل = _تنظيف_نص_واجهة(حقل[1])
            }
            إذا (طول(حقل) > 2) {
                نوع_ادخال = _تنظيف_نص_واجهة(حقل[2])
            }
            إذا (طول(حقل) > 3) {
                تلميح = _تنظيف_نص_واجهة(حقل[3])
            }
            كود_الحقول = كود_الحقول + "<div class=\"lsn-field\"><label>" + تسمية_حقل + "</label><input name=\"" + اسم_حقل + "\" type=\"" + نوع_ادخال + "\" placeholder=\"" + تلميح + "\" required></div>"
        }
        دع كود = "<form class=\"lsn-mobile-form\" id=\"form-" + _تنظيف_نص_واجهة(معرف) + "\" onsubmit=\"handleMobileForm(event, '" + معرف_js + "', '" + نجاح_js + "')\"><h3>" + _تنظيف_نص_واجهة(عنوان) + "</h3>" + كود_الحقول + "<button class=\"lsn-btn\" type=\"submit\">" + أيقونة("صح"، 18، "#ffffff") + "<span>" + _تنظيف_نص_واجهة(نص_الزر) + "</span></button></form>"
        هذا.العناصر = أضف(هذا.العناصر، كود)
        أرجع كود
    }

    دالة بناء_واجهة() {
        دع كل_العناصر = []
        إذا (طول(هذا.الشاشات) > 0) {
            دع هناك_رئيسية_عامة = طول(هذا.العناصر) > 0
            إذا (هناك_رئيسية_عامة) {
                دع محتوى_رئيسي = ادمج(هذا.العناصر، "\n")
                كل_العناصر = أضف(كل_العناصر، "<section class=\"lsn-screen active\" id=\"screen-main\" data-screen=\"main\">" + محتوى_رئيسي + "</section>")
                كل_العناصر = أضف(كل_العناصر، "<!--LSN_TAB:main|الرئيسية|" + هذا.أيقونة_العنوان + "-->")
            }
            دع فهرس = 0
            لكل شاشة في هذا.الشاشات {
                دع ش_معرف = شاشة[0]
                دع ش_عنوان = شاشة[1]
                دع ش_أيقونة = شاشة[2]
                دع ش_محتوى = شاشة[3]
                دع فئة_نشطة = "lsn-screen"
                إذا (ليس هناك_رئيسية_عامة و فهرس == 0) {
                    فئة_نشطة = "lsn-screen active"
                }
                كل_العناصر = أضف(كل_العناصر، "<section class=\"" + فئة_نشطة + "\" id=\"screen-" + ش_معرف + "\" data-screen=\"" + ش_معرف + "\">" + ش_محتوى + "</section>")
                كل_العناصر = أضف(كل_العناصر، "<!--LSN_TAB:" + ش_معرف + "|" + ش_عنوان + "|" + ش_أيقونة + "-->")
                فهرس = فهرس + 1
            }
        } وإلا {
            لكل عنصر_عادي في هذا.العناصر {
                كل_العناصر = أضف(كل_العناصر، عنصر_عادي)
            }
            لكل تبويب في هذا.التبويبات {
                كل_العناصر = أضف(كل_العناصر، "<!--LSN_TAB:|" + تبويب[0] + "|" + تبويب[1] + "-->")
            }
        }
        لكل تراكب في هذا.التراكبات {
            كل_العناصر = أضف(كل_العناصر، تراكب)
        }
        أرجع بناء_تطبيق_هاتف(هذا.اسم_التطبيق، هذا.النسق، هذا.أيقونة_العنوان، كل_العناصر)
    }

    دالة تصدير(مجلد_الإخراج) {
        دع html = هذا.بناء_واجهة()
        أرجع تصدير_تطبيق_هاتف(هذا.اسم_التطبيق، html، مجلد_الإخراج)
    }
}

صنف موقع_ويب {
    دالة بناء(عنوان_الموقع، النسق = "ملكي") {
        هذا.عنوان_الموقع = عنوان_الموقع
        هذا.النسق = النسق
        هذا.الأقسام = []
    }

    دالة أضف_تنبيه_أعلى(رسالة، نوع_التنبيه = "معلومات") {
        دع فئة_لون = "web-banner-info"
        دع رمز_اسم = "معلومات"
        إذا (نوع_التنبيه == "نجاح") {
            فئة_لون = "web-banner-success"
            رمز_اسم = "صح"
        }
        إذا (نوع_التنبيه == "تحذير") {
            فئة_لون = "web-banner-warning"
            رمز_اسم = "تحذير"
        }
        إذا (نوع_التنبيه == "خطر" أو نوع_التنبيه == "تنبيه") {
            فئة_لون = "web-banner-danger"
            رمز_اسم = "تحذير"
        }
        دع كود = "<div class=\"web-banner " + فئة_لون + "\" role=\"status\">" + أيقونة(رمز_اسم، 18، "currentColor") + "<span>" + _تنظيف_نص_واجهة(رسالة) + "</span></div>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة شريط_تنقل(اسم_الموقع، أيقونة_الشعار = "كود") {
        دع رمز = أيقونة(أيقونة_الشعار، 24، "var(--primary)")
        دع زر_النسق = "<button class=\"theme-toggle-btn\" type=\"button\" onclick=\"toggleTheme()\">" + أيقونة("قمر"، 16، "currentColor") + " <span>النسق</span></button>"
        دع كود = "<header class=\"web-nav\"><div class=\"web-brand\">" + رمز + "<span>" + _تنظيف_نص_واجهة(اسم_الموقع) + "</span></div><nav class=\"web-links\"><a href=\"#hero\">الرئيسية</a><a href=\"#features\">الميزات</a><a href=\"#stats\">إحصائيات</a><a href=\"#contact\">تواصل</a></nav>" + زر_النسق + "</header>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة قسم_رئيسي(عنوان، وصف، نص_الزر = "ابدأ الآن"، اسم_الأيقونة = "روبوت") {
        دع رمز = أيقونة(اسم_الأيقونة، 42، "var(--primary)")
        دع زر = عنصر_زر(_تنظيف_نص_واجهة(نص_الزر)، "تشغيل"، "مرحباً بكم في " + _تنظيف_نص_js(عنوان))
        دع قسم = "<section id=\"hero\" class=\"web-hero\">" + رمز + "<h1>" + _تنظيف_نص_واجهة(عنوان) + "</h1><p>" + _تنظيف_نص_واجهة(وصف) + "</p>" + زر + "</section>"
        هذا.الأقسام = أضف(هذا.الأقسام، قسم)
        أرجع قسم
    }

    دالة أضف_بطاقة(عنوان، وصف، اسم_الأيقونة = "نجمة") {
        دع كود = عنصر_بطاقة(_تنظيف_نص_واجهة(عنوان)، _تنظيف_نص_واجهة(وصف)، اسم_الأيقونة)
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_إحصائية(عنوان، قيمة، اسم_الأيقونة = "شريحة") {
        دع كود = عنصر_إحصائية(_تنظيف_نص_واجهة(عنوان)، _تنظيف_نص_واجهة(قيمة)، اسم_الأيقونة)
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_إحصائيات(قائمة_الإحصائيات) {
        دع محتوى = ""
        لكل إحصاء في قائمة_الإحصائيات {
            دع عنوان = إحصاء[0]
            دع قيمة = إحصاء[1]
            دع رمز = "شريحة"
            إذا (طول(إحصاء) > 2) {
                رمز = إحصاء[2]
            }
            محتوى = محتوى + عنصر_إحصائية(_تنظيف_نص_واجهة(عنوان)، _تنظيف_نص_واجهة(قيمة)، رمز)
        }
        دع كود = "<section id=\"stats\" class=\"web-stats\">" + محتوى + "</section>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_شبكة(قائمة_البطاقات) {
        دع محتوى = ""
        لكل بطاقة في قائمة_البطاقات {
            دع عنوان = بطاقة[0]
            دع وصف = بطاقة[1]
            دع رمز = "نجمة"
            إذا (طول(بطاقة) > 2) {
                رمز = بطاقة[2]
            }
            محتوى = محتوى + عنصر_بطاقة(_تنظيف_نص_واجهة(عنوان)، _تنظيف_نص_واجهة(وصف)، رمز)
        }
        دع كود = "<section id=\"features\" class=\"web-grid\">" + محتوى + "</section>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_أسعار(قائمة_الباقات) {
        دع محتوى = ""
        لكل باقة في قائمة_الباقات {
            دع اسم_الباقة = ""
            دع السعر = ""
            دع الوصف = ""
            دع المزايا = []
            دع مميزة = خطأ
            دع نص_الزر = "اختر الباقة"
            إذا (نوع(باقة) == "قاموس") {
                إذا (يحتوي(باقة، "اسم")) { اسم_الباقة = باقة["اسم"] }
                إذا (يحتوي(باقة، "سعر")) { السعر = باقة["سعر"] }
                إذا (يحتوي(باقة، "وصف")) { الوصف = باقة["وصف"] }
                إذا (يحتوي(باقة، "مزايا")) { المزايا = باقة["مزايا"] }
                إذا (يحتوي(باقة، "مميزة")) { مميزة = باقة["مميزة"] }
                إذا (يحتوي(باقة، "زر")) { نص_الزر = باقة["زر"] }
            } وإلا {
                اسم_الباقة = باقة[0]
                السعر = باقة[1]
                إذا (طول(باقة) > 2) { الوصف = باقة[2] }
                إذا (طول(باقة) > 3) { المزايا = باقة[3] }
                إذا (طول(باقة) > 4) { مميزة = باقة[4] }
                إذا (طول(باقة) > 5) { نص_الزر = باقة[5] }
            }
            دع فئة_بطاقة = "web-price-card"
            دع شارة_مميزة = ""
            إذا (مميزة) {
                فئة_بطاقة = "web-price-card featured"
                شارة_مميزة = "<span class=\"web-price-badge\">الأكثر شيوعاً</span>"
            }
            دع قائمة_html = ""
            لكل ميزة في المزايا {
                قائمة_html = قائمة_html + "<li>" + أيقونة("صح"، 16، "var(--primary)") + "<span>" + _تنظيف_نص_واجهة(ميزة) + "</span></li>"
            }
            دع زر_باقة = "<button class=\"lsn-btn full-btn\" type=\"button\" onclick=\"showToast('تم اختيار باقة: " + _تنظيف_نص_js(اسم_الباقة) + "')\">" + _تنظيف_نص_واجهة(نص_الزر) + "</button>"
            محتوى = محتوى + "<article class=\"" + فئة_بطاقة + "\">" + شارة_مميزة + "<h3>" + _تنظيف_نص_واجهة(اسم_الباقة) + "</h3><div class=\"web-price-val\">" + _تنظيف_نص_واجهة(السعر) + "</div><p class=\"web-price-desc\">" + _تنظيف_نص_واجهة(الوصف) + "</p><ul class=\"web-price-list\">" + قائمة_html + "</ul>" + زر_باقة + "</article>"
        }
        دع كود = "<section id=\"pricing\" class=\"web-pricing\">" + محتوى + "</section>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_أسئلة_شائعة(قائمة_الأسئلة) {
        دع محتوى = ""
        لكل بند في قائمة_الأسئلة {
            دع السؤال = ""
            دع الجواب = ""
            إذا (نوع(بند) == "قاموس") {
                إذا (يحتوي(بند، "سؤال")) { السؤال = بند["سؤال"] }
                إذا (يحتوي(بند، "جواب")) { الجواب = بند["جواب"] }
            } وإلا {
                السؤال = بند[0]
                الجواب = بند[1]
            }
            محتوى = محتوى + "<details class=\"web-faq-item\"><summary>" + _تنظيف_نص_واجهة(السؤال) + "</summary><p>" + _تنظيف_نص_واجهة(الجواب) + "</p></details>"
        }
        دع كود = "<section id=\"faq\" class=\"web-faq\">" + محتوى + "</section>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_جدول(ترويسات، صفوف) {
        دع رأس_الجدول = ""
        لكل ترويسة في ترويسات {
            رأس_الجدول = رأس_الجدول + "<th>" + _تنظيف_نص_واجهة(ترويسة) + "</th>"
        }
        دع جسم_الجدول = ""
        لكل صف_بيانات في صفوف {
            دع خلايا = ""
            لكل خلية في صف_بيانات {
                خلايا = خلايا + "<td>" + _تنظيف_نص_واجهة(خلية) + "</td>"
            }
            جسم_الجدول = جسم_الجدول + "<tr>" + خلايا + "</tr>"
        }
        دع كود = "<section class=\"web-table-wrap\"><table class=\"web-table\"><thead><tr>" + رأس_الجدول + "</tr></thead><tbody>" + جسم_الجدول + "</tbody></table></section>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_شهادات(قائمة_آراء) {
        دع محتوى = ""
        لكل شهادة في قائمة_آراء {
            دع الاسم = ""
            دع الصفة = ""
            دع الرأي = ""
            دع التقييم = "★★★★★"
            إذا (نوع(شهادة) == "قاموس") {
                إذا (يحتوي(شهادة، "اسم")) { الاسم = شهادة["اسم"] }
                إذا (يحتوي(شهادة، "صفة")) { الصفة = شهادة["صفة"] }
                إذا (يحتوي(شهادة، "رأي")) { الرأي = شهادة["رأي"] }
                إذا (يحتوي(شهادة، "تقييم")) { التقييم = شهادة["تقييم"] }
            } وإلا {
                الاسم = شهادة[0]
                الصفة = شهادة[1]
                الرأي = شهادة[2]
                إذا (طول(شهادة) > 3) { التقييم = شهادة[3] }
            }
            محتوى = محتوى + "<blockquote class=\"web-quote-card\"><div class=\"web-quote-stars\">" + _تنظيف_نص_واجهة(التقييم) + "</div><p class=\"web-quote-text\">«" + _تنظيف_نص_واجهة(الرأي) + "»</p><footer class=\"web-quote-author\"><strong>" + _تنظيف_نص_واجهة(الاسم) + "</strong><span>" + _تنظيف_نص_واجهة(الصفة) + "</span></footer></blockquote>"
        }
        دع كود = "<section id=\"testimonials\" class=\"web-testimonials\">" + محتوى + "</section>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_معرض(قائمة_عناصر) {
        دع محتوى = ""
        لكل عمل في قائمة_عناصر {
            دع العنوان = ""
            دع الوصف = ""
            دع الوسم = "مشروع"
            دع الرمز = "صاروخ"
            إذا (نوع(عمل) == "قاموس") {
                إذا (يحتوي(عمل، "عنوان")) { العنوان = عمل["عنوان"] }
                إذا (يحتوي(عمل، "وصف")) { الوصف = عمل["وصف"] }
                إذا (يحتوي(عمل، "وسم")) { الوسم = عمل["وسم"] }
                إذا (يحتوي(عمل، "أيقونة")) { الرمز = عمل["أيقونة"] }
            } وإلا {
                العنوان = عمل[0]
                الوصف = عمل[1]
                إذا (طول(عمل) > 2) { الوسم = عمل[2] }
                إذا (طول(عمل) > 3) { الرمز = عمل[3] }
            }
            محتوى = محتوى + "<article class=\"web-gallery-card\"><div class=\"web-gallery-visual\">" + أيقونة(الرمز، 44، "var(--primary)") + "</div><div class=\"web-gallery-body\"><span class=\"web-gallery-tag\">" + _تنظيف_نص_واجهة(الوسم) + "</span><h3>" + _تنظيف_نص_واجهة(العنوان) + "</h3><p>" + _تنظيف_نص_واجهة(الوصف) + "</p></div></article>"
        }
        دع كود = "<section id=\"gallery\" class=\"web-gallery\">" + محتوى + "</section>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_قسم(عنوان، محتوى_نصي، اسم_الأيقونة = "معلومات") {
        دع رمز = أيقونة(اسم_الأيقونة، 24، "var(--primary)")
        دع كود = "<section class=\"web-section\"><h2>" + رمز + " " + _تنظيف_نص_واجهة(عنوان) + "</h2><p>" + _تنظيف_نص_واجهة(محتوى_نصي) + "</p></section>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_نموذج_تواصل(عنوان = "تواصل معنا", وصف = "أرسل لنا رسالتك وسنرد عليك بأسرع وقت", نص_الزر = "إرسال الرسالة") {
        دع كود = "<section id=\"contact\" class=\"web-form-box\"><h2>" + _تنظيف_نص_واجهة(عنوان) + "</h2><p>" + _تنظيف_نص_واجهة(وصف) + "</p><form onsubmit=\"handleContact(event)\"><div class=\"form-group\"><label>الاسم الكريم</label><input type=\"text\" required placeholder=\"اكتب اسمك هنا...\" class=\"lsn-input\"></div><div class=\"form-group\"><label>البريد الإلكتروني</label><input type=\"email\" required placeholder=\"name@example.com\" class=\"lsn-input\"></div><div class=\"form-group\"><label>الرسالة</label><textarea required rows=\"4\" placeholder=\"اكتب استفسارك هنا...\" class=\"lsn-input\"></textarea></div><button type=\"submit\" class=\"lsn-btn full-btn\">" + أيقونة("رسالة"، 18، "#fff") + " <span>" + _تنظيف_نص_واجهة(نص_الزر) + "</span></button></form></section>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_تذييل(نص_الحقوق = "جميع الحقوق محفوظة © لغة لسان") {
        دع كود = "<footer class=\"web-footer\"><p>" + _تنظيف_نص_واجهة(نص_الحقوق) + "</p><div class=\"footer-badge\">" + أيقونة("درع"، 16، "var(--primary)") + " <span>صُنع بلغة البرمجة العربية لسان</span></div></footer>"
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة أضف_عنصر(كود_html) {
        هذا.الأقسام = أضف(هذا.الأقسام، كود_html)
        أرجع كود_html
    }

    دالة بناء_html() {
        أرجع بناء_صفحة_ويب(هذا.عنوان_الموقع، هذا.النسق، هذا.الأقسام)
    }

    دالة حفظ(مسار_الملف) {
        دع كود = هذا.بناء_html()
        اكتب_ملف(مسار_الملف، كود)
        أرجع مسار_الملف
    }

    دالة تصدير(مجلد_الموقع) {
        دع كود = هذا.بناء_html()
        أرجع تصدير_صفحة_ويب(هذا.عنوان_الموقع، كود، مجلد_الموقع)
    }
}
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_catalog_expanded_and_valid_svg() {
        assert!(ICON_NAMES.len() >= 66);
        for name in ICON_NAMES {
            let svg = get_icon_svg(name, 24, "#091d37");
            assert!(svg.starts_with("<svg"), "icon '{}' must produce svg", name);
            assert!(svg.contains("</svg>"), "icon '{}' must close svg", name);
        }
        // تحقق من تهريب لون الأيقونة ضد الحقن
        let safe = get_icon_svg("صاروخ", 20, "\" onclick=\"alert(1)");
        assert!(!safe.contains("\" onclick="));
        assert!(safe.contains("&quot;"));
    }

    #[test]
    fn test_mobile_app_html_screens_fab_and_sheet() {
        let items = vec![
            r#"<section class="lsn-screen active" id="screen-home">محتوى الرئيسية</section>"#.to_string(),
            "<!--LSN_TAB:home|الرئيسية|رئيسية-->".to_string(),
            r#"<section class="lsn-screen" id="screen-shop">محتوى المتجر</section>"#.to_string(),
            "<!--LSN_TAB:shop|المتجر|متجر-->".to_string(),
            r#"<!--LSN_FAB--><button class="lsn-fab">إضافة</button>"#.to_string(),
            r#"<!--LSN_SHEET--><div class="lsn-sheet-backdrop" id="sheet-add">نافذة</div>"#.to_string(),
        ];
        let html = build_mobile_app_html("تطبيق شامل", "لسان", "صاروخ", &items, &[]);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("selectScreen(this, 'home'"));
        assert!(html.contains("selectScreen(this, 'shop'"));
        assert!(html.contains("toggleMobileTheme()"));
        assert!(html.contains("lsnSaveState"));
        assert!(html.contains("lsn-fab"));
        assert!(html.contains("lsn-sheet-backdrop"));
        assert!(!html.contains("<!--LSN_TAB:"));
    }

    #[test]
    fn test_web_page_html_escapes_title_and_includes_v2_styles() {
        let html = build_web_page_html("موقع <script>", "لسان", &["<div>مرحبا</div>".to_string()]);
        assert!(html.contains("<title>موقع &lt;script&gt;</title>"));
        assert!(html.contains(".web-pricing"));
        assert!(html.contains(".web-faq-item"));
        assert!(html.contains(".web-table"));
        assert!(html.contains(".web-testimonials"));
        assert!(html.contains(".web-gallery"));
        assert!(html.contains(".web-banner"));
    }

    #[test]
    fn test_stdlib_ui_and_mobile_parses_cleanly() {
        let tokens = crate::lexer::tokenize(STDLIB_UI_AND_MOBILE)
            .expect("STDLIB_UI_AND_MOBILE must tokenize");
        let stmts = crate::parser::parse(tokens)
            .expect("STDLIB_UI_AND_MOBILE must parse");
        assert!(stmts.len() >= 4);
    }
}
