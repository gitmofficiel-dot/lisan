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
];

/// إرجاع كود SVG المتجه لأي أيقونة عربية مع الحجم واللون المطلوبين
pub fn get_icon_svg(name: &str, size: u32, color: &str) -> String {
    let norm = crate::diagnostics::normalize_arabic(name.trim());
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
        "سله" | "متجر" | "cart" => {
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
        "موقع" | "خريطه" | "location" => {
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
        _ => {
            r#"<circle cx="12" cy="12" r="9"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>"#
        }
    };

    format!(
        r#"<svg class="lsn-icon" xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{path_data}</svg>"#
    )
}

/// إرجاع متغيرات CSS للنسق اللوني المطلوب (داكن، فاتح، زمردي، ملكي، غروب، محيطي)
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
        _ => {
            ":root { --bg: #0f172a; --surface: #1e293b; --surface-2: #334155; --text: #f8fafc; --muted: #94a3b8; --primary: #3b82f6; --accent: #10b981; --border: #334155; }"
        }
    }
}

/// بناء صفحة HTML5 متكاملة لتطبيق الهاتف الذكي (مع إطار هاتف تفاعلي ودعم PWA)
pub fn build_mobile_app_html(
    app_name: &str,
    theme: &str,
    header_icon: &str,
    items_html: &[String],
    tabs: &[(String, String)],
) -> String {
    let theme_vars = get_theme_css(theme);
    let top_icon = get_icon_svg(header_icon, 22, "currentColor");
    let body_content = items_html.join("\n");

    let mut tabs_html = String::new();
    if tabs.is_empty() {
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
                r#"<button class="lsn-tab {active}" onclick="selectTab(this, '{label}')">{svg}<span>{label}</span></button>"#
            ));
        }
    } else {
        for (idx, (label, ic)) in tabs.iter().enumerate() {
            let active = if idx == 0 { "active" } else { "" };
            let svg = get_icon_svg(ic, 20, "currentColor");
            tabs_html.push_str(&format!(
                r#"<button class="lsn-tab {active}" onclick="selectTab(this, '{label}')">{svg}<span>{label}</span></button>"#
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
<title>{app_name}</title>
<style>
{theme_vars}
* {{ box-sizing: border-box; margin: 0; padding: 0; font-family: 'Segoe UI', Tahoma, system-ui, sans-serif; }}
body {{ background: var(--bg); color: var(--text); display: flex; justify-content: center; align-items: center; min-height: 100vh; padding: 16px; }}
.phone-frame {{ width: 100%; max-width: 390px; height: 780px; background: var(--bg); border: 8px solid var(--surface-2); border-radius: 42px; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 25px 60px rgba(0,0,0,0.45); position: relative; }}
.status-bar {{ display: flex; justify-content: space-between; align-items: center; padding: 10px 22px 6px; font-size: 12px; color: var(--muted); background: var(--surface); }}
.app-header {{ display: flex; align-items: center; justify-content: space-between; padding: 14px 20px; background: var(--surface); border-bottom: 1px solid var(--border); }}
.app-header h1 {{ font-size: 18px; font-weight: 700; display: flex; align-items: center; gap: 10px; color: var(--text); }}
.app-content {{ flex: 1; overflow-y: auto; padding: 16px; display: flex; flex-direction: column; gap: 12px; }}
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
.lsn-toast {{ position: absolute; bottom: 76px; right: 20px; left: 20px; background: var(--primary); color: #fff; padding: 10px 14px; border-radius: 12px; font-size: 13px; text-align: center; display: none; z-index: 20; }}
.bottom-nav {{ display: flex; justify-content: space-around; background: var(--surface); border-top: 1px solid var(--border); padding: 8px 6px 14px; }}
.lsn-tab {{ background: none; border: none; color: var(--muted); display: flex; flex-direction: column; align-items: center; gap: 4px; font-size: 11px; cursor: pointer; padding: 4px 10px; border-radius: 10px; }}
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
    <h1>{top_icon} <span>{app_name}</span></h1>
  </div>
  <div class="app-content" id="appContent">
    {body_content}
  </div>
  <div class="lsn-toast" id="toastMsg"></div>
  <nav class="bottom-nav">
    {tabs_html}
  </nav>
</div>
<script>
function showToast(msg) {{
  const t = document.getElementById('toastMsg');
  t.textContent = msg;
  t.style.display = 'block';
  setTimeout(() => {{ t.style.display = 'none'; }}, 2200);
}}
function toggleSwitch(btn, title) {{
  btn.classList.toggle('on');
  const state = btn.classList.contains('on') ? 'مُفعّل' : 'مُعطّل';
  showToast(title + ': ' + state);
}}
function selectTab(btn, name) {{
  document.querySelectorAll('.lsn-tab').forEach(b => b.classList.remove('active'));
  btn.classList.add('active');
  showToast('الانتقال إلى: ' + name);
}}
</script>
</body>
</html>"##
    )
}

/// تصدير تطبيق الهاتف الذكي كحزمة PWA قابلة للتثبيت على الهواتف (Android / iOS)
pub fn export_mobile_pwa(app_name: &str, html: &str, out_dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("تعذر إنشاء مجلد تطبيق الهاتف '{}': {}", out_dir.display(), e))?;

    let index_path = out_dir.join("index.html");
    std::fs::write(&index_path, html)
        .map_err(|e| format!("تعذر حفظ ملف الواجهة '{}': {}", index_path.display(), e))?;

    let manifest = format!(
        r##"{{
  "name": "{app_name}",
  "short_name": "{app_name}",
  "lang": "ar",
  "dir": "rtl",
  "start_url": "./index.html",
  "display": "standalone",
  "background_color": "#0f172a",
  "theme_color": "#3b82f6"
}}"##
    );
    std::fs::write(out_dir.join("manifest.json"), manifest)
        .map_err(|e| format!("تعذر حفظ manifest.json: {}", e))?;

    let sw = r#"self.addEventListener('install', e => self.skipWaiting());
self.addEventListener('fetch', e => e.respondWith(fetch(e.request).catch(() => caches.match(e.request))));
"#;
    std::fs::write(out_dir.join("sw.js"), sw)
        .map_err(|e| format!("تعذر حفظ sw.js: {}", e))?;

    Ok(index_path)
}

/// بناء صفحة موقع ويب متكاملة ومتجاوبة بالعربية
pub fn build_web_page_html(title: &str, theme: &str, sections_html: &[String]) -> String {
    let theme_vars = get_theme_css(theme);
    let body = sections_html.join("\n");
    format!(
        r#"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
{theme_vars}
* {{ box-sizing: border-box; margin: 0; padding: 0; font-family: 'Segoe UI', Tahoma, system-ui, sans-serif; }}
body {{ background: var(--bg); color: var(--text); line-height: 1.6; }}
.container {{ max-width: 1100px; margin: 0 auto; padding: 24px; display: flex; flex-direction: column; gap: 28px; }}
.web-hero {{ background: linear-gradient(135deg, var(--surface), var(--surface-2)); border: 1px solid var(--border); border-radius: 24px; padding: 48px 32px; text-align: center; display: flex; flex-direction: column; align-items: center; gap: 16px; }}
.web-hero h1 {{ font-size: 34px; font-weight: 800; }}
.web-hero p {{ color: var(--muted); max-width: 650px; font-size: 17px; }}
.lsn-card {{ background: var(--surface); border: 1px solid var(--border); border-radius: 18px; padding: 20px; display: flex; align-items: flex-start; gap: 14px; }}
.lsn-card-icon {{ width: 46px; height: 46px; border-radius: 12px; background: var(--surface-2); color: var(--primary); display: flex; align-items: center; justify-content: center; flex-shrink: 0; }}
.lsn-btn {{ padding: 12px 24px; border: none; border-radius: 12px; background: linear-gradient(135deg, var(--primary), var(--accent)); color: #fff; font-size: 15px; font-weight: 700; cursor: pointer; display: inline-flex; align-items: center; gap: 8px; }}
.web-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 16px; }}
</style>
</head>
<body>
<div class="container">
{body}
</div>
</body>
</html>"#
    )
}

/// المكتبة القياسية المدمجة للأيقونات والتصميمات وتطبيقات الهواتف والويب بلغة لسان
pub const STDLIB_UI_AND_MOBILE: &str = r#"
# مكتبة الأيقونات والتصميمات وتطبيقات الهواتف الذكية والويب في لغة لسان

صنف تطبيق_هاتف {
    دالة بناء(اسم_التطبيق، النسق) {
        هذا.اسم_التطبيق = اسم_التطبيق
        هذا.النسق = النسق
        هذا.أيقونة_العنوان = "هاتف"
        هذا.العناصر = []
        هذا.التبويبات = []
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

    دالة أضف_تبويب(عنوان، اسم_الأيقونة) {
        هذا.التبويبات = أضف(هذا.التبويبات، [عنوان، اسم_الأيقونة])
        أرجع طول(هذا.التبويبات)
    }

    دالة بناء_واجهة() {
        أرجع بناء_تطبيق_هاتف(هذا.اسم_التطبيق، هذا.النسق، هذا.أيقونة_العنوان، هذا.العناصر)
    }

    دالة تصدير(مجلد_الإخراج) {
        دع html = هذا.بناء_واجهة()
        أرجع تصدير_تطبيق_هاتف(هذا.اسم_التطبيق، html، مجلد_الإخراج)
    }
}

صنف موقع_ويب {
    دالة بناء(عنوان_الموقع، النسق) {
        هذا.عنوان_الموقع = عنوان_الموقع
        هذا.النسق = النسق
        هذا.الأقسام = []
    }

    دالة قسم_رئيسي(عنوان، وصف، نص_الزر، اسم_الأيقونة) {
        دع رمز = أيقونة(اسم_الأيقونة، 40، "var(--primary)")
        دع زر = عنصر_زر(نص_الزر، "تشغيل"، عنوان)
        دع قسم = "<section class=\"web-hero\">" + رمز + "<h1>" + عنوان + "</h1><p>" + وصف + "</p>" + زر + "</section>"
        هذا.الأقسام = أضف(هذا.الأقسام، قسم)
        أرجع قسم
    }

    دالة أضف_بطاقة(عنوان، وصف، اسم_الأيقونة) {
        دع كود = عنصر_بطاقة(عنوان، وصف، اسم_الأيقونة)
        هذا.الأقسام = أضف(هذا.الأقسام، كود)
        أرجع كود
    }

    دالة بناء_html() {
        أرجع بناء_صفحة_ويب(هذا.عنوان_الموقع، هذا.النسق، هذا.الأقسام)
    }

    دالة حفظ(مسار_الملف) {
        دع كود = هذا.بناء_html()
        اكتب_ملف(مسار_الملف، كود)
        أرجع مسار_الملف
    }
}
"#;
