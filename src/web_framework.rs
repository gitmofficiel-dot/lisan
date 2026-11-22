//! إطار عمل الويب الكامل للغة لسان (محرك قوالب آمن، توجيه مسارات، استجابات HTTP/1.1، عميل وخادم ويب).
//! مبني بالكامل بمكتبة Rust القياسية (`std`) فقط، مع توافق كامل مع بيئة WebAssembly (`wasm32`).

use crate::runtime::Value;

/// جميع أسماء الدوال المدمجة الخاصة بإطار عمل الويب (بالعربية والإنجليزية)
pub const ALL_WEB_BUILTINS: &[&str] = &[
    "عرض_قالب",
    "render_template",
    "تنظيف_html",
    "escape_html",
    "بناء_استجابة_http",
    "build_http_response",
    "تحليل_استعلام_url",
    "parse_query",
    "طلب_http_متقدم",
    "http_request",
    "تشغيل_تطبيق_ويب",
    "run_web_app",
];

/// التحقق مما إذا كان الاسم يشير إلى دالة مدمجة في إطار عمل الويب
pub fn is_web_builtin(name: &str) -> bool {
    ALL_WEB_BUILTINS.contains(&name)
}

fn expect_text<'a>(fn_name: &str, val: &'a Value) -> Result<&'a str, String> {
    match val {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("الدالة '{}' تتطلب نصًا", fn_name)),
    }
}

/// تنفيذ دالة مدمجة من إطار عمل الويب
pub fn eval_web_builtin(name: &str, args: Vec<Value>) -> Result<Value, String> {
    match name {
        "عرض_قالب" | "render_template" => {
            if args.len() != 2 {
                return Err(format!(
                    "الدالة '{}' تتوقع معاملين (قالب_نص، بيانات_قاموس)، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let template = expect_text(name, &args[0])?;
            match &args[1] {
                Value::Map(_) | Value::Instance { .. } => {
                    Ok(Value::Text(render_template(template, &args[1])))
                }
                _ => Err(format!(
                    "الدالة '{}' تتطلب قاموس بيانات أو كائنًا في المعامل الثاني",
                    name
                )),
            }
        }
        "تنظيف_html" | "escape_html" => {
            if args.len() != 1 {
                return Err(format!(
                    "الدالة '{}' تتوقع معاملًا واحدًا (نص)، استلمت {}",
                    name,
                    args.len()
                ));
            }
            Ok(Value::Text(escape_html(&args[0].to_string())))
        }
        "بناء_استجابة_http" | "build_http_response" => {
            if args.len() < 3 || args.len() > 4 {
                return Err(format!(
                    "الدالة '{}' تتوقع 3 أو 4 معاملات (رمز_حالة، نوع_محتوى، جسم، [ترويسات])، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let status = match &args[0] {
                Value::Number(n) if n.is_finite() && *n >= 100.0 && *n <= 599.0 => *n as u16,
                _ => return Err("رمز حالة HTTP يجب أن يكون عددًا بين 100 و 599".into()),
            };
            let content_type = expect_text(name, &args[1])?;
            let body = args[2].to_string();
            let extra_headers = if args.len() == 4 {
                extract_headers_vec(&args[3])?
            } else {
                Vec::new()
            };
            Ok(Value::Text(build_http_response(
                status,
                content_type,
                &body,
                &extra_headers,
            )))
        }
        "تحليل_استعلام_url" | "parse_query" => {
            if args.len() != 1 {
                return Err(format!(
                    "الدالة '{}' تتوقع معاملًا واحدًا (نص_استعلام)، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let query = expect_text(name, &args[0])?;
            Ok(parse_query_to_value(query))
        }
        "طلب_http_متقدم" | "http_request" => {
            if args.len() < 2 || args.len() > 4 {
                return Err(format!(
                    "الدالة '{}' تتوقع 2 إلى 4 معاملات (طريقة، رابط، [ترويسات]، [جسم])، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let method = expect_text(name, &args[0])?;
            let url = expect_text(name, &args[1])?;
            let headers = if args.len() >= 3 {
                extract_headers_vec(&args[2])?
            } else {
                Vec::new()
            };
            let body = if args.len() == 4 {
                args[3].to_string()
            } else {
                String::new()
            };
            http_request_impl(method, url, &headers, &body)
        }
        "تشغيل_تطبيق_ويب" | "run_web_app" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(format!(
                    "الدالة '{}' تتوقع 2 أو 3 معاملات (منفذ، مسارات_قاموس، [حد_الطلبات])، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let port = match &args[0] {
                Value::Number(n) if n.is_finite() && *n >= 1.0 && *n <= 65535.0 => *n as u16,
                _ => return Err("المنفذ يجب أن يكون عددًا صحيحًا بين 1 و 65535".into()),
            };
            let routes = match &args[1] {
                Value::Map(m) => m
                    .borrow()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect::<Vec<_>>(),
                _ => return Err("المسارات يجب أن تكون قاموسًا يربط المسار بمحتوى الرد".into()),
            };
            let req_limit = if args.len() == 3 {
                match &args[2] {
                    Value::Number(n) if n.is_finite() && *n >= 0.0 => *n as usize,
                    _ => return Err("حد الطلبات يجب أن يكون عددًا غير سالب".into()),
                }
            } else {
                1
            };
            run_web_app_impl(port, &routes, req_limit)
        }
        _ => Err(format!("دالة إطار الويب غير معروفة: '{}'", name)),
    }
}

fn extract_headers_vec(val: &Value) -> Result<Vec<(String, String)>, String> {
    match val {
        Value::Map(m) => Ok(m
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect()),
        Value::Null => Ok(Vec::new()),
        _ => Err("الترويسات يجب أن تكون قاموسًا (Map)".into()),
    }
}

// ============================================================================
// 1. تهريب HTML ومحرك القوالب الآمن
// ============================================================================

/// تهريب الرموز الخاصة في HTML (`& < > " '`) للحماية من ثغرات XSS
pub fn escape_html(input: &str) -> String {
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

/// استرجاع قيمة متداخلة عبر مسار منقط مثل `مستخدم.اسم` أو `قائمة.0`
fn lookup_dotted_path(root: &Value, path: &str) -> Option<Value> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut current = root.clone();
    for segment in trimmed.split('.') {
        let key = segment.trim();
        if key.is_empty() {
            return None;
        }
        let next = match &current {
            Value::Map(pairs) => pairs
                .borrow()
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone()),
            Value::Instance { fields, .. } => fields
                .borrow()
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone()),
            Value::List(items) => {
                let idx = key.parse::<usize>().ok()?;
                items.borrow().get(idx).cloned()
            }
            Value::Tuple(items) => {
                let idx = key.parse::<usize>().ok()?;
                items.get(idx).cloned()
            }
            _ => None,
        };
        current = next?;
    }
    Some(current)
}

fn value_to_template_text(val: &Value) -> String {
    match val {
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// تصيير قالب نصي يدعم `{{مفتاح}}` (مهرب ضد XSS) و `{{{مفتاح}}}` (خام) مع دعم المسارات المنقطة `{{مستخدم.اسم}}`
pub fn render_template(template: &str, data: &Value) -> String {
    let mut out = String::with_capacity(template.len());
    let mut idx = 0;
    let bytes = template.as_bytes();
    let len = bytes.len();

    while idx < len {
        if template[idx..].starts_with("{{{") {
            if let Some(rel_end) = template[idx + 3..].find("}}}") {
                let expr = &template[idx + 3..idx + 3 + rel_end];
                if let Some(val) = lookup_dotted_path(data, expr) {
                    out.push_str(&value_to_template_text(&val));
                }
                idx += 3 + rel_end + 3;
                continue;
            }
        }
        if template[idx..].starts_with("{{") {
            if let Some(rel_end) = template[idx + 2..].find("}}") {
                let expr = &template[idx + 2..idx + 2 + rel_end];
                if let Some(val) = lookup_dotted_path(data, expr) {
                    out.push_str(&escape_html(&value_to_template_text(&val)));
                }
                idx += 2 + rel_end + 2;
                continue;
            }
        }
        let ch = template[idx..].chars().next().unwrap();
        out.push(ch);
        idx += ch.len_utf8();
    }

    out
}

// ============================================================================
// 2. تحليل استعلامات URL وبناء استجابات HTTP/1.1
// ============================================================================

fn from_hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// فك ترميز نسبة مئوية (Percent-decoding) مع تحويل `+` إلى مسافة
pub fn decode_query_component(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let (Some(hi), Some(lo)) = (from_hex_digit(bytes[i + 1]), from_hex_digit(bytes[i + 2])) {
                    out.push((hi << 4) | lo);
                    i += 3;
                } else {
                    out.push(b'%');
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// تحليل سلسلة استعلام URL مثل `a=1&b=hello` إلى قاموس لسان
pub fn parse_query_to_value(raw: &str) -> Value {
    let trimmed = raw.trim();
    let query_part = if let Some(pos) = trimmed.find('?') {
        &trimmed[pos + 1..]
    } else {
        trimmed
    };
    let query_part = query_part.split('#').next().unwrap_or(query_part);

    let mut pairs: Vec<(String, Value)> = Vec::new();
    if query_part.is_empty() {
        return Value::map(pairs);
    }

    for entry in query_part.split('&') {
        if entry.is_empty() {
            continue;
        }
        let (raw_k, raw_v) = match entry.find('=') {
            Some(eq) => (&entry[..eq], &entry[eq + 1..]),
            None => (entry, ""),
        };
        let key = decode_query_component(raw_k);
        let val = decode_query_component(raw_v);
        if let Some(existing) = pairs.iter_mut().find(|(k, _)| k == &key) {
            existing.1 = Value::Text(val);
        } else {
            pairs.push((key, Value::Text(val)));
        }
    }
    Value::map(pairs)
}

/// العبارة القياسية لرمز حالة HTTP
pub fn http_status_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

fn sanitize_header_text(s: &str) -> String {
    s.chars().filter(|&c| c != '\r' && c != '\n').collect()
}

/// بناء استجابة HTTP/1.1 صالحة بالكامل مع منع حقن أسطر الترويسات (CRLF)
pub fn build_http_response(
    status: u16,
    content_type: &str,
    body: &str,
    extra_headers: &[(String, String)],
) -> String {
    let reason = http_status_reason(status);
    let safe_ct = sanitize_header_text(content_type);
    let mut resp = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n",
        status,
        reason,
        safe_ct,
        body.len()
    );
    let mut has_connection = false;
    for (k, v) in extra_headers {
        let sk = sanitize_header_text(k).trim().to_string();
        if sk.is_empty() {
            continue;
        }
        let lower = sk.to_ascii_lowercase();
        if lower == "content-type" || lower == "content-length" {
            continue;
        }
        if lower == "connection" {
            has_connection = true;
        }
        let sv = sanitize_header_text(v);
        resp.push_str(&format!("{}: {}\r\n", sk, sv));
    }
    if !has_connection {
        resp.push_str("Connection: close\r\n");
    }
    resp.push_str("\r\n");
    resp.push_str(body);
    resp
}

/// اكتشاف نوع المحتوى (Content-Type) تلقائيًا من المسار والمحتوى
pub fn detect_content_type(path: &str, body: &str) -> &'static str {
    let clean_path = path.split('?').next().unwrap_or(path).to_ascii_lowercase();
    if clean_path.ends_with(".html") || clean_path.ends_with(".htm") {
        return "text/html; charset=utf-8";
    }
    if clean_path.ends_with(".json") {
        return "application/json; charset=utf-8";
    }
    if clean_path.ends_with(".css") {
        return "text/css; charset=utf-8";
    }
    if clean_path.ends_with(".js") || clean_path.ends_with(".mjs") {
        return "application/javascript; charset=utf-8";
    }
    if clean_path.ends_with(".svg") {
        return "image/svg+xml; charset=utf-8";
    }

    let trimmed = body.trim_start();
    if trimmed.starts_with("<svg") {
        "image/svg+xml; charset=utf-8"
    } else if trimmed.starts_with("<!DOCTYPE")
        || trimmed.starts_with("<!doctype")
        || trimmed.starts_with("<html")
        || trimmed.starts_with('<')
    {
        "text/html; charset=utf-8"
    } else if (trimmed.starts_with('{') && trimmed.trim_end().ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.trim_end().ends_with(']'))
    {
        "application/json; charset=utf-8"
    } else {
        "text/plain; charset=utf-8"
    }
}

/// توليد صفحة 404 عربية منسقة بألوان هوية لسان
pub fn arabic_404_page(path: &str) -> String {
    let safe_path = escape_html(path);
    format!(
        r##"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>404 — الصفحة غير موجودة</title>
<style>
body{{margin:0;min-height:100vh;display:flex;align-items:center;justify-content:center;background:#091d37;color:#f8fafc;font-family:system-ui,'Segoe UI',Tahoma,sans-serif;padding:20px}}
.box{{max-width:480px;width:100%;background:#102a4c;border:1px solid #1f4474;border-top:4px solid #e65c1a;border-radius:20px;padding:36px 28px;text-align:center;box-shadow:0 20px 50px rgba(0,0,0,.35)}}
h1{{margin:0 0 8px;font-size:64px;font-weight:900;color:#e65c1a;line-height:1}}
h2{{margin:0 0 12px;font-size:22px}}
p{{margin:0 0 20px;color:#b8c9e0;line-height:1.7}}
code{{background:#06142a;color:#ffb98a;padding:3px 10px;border-radius:8px;font-size:14px}}
a{{display:inline-block;padding:10px 24px;border-radius:12px;background:#e65c1a;color:#fff;text-decoration:none;font-weight:700}}
</style>
</head>
<body>
<main class="box">
  <h1>404</h1>
  <h2>الصفحة غير موجودة</h2>
  <p>تعذّر العثور على المسار المطلوب <code>{safe_path}</code> في خادم تطبيق لسان.</p>
  <a href="/">العودة إلى الرئيسية</a>
</main>
</body>
</html>"##
    )
}

/// مطابقة مسار HTTP مع جدول المسارات وإرجاع (رمز الحالة، نوع المحتوى، جسم الرد)
pub fn match_route(method: &str, path: &str, routes: &[(String, String)]) -> (u16, &'static str, String) {
    let upper_method = method.trim().to_ascii_uppercase();
    let clean_path = path.split('?').next().unwrap_or(path);
    let method_key = format!("{} {}", upper_method, clean_path);

    if let Some((_, body)) = routes.iter().find(|(k, _)| k == &method_key) {
        return (200, detect_content_type(clean_path, body), body.clone());
    }
    if let Some((_, body)) = routes.iter().find(|(k, _)| k == clean_path) {
        return (200, detect_content_type(clean_path, body), body.clone());
    }
    if clean_path == "/" {
        if let Some((_, body)) = routes.iter().find(|(k, _)| k == "GET /" || k == "/") {
            return (200, detect_content_type("/", body), body.clone());
        }
    }
    if let Some((_, body)) = routes.iter().find(|(k, _)| k == "*") {
        return (200, detect_content_type(clean_path, body), body.clone());
    }
    (404, "text/html; charset=utf-8", arabic_404_page(clean_path))
}

// ============================================================================
// 3. عميل HTTP/1.1 المتقدم وخادم تطبيقات الويب (std فقط + توافق wasm32)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
fn decode_chunked_body(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < raw.len() {
        let line_end = match raw[pos..].windows(2).position(|w| w == b"\r\n") {
            Some(rel) => pos + rel,
            None => break,
        };
        let size_str = String::from_utf8_lossy(&raw[pos..line_end]);
        let size_hex = size_str.split(';').next().unwrap_or("").trim();
        let chunk_size = match usize::from_str_radix(size_hex, 16) {
            Ok(n) => n,
            Err(_) => break,
        };
        pos = line_end + 2;
        if chunk_size == 0 {
            break;
        }
        let end = (pos + chunk_size).min(raw.len());
        out.extend_from_slice(&raw[pos..end]);
        pos = end;
        if pos + 2 <= raw.len() && &raw[pos..pos + 2] == b"\r\n" {
            pos += 2;
        }
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn http_request_impl(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: &str,
) -> Result<Value, String> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let upper_method = method.trim().to_ascii_uppercase();
    match upper_method.as_str() {
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" => {}
        _ => {
            return Err(format!(
                "طريقة HTTP غير مدعومة '{}': استخدم GET أو POST أو PUT أو DELETE أو PATCH",
                method
            ));
        }
    }

    let stripped = if let Some(s) = url.strip_prefix("http://") {
        s
    } else if url.starts_with("https://") {
        return Err(
            "طلب_http_متقدم المدمجة بدون اعتماديات خارجية تدعم http:// المباشر؛ لاتصالات https:// استخدم خادمًا وسيطًا محليًا"
                .into(),
        );
    } else {
        url
    };

    let (host_port, path) = match stripped.find('/') {
        Some(idx) => (&stripped[..idx], &stripped[idx..]),
        None => (stripped, "/"),
    };
    if host_port.is_empty() {
        return Err("الرابط غير صالح: اسم المضيف فارغ".into());
    }

    let addr = if host_port.contains(':') {
        host_port.to_string()
    } else {
        format!("{}:80", host_port)
    };
    let host_header = host_port;

    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| format!("تعذر الاتصال بالخادم '{}': {}", addr, e))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));

    let mut req = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: Lisan-Web/2.0\r\nConnection: close\r\n",
        upper_method,
        sanitize_header_text(path),
        sanitize_header_text(host_header)
    );

    let mut has_content_length = false;
    for (k, v) in headers {
        let sk = sanitize_header_text(k).trim().to_string();
        if sk.is_empty() {
            continue;
        }
        let lower = sk.to_ascii_lowercase();
        if lower == "host" || lower == "connection" {
            continue;
        }
        if lower == "content-length" {
            has_content_length = true;
        }
        req.push_str(&format!("{}: {}\r\n", sk, sanitize_header_text(v)));
    }
    if !has_content_length
        && (!body.is_empty() || matches!(upper_method.as_str(), "POST" | "PUT" | "PATCH"))
    {
        req.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    req.push_str("\r\n");

    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("فشل إرسال ترويسات طلب HTTP: {}", e))?;
    if !body.is_empty() {
        stream
            .write_all(body.as_bytes())
            .map_err(|e| format!("فشل إرسال جسم طلب HTTP: {}", e))?;
    }

    let mut raw_response = Vec::new();
    stream
        .read_to_end(&mut raw_response)
        .map_err(|e| format!("فشل قراءة استجابة HTTP: {}", e))?;

    let split_pos = raw_response
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| (p, 4))
        .or_else(|| {
            raw_response
                .windows(2)
                .position(|w| w == b"\n\n")
                .map(|p| (p, 2))
        });

    let (header_bytes, body_bytes) = match split_pos {
        Some((pos, sep_len)) => (&raw_response[..pos], &raw_response[pos + sep_len..]),
        None => (&raw_response[..], &[][..]),
    };

    let header_text = String::from_utf8_lossy(header_bytes);
    let mut lines = header_text.lines();
    let mut status_code = 200.0;
    if let Some(status_line) = lines.next() {
        let mut parts = status_line.split_whitespace();
        let _proto = parts.next();
        if let Some(code_str) = parts.next() {
            if let Ok(code) = code_str.parse::<f64>() {
                status_code = code;
            }
        }
    }

    let mut resp_headers: Vec<(String, Value)> = Vec::new();
    let mut is_chunked = false;
    for line in lines {
        if let Some(colon) = line.find(':') {
            let k = line[..colon].trim().to_string();
            let v = line[colon + 1..].trim().to_string();
            if k.eq_ignore_ascii_case("transfer-encoding")
                && v.to_ascii_lowercase().contains("chunked")
            {
                is_chunked = true;
            }
            resp_headers.push((k, Value::Text(v)));
        }
    }

    let decoded_body = if is_chunked {
        String::from_utf8_lossy(&decode_chunked_body(body_bytes)).into_owned()
    } else {
        String::from_utf8_lossy(body_bytes).into_owned()
    };

    let success = (200.0..400.0).contains(&status_code);
    Ok(Value::map(vec![
        ("نجاح".to_string(), Value::Bool(success)),
        ("حالة".to_string(), Value::Number(status_code)),
        ("نص".to_string(), Value::Text(decoded_body)),
        ("ترويسات".to_string(), Value::map(resp_headers)),
    ]))
}

#[cfg(target_arch = "wasm32")]
fn http_request_impl(
    _method: &str,
    _url: &str,
    _headers: &[(String, String)],
    _body: &str,
) -> Result<Value, String> {
    Err("طلب_http_متقدم عبر المقابس المباشرة غير متاح داخل WebAssembly".into())
}

#[cfg(not(target_arch = "wasm32"))]
fn cors_headers() -> Vec<(String, String)> {
    vec![
        ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
        (
            "Access-Control-Allow-Methods".to_string(),
            "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string(),
        ),
        (
            "Access-Control-Allow-Headers".to_string(),
            "Content-Type, Authorization, X-Requested-With".to_string(),
        ),
        ("X-Powered-By".to_string(), "Lisan-Web/2.0".to_string()),
    ]
}

#[cfg(not(target_arch = "wasm32"))]
fn run_web_app_impl(
    port: u16,
    routes: &[(String, String)],
    req_limit: usize,
) -> Result<Value, String> {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::Duration;

    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("تعذر تشغيل خادم لسان على المنفذ {}: {}", port, e))?;

    let mut served: usize = 0;
    let headers = cors_headers();

    for incoming in listener.incoming() {
        let mut stream = match incoming {
            Ok(s) => s,
            Err(_) => continue,
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

        let mut buf = [0u8; 8192];
        let n = match stream.read(&mut buf) {
            Ok(0) | Err(_) => continue,
            Ok(n) => n,
        };

        let req_text = String::from_utf8_lossy(&buf[..n]);
        let first_line = req_text.lines().next().unwrap_or("GET / HTTP/1.1");
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or("GET");
        let raw_path = parts.next().unwrap_or("/");

        let response = if method.eq_ignore_ascii_case("OPTIONS") {
            build_http_response(204, "text/plain; charset=utf-8", "", &headers)
        } else {
            let (status, ct, body) = match_route(method, raw_path, routes);
            build_http_response(status, ct, &body, &headers)
        };

        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();

        served += 1;
        if req_limit > 0 && served >= req_limit {
            break;
        }
    }

    Ok(Value::Text(format!(
        "خادم لسان أتمّ خدمة {} طلبًا بنجاح على http://127.0.0.1:{}",
        served, port
    )))
}

#[cfg(target_arch = "wasm32")]
fn run_web_app_impl(
    port: u16,
    routes: &[(String, String)],
    _req_limit: usize,
) -> Result<Value, String> {
    let route_list: Vec<&str> = routes.iter().map(|(k, _)| k.as_str()).collect();
    Ok(Value::Text(format!(
        "محاكاة تطبيق ويب لسان (WebAssembly) — المنفذ: {} — المسارات ({}): {}",
        port,
        route_list.len(),
        route_list.join("، ")
    )))
}

// ============================================================================
// 4. المكتبة القياسية الكائنية لإطار عمل الويب بلغة لسان
// ============================================================================

/// أصناف إطار عمل الويب المدمجة بلغة لسان (`تطبيق_ويب`، `قالب_صفحة`، `عميل_api`)
pub const STDLIB_WEB_FRAMEWORK: &str = r##"
# إطار عمل الويب الحديث للغة لسان (تطبيق_ويب، قالب_صفحة، عميل_api)

صنف تطبيق_ويب {
    دالة بناء(اسم، منفذ = 8080) {
        هذا.اسم = اسم
        هذا.منفذ = منفذ
        هذا.مسارات = {}
    }

    دالة مسار_get(مسار، رد) {
        دع جدول_المسارات = هذا.مسارات
        جدول_المسارات["GET " + مسار] = نص(رد)
        هذا.مسارات = جدول_المسارات
        أرجع هذا
    }

    دالة مسار_post(مسار، رد) {
        دع جدول_المسارات = هذا.مسارات
        جدول_المسارات["POST " + مسار] = نص(رد)
        هذا.مسارات = جدول_المسارات
        أرجع هذا
    }

    دالة رد_json(مسار، بيانات، طريقة = "GET") {
        دع نص_json = ""
        إذا (نوع(بيانات) == "نص") {
            نص_json = بيانات
        } وإلا {
            نص_json = ترميز_json(بيانات)
        }
        دع جدول_المسارات = هذا.مسارات
        جدول_المسارات[أحرف_كبيرة(طريقة) + " " + مسار] = نص_json
        هذا.مسارات = جدول_المسارات
        أرجع هذا
    }

    دالة رد_html(مسار، محتوى_html، طريقة = "GET") {
        دع جدول_المسارات = هذا.مسارات
        جدول_المسارات[أحرف_كبيرة(طريقة) + " " + مسار] = نص(محتوى_html)
        هذا.مسارات = جدول_المسارات
        أرجع هذا
    }

    دالة عدد_المسارات() {
        أرجع طول(مفاتيح(هذا.مسارات))
    }

    دالة محاكاة_طلب(طريقة، مسار) {
        دع الطريقة_الكبيرة = أحرف_كبيرة(نظّف(طريقة))
        دع المسار_النظيف = مسار
        دع نص_الاستعلام = ""
        إذا (يحتوي(مسار، "?")) {
            دع أجزاء = قسّم(مسار، "?")
            المسار_النظيف = أجزاء[0]
            إذا (طول(أجزاء) > 1) {
                نص_الاستعلام = أجزاء[1]
            }
        }
        دع معاملات_الاستعلام = تحليل_استعلام_url(نص_الاستعلام)
        دع مسارات_حالية = هذا.مسارات
        دع مفتاح_كامل = الطريقة_الكبيرة + " " + المسار_النظيف

        دع تم_العثور = خطأ
        دع جسم_الرد = ""
        دع رمز_الرد = 404

        إذا (يحتوي(مسارات_حالية، مفتاح_كامل)) {
            تم_العثور = صحيح
            جسم_الرد = مسارات_حالية[مفتاح_كامل]
            رمز_الرد = 200
        } وإلا {
            إذا (يحتوي(مسارات_حالية، المسار_النظيف)) {
                تم_العثور = صحيح
                جسم_الرد = مسارات_حالية[المسار_النظيف]
                رمز_الرد = 200
            } وإلا {
                إذا (يحتوي(مسارات_حالية، "*")) {
                    تم_العثور = صحيح
                    جسم_الرد = مسارات_حالية["*"]
                    رمز_الرد = 200
                }
            }
        }

        إذا (ليس تم_العثور) {
            جسم_الرد = "<!DOCTYPE html><html lang=\"ar\" dir=\"rtl\"><head><meta charset=\"UTF-8\"><title>404 — غير موجود</title></head><body><h1>404</h1><p>الصفحة غير موجودة: " + تنظيف_html(المسار_النظيف) + "</p></body></html>"
        }

        دع منظف = نظّف(جسم_الرد)
        دع نوع_المحتوى = "text/plain; charset=utf-8"
        إذا (ينتهي_ب(المسار_النظيف، ".css")) {
            نوع_المحتوى = "text/css; charset=utf-8"
        } وإلا {
            إذا (ينتهي_ب(المسار_النظيف، ".js")) {
                نوع_المحتوى = "application/javascript; charset=utf-8"
            } وإلا {
                إذا (ينتهي_ب(المسار_النظيف، ".svg") أو يبدأ_ب(منظف، "<svg")) {
                    نوع_المحتوى = "image/svg+xml; charset=utf-8"
                } وإلا {
                    إذا (ينتهي_ب(المسار_النظيف، ".json") أو يبدأ_ب(منظف، "{") أو يبدأ_ب(منظف، "[")) {
                        نوع_المحتوى = "application/json; charset=utf-8"
                    } وإلا {
                        إذا (يبدأ_ب(منظف، "<")) {
                            نوع_المحتوى = "text/html; charset=utf-8"
                        }
                    }
                }
            }
        }

        دع ترويسات_الرد = {
            "Access-Control-Allow-Origin": "*",
            "X-Powered-By": "Lisan-Web/2.0"
        }
        دع استجابة_نصية = بناء_استجابة_http(رمز_الرد، نوع_المحتوى، جسم_الرد، ترويسات_الرد)

        أرجع {
            "نجاح": تم_العثور،
            "حالة": رمز_الرد،
            "مسار": المسار_النظيف،
            "طريقة": الطريقة_الكبيرة،
            "استعلام": معاملات_الاستعلام،
            "نوع_المحتوى": نوع_المحتوى،
            "جسم": جسم_الرد،
            "استجابة_http": استجابة_نصية
        }
    }

    دالة شغّل(حد_طلبات = 1) {
        أرجع تشغيل_تطبيق_ويب(هذا.منفذ، هذا.مسارات، حد_طلبات)
    }

    دالة شغل(حد_طلبات = 1) {
        أرجع هذا.شغّل(حد_طلبات)
    }
}

صنف قالب_صفحة {
    دالة بناء(قالب_نص) {
        هذا.قالب = قالب_نص
    }

    دالة عرض(بيانات_قاموس = {}) {
        أرجع عرض_قالب(هذا.قالب، بيانات_قاموس)
    }
}

صنف عميل_api {
    دالة بناء(رابط_أساسي، ترويسات = {}) {
        هذا.رابط_أساسي = رابط_أساسي
        هذا.ترويسات = ترويسات
    }

    دالة بناء_الرابط(مسار) {
        دع أساسي = هذا.رابط_أساسي
        إذا (ينتهي_ب(أساسي، "/") و يبدأ_ب(مسار، "/")) {
            أرجع أساسي + اقتطع(مسار، 1، طول(مسار))
        }
        إذا (ليس ينتهي_ب(أساسي، "/") و ليس يبدأ_ب(مسار، "/")) {
            أرجع أساسي + "/" + مسار
        }
        أرجع أساسي + مسار
    }

    دالة جلب(مسار) {
        أرجع طلب_http_متقدم("GET"، هذا.بناء_الرابط(مسار)، هذا.ترويسات، "")
    }

    دالة إرسال(مسار، بيانات_قاموس، طريقة = "POST") {
        دع رؤوس_الطلب = {}
        دع مفاتيح_أساسية = مفاتيح(هذا.ترويسات)
        لكل مفتاح في مفاتيح_أساسية {
            رؤوس_الطلب[مفتاح] = هذا.ترويسات[مفتاح]
        }
        رؤوس_الطلب["Content-Type"] = "application/json; charset=utf-8"
        دع جسم_الطلب = ""
        إذا (نوع(بيانات_قاموس) == "نص") {
            جسم_الطلب = بيانات_قاموس
        } وإلا {
            جسم_الطلب = ترميز_json(بيانات_قاموس)
        }
        أرجع طلب_http_متقدم(أحرف_كبيرة(طريقة)، هذا.بناء_الرابط(مسار)، رؤوس_الطلب، جسم_الطلب)
    }
}
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html_prevents_xss() {
        let raw = r#"<script>alert("xss" & '1')</script>"#;
        let escaped = escape_html(raw);
        assert_eq!(
            escaped,
            "&lt;script&gt;alert(&quot;xss&quot; &amp; &#39;1&#39;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_render_template_escaped_raw_and_dotted_paths() {
        let user_map = Value::map(vec![
            ("اسم".to_string(), Value::Text("أحمد <مشرف>".to_string())),
            ("عمر".to_string(), Value::Number(28.0)),
        ]);
        let items = Value::list(vec![
            Value::Text("كتاب لسان".to_string()),
            Value::Text("دليل الويب".to_string()),
        ]);
        let data = Value::map(vec![
            ("مستخدم".to_string(), user_map),
            ("شارة".to_string(), Value::Text("<b>مميز</b>".to_string())),
            ("عناصر".to_string(), items),
        ]);

        let tpl = "<h1>مرحباً {{ مستخدم.اسم }}</h1><div>{{{ شارة }}}</div><p>{{عناصر.0}} - {{مستخدم.عمر}} - {{غير_موجود}}</p>";
        let rendered = render_template(tpl, &data);
        assert_eq!(
            rendered,
            "<h1>مرحباً أحمد &lt;مشرف&gt;</h1><div><b>مميز</b></div><p>كتاب لسان - 28 - </p>"
        );
    }

    #[test]
    fn test_parse_query_basic_and_percent_decoded_utf8() {
        let val = eval_web_builtin(
            "تحليل_استعلام_url",
            vec![Value::Text("/search?a=1&b=hello+world&name=%D9%84%D8%B3%D8%A7%D9%86".into())],
        )
        .unwrap();
        match val {
            Value::Map(m) => {
                let pairs = m.borrow();
                assert!(pairs.iter().any(|(k, v)| k == "a" && *v == Value::Text("1".into())));
                assert!(pairs
                    .iter()
                    .any(|(k, v)| k == "b" && *v == Value::Text("hello world".into())));
                assert!(pairs
                    .iter()
                    .any(|(k, v)| k == "name" && *v == Value::Text("لسان".into())));
            }
            other => panic!("Expected Map, got {:?}", other),
        }
    }

    #[test]
    fn test_build_http_response_headers_length_and_crlf_safety() {
        let headers = Value::map(vec![
            ("X-Custom".to_string(), Value::Text("ok\r\nInjected: bad".into())),
        ]);
        let body = "مرحباً بكم";
        let resp = eval_web_builtin(
            "بناء_استجابة_http",
            vec![
                Value::Number(201.0),
                Value::Text("text/plain; charset=utf-8".into()),
                Value::Text(body.into()),
                headers,
            ],
        )
        .unwrap();
        let text = match resp {
            Value::Text(s) => s,
            other => panic!("Expected Text, got {:?}", other),
        };
        assert!(text.starts_with("HTTP/1.1 201 Created\r\n"));
        assert!(text.contains(&format!("Content-Length: {}\r\n", body.len())));
        assert!(text.contains("X-Custom: okInjected: bad\r\n"));
        assert!(!text.contains("\r\nInjected:"));
        assert!(text.ends_with(&format!("\r\n\r\n{}", body)));
    }

    #[test]
    fn test_route_matching_and_content_type_detection() {
        let routes = vec![
            ("GET /".to_string(), "<!DOCTYPE html><html><body>الرئيسية</body></html>".to_string()),
            ("GET /api/info".to_string(), r#"{"lang":"lisan","ver":2}"#.to_string()),
            ("POST /api/echo".to_string(), r#"{"ok":true}"#.to_string()),
            ("/style.css".to_string(), "body{color:#091d37}".to_string()),
        ];

        let (st, ct, _) = match_route("GET", "/", &routes);
        assert_eq!(st, 200);
        assert_eq!(ct, "text/html; charset=utf-8");

        let (st, ct, _) = match_route("GET", "/api/info?x=1", &routes);
        assert_eq!(st, 200);
        assert_eq!(ct, "application/json; charset=utf-8");

        let (st, ct, _) = match_route("POST", "/api/echo", &routes);
        assert_eq!(st, 200);
        assert_eq!(ct, "application/json; charset=utf-8");

        let (st, ct, _) = match_route("GET", "/style.css", &routes);
        assert_eq!(st, 200);
        assert_eq!(ct, "text/css; charset=utf-8");

        let (st, ct, body) = match_route("GET", "/missing", &routes);
        assert_eq!(st, 404);
        assert_eq!(ct, "text/html; charset=utf-8");
        assert!(body.contains("الصفحة غير موجودة"));
    }

    #[test]
    fn test_stdlib_web_framework_lexes_parses_and_loads() {
        let tokens = crate::lexer::tokenize(STDLIB_WEB_FRAMEWORK)
            .expect("STDLIB_WEB_FRAMEWORK must tokenize");
        let stmts = crate::parser::parse(tokens)
            .expect("STDLIB_WEB_FRAMEWORK must parse");
        assert_eq!(stmts.len(), 3);

        // تشغيل تعريفات الأصناف وتنفيذ سيناريو كامل داخل المفسّر
        let script = format!(
            "{}\n{}",
            STDLIB_WEB_FRAMEWORK,
            r##"
دع قالب = جديد قالب_صفحة("<h1>مرحباً {{مستخدم.اسم}}</h1><div>{{{شارة}}}</div>")
دع ناتج_قالب = قالب.عرض({"مستخدم": {"اسم": "سارة <طالب>"}, "شارة": "<b>نشط</b>"})
اطبع(يحتوي(ناتج_قالب، "سارة &lt;طالب&gt;"))
اطبع(يحتوي(ناتج_قالب، "<b>نشط</b>"))

دع تطبيق = جديد تطبيق_ويب("بوابة الاختبار"، 9090)
تطبيق.رد_html("/"، "<h1>أهلاً</h1>")
تطبيق.رد_json("/api/status"، {"خادم": "لسان"، "إصدار": 2})

دع رد1 = تطبيق.محاكاة_طلب("GET"، "/api/status?lang=ar")
اطبع(رد1["حالة"])
اطبع(رد1["استعلام"]["lang"])
اطبع(يحتوي(رد1["نوع_المحتوى"]، "application/json"))

دع رد2 = تطبيق.محاكاة_طلب("GET"، "/غير_موجود")
اطبع(رد2["حالة"])
اطبع(رد2["نجاح"])
"##
        );
        let mut interp = crate::runtime::Interpreter::default();
        let out = interp
            .run_source(&script)
            .expect("STDLIB_WEB_FRAMEWORK must execute cleanly");
        assert_eq!(
            out,
            vec!["صحيح", "صحيح", "200", "ar", "صحيح", "404", "خطأ"]
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_live_http_server_and_client_loopback() {
        use std::net::TcpListener;
        use std::thread;

        // احجز منفذًا حرًا من النظام ثم أغلقه فورًا لتشغيل الخادم عليه
        let probe = TcpListener::bind(("127.0.0.1", 0)).expect("bind ephemeral port");
        let port = probe.local_addr().unwrap().port();
        drop(probe);

        let routes = vec![
            (
                "GET /api/ping".to_string(),
                r#"{"status":"ok","lang":"لسان"}"#.to_string(),
            ),
        ];

        let server_handle =
            thread::spawn(move || run_web_app_impl(port, &routes, 1).map(|v| v.to_string()));
        let url = format!("http://127.0.0.1:{}/api/ping", port);
        let mut resp = Err("لم يبدأ الاتصال بعد".to_string());
        // runners بطيئة (macOS CI) قد تتأخر في ربط المنفذ: انتظر حتى ~6 ثوانٍ بدل 300ms
        for _ in 0..240 {
            thread::sleep(std::time::Duration::from_millis(25));
            resp = http_request_impl("GET", &url, &[], "");
            if resp.is_ok() {
                break;
            }
        }
        let resp = resp.expect("HTTP client request must succeed");
        let server_res = server_handle.join().expect("server thread must finish");
        assert!(server_res.is_ok());

        match resp {
            Value::Map(m) => {
                let pairs = m.borrow();
                assert!(pairs.iter().any(|(k, v)| k == "نجاح" && *v == Value::Bool(true)));
                assert!(pairs.iter().any(|(k, v)| k == "حالة" && *v == Value::Number(200.0)));
                assert!(pairs.iter().any(|(k, v)| {
                    k == "نص" && matches!(v, Value::Text(s) if s.contains("لسان"))
                }));
            }
            other => panic!("Expected response Map, got {:?}", other),
        }
    }
}
