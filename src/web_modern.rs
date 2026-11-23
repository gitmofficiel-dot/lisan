//! الويب التفاعلي الحديث، مقابس الويب الحيّة (RFC 6455 WebSocket)،
//! وتصدير التطبيقات الأصلية (Capacitor للأندرويد وiOS، وTauri لسطح المكتب، ورموز التصميم W3C / Figma).
//!
//! مبني بالكامل بمكتبة Rust القياسية (`std`) فقط دون أي حزم خارجية،
//! مع توافق كامل مع البناء الأصلي (Native) ومتصفح WebAssembly (`wasm32-unknown-unknown`).

use crate::runtime::Value;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

/// الثابت السحري القياسي لبروتوكول مقابس الويب RFC 6455
const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// الحد الأقصى لحجم حمولة إطار WebSocket المقبول في الفك (16 ميغابايت) لحماية الذاكرة
const MAX_WS_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;

// ============================================================================
// 1. خوارزمية SHA-1 (RFC 3174) وترميز Base64 (RFC 4648) النقيّان بلا اعتمادات
// ============================================================================

/// حساب بصمة SHA-1 القياسية (RFC 3174) بدقة 160 بت (20 بايت)
fn sha1_digest(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [
        0x6745_2301,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(data.len() + 72);
    padded.extend_from_slice(data);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0x00);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut w = [0u32; 80];
    for chunk in padded.chunks_exact(64) {
        for (i, word) in w.iter_mut().take(16).enumerate() {
            let b = &chunk[i * 4..i * 4 + 4];
            *word = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6u32),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }

    let mut out = [0u8; 20];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// ترميز مصفوفة بايتات إلى Base64 القياسي (RFC 4648)
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((triple >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(triple & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}

// ============================================================================
// 2. بروتوكول WebSocket (RFC 6455): مفتاح القبول، المصافحة، وترميز/فك الإطارات
// ============================================================================

/// حساب قيمة الترويسة `Sec-WebSocket-Accept` القياسية وفق RFC 6455.
///
/// مثال المتجه القياسي RFC 6455:
/// `"dGhlIHNhbXBsZSBub25jZQ=="` -> `"s3pPLMBiTxaQ9kYGzzhZRbK+xOo="`
pub fn websocket_accept_key(client_key: &str) -> String {
    let trimmed = client_key.trim();
    let mut combined = String::with_capacity(trimmed.len() + WEBSOCKET_GUID.len());
    combined.push_str(trimmed);
    combined.push_str(WEBSOCKET_GUID);
    let digest = sha1_digest(combined.as_bytes());
    base64_encode(&digest)
}

/// بناء استجابة المصافحة الكاملة `HTTP/1.1 101 Switching Protocols` وفق RFC 6455
pub fn build_websocket_handshake_response(client_key: &str) -> String {
    let accept = websocket_accept_key(client_key);
    format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
        accept
    )
}

/// ترميز نص UTF-8 إلى إطار WebSocket نصي غير مقنّع (من الخادم إلى العميل) وفق RFC 6455
pub fn encode_websocket_text_frame(message: &str) -> Vec<u8> {
    let payload = message.as_bytes();
    let len = payload.len();
    let mut frame = Vec::with_capacity(len + 10);
    // FIN = 1 (0x80) | Opcode Text = 0x1 -> 0x81
    frame.push(0x81);
    if len <= 125 {
        frame.push(len as u8);
    } else if len <= 65_535 {
        frame.push(126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }
    frame.extend_from_slice(payload);
    frame
}

/// ترميز نص UTF-8 إلى إطار WebSocket نصي مقنّع (من العميل إلى الخادم) وفق RFC 6455
pub fn encode_websocket_masked_text_frame(message: &str, mask: [u8; 4]) -> Vec<u8> {
    let payload = message.as_bytes();
    let len = payload.len();
    let mut frame = Vec::with_capacity(len + 14);
    frame.push(0x81);
    if len <= 125 {
        frame.push(0x80 | (len as u8));
    } else if len <= 65_535 {
        frame.push(0x80 | 126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(0x80 | 127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }
    frame.extend_from_slice(&mask);
    for (i, &b) in payload.iter().enumerate() {
        frame.push(b ^ mask[i % 4]);
    }
    frame
}

/// فك إطار WebSocket وفق RFC 6455 (يدعم الإطارات المقنّعة من العميل وغير المقنّعة من الخادم)
pub fn decode_websocket_frame(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() < 2 {
        return Err("إطار مقبس الويب قصير جدًا: يجب أن يحتوي على بايتين على الأقل".into());
    }

    let opcode = bytes[0] & 0x0F;
    if opcode == 0x8 {
        return Err("إطار إغلاق اتصال WebSocket (opcode 0x8)".into());
    }
    if opcode != 0x1 && opcode != 0x0 && opcode != 0x9 && opcode != 0xA {
        return Err(format!(
            "نوع إطار مقبس الويب غير مدعوم كنص (opcode 0x{:X})",
            opcode
        ));
    }

    let masked = (bytes[1] & 0x80) != 0;
    let len_indicator = bytes[1] & 0x7F;
    let mut offset = 2usize;

    let payload_len: usize = match len_indicator {
        0..=125 => len_indicator as usize,
        126 => {
            if bytes.len() < offset + 2 {
                return Err("إطار مقبس الويب مبتور في طول الحمولة الموسّع (16 بت)".into());
            }
            let len = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
            offset += 2;
            len
        }
        127 => {
            if bytes.len() < offset + 8 {
                return Err("إطار مقبس الويب مبتور في طول الحمولة الموسّع (64 بت)".into());
            }
            let len_u64 = u64::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
            offset += 8;
            if len_u64 > MAX_WS_PAYLOAD_BYTES as u64 {
                return Err("حجم حمولة إطار مقبس الويب يتجاوز الحد الآمن المسموح".into());
            }
            len_u64 as usize
        }
        _ => unreachable!(),
    };

    if payload_len > MAX_WS_PAYLOAD_BYTES {
        return Err("حجم حمولة إطار مقبس الويب يتجاوز الحد الآمن المسموح".into());
    }

    let mask_key = if masked {
        if bytes.len() < offset + 4 {
            return Err("إطار مقبس الويب مبتور في مفتاح القناع (Masking Key)".into());
        }
        let key = [
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ];
        offset += 4;
        Some(key)
    } else {
        None
    };

    let end = offset
        .checked_add(payload_len)
        .ok_or_else(|| "تجاوز عددي في طول إطار مقبس الويب".to_string())?;
    if bytes.len() < end {
        return Err(format!(
            "إطار مقبس الويب غير مكتمل: متوقع {} بايت للحمولة، المتوفر {}",
            payload_len,
            bytes.len().saturating_sub(offset)
        ));
    }

    let raw = &bytes[offset..end];
    let decoded = if let Some(mask) = mask_key {
        let mut unmasked = Vec::with_capacity(payload_len);
        for (i, &b) in raw.iter().enumerate() {
            unmasked.push(b ^ mask[i % 4]);
        }
        unmasked
    } else {
        raw.to_vec()
    };

    String::from_utf8(decoded).map_err(|_| "حمولة إطار مقبس الويب ليست نص UTF-8 صالحًا".into())
}

// ============================================================================
// 3. أدوات التصدير (Capacitor للأندرويد وiOS، Tauri لسطح المكتب، وW3C / Figma Tokens)
// ============================================================================

/// تهريب نص JSON بأمان
fn escape_json_str(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
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

/// تحويل اسم أو معرف إلى معرّف حزمة npm آمن بالأحرف اللاتينية الصغيرة
fn sanitize_npm_slug(bundle_id: &str) -> String {
    let slug: String = bundle_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "lisan-app".to_string()
    } else {
        trimmed.to_string()
    }
}

/// توليد ملفات مشروع Capacitor (Android / iOS) في الذاكرة — متوافق مع Native وWASM
pub fn build_capacitor_project_files(
    app_name: &str,
    bundle_id: &str,
    html_content: &str,
) -> Vec<(String, String)> {
    let slug = sanitize_npm_slug(bundle_id);
    let safe_name = escape_json_str(app_name);
    let safe_id = escape_json_str(bundle_id);
    let safe_slug = escape_json_str(&slug);

    let package_json = format!(
        r#"{{
  "name": "{safe_slug}",
  "version": "1.0.0",
  "description": "تطبيق {safe_name} الأصلي المصدّر من لغة البرمجة العربية لسان",
  "private": true,
  "scripts": {{
    "cap:sync": "npx cap sync",
    "cap:android": "npx cap add android && npx cap sync android && npx cap open android",
    "cap:ios": "npx cap add ios && npx cap sync ios && npx cap open ios"
  }},
  "dependencies": {{
    "@capacitor/core": "^6.0.0"
  }},
  "devDependencies": {{
    "@capacitor/cli": "^6.0.0",
    "@capacitor/android": "^6.0.0",
    "@capacitor/ios": "^6.0.0"
  }}
}}
"#
    );

    let capacitor_config = format!(
        r#"{{
  "appId": "{safe_id}",
  "appName": "{safe_name}",
  "webDir": "www",
  "bundledWebRuntime": false,
  "server": {{
    "androidScheme": "https"
  }}
}}
"#
    );

    let manifest_json = format!(
        r##"{{
  "name": "{safe_name}",
  "short_name": "{safe_name}",
  "lang": "ar",
  "dir": "rtl",
  "start_url": "./index.html",
  "display": "standalone",
  "background_color": "#0b1220",
  "theme_color": "#0ea5e9",
  "icons": [
    {{
      "src": "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 128 128'%3E%3Crect width='128' height='128' rx='28' fill='%230ea5e9'/%3E%3Ctext x='64' y='82' font-size='64' text-anchor='middle' fill='white'%3E%D9%84%3C/text%3E%3C/svg%3E",
      "sizes": "192x192",
      "type": "image/svg+xml"
    }}
  ]
}}
"##
    );

    let sw_js = format!(
        r#"// عامل الخدمة (Service Worker) المولّد تلقائيًا من لغة لسان
const CACHE_NAME = 'lisan-cap-{safe_slug}-v1';
const ASSETS = ['./', './index.html', './manifest.json'];

self.addEventListener('install', (event) => {{
  event.waitUntil(caches.open(CACHE_NAME).then((cache) => cache.addAll(ASSETS)));
  self.skipWaiting();
}});

self.addEventListener('activate', (event) => {{
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k)))
    )
  );
  self.clients.claim();
}});

self.addEventListener('fetch', (event) => {{
  event.respondWith(
    caches.match(event.request).then((cached) => cached || fetch(event.request))
  );
}});
"#
    );

    let readme_build = format!(
        r#"# دليل بناء التطبيق الأصلي ({app_name}) عبر Capacitor

تم توليد هذا المشروع بالكامل من لغة البرمجة العربية **لسان (Lisan)**.

- **اسم التطبيق:** `{app_name}`
- **معرّف الحزمة (App ID):** `{bundle_id}`
- **مجلد الواجهة التفاعلية:** `www/`

## خطوات البناء السريع لأندرويد (APK / AAB) وiOS (IPA)

```bash
# 1. تثبيت حزم Capacitor
npm install

# 2. إضافة منصة أندرويد ومزامنة الملفات
npm run cap:android

# 3. أو إضافة منصة iOS (على macOS مع Xcode)
npm run cap:ios
```
"#
    );

    vec![
        ("package.json".to_string(), package_json),
        ("capacitor.config.json".to_string(), capacitor_config),
        ("www/index.html".to_string(), html_content.to_string()),
        ("www/manifest.json".to_string(), manifest_json),
        ("www/sw.js".to_string(), sw_js),
        ("README_BUILD.md".to_string(), readme_build),
    ]
}

/// توليد ملفات مشروع Tauri (تطبيق سطح المكتب الأصلي) في الذاكرة — متوافق مع Native وWASM
pub fn build_tauri_project_files(
    app_name: &str,
    bundle_id: &str,
    html_content: &str,
) -> Vec<(String, String)> {
    let slug = sanitize_npm_slug(bundle_id);
    let safe_name = escape_json_str(app_name);
    let safe_id = escape_json_str(bundle_id);
    let safe_slug = escape_json_str(&slug);

    let tauri_conf = format!(
        r#"{{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "{safe_name}",
  "version": "1.0.0",
  "identifier": "{safe_id}",
  "build": {{
    "frontendDist": "../dist"
  }},
  "app": {{
    "withGlobalTauri": true,
    "windows": [
      {{
        "title": "{safe_name}",
        "width": 1100,
        "height": 740,
        "resizable": true
      }}
    ],
    "security": {{
      "csp": null
    }}
  }},
  "bundle": {{
    "active": true,
    "targets": "all"
  }}
}}
"#
    );

    let package_json = format!(
        r#"{{
  "name": "{safe_slug}",
  "version": "1.0.0",
  "description": "تطبيق سطح مكتب {safe_name} المصدّر من لغة البرمجة العربية لسان",
  "private": true,
  "scripts": {{
    "dev": "tauri dev",
    "build": "tauri build"
  }},
  "devDependencies": {{
    "@tauri-apps/cli": "^2.0.0"
  }}
}}
"#
    );

    let readme_build = format!(
        r#"# دليل بناء تطبيق سطح المكتب ({app_name}) عبر Tauri

تم توليد هذا المشروع بالكامل من لغة البرمجة العربية **لسان (Lisan)**.

- **اسم التطبيق:** `{app_name}`
- **معرّف الحزمة (Identifier):** `{bundle_id}`
- **مجلد الواجهة:** `dist/index.html`

## خطوات التشغيل والبناء الأصلي (Linux / macOS / Windows)

```bash
# 1. تثبيت أداة Tauri CLI
npm install

# 2. تهيئة نواة Rust إن لم تكن موجودة وتشغيل وضع التطوير
npm run dev

# 3. بناء حزمة التثبيت النهائية (AppImage / deb / dmg / msi)
npm run build
```
"#
    );

    vec![
        ("src-tauri/tauri.conf.json".to_string(), tauri_conf),
        ("dist/index.html".to_string(), html_content.to_string()),
        ("package.json".to_string(), package_json),
        ("README_BUILD.md".to_string(), readme_build),
    ]
}

/// توليد رموز التصميم القياسية W3C Design Tokens / Figma Tokens Studio بصيغة JSON صالحة
pub fn export_figma_tokens(theme_name: &str, colors: &[(String, String)]) -> String {
    let default_colors: Vec<(String, String)> = vec![
        ("أساسي".into(), "#0ea5e9".into()),
        ("ثانوي".into(), "#6366f1".into()),
        ("خلفية".into(), "#0b1220".into()),
        ("سطح".into(), "#111827".into()),
        ("نص".into(), "#f8fafc".into()),
        ("نجاح".into(), "#10b981".into()),
        ("تحذير".into(), "#f59e0b".into()),
        ("تنبيه".into(), "#ef4444".into()),
    ];
    let palette = if colors.is_empty() {
        &default_colors
    } else {
        colors
    };

    let mut color_entries = Vec::with_capacity(palette.len());
    for (k, v) in palette {
        color_entries.push(format!(
            "    \"{}\": {{ \"$value\": \"{}\", \"$type\": \"color\" }}",
            escape_json_str(k),
            escape_json_str(v)
        ));
    }

    let safe_theme = escape_json_str(theme_name);
    format!(
        r#"{{
  "$metadata": {{
    "generator": "Lisan Design Tokens Exporter",
    "theme": "{safe_theme}",
    "direction": "rtl",
    "language": "ar",
    "tokenSetOrder": ["{safe_theme}"]
  }},
  "{safe_theme}": {{
    "color": {{
{colors_json}
    }},
    "typography": {{
      "fontFamily": {{
        "arabic": {{ "$value": "Cairo, Tajawal, system-ui, sans-serif", "$type": "fontFamily" }}
      }},
      "fontSize": {{
        "sm": {{ "$value": "14px", "$type": "dimension" }},
        "base": {{ "$value": "16px", "$type": "dimension" }},
        "lg": {{ "$value": "20px", "$type": "dimension" }},
        "xl": {{ "$value": "28px", "$type": "dimension" }}
      }}
    }},
    "spacing": {{
      "xs": {{ "$value": "4px", "$type": "dimension" }},
      "sm": {{ "$value": "8px", "$type": "dimension" }},
      "md": {{ "$value": "16px", "$type": "dimension" }},
      "lg": {{ "$value": "24px", "$type": "dimension" }}
    }},
    "borderRadius": {{
      "sm": {{ "$value": "8px", "$type": "dimension" }},
      "md": {{ "$value": "14px", "$type": "dimension" }},
      "lg": {{ "$value": "20px", "$type": "dimension" }},
      "pill": {{ "$value": "9999px", "$type": "dimension" }}
    }}
  }}
}}"#,
        safe_theme = safe_theme,
        colors_json = color_entries.join(",\n")
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn write_project_files(
    output_dir: &str,
    bundle_id: &str,
    files: &[(String, String)],
) -> Result<Value, String> {
    let base = Path::new(output_dir);
    if output_dir.trim().is_empty() {
        return Err("مسار مجلد التصدير لا يمكن أن يكون فارغًا".into());
    }
    std::fs::create_dir_all(base)
        .map_err(|e| format!("تعذّر إنشاء مجلد التصدير '{}': {}", output_dir, e))?;

    let mut written = Vec::with_capacity(files.len());
    for (rel_path, content) in files {
        let full = base.join(rel_path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("تعذّر إنشاء المجلد الفرعي '{}': {}", parent.display(), e))?;
        }
        std::fs::write(&full, content)
            .map_err(|e| format!("تعذّر كتابة الملف '{}': {}", full.display(), e))?;
        written.push(Value::Text(rel_path.clone()));
    }

    Ok(Value::map(vec![
        ("نجاح".to_string(), Value::Bool(true)),
        ("المجلد".to_string(), Value::Text(output_dir.to_string())),
        ("المعرف".to_string(), Value::Text(bundle_id.to_string())),
        ("عدد_الملفات".to_string(), Value::Number(written.len() as f64)),
        ("ملفات".to_string(), Value::list(written)),
    ]))
}

/// تصدير مشروع Capacitor متكامل (Android / iOS) إلى القرص (البيئة الأصلية)
#[cfg(not(target_arch = "wasm32"))]
pub fn export_capacitor_project(
    app_name: &str,
    bundle_id: &str,
    html_content: &str,
    output_dir: &str,
) -> Result<Value, String> {
    let files = build_capacitor_project_files(app_name, bundle_id, html_content);
    write_project_files(output_dir, bundle_id, &files)
}

/// نسخة بديلة على WebAssembly لضمان التوافق عند البناء لهدف `wasm32-unknown-unknown`
#[cfg(target_arch = "wasm32")]
pub fn export_capacitor_project(
    _app_name: &str,
    _bundle_id: &str,
    _html_content: &str,
    _output_dir: &str,
) -> Result<Value, String> {
    Err("تصدير مشروع كاباسيتور إلى القرص غير متاح في بيئة المتصفح (WebAssembly)".into())
}

/// تصدير مشروع Tauri متكامل (سطح المكتب) إلى القرص (البيئة الأصلية)
#[cfg(not(target_arch = "wasm32"))]
pub fn export_tauri_project(
    app_name: &str,
    bundle_id: &str,
    html_content: &str,
    output_dir: &str,
) -> Result<Value, String> {
    let files = build_tauri_project_files(app_name, bundle_id, html_content);
    write_project_files(output_dir, bundle_id, &files)
}

/// نسخة بديلة على WebAssembly لضمان التوافق عند البناء لهدف `wasm32-unknown-unknown`
#[cfg(target_arch = "wasm32")]
pub fn export_tauri_project(
    _app_name: &str,
    _bundle_id: &str,
    _html_content: &str,
    _output_dir: &str,
) -> Result<Value, String> {
    Err("تصدير مشروع تاوري إلى القرص غير متاح في بيئة المتصفح (WebAssembly)".into())
}

// ============================================================================
// 4. ربط الدوال العربية المدمجة مع بيئة تشغيل لسان (Runtime Builtin Hooks)
// ============================================================================

/// جميع أسماء الدوال العربية الخالصة المدمجة في وحدة الويب الحديث
pub const ALL_WEB_MODERN_BUILTINS: &[&str] = &[
    "مفتاح_مقبس_حي",
    "مصافحة_مقبس_حي",
    "تصدير_كاباسيتور",
    "تصدير_تاوري",
    "تصدير_فيجما",
];

/// التحقق مما إذا كان الاسم يشير إلى دالة مدمجة في وحدة الويب الحديث
pub fn is_web_modern_builtin(name: &str) -> bool {
    ALL_WEB_MODERN_BUILTINS.contains(&name)
}

fn expect_text<'a>(fn_name: &str, val: &'a Value) -> Result<&'a str, String> {
    match val {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("الدالة '{}' تتطلب نصًا", fn_name)),
    }
}

fn extract_color_pairs(fn_name: &str, val: &Value) -> Result<Vec<(String, String)>, String> {
    match val {
        Value::Map(m) => {
            let borrowed = m.borrow();
            Ok(borrowed
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect())
        }
        _ => Err(format!(
            "الدالة '{}' تتطلب قاموس ألوان في المعامل الثاني",
            fn_name
        )),
    }
}

/// تنفيذ دالة مدمجة من وحدة الويب الحديث
pub fn eval_web_modern_builtin(name: &str, args: Vec<Value>) -> Result<Value, String> {
    match name {
        "مفتاح_مقبس_حي" => {
            if args.len() != 1 {
                return Err(format!(
                    "الدالة '{}' تتوقع معاملًا واحدًا (مفتاح_العميل)، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let key = expect_text(name, &args[0])?;
            Ok(Value::Text(websocket_accept_key(key)))
        }
        "مصافحة_مقبس_حي" => {
            if args.len() != 1 {
                return Err(format!(
                    "الدالة '{}' تتوقع معاملًا واحدًا (مفتاح_العميل)، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let key = expect_text(name, &args[0])?;
            Ok(Value::Text(build_websocket_handshake_response(key)))
        }
        "تصدير_كاباسيتور" => {
            if args.len() != 4 {
                return Err(format!(
                    "الدالة '{}' تتوقع 4 معاملات (اسم_التطبيق، معرف_الحزمة، محتوى_صفحة، مسار_المجلد)، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let app_name = expect_text(name, &args[0])?;
            let bundle_id = expect_text(name, &args[1])?;
            let html = expect_text(name, &args[2])?;
            let dir = expect_text(name, &args[3])?;
            export_capacitor_project(app_name, bundle_id, html, dir)
        }
        "تصدير_تاوري" => {
            if args.len() != 4 {
                return Err(format!(
                    "الدالة '{}' تتوقع 4 معاملات (اسم_التطبيق، معرف_الحزمة، محتوى_صفحة، مسار_المجلد)، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let app_name = expect_text(name, &args[0])?;
            let bundle_id = expect_text(name, &args[1])?;
            let html = expect_text(name, &args[2])?;
            let dir = expect_text(name, &args[3])?;
            export_tauri_project(app_name, bundle_id, html, dir)
        }
        "تصدير_فيجما" => {
            if args.is_empty() || args.len() > 2 {
                return Err(format!(
                    "الدالة '{}' تتوقع معاملًا واحدًا أو معاملين (اسم_النسق، [ألوان_النسق])، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let theme_name = expect_text(name, &args[0])?;
            let colors = if args.len() == 2 {
                extract_color_pairs(name, &args[1])?
            } else {
                Vec::new()
            };
            Ok(Value::Text(export_figma_tokens(theme_name, &colors)))
        }
        _ => Err(format!("دالة الويب الحديث غير معروفة: '{}'", name)),
    }
}

// ============================================================================
// 5. المكتبة القياسية العربية الخالصة 100% للويب التفاعلي ومقابس الويب الحيّة
// ============================================================================

/// المكتبة القياسية العربية المدمجة لبناء واجهات تفاعلية وتطبيقات أصلية وقنوات مقابس حيّة
pub const STDLIB_WEB_MODERN: &str = r##"
# مكتبة الويب التفاعلي الحديث وتصدير التطبيقات الأصلية في لغة لسان (عربية خالصة 100%)

دالة _وسم_آمن(الوسم) {
    دع منظف = أحرف_صغيرة(نظّف(نص(الوسم)))
    إذا (منظف == "عنوان") { أرجع "h2" }
    إذا (منظف == "فقرة") { أرجع "p" }
    إذا (منظف == "بطاقة") { أرجع "div" }
    إذا (منظف == "قسم") { أرجع "section" }
    دع مسموح = [
        "div"، "p"، "h1"، "h2"، "h3"، "h4"، "span"، "section"،
        "article"، "header"، "footer"، "ul"، "ol"، "li"، "strong"،
        "em"، "small"، "code"، "pre"، "blockquote"، "label"
    ]
    إذا (منظف في مسموح) {
        أرجع منظف
    }
    أرجع "div"
}

دالة _تهريب_قيمة_جافاسكربت(قيمة) {
    دع س = تنظيف_صفحة(نص(قيمة))
    س = استبدل(س، "\\"، "\\\\")
    س = استبدل(س، "'"، "\\'")
    أرجع س
}

صنف مكون_تفاعلي {
    دالة بناء(اسم) {
        هذا.اسم = نص(اسم)
        هذا.القيم = {}
        هذا.العناصر = []
    }

    دالة أضف_حالة(مفتاح، قيمة) {
        دع جدول = هذا.القيم
        جدول[نص(مفتاح)] = قيمة
        هذا.القيم = جدول
        أرجع هذا
    }

    دالة أضف_عنصر(وسم، محتوى، خصائص = {}) {
        دع وسم_نظيف = _وسم_آمن(وسم)
        دع سمات_نص = ""
        إذا (نوع(خصائص) == "قاموس") {
            دع أسماء_السمات = مفاتيح(خصائص)
            لكل مفتاح_سمة في أسماء_السمات {
                دع اسم_منظف = أحرف_صغيرة(نظّف(نص(مفتاح_سمة)))
                دع قيمة_سمة = نص(خصائص[مفتاح_سمة])
                إذا (ليس يبدأ_ب(اسم_منظف، "on") و ليس يبدأ_ب(أحرف_صغيرة(نظّف(قيمة_سمة))، "javascript:")) {
                    سمات_نص = سمات_نص + " " + تنظيف_صفحة(اسم_منظف) + "=\"" + تنظيف_صفحة(قيمة_سمة) + "\""
                }
            }
        }
        دع محتوى_آمن = تنظيف_صفحة(نص(محتوى))
        دع قطعة = "<" + وسم_نظيف + سمات_نص + ">" + محتوى_آمن + "</" + وسم_نظيف + ">"
        هذا.العناصر = أضف(هذا.العناصر، قطعة)
        أرجع هذا
    }

    دالة أضف_زر(تسمية، فعل_تفاعلي) {
        دع نص_آمن = تنظيف_صفحة(نص(تسمية))
        دع فعل_آمن = _تهريب_قيمة_جافاسكربت(فعل_تفاعلي)
        دع قطعة = "<button type=\"button\" class=\"lsn-r-btn\" data-lsn-action=\"" + تنظيف_صفحة(نص(فعل_تفاعلي)) + "\" onclick=\"lsnDispatch('" + فعل_آمن + "')\">" + نص_آمن + "</button>"
        هذا.العناصر = أضف(هذا.العناصر، قطعة)
        أرجع هذا
    }

    دالة أضف_إدخال(مفتاح، تلميح = "") {
        دع مفتاح_نص = نص(مفتاح)
        دع مفتاح_آمن = تنظيف_صفحة(مفتاح_نص)
        دع مفتاح_حدث = _تهريب_قيمة_جافاسكربت(مفتاح_نص)
        دع تلميح_آمن = تنظيف_صفحة(نص(تلميح))
        دع قطعة = "<div class=\"lsn-r-field\"><label class=\"lsn-r-label\">" + مفتاح_آمن + "</label><input type=\"text\" class=\"lsn-r-input\" data-lsn-model=\"" + مفتاح_آمن + "\" placeholder=\"" + تلميح_آمن + "\" oninput=\"lsnSetState('" + مفتاح_حدث + "', this.value)\"></div>"
        هذا.العناصر = أضف(هذا.العناصر، قطعة)
        أرجع هذا
    }

    دالة عرض_صفحة() {
        دع داخل = ""
        لكل عنصر في هذا.العناصر {
            داخل = داخل + عنصر + "\n"
        }
        أرجع "<section class=\"lsn-component\" data-component=\"" + تنظيف_صفحة(هذا.اسم) + "\"><h2 class=\"lsn-comp-title\">" + تنظيف_صفحة(هذا.اسم) + "</h2><div class=\"lsn-comp-body\">" + داخل + "</div></section>"
    }
}

صنف تطبيق_تفاعلي {
    دالة بناء(عنوان، اسم_النسق = "لسان_حديث") {
        هذا.العنوان = نص(عنوان)
        هذا.اسم_النسق = نص(اسم_النسق)
        هذا.المكونات = []
        هذا.البيانات = {}
        هذا.الألوان = {
            "أساسي": "#0ea5e9"،
            "ثانوي": "#6366f1"،
            "خلفية": "#0b1220"،
            "سطح": "#111827"،
            "نص": "#f8fafc"،
            "نجاح": "#10b981"
        }
    }

    دالة أضف_لون(مفتاح، قيمة) {
        دع ألوان = هذا.الألوان
        ألوان[نص(مفتاح)] = نص(قيمة)
        هذا.الألوان = ألوان
        أرجع هذا
    }

    دالة أضف_حالة(مفتاح، قيمة) {
        دع بيانات = هذا.البيانات
        بيانات[نص(مفتاح)] = قيمة
        هذا.البيانات = بيانات
        أرجع هذا
    }

    دالة أضف_مكون(مكون) {
        هذا.المكونات = أضف(هذا.المكونات، مكون)
        دع مفاتيح_مكون = مفاتيح(مكون.القيم)
        دع بيانات = هذا.البيانات
        لكل مفتاح في مفاتيح_مكون {
            بيانات[مفتاح] = مكون.القيم[مفتاح]
        }
        هذا.البيانات = بيانات
        أرجع هذا
    }

    دالة بناء_تطبيق() {
        دع عنوان_آمن = تنظيف_صفحة(هذا.العنوان)
        دع أزرار_التبويب = ""
        دع لوحات_التبويب = ""
        دع فهرس = 0
        لكل مكون في هذا.المكونات {
            دع نشط = ""
            إذا (فهرس == 0) {
                نشط = " active"
            }
            دع اسم_آمن = تنظيف_صفحة(مكون.اسم)
            أزرار_التبويب = أزرار_التبويب + "<button type=\"button\" class=\"lsn-tab" + نشط + "\" data-tab-index=\"" + نص(فهرس) + "\" onclick=\"lsnSelectTab(" + نص(فهرس) + ")\">" + اسم_آمن + "</button>"
            لوحات_التبويب = لوحات_التبويب + "<div class=\"lsn-pane" + نشط + "\" data-pane-index=\"" + نص(فهرس) + "\">" + مكون.عرض_صفحة() + "</div>"
            فهرس = فهرس + 1
        }

        دع بيانات_أولية = ترميز_جيسون(هذا.البيانات)
        دع لون_أساسي = تنظيف_صفحة(هذا.الألوان["أساسي"])
        دع لون_ثانوي = تنظيف_صفحة(هذا.الألوان["ثانوي"])
        دع لون_خلفية = تنظيف_صفحة(هذا.الألوان["خلفية"])
        دع لون_سطح = تنظيف_صفحة(هذا.الألوان["سطح"])
        دع لون_نص = تنظيف_صفحة(هذا.الألوان["نص"])

        دع النمط = "<style>:root{--primary:" + لون_أساسي + ";--secondary:" + لون_ثانوي + ";--bg:" + لون_خلفية + ";--surface:" + لون_سطح + ";--text:" + لون_نص + ";--border:rgba(148,163,184,0.22)}body.light{--bg:#f8fafc;--surface:#ffffff;--text:#0f172a;--border:rgba(15,23,42,0.12)}*{box-sizing:border-box}body{margin:0;font-family:Cairo,Tajawal,system-ui,sans-serif;background:var(--bg);color:var(--text);transition:background .25s,color .25s}.lsn-shell{max-width:980px;margin:0 auto;padding:20px}.lsn-header{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:16px 20px;border-radius:16px;background:var(--surface);border:1px solid var(--border);margin-bottom:16px}.lsn-tabs{display:flex;flex-wrap:wrap;gap:8px;margin-bottom:16px}.lsn-tab{padding:10px 16px;border-radius:12px;border:1px solid var(--border);background:var(--surface);color:var(--text);cursor:pointer;font-weight:600}.lsn-tab.active{background:linear-gradient(135deg,var(--primary),var(--secondary));color:#fff;border-color:transparent}.lsn-pane{display:none}.lsn-pane.active{display:block}.lsn-component{background:var(--surface);border:1px solid var(--border);border-radius:18px;padding:20px;margin-bottom:16px}.lsn-r-btn{background:linear-gradient(135deg,var(--primary),var(--secondary));color:#fff;border:none;border-radius:12px;padding:10px 18px;cursor:pointer;font-weight:700;margin:6px 0}.lsn-r-field{display:flex;flex-direction:column;gap:6px;margin:10px 0}.lsn-r-input{padding:10px 14px;border-radius:10px;border:1px solid var(--border);background:var(--bg);color:var(--text)}.lsn-state-bar{font-size:13px;opacity:.85;padding:10px 14px;border-radius:12px;background:var(--surface);border:1px solid var(--border);margin-top:12px}</style>"

        دع السكربت = "<script>window.lsnState=" + بيانات_أولية + ";function lsnRender(){document.querySelectorAll('[data-lsn-bind]').forEach(function(el){var k=el.getAttribute('data-lsn-bind');if(k in window.lsnState){el.textContent=String(window.lsnState[k]);}});document.querySelectorAll('[data-lsn-model]').forEach(function(el){var k=el.getAttribute('data-lsn-model');if(k in window.lsnState&&el.value!==String(window.lsnState[k])){el.value=String(window.lsnState[k]);}});var preview=document.getElementById('lsn-state-preview');if(preview){preview.textContent=JSON.stringify(window.lsnState);}}function lsnSetState(k,v){window.lsnState[k]=v;lsnRender();}function lsnDispatch(action){window.lsnState['آخر_فعل']=action;if(action==='زيادة'){window.lsnState['عداد']=(Number(window.lsnState['عداد']||0)+1);}if(action==='إنقاص'){window.lsnState['عداد']=(Number(window.lsnState['عداد']||0)-1);}lsnRender();}function lsnToggleTheme(){document.body.classList.toggle('light');}function lsnSelectTab(idx){document.querySelectorAll('.lsn-tab').forEach(function(b){b.classList.toggle('active',Number(b.getAttribute('data-tab-index'))===idx);});document.querySelectorAll('.lsn-pane').forEach(function(p){p.classList.toggle('active',Number(p.getAttribute('data-pane-index'))===idx);});}document.addEventListener('DOMContentLoaded',lsnRender);</script>"

        أرجع "<!DOCTYPE html><html lang=\"ar\" dir=\"rtl\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>" + عنوان_آمن + "</title>" + النمط + "</head><body><main class=\"lsn-shell\"><header class=\"lsn-header\"><h1>" + عنوان_آمن + "</h1><button type=\"button\" class=\"lsn-r-btn\" onclick=\"lsnToggleTheme()\">تبديل المظهر</button></header><nav class=\"lsn-tabs\">" + أزرار_التبويب + "</nav>" + لوحات_التبويب + "<footer class=\"lsn-state-bar\">حالة التطبيق الحية: <code id=\"lsn-state-preview\"></code></footer></main>" + السكربت + "</body></html>"
    }

    دالة تصدير_تطبيق_أصلي(معرف_الحزمة، مجلد) {
        أرجع تصدير_كاباسيتور(هذا.العنوان، معرف_الحزمة، هذا.بناء_تطبيق()، مجلد)
    }

    دالة تصدير_سطح_مكتب(معرف_الحزمة، مجلد) {
        أرجع تصدير_تاوري(هذا.العنوان، معرف_الحزمة، هذا.بناء_تطبيق()، مجلد)
    }

    دالة تصدير_رموز_فيجما() {
        أرجع تصدير_فيجما(هذا.اسم_النسق، هذا.الألوان)
    }
}

صنف قناة_مقبس_حي {
    دالة بناء(اسم_الغرفة) {
        هذا.اسم_الغرفة = نص(اسم_الغرفة)
        هذا.المشتركون = []
        هذا.الرسائل = []
    }

    دالة مفتاح_قبول(مفتاح_العميل) {
        أرجع مفتاح_مقبس_حي(مفتاح_العميل)
    }

    دالة مصافحة(مفتاح_العميل) {
        أرجع مصافحة_مقبس_حي(مفتاح_العميل)
    }

    دالة انضمام(اسم_المستخدم) {
        دع مستخدم = نص(اسم_المستخدم)
        إذا (مستخدم ليس في هذا.المشتركون) {
            هذا.المشتركون = أضف(هذا.المشتركون، مستخدم)
        }
        أرجع طول(هذا.المشتركون)
    }

    دالة مغادرة(اسم_المستخدم) {
        دع مستخدم = نص(اسم_المستخدم)
        دع قائمة_جديدة = []
        لكل مشترك في هذا.المشتركون {
            إذا (مشترك != مستخدم) {
                قائمة_جديدة = أضف(قائمة_جديدة، مشترك)
            }
        }
        هذا.المشتركون = قائمة_جديدة
        أرجع طول(هذا.المشتركون)
    }

    دالة بث_رسالة(المرسل، المحتوى) {
        دع سجل = {
            "الغرفة": هذا.اسم_الغرفة،
            "المرسل": نص(المرسل)،
            "المحتوى": نص(المحتوى)،
            "عدد_المستلمين": طول(هذا.المشتركون)
        }
        هذا.الرسائل = أضف(هذا.الرسائل، سجل)
        أرجع سجل
    }

    دالة سجل_الرسائل() {
        أرجع هذا.الرسائل
    }
}
"##;

// ============================================================================
// 6. اختبارات الوحدة الشاملة (Unit Tests)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc6455_websocket_accept_key_and_handshake() {
        // المتجه الرسمي المعتمد في RFC 6455 القسم 1.3
        let client_key = "dGhlIHNhbXBsZSBub25jZQ==";
        let accept = websocket_accept_key(client_key);
        assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");

        // عبر دالة بيئة التشغيل العربية المدمجة
        assert!(is_web_modern_builtin("مفتاح_مقبس_حي"));
        assert!(is_web_modern_builtin("مصافحة_مقبس_حي"));
        let val = eval_web_modern_builtin("مفتاح_مقبس_حي", vec![Value::Text(client_key.into())])
            .expect("مفتاح_مقبس_حي must succeed");
        assert_eq!(val, Value::Text("s3pPLMBiTxaQ9kYGzzhZRbK+xOo=".into()));

        let hs = eval_web_modern_builtin("مصافحة_مقبس_حي", vec![Value::Text(client_key.into())])
            .expect("مصافحة_مقبس_حي must succeed");
        let hs_text = match hs {
            Value::Text(s) => s,
            other => panic!("Expected Text handshake, got {:?}", other),
        };
        assert!(hs_text.starts_with("HTTP/1.1 101 Switching Protocols\r\n"));
        assert!(hs_text.contains("Upgrade: websocket\r\n"));
        assert!(hs_text.contains("Connection: Upgrade\r\n"));
        assert!(hs_text.contains("Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n"));
    }

    #[test]
    fn test_websocket_frame_roundtrip_unmasked_and_masked_arabic_utf8() {
        let short_msg = "مرحبًا بك في مقبس لسان الحي! 🚀";
        let unmasked = encode_websocket_text_frame(short_msg);
        assert_eq!(unmasked[0], 0x81);
        assert_eq!(decode_websocket_frame(&unmasked).unwrap(), short_msg);

        let mask = [0x37, 0xFA, 0x21, 0x3D];
        let masked = encode_websocket_masked_text_frame(short_msg, mask);
        assert_eq!(masked[0], 0x81);
        assert_ne!(masked[1] & 0x80, 0);
        assert_eq!(decode_websocket_frame(&masked).unwrap(), short_msg);

        // رسالة أطول من 125 بايت لاختبار ترميز الطول الموسّع 16 بت (126)
        let long_msg = "لغة البرمجة العربية لسان تدعم مقابس الويب الحيّة بالكامل — ".repeat(6);
        assert!(long_msg.len() > 125);
        let long_unmasked = encode_websocket_text_frame(&long_msg);
        assert_eq!(long_unmasked[1], 126);
        assert_eq!(decode_websocket_frame(&long_unmasked).unwrap(), long_msg);

        let long_masked = encode_websocket_masked_text_frame(&long_msg, [0xAA, 0x55, 0x10, 0x99]);
        assert_eq!(long_masked[1] & 0x7F, 126);
        assert_eq!(decode_websocket_frame(&long_masked).unwrap(), long_msg);

        // رفض الإطار المبتور أو غير الصالح
        assert!(decode_websocket_frame(&[0x81]).is_err());
        assert!(decode_websocket_frame(&[0x81, 0x05, b'a']).is_err());
    }

    #[test]
    fn test_stdlib_web_modern_tokenizes_parses_pure_arabic_and_renders_html() {
        let tokens = crate::lexer::tokenize(STDLIB_WEB_MODERN)
            .expect("STDLIB_WEB_MODERN must tokenize cleanly");
        let stmts = crate::parser::parse(tokens).expect("STDLIB_WEB_MODERN must parse cleanly");
        assert!(stmts.len() >= 5);

        let lints = crate::diagnostics::lint_arabic_only(STDLIB_WEB_MODERN);
        let english_errors: Vec<_> = lints.iter().filter(|d| d.code == "ل0001").collect();
        assert_eq!(
            english_errors.len(),
            0,
            "STDLIB_WEB_MODERN must have zero ل0001 errors, got: {:?}",
            english_errors
        );
        let legacy_warnings: Vec<_> = lints.iter().filter(|d| d.code == "ل0004").collect();
        assert_eq!(
            legacy_warnings.len(),
            0,
            "STDLIB_WEB_MODERN must have zero ل0004 warnings, got: {:?}",
            legacy_warnings
        );

        let script = format!(
            "{}\n{}",
            STDLIB_WEB_MODERN,
            r##"
دع لوحة = جديد مكون_تفاعلي("لوحة <مشرف>")
لوحة.أضف_حالة("عداد"، 3)
لوحة.أضف_عنصر("عنوان"، "أهلاً بكم <script>alert(1)</script>")
لوحة.أضف_إدخال("الاسم"، "اكتب اسمك")
لوحة.أضف_زر("زيادة العداد"، "زيادة")

دع تطبيق = جديد تطبيق_تفاعلي("تطبيق <لسان> التفاعلي")
تطبيق.أضف_مكون(لوحة)
دع صفحة = تطبيق.بناء_تطبيق()
اطبع(يحتوي(صفحة، "<!DOCTYPE html><html lang=\"ar\" dir=\"rtl\">"))
اطبع(يحتوي(صفحة، "&lt;script&gt;alert(1)&lt;/script&gt;"))
اطبع(ليس يحتوي(صفحة، "<script>alert(1)</script>"))
اطبع(يحتوي(صفحة، "lsnDispatch"))
اطبع(يحتوي(صفحة، "lsnToggleTheme"))
اطبع(يحتوي(صفحة، "lsnSelectTab"))
"##
        );
        let mut interp = crate::runtime::Interpreter::default();
        let out = interp
            .run_source(&script)
            .expect("STDLIB_WEB_MODERN reactive app must execute cleanly");
        assert_eq!(
            out,
            vec!["صحيح", "صحيح", "صحيح", "صحيح", "صحيح", "صحيح"]
        );
    }

    #[test]
    fn test_figma_tokens_export_valid_json() {
        let colors = Value::map(vec![
            ("أساسي".into(), Value::Text("#0ea5e9".into())),
            ("خلفية".into(), Value::Text("#0b1220".into())),
        ]);
        let res = eval_web_modern_builtin(
            "تصدير_فيجما",
            vec![Value::Text("نسق_لسان".into()), colors],
        )
        .expect("تصدير_فيجما must succeed");
        let json_str = match res {
            Value::Text(s) => s,
            other => panic!("Expected JSON text, got {:?}", other),
        };
        let parsed = crate::json::parse_json(&json_str).expect("Figma tokens must be valid JSON");
        assert!(json_str.contains("\"$value\": \"#0ea5e9\""));
        assert!(json_str.contains("\"$type\": \"color\""));
        assert!(json_str.contains("\"direction\": \"rtl\""));
        assert!(matches!(parsed, Value::Map(_)));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_capacitor_and_tauri_export_to_temp_dir_and_cleanup() {
        let uniq = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let cap_dir = std::env::temp_dir().join(format!("lisan_cap_test_{}", uniq));
        let tauri_dir = std::env::temp_dir().join(format!("lisan_tauri_test_{}", uniq));
        let html = "<!DOCTYPE html><html lang=\"ar\" dir=\"rtl\"><body><h1>تطبيق لسان</h1></body></html>";

        let cap_res = eval_web_modern_builtin(
            "تصدير_كاباسيتور",
            vec![
                Value::Text("تطبيق لسان".into()),
                Value::Text("org.lisan.demo".into()),
                Value::Text(html.into()),
                Value::Text(cap_dir.to_string_lossy().to_string()),
            ],
        )
        .expect("تصدير_كاباسيتور must succeed");
        assert!(matches!(cap_res, Value::Map(_)));
        for expected in &[
            "package.json",
            "capacitor.config.json",
            "www/index.html",
            "www/manifest.json",
            "www/sw.js",
            "README_BUILD.md",
        ] {
            let p = cap_dir.join(expected);
            assert!(p.exists(), "Capacitor file missing: {}", p.display());
        }
        let cap_pkg = std::fs::read_to_string(cap_dir.join("package.json")).unwrap();
        crate::json::parse_json(&cap_pkg).expect("Capacitor package.json must be valid JSON");
        let cap_cfg = std::fs::read_to_string(cap_dir.join("capacitor.config.json")).unwrap();
        crate::json::parse_json(&cap_cfg).expect("capacitor.config.json must be valid JSON");
        let cap_manifest = std::fs::read_to_string(cap_dir.join("www/manifest.json")).unwrap();
        crate::json::parse_json(&cap_manifest).expect("manifest.json must be valid JSON");

        let tauri_res = eval_web_modern_builtin(
            "تصدير_تاوري",
            vec![
                Value::Text("سطح مكتب لسان".into()),
                Value::Text("org.lisan.desktop".into()),
                Value::Text(html.into()),
                Value::Text(tauri_dir.to_string_lossy().to_string()),
            ],
        )
        .expect("تصدير_تاوري must succeed");
        assert!(matches!(tauri_res, Value::Map(_)));
        for expected in &[
            "src-tauri/tauri.conf.json",
            "dist/index.html",
            "package.json",
            "README_BUILD.md",
        ] {
            let p = tauri_dir.join(expected);
            assert!(p.exists(), "Tauri file missing: {}", p.display());
        }
        let tauri_cfg =
            std::fs::read_to_string(tauri_dir.join("src-tauri/tauri.conf.json")).unwrap();
        crate::json::parse_json(&tauri_cfg).expect("tauri.conf.json must be valid JSON");
        let tauri_pkg = std::fs::read_to_string(tauri_dir.join("package.json")).unwrap();
        crate::json::parse_json(&tauri_pkg).expect("Tauri package.json must be valid JSON");

        let _ = std::fs::remove_dir_all(&cap_dir);
        let _ = std::fs::remove_dir_all(&tauri_dir);
    }
}
