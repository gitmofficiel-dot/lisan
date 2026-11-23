//! خادم بروتوكول سياق النموذج (Model Context Protocol - MCP) للغة لسان (Lisan).
//!
//! يوفر خادم JSON-RPC 2.0 قياسي لوكلاء الذكاء الاصطناعي (Claude / Gemini / GPT / Cursor / Antigravity)
//! عبر الإدخال والإخراج القياسي (يدعم كلاً من JSON-RPC المفصول بسطر جديد وتأطير `Content-Length:`)،
//! بالإضافة إلى دالة معالجة نقية قابلة للاختبار والتشغيل على WebAssembly (`handle_mcp_request`).
//!
//! الأدوات المتاحة:
//! 1. `lisan_run` — تشغيل شيفرة لسان بأمان داخل الصندوق الآمن `Limits::sandbox()`.
//! 2. `lisan_check` — فحص القواعد النحوية واكتشاف الكلمات المفتاحية الإنجليزية العرضية واقتراح بديلها العربي.
//! 3. `lisan_format` — تنسيق شيفرة لسان عبر `crate::fmt::format_source`.
//! 4. `lisan_transpile` — ترجمة شيفرة لسان إلى أهداف العتاد (`arduino`, `esp32`, `stm32`, `raspberry`, `pico`).
//! 5. `lisan_docs` — البحث في المرجع العربي المدمج الشامل للغة لسان.
//! 6. `lisan_from_python` — محوّل حتمي من شيفرة بايثون الشائعة إلى شيفرة لسان عربية خالصة.
//!
//! الموارد المتاحة:
//! - `lisan://llms.txt` — المرجع المختصر المعتمد للغة لسان لوكلاء النماذج اللغوية.

use crate::json::parse_json;
use crate::lexer::TokenKind;
use crate::runtime::{Interpreter, Limits, Value};

#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, BufRead, Read, Write};

/// إصدار بروتوكول MCP المدعوم.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// اسم الخادم وإصداره.
pub const MCP_SERVER_NAME: &str = "lisan-mcp";
pub const MCP_SERVER_VERSION: &str = "1.2.0";

/// معرف المورد المدمج لملف llms.txt.
pub const RESOURCE_LLMS_TXT_URI: &str = "lisan://llms.txt";

/// المرجع المختصر للغة لسان بصيغة llms.txt (مضمّن مباشرة من docs/llms.txt).
pub const EMBEDDED_LLMS_TXT: &str = include_str!("../docs/llms.txt");

// ============================================================================
// خادم الإدخال/الإخراج القياسي (Native stdio server)
// ============================================================================

/// تشغيل خادم MCP عبر الإدخال والإخراج القياسي (stdio).
///
/// يدعم كلاً من:
/// - رسائل JSON-RPC المفصولة بسطر جديد (`\n`) القياسية في MCP
/// - رسائل JSON-RPC المؤطّرة بترويسة `Content-Length: N\r\n\r\n`
/// ويُعيد الاستجابة بنفس أسلوب التأطير الذي استخدمه العميل.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_mcp_server() -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    loop {
        let mut first_line = String::new();
        let bytes_read = stdin_lock
            .read_line(&mut first_line)
            .map_err(|e| format!("فشل قراءة الإدخال في خادم MCP: {}", e))?;
        if bytes_read == 0 {
            return Ok(()); // EOF
        }

        let trimmed = first_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // الحالة 1: JSON-RPC مفصول بسطر جديد (MCP stdio الافتراضي)
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if let Some(response) = handle_mcp_request(trimmed) {
                writeln!(stdout_lock, "{}", response)
                    .map_err(|e| format!("فشل كتابة استجابة MCP: {}", e))?;
                stdout_lock
                    .flush()
                    .map_err(|e| format!("فشل تفريغ مخرجات MCP: {}", e))?;
            }
            continue;
        }

        // الحالة 2: JSON-RPC مؤطّر بـ Content-Length (نمط LSP)
        let mut content_length = parse_content_length_header(trimmed);
        loop {
            let mut header_line = String::new();
            let n = stdin_lock
                .read_line(&mut header_line)
                .map_err(|e| format!("فشل قراءة ترويسة MCP: {}", e))?;
            if n == 0 {
                return Ok(());
            }
            let h = header_line.trim();
            if h.is_empty() {
                break;
            }
            if content_length.is_none() {
                content_length = parse_content_length_header(h);
            }
        }

        let length = match content_length {
            Some(l) if l <= 16 * 1024 * 1024 => l,
            _ => continue,
        };

        let mut body_bytes = vec![0u8; length];
        stdin_lock
            .read_exact(&mut body_bytes)
            .map_err(|e| format!("فشل قراءة جسم رسالة MCP: {}", e))?;
        let body_str = String::from_utf8_lossy(&body_bytes);

        if let Some(response) = handle_mcp_request(&body_str) {
            let resp_bytes = response.as_bytes();
            write!(stdout_lock, "Content-Length: {}\r\n\r\n", resp_bytes.len())
                .map_err(|e| format!("فشل كتابة ترويسة استجابة MCP: {}", e))?;
            stdout_lock
                .write_all(resp_bytes)
                .map_err(|e| format!("فشل كتابة جسم استجابة MCP: {}", e))?;
            stdout_lock
                .flush()
                .map_err(|e| format!("فشل تفريغ مخرجات MCP: {}", e))?;
        }
    }
}

/// نسخة بديلة على WebAssembly لضمان التوافق عند البناء لهدف `wasm32-unknown-unknown`.
#[cfg(target_arch = "wasm32")]
pub fn run_mcp_server() -> Result<(), String> {
    Err("خادم MCP عبر الإدخال/الإخراج القياسي غير متاح على WebAssembly؛ استخدم handle_mcp_request مباشرة".into())
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_content_length_header(line: &str) -> Option<usize> {
    let lower = line.to_ascii_lowercase();
    let rest = lower.strip_prefix("content-length:")?;
    rest.trim().parse::<usize>().ok()
}

// ============================================================================
// معالج JSON-RPC 2.0 النقي العابر للمنصات (Pure Cross-Platform Dispatcher)
// ============================================================================

/// معالجة طلب JSON-RPC 2.0 واحد وفق بروتوكول MCP وإرجاع استجابة JSON إن كانت مطلوبة.
///
/// تعيد `None` للإشعارات (مثل `notifications/initialized`) التي لا تتطلب رداً.
pub fn handle_mcp_request(raw_json: &str) -> Option<String> {
    let trimmed = raw_json.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parsed = match parse_json(trimmed) {
        Ok(v) => v,
        Err(err) => {
            return Some(jsonrpc_error("null", -32700, &format!("خطأ في تحليل JSON: {}", err)));
        }
    };

    let method = match map_get_str(&parsed, "method") {
        Some(m) => m,
        None => {
            // إذا لم يوجد method فربما كان رداً أو طلباً غير صالح
            if let Some(id_json) = extract_id_json(&parsed) {
                return Some(jsonrpc_error(&id_json, -32600, "طلب JSON-RPC غير صالح: الحقل 'method' مفقود"));
            }
            return None;
        }
    };

    let id_json = extract_id_json(&parsed);

    // الإشعارات لا تنتج رداً
    if method == "notifications/initialized"
        || method == "initialized"
        || method == "notifications/cancelled"
        || method.starts_with("notifications/")
    {
        return None;
    }

    let req_id = match id_json {
        Some(id) => id,
        None => return None, // رسالة بلا id تعتبر إشعاراً
    };

    let params = map_get(&parsed, "params");

    match method.as_str() {
        "initialize" => {
            let result = format!(
                r#"{{"protocolVersion":"{}","capabilities":{{"tools":{{"listChanged":false}},"resources":{{"subscribe":false,"listChanged":false}}}},"serverInfo":{{"name":"{}","version":"{}"}},"instructions":"{}"}}"#,
                MCP_PROTOCOL_VERSION,
                MCP_SERVER_NAME,
                MCP_SERVER_VERSION,
                json_escape("لغة لسان (Lisan) لغة برمجة عربية خالصة 100% بلا كلمات مفتاحية إنجليزية. استخدم lisan_check لفحص الكود، وlisan_run لتشغيله بأمان، وlisan_docs للبحث في المرجع، وlisan_from_python لتحويل شيفرة بايثون إلى لسان.")
            );
            Some(jsonrpc_result(&req_id, &result))
        }
        "ping" => Some(jsonrpc_result(&req_id, "{}")),
        "tools/list" => Some(jsonrpc_result(&req_id, &build_tools_list_json())),
        "tools/call" => {
            let params_val = match params {
                Some(p) => p,
                None => {
                    return Some(jsonrpc_error(&req_id, -32602, "المعاملات 'params' مطلوبة لاستدعاء الأداة"));
                }
            };
            let tool_name = match map_get_str(&params_val, "name") {
                Some(name) => name,
                None => {
                    return Some(jsonrpc_error(&req_id, -32602, "اسم الأداة 'name' مطلوب في tools/call"));
                }
            };
            let arguments = map_get(&params_val, "arguments").unwrap_or_else(|| Value::map(Vec::new()));
            let (payload_text, is_error) = dispatch_tool(&tool_name, &arguments);
            let result = format!(
                r#"{{"content":[{{"type":"text","text":"{}"}}],"isError":{}}}"#,
                json_escape(&payload_text),
                if is_error { "true" } else { "false" }
            );
            Some(jsonrpc_result(&req_id, &result))
        }
        "resources/list" => {
            let result = format!(
                r#"{{"resources":[{{"uri":"{}","name":"Lisan Compact Reference (llms.txt)","description":"{}","mimeType":"text/plain; charset=utf-8"}}]}}"#,
                RESOURCE_LLMS_TXT_URI,
                json_escape("المرجع المختصر الرسمي للغة لسان العربية الخالصة (القواعد، الكلمات المحجوزة، المكتبة القياسية، العتاد، وأوامر الوكلاء).")
            );
            Some(jsonrpc_result(&req_id, &result))
        }
        "resources/read" => {
            let params_val = match params {
                Some(p) => p,
                None => {
                    return Some(jsonrpc_error(&req_id, -32602, "المعامل 'uri' مطلوب في resources/read"));
                }
            };
            let uri = match map_get_str(&params_val, "uri") {
                Some(u) => u,
                None => {
                    return Some(jsonrpc_error(&req_id, -32602, "الحقل 'uri' مطلوب في resources/read"));
                }
            };
            if uri == RESOURCE_LLMS_TXT_URI {
                let result = format!(
                    r#"{{"contents":[{{"uri":"{}","mimeType":"text/plain; charset=utf-8","text":"{}"}}]}}"#,
                    RESOURCE_LLMS_TXT_URI,
                    json_escape(EMBEDDED_LLMS_TXT)
                );
                Some(jsonrpc_result(&req_id, &result))
            } else {
                Some(jsonrpc_error(
                    &req_id,
                    -32002,
                    &format!("المورد '{}' غير موجود؛ المورد المتاح هو '{}'", uri, RESOURCE_LLMS_TXT_URI),
                ))
            }
        }
        _ => Some(jsonrpc_error(
            &req_id,
            -32601,
            &format!("الطريقة '{}' غير مدعومة في خادم Lisan MCP", method),
        )),
    }
}

fn build_tools_list_json() -> String {
    let tools = [
        (
            "lisan_run",
            "Run Lisan (لسان) Arabic source code safely inside Limits::sandbox() (or custom max_steps) and return stdout lines, execution status, and structured Arabic errors.",
            r#"{"type":"object","properties":{"code":{"type":"string","description":"Lisan (.lsn) Arabic source code to execute."},"max_steps":{"type":"integer","description":"Optional execution step limit (1 to 2000000; default 250000)."}},"required":["code"]}"#,
        ),
        (
            "lisan_check",
            "Tokenize and parse Lisan source code, detect accidental English keywords (if, else, def, return, import, class, while, for, print, true, false, let, var, function, try, except, catch, from, as, and, or, not, in, etc.) with exact pure Arabic Lisan suggestions, and return JSON diagnostics.",
            r#"{"type":"object","properties":{"code":{"type":"string","description":"Lisan source code to lint and syntax-check."}},"required":["code"]}"#,
        ),
        (
            "lisan_format",
            "Format Lisan source code cleanly using the official Lisan formatter (crate::fmt::format_source).",
            r#"{"type":"object","properties":{"code":{"type":"string","description":"Lisan source code to format."}},"required":["code"]}"#,
        ),
        (
            "lisan_transpile",
            "Transpile Lisan source code to arduino (C++), esp32 (C++), stm32 (C++), raspberry (Python 3), pico (MicroPython RP2040), python (Python 3), or js (ES2022 JavaScript).",
            r#"{"type":"object","properties":{"code":{"type":"string","description":"Lisan source code to transpile."},"target":{"type":"string","enum":["arduino","esp32","stm32","raspberry","pico","python","js"],"description":"Target platform or language."}},"required":["code","target"]}"#,
        ),
        (
            "lisan_docs",
            "Search Lisan's embedded comprehensive Arabic reference covering syntax, types, stdlib, web, mobile PWA, AI/ML/SVG, robotics, microcontrollers (105+ classes), pure Arabic aliases, and Python-to-Lisan conversion table.",
            r#"{"type":"object","properties":{"query":{"type":"string","description":"Topic or symbol to search (e.g. 'syntax', 'types', 'stdlib', 'web', 'mobile', 'ai', 'robotics', 'microcontrollers', 'aliases', 'python', or any Arabic keyword/class). Leave empty for overview."}}}"#,
        ),
        (
            "lisan_from_python",
            "Deterministically convert common Python code (def, if/elif/else, for x in range(...), while, class, print, True/False/None, and/or/not, return, try/except, import, # comments) into valid, pure Arabic Lisan source code.",
            r#"{"type":"object","properties":{"python_code":{"type":"string","description":"Python source code to translate into pure Arabic Lisan."}},"required":["python_code"]}"#,
        ),
    ];

    let mut items = Vec::with_capacity(tools.len());
    for (name, desc, schema) in tools {
        items.push(format!(
            r#"{{"name":"{}","description":"{}","inputSchema":{}}}"#,
            name,
            json_escape(desc),
            schema
        ));
    }
    format!(r#"{{"tools":[{}]}}"#, items.join(","))
}

fn dispatch_tool(name: &str, args: &Value) -> (String, bool) {
    match name {
        "lisan_run" => tool_lisan_run(args),
        "lisan_check" => tool_lisan_check(args),
        "lisan_format" => tool_lisan_format(args),
        "lisan_transpile" => tool_lisan_transpile(args),
        "lisan_docs" => tool_lisan_docs(args),
        "lisan_from_python" => tool_lisan_from_python(args),
        _ => (
            format!(
                r#"{{"ok":false,"error":"أداة غير معروفة: '{}'"}}"#,
                json_escape(name)
            ),
            true,
        ),
    }
}

// ============================================================================
// الأداة 1: lisan_run
// ============================================================================

fn tool_lisan_run(args: &Value) -> (String, bool) {
    let code = match map_get_str(args, "code") {
        Some(c) => c,
        None => {
            return (
                r#"{"ok":false,"status":"خطأ في المعاملات","error":"المعامل 'code' مطلوب وهو نص شيفرة لسان"}"#.into(),
                true,
            );
        }
    };

    let mut limits = Limits::sandbox();
    if let Some(steps) = map_get_usize(args, "max_steps") {
        limits.max_steps = Some(steps.clamp(1, 2_000_000));
    }

    let mut interp = Interpreter::with_limits(limits);
    match interp.run_source(&code) {
        Ok(lines) => {
            let stdout_items: Vec<String> = lines
                .iter()
                .map(|l| format!("\"{}\"", json_escape(l)))
                .collect();
            let joined = lines.join("\n");
            let json = format!(
                r#"{{"ok":true,"status":"نجح التنفيذ","stdout":[{}],"output":"{}","error":null}}"#,
                stdout_items.join(","),
                json_escape(&joined)
            );
            (json, false)
        }
        Err(err) => {
            let (line0, col0, msg) = crate::lsp::parse_diagnostic_location(&err);
            let line = if err.contains("السطر ") { line0 + 1 } else { 0 };
            let col = if err.contains("العمود ") { col0 + 1 } else { 0 };
            let json = format!(
                r#"{{"ok":false,"status":"فشل التنفيذ","stdout":[],"output":"","error":{{"line":{},"column":{},"message":"{}"}}}}"#,
                line,
                col,
                json_escape(&msg)
            );
            (json, true)
        }
    }
}

// ============================================================================
// الأداة 2: lisan_check
// ============================================================================

/// جدول الكلمات المفتاحية الإنجليزية الممنوعة وبدائلها العربية الدقيقة في لسان.
fn english_keyword_suggestion(word: &str) -> Option<&'static str> {
    match word {
        "if" => Some("إذا"),
        "elif" => Some("وإلا { إذا (...) { ... } }"),
        "else" => Some("وإلا"),
        "def" | "function" | "fn" | "func" => Some("دالة"),
        "return" => Some("أرجع"),
        "import" => Some("استورد"),
        "from" => Some("من"),
        "as" => Some("كـ"),
        "class" | "struct" => Some("صنف"),
        "while" => Some("طالما"),
        "for" => Some("لكل"),
        "in" => Some("في"),
        "print" => Some("اطبع"),
        "true" | "True" => Some("صحيح"),
        "false" | "False" => Some("خطأ"),
        "null" | "None" | "nil" => Some("لاشيء"),
        "let" | "var" | "const" => Some("دع"),
        "try" => Some("حاول"),
        "except" | "catch" => Some("التقط"),
        "finally" => Some("أخيرًا"),
        "raise" | "throw" => Some("ارمِ"),
        "and" => Some("و"),
        "or" => Some("أو"),
        "not" => Some("ليس"),
        "break" => Some("توقف"),
        "continue" => Some("تابع"),
        "with" => Some("مع"),
        "match" | "switch" => Some("اختر"),
        "case" => Some("حالة"),
        "default" => Some("افتراضي"),
        "new" => Some("جديد"),
        "self" | "this" => Some("هذا"),
        "extends" => Some("يرث"),
        "lambda" => Some("دالة(...) => ..."),
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct CheckDiagnostic {
    severity: &'static str,
    code: &'static str,
    line: usize,
    column: usize,
    found: Option<String>,
    suggestion: Option<String>,
    message: String,
}

fn run_lisan_diagnostics(source: &str) -> Vec<CheckDiagnostic> {
    let mut diagnostics = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    let tokens = match crate::lexer::tokenize(source) {
        Ok(toks) => toks,
        Err(err) => {
            let (l0, c0, msg) = crate::lsp::parse_diagnostic_location(&err);
            diagnostics.push(CheckDiagnostic {
                severity: "error",
                code: "lexer-error",
                line: if err.contains("السطر ") { (l0 + 1) as usize } else { 1 },
                column: if err.contains("العمود ") { (c0 + 1) as usize } else { 1 },
                found: None,
                suggestion: None,
                message: msg,
            });
            return diagnostics;
        }
    };

    // فحص الكلمات المفتاحية الإنجليزية العرضية (بما في ذلك as و from التي يمررها المحلل المعجمي)
    for tok in &tokens {
        match &tok.kind {
            TokenKind::Identifier(word) => {
                if let Some(arabic) = english_keyword_suggestion(word) {
                    diagnostics.push(CheckDiagnostic {
                        severity: "error",
                        code: "english-keyword",
                        line: tok.line,
                        column: tok.column,
                        found: Some(word.clone()),
                        suggestion: Some(arabic.to_string()),
                        message: format!(
                            "السطر {}، العمود {}: الكلمة المفتاحية الإنجليزية '{}' غير مسموحة في لسان — استخدم البديل العربي الخالص '{}'",
                            tok.line, tok.column, word, arabic
                        ),
                    });
                }
            }
            TokenKind::As => {
                if let Some(src_word) = extract_source_word(&lines, tok.line, tok.column) {
                    if src_word == "as" {
                        diagnostics.push(CheckDiagnostic {
                            severity: "error",
                            code: "english-keyword",
                            line: tok.line,
                            column: tok.column,
                            found: Some("as".into()),
                            suggestion: Some("كـ".into()),
                            message: format!(
                                "السطر {}، العمود {}: الكلمة المفتاحية الإنجليزية 'as' غير مسموحة — استخدم البديل العربي الخالص 'كـ'",
                                tok.line, tok.column
                            ),
                        });
                    }
                }
            }
            TokenKind::From => {
                if let Some(src_word) = extract_source_word(&lines, tok.line, tok.column) {
                    if src_word == "from" {
                        diagnostics.push(CheckDiagnostic {
                            severity: "error",
                            code: "english-keyword",
                            line: tok.line,
                            column: tok.column,
                            found: Some("from".into()),
                            suggestion: Some("من".into()),
                            message: format!(
                                "السطر {}، العمود {}: الكلمة المفتاحية الإنجليزية 'from' غير مسموحة — استخدم البديل العربي الخالص 'من'",
                                tok.line, tok.column
                            ),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    if let Err(err) = crate::parser::parse(tokens) {
        let (l0, c0, msg) = crate::lsp::parse_diagnostic_location(&err);
        diagnostics.push(CheckDiagnostic {
            severity: "error",
            code: "syntax-error",
            line: if err.contains("السطر ") { (l0 + 1) as usize } else { 1 },
            column: if err.contains("العمود ") { (c0 + 1) as usize } else { 1 },
            found: None,
            suggestion: None,
            message: msg,
        });
    }

    diagnostics
}

fn extract_source_word(lines: &[&str], line_1based: usize, col_1based: usize) -> Option<String> {
    if line_1based == 0 || col_1based == 0 {
        return None;
    }
    let line = lines.get(line_1based - 1)?;
    let chars: Vec<char> = line.chars().collect();
    let start = col_1based - 1;
    if start >= chars.len() {
        return None;
    }
    let mut end = start;
    while end < chars.len() && (chars[end].is_alphabetic() || chars[end] == '_') {
        end += 1;
    }
    Some(chars[start..end].iter().collect())
}

fn tool_lisan_check(args: &Value) -> (String, bool) {
    let code = match map_get_str(args, "code") {
        Some(c) => c,
        None => {
            return (
                r#"{"ok":false,"error":"المعامل 'code' مطلوب لفحص الشيفرة"}"#.into(),
                true,
            );
        }
    };

    let diags = run_lisan_diagnostics(&code);
    let ok = diags.is_empty();
    let mut diag_items = Vec::with_capacity(diags.len());
    let mut eng_items = Vec::new();

    for d in &diags {
        let found_json = match &d.found {
            Some(f) => format!("\"{}\"", json_escape(f)),
            None => "null".into(),
        };
        let sugg_json = match &d.suggestion {
            Some(s) => format!("\"{}\"", json_escape(s)),
            None => "null".into(),
        };
        if d.code == "english-keyword" {
            if let (Some(f), Some(s)) = (&d.found, &d.suggestion) {
                eng_items.push(format!(
                    r#"{{"keyword":"{}","arabic_equivalent":"{}","line":{},"column":{}}}"#,
                    json_escape(f),
                    json_escape(s),
                    d.line,
                    d.column
                ));
            }
        }
        diag_items.push(format!(
            r#"{{"severity":"{}","code":"{}","line":{},"column":{},"found":{},"suggestion":{},"message":"{}"}}"#,
            d.severity,
            d.code,
            d.line,
            d.column,
            found_json,
            sugg_json,
            json_escape(&d.message)
        ));
    }

    let json = format!(
        r#"{{"ok":{},"error_count":{},"english_keywords":[{}],"diagnostics":[{}]}}"#,
        if ok { "true" } else { "false" },
        diags.len(),
        eng_items.join(","),
        diag_items.join(",")
    );
    (json, false)
}

// ============================================================================
// الأداة 3: lisan_format
// ============================================================================

fn tool_lisan_format(args: &Value) -> (String, bool) {
    let code = match map_get_str(args, "code") {
        Some(c) => c,
        None => {
            return (
                r#"{"ok":false,"error":"المعامل 'code' مطلوب لتنسيق الشيفرة"}"#.into(),
                true,
            );
        }
    };

    let formatted = crate::fmt::format_source(&code);
    let changed = formatted != code;
    let json = format!(
        r#"{{"ok":true,"changed":{},"formatted":"{}"}}"#,
        if changed { "true" } else { "false" },
        json_escape(&formatted)
    );
    (json, false)
}

// ============================================================================
// الأداة 4: lisan_transpile
// ============================================================================

fn tool_lisan_transpile(args: &Value) -> (String, bool) {
    let code = match map_get_str(args, "code") {
        Some(c) => c,
        None => {
            return (
                r#"{"ok":false,"error":"المعامل 'code' مطلوب للترجمة"}"#.into(),
                true,
            );
        }
    };
    let target_raw = match map_get_str(args, "target") {
        Some(t) => t,
        None => {
            return (
                r#"{"ok":false,"error":"المعامل 'target' مطلوب: arduino | esp32 | stm32 | raspberry | pico"}"#.into(),
                true,
            );
        }
    };

    let norm_target = crate::diagnostics::normalize_arabic(target_raw.trim()).to_ascii_lowercase();
    let (canonical, lang, ext, result) = match norm_target.as_str() {
        "arduino" | "اردوينو" => (
            "arduino",
            "cpp",
            "ino",
            crate::arduino::transpile_arduino(&code),
        ),
        "esp32" | "esp" | "اي_اس_بي" | "اي_اس_بي32" => (
            "esp32",
            "cpp",
            "ino",
            crate::arduino::transpile_esp32(&code),
        ),
        "stm32" | "stm" | "اس_تي_ام" | "اس_تي_ام32" => (
            "stm32",
            "cpp",
            "ino",
            crate::arduino::transpile_stm32(&code),
        ),
        "raspberry" | "rpi" | "راسبيري" | "راسبيري_باي" => (
            "raspberry",
            "python",
            "py",
            crate::raspberry::transpile_raspberry_python(&code),
        ),
        "pico" | "rp2040" | "بيكو" | "راسبيري_بيكو" => (
            "pico",
            "micropython",
            "py",
            crate::pico::transpile_pico_micropython(&code),
        ),
        "python" | "py" | "بايثون" => (
            "python",
            "python",
            "py",
            crate::interop::transpile_to_python(&code),
        ),
        "js" | "javascript" | "node" | "جافاسكربت" => (
            "js",
            "javascript",
            "js",
            crate::interop::transpile_to_js(&code),
        ),
        _ => {
            return (
                format!(
                    r#"{{"ok":false,"error":"الهدف '{}' غير مدعوم. الأهداف المدعومة: arduino, esp32, stm32, raspberry, pico, python, js"}}"#,
                    json_escape(&target_raw)
                ),
                true,
            );
        }
    };

    match result {
        Ok(transpiled) => {
            let json = format!(
                r#"{{"ok":true,"target":"{}","output_language":"{}","extension":"{}","code":"{}"}}"#,
                canonical,
                lang,
                ext,
                json_escape(&transpiled)
            );
            (json, false)
        }
        Err(err) => {
            let json = format!(
                r#"{{"ok":false,"target":"{}","error":"{}"}}"#,
                canonical,
                json_escape(&err)
            );
            (json, true)
        }
    }
}

// ============================================================================
// الأداة 5: lisan_docs
// ============================================================================

struct DocTopic {
    id: &'static str,
    title: &'static str,
    keywords: &'static [&'static str],
    body: &'static str,
}

const DOC_TOPICS: &[DocTopic] = &[
    DocTopic {
        id: "syntax",
        title: "القواعد الأساسية والكلمات المفتاحية (Syntax & Keywords)",
        keywords: &["syntax", "keywords", "قواعد", "كلمات", "متغير", "دالة", "إذا", "لكل", "طالما", "اختر", "حاول", "محجوزة", "reserved"],
        body: r#"لسان لغة برمجة عربية خالصة 100% (بلا كلمات مفتاحية إنجليزية):
- المتغيرات: `دع س = 10` (أيضاً `ليكن`، `متغير`)، والتفكيك `دع أ، ب = [1، 2]`.
- الطباعة: `اطبع(تعبير)` تقبل تعبيراً واحداً — استخدم `ن"الناتج: {س}"` أو `"الناتج: " + نص(س)`.
- الشروط: `إذا (شرط) { ... } وإلا { ... }` (لتعدد الفروع: `وإلا { إذا (شرط2) { ... } }`).
- الحلقات: `طالما (شرط) { ... }` أو `بينما (شرط) { ... }`، و`لكل (ع في قائمة) { ... }`، مع `توقف` و`تابع`.
- الدوال والإغلاقات: `دالة جمع(أ، ب = 0) { أرجع أ + ب }` أو مختصرة `دالة ضعف(س) => س * 2`، ودالة مجهولة `دالة(س) => س + 1`.
- الأصناف والوراثة: `صنف ابن يرث أب { دالة بناء(اسم) { هذا.اسم = اسم } }` والإنشاء بـ `جديد ابن("زيد")`.
- المطابقة: `اختر (قيمة) { حالة 1: { ... } افتراضي: { ... } }`.
- الاستثناءات: `حاول { ارمِ "عطل" } التقط (رسالة) { ... } أخيرًا { ... }`.
- مدير السياق: `مع افتح("ملف.txt"، "ك") كـ م { م.اكتب("مرحبا") }`.
- كلمات محجوزة لا تستخدمها كأسماء متغيرات: `حالة`، `افتراضي`، `و`، `أو`، `خطأ`، `صحيح`، `لاشيء`، `فارغ`، `في`، `من`، `مع`، `باسم`، `فئة`، `هيكل`، `كـ`، `استمر`، `أطلق`، `نهائيا`."#,
    },
    DocTopic {
        id: "types",
        title: "أنواع البيانات والتراكيب (Types, Closures & Comprehensions)",
        keywords: &["types", "أنواع", "قائمة", "قاموس", "صف", "مجموعة", "تقطيع", "slice", "closure", "comprehension", "lambda", "fstring", "نصوص"],
        body: r#"أنواع البيانات في لسان:
- الأرقام: أرقام عشرية 64-بت (`42`، `3.14`، `1e3`، والأرقام العربية الهندية `١٢٣`).
- النصوص: بعلامتي تنصيص مزدوجتين `"مرحبا"`، والنصوص المنسّقة `ن"مرحبا {الاسم}، المجموع = {أ + ب}"`.
- المنطق والعدم: `صحيح`، `خطأ`، `لاشيء`.
- القوائم `[1، 2، 3]` مع التقطيع `قائمة[1:4]` و`قائمة[::-1]`، والتوليد المختصر `[س * 2 لكل س في مدى(10) إذا س % 2 == 0]`.
- القواميس `{"اسم": "سلمى"، "عمر": 25}` والصفوف الثابتة `صف([1، 2])` والمجموعات الفريدة `مجموعة([1، 1، 2])`.
- الدوال قيم من الدرجة الأولى مع الإغلاقات المعجمية والاستدعاء المتسلسل `مصنع(5)(3)`."#,
    },
    DocTopic {
        id: "stdlib",
        title: "المكتبة القياسية المدمجة (Standard Library)",
        keywords: &["stdlib", "مكتبة", "رياضيات", "إحصاء", "ملفات", "وقت", "تاريخ", "عشوائي", "جيسون", "شبكة", "نصوص", "نظام", "قاعدة_بيانات", "تشفير", "أنماط", "csv", "sha256"],
        body: r#"الوحدات القياسية (`استورد <الوحدة> كـ <اختصار>` أو `من <الوحدة> استورد ...`):
- `رياضيات`: `جذر`، `جا`، `جتا`، `ظا`، `لوغاريتم`، `أسي`، `مضروب`، `توافيق`، `تباديل`، `قاسم_مشترك`، `مضاعف_مشترك`، `تقارب`، الثوابت `ط` و`هـ`.
- `إحصاء`: `متوسط`، `وسيط`، `منوال`، `تباين`، `انحراف_معياري`، `ارتباط`.
- `نصوص`: `ترميز_قاعدة64`، `فك_قاعدة64`، `ترميز_url`، `فك_url`، إضافة للدوال المدمجة `نظّف`، `قسّم`، `ادمج`، `استبدل`، `اقتطع`، `يبدأ_ب`، `ينتهي_ب`.
- `ملفات` و`نظام`: `افتح`، `اقرأ_ملف`، `اكتب_ملف`، `ألحق_ملف`، `ملف_موجود`، `مجلد_موجود`، `أنشئ_مجلد`، `قائمة_ملفات`، `نظام_تشغيل`، `معلومات_النظام`.
- `وقت` و`تاريخ`: `طابع_زمني`، `انتظر_ملي`، `تاريخ_اليوم`، `فرق_أيام`، `أضف_أيام`، `يوم_الأسبوع`.
- `عشوائي` و`جيسون` و`شبكة`: `عشوائي`، `عدد_عشوائي`، `عنصر_عشوائي`، `خلط`، `تحليل_جيسون`، `إلى_جيسون`، `طلب_وب`.
- `استورد "قاعدة_بيانات"`: `قاعدة_بيانات`، `تشفير_أمان` (`تجزئة_sha256`، `توقيع_hmac`، `شفر_نص`، `فك_تشفير_نص`، `معرف_فريد`)، `محلل_أنماط` (`يطابق_نمط`، `استخرج_نمط`، `استبدل_نمط`)."#,
    },
    DocTopic {
        id: "web",
        title: "تطوير الويب والخوادم والمواقع (Web Framework & Static Sites)",
        keywords: &["web", "ويب", "موقع", "موقع_ويب", "إطار_ويب", "تطبيق_ويب", "قالب_صفحة", "عميل_api", "http", "خادم"],
        body: r#"مساران لتطوير الويب في لسان:
1) بناء مواقع عربية متجاوبة (`استورد "موقع"`):
   `دع م = جديد موقع_ويب("بوابتي"، "ملكي")` — `شريط_تنقل`، `قسم_رئيسي`، `أضف_بطاقة`، `أضف_إحصائيات`، `أضف_شبكة`، `أضف_أسعار`، `أضف_جدول`، `أضف_نموذج_تواصل`، `حفظ("index.html")`.
2) إطار تطبيقات وخوادم ويب (`استورد "إطار_ويب"`):
   - `تطبيق_ويب(اسم، منفذ)`: `مسار_get`، `مسار_post`، `رد_json`، `رد_html`، `محاكاة_طلب`، `شغّل`.
   - `قالب_صفحة(قالب)`: `عرض(قاموس)` مع تهريب HTML تلقائي.
   - `عميل_api(رابط_أساسي، ترويسات)`: `جلب`، `إرسال`."#,
    },
    DocTopic {
        id: "mobile",
        title: "تطبيقات الهواتف الذكية PWA 2.0 (Mobile Apps)",
        keywords: &["mobile", "هاتف", "جوال", "تطبيق_هاتف", "pwa", "شاشة", "تبويب"],
        body: r#"تطوير وتصدير تطبيقات الهواتف الذكية (`استورد "هاتف"`):
- `دع تطبيق = جديد تطبيق_هاتف("تطبيقي"، "زمردي")`
- المكونات: `أضف_بطاقة`، `أضف_إحصائية`، `أضف_مفتاح_تحكم`، `أضف_زر`، `أضف_تبويب`، `أضف_شاشة` (تعدد الشاشات)، `أضف_زر_عائم` (FAB)، `أضف_نافذة_سفلية` (Bottom Sheet)، `أضف_نموذج_هاتف`.
- التصدير: `تطبيق.تصدير("dist_app")` يولّد تطبيق PWA كامل (`index.html` + `manifest.json` + `sw.js`) قابل للتثبيت والعمل دون اتصال، أو عبر `lisan --mobile ملف.lsn`."#,
    },
    DocTopic {
        id: "ai",
        title: "الذكاء الاصطناعي وتعلم الآلة وعلوم البيانات والرسوم (AI / ML / Data / SVG)",
        keywords: &["ai", "ml", "ذكاء", "بيانات", "مصفوفة", "جدول_بيانات", "انحدار_خطي", "knn", "شبكة_عصبية", "محلل_نص_عربي", "مخطط", "svg"],
        body: r#"وحدة الذكاء الاصطناعي وعلوم البيانات (`استورد "ذكاء"`):
- الجبر الخطي: `مصفوفة` (`جمع`، `طرح`، `ضرب`، `منقول`، `تفعيل_سيجمويد`، `تفعيل_ريلو`).
- تحليل البيانات: `جدول_بيانات` (`أضف_صف`، `تصفية`، `مجموع_عمود`، `متوسط_عمود`، `تجميع_عدد`، `إلى_csv`، `من_csv`، `ملخص`).
- تعلم الآلة: `انحدار_خطي` (`درّب`، `توقّع`، `دقة_r2`)، `مصنف_جيران_knn`، `شبكة_عصبية` (انتشار عكسي كامل).
- معالجة اللغة العربية: `محلل_نص_عربي` (`أزل_التشكيل`، `وحّد_الحروف`، `كلمات`، `تكرار_الكلمات`، `تحليل_المشاعر`).
- الرسوم البيانية SVG: `مخطط_أعمدة`، `مخطط_خطي`، `مخطط_دائري`، `لوحة_رسم_svg`."#,
    },
    DocTopic {
        id: "robotics",
        title: "الروبوتات ونظرية التحكم والبروتوكولات الصناعية (Robotics & Control)",
        keywords: &["robotics", "روبوتات", "روبوت", "pid", "كالمان", "ذراع_آلي", "آلة_حالات", "مودباس", "lora", "gsm", "bms", "a4988"],
        body: r#"وحدة الروبوتات والتحكم الصناعي (`استورد "روبوتات"` — 10 فئات):
- التحكم والتنقية: `متحكم_pid`، `مرشح_كالمان`، `آلة_حالات`.
- الحركية: `روبوت_متحرك` (ثنائي العجلات والأودومتري)، `ذراع_آلي` (الحركية الأمامية ثنائية المحور).
- الصناعة والاتصالات: `مودباس_صناعي` (Modbus RTU/TCP)، `لورا_lora` (SX1278)، `شريحة_اتصال_gsm`، `سائق_خطوي_a4988`، `مراقب_بطارية_bms`."#,
    },
    DocTopic {
        id: "microcontrollers",
        title: "المتحكمات الدقيقة والحواسيب أحادية اللوحة (Arduino / ESP32 / STM32 / Raspberry Pi / Pico)",
        keywords: &["microcontrollers", "hardware", "عتاد", "arduino", "اردوينو", "esp32", "stm32", "raspberry", "راسبيري", "pico", "بيكو", "متحكمات", "gpio", "oled", "rfid", "gps"],
        body: r#"تدعم لسان 5 منصات عتادية (95+ فئة عتادية + 10 فئات روبوتات = 105 فئة) مع المحاكاة والترجمة الأصلية:
- أردوينو (`استورد "اردوينو"` / `--arduino`): 38 فئة منها `محرك_سيرفو`، `محرك_خطوي`، `حساس_مسافة`، `حساس_حرارة`، `شاشة_كريستال`، `شاشة_oled`، `شريط_إضاءة`، `مرحل_كهربائي`، `حساس_حركة_pir`، `حساس_غاز_mq`، `محدد_مواقع_gps`، `قارئ_rfid`، `حساس_تسارع_جيروسكوب`، `حساس_ضغط_جوي`، `خلية_وزن_hx711`، `حساس_نبض_max30102`، `ناقل_i2c`، `ناقل_spi`.
- ESP32 (`استورد "اي_اس_بي32"` / `--esp32`): 18 فئة منها `واي_فاي_esp`، `خادم_ويب_esp`، `عميل_mqtt`، `بلوتوث_ble`، `تحديث_هوائي_ota`، `اتصال_مباشر_espnow`، `كاميرا_esp32`، `نوم_عميق_esp`، `مهمة_متوازية_rtos`.
- STM32 (`استورد "اس_تي_ام32"` / `--stm32`): 15 فئة منها `مؤقت_عتادي_stm32`، `ناقل_can_stm32`، `ناقل_مباشر_dma`، `ساعة_الوقت_الحقيقي_rtc`، `مراقب_النظام_iwdg`، `مقاطعة_خارجية_exti`.
- راسبيري باي (`استورد "راسبيري"` / `--raspberry`): 12 فئة منها `لوحة_راسبيري`، `منفذ_gpio_راسبيري`، `نبضة_pwm_راسبيري`، `كاميرا_راسبيري`، `قبعة_الحساسات_sensehat`، `قبعة_محركات_pca9685`، `صوت_راسبيري`، `شاشة_حبر_إلكتروني_epaper`.
- راسبيري بيكو (`استورد "بيكو"` / `--pico`): 12 فئة منها `لوحة_بيكو`، `منفذ_بيكو`، `نبضة_بيكو`، `وحدة_pio_بيكو`، `واي_فاي_بيكو_w`، `شريط_نيوبكسل_بيكو`، `مستشعر_حرارة_داخلي_بيكو`."#,
    },
    DocTopic {
        id: "systems_games",
        title: "الأنظمة وأدوات الطرفية ومحرك الألعاب ثنائية الأبعاد (Systems, CLI & 2D Games)",
        keywords: &["games", "ألعاب", "لعبة", "systems", "أنظمة", "cli", "طرفية", "سجل_أحداث", "مجدول_مهام", "crc32", "rle"],
        body: r#"وحدة الأنظمة والألعاب (`استورد "ألعاب"` أو `استورد "أنظمة"`):
- `لعبة_ثنائية_الأبعاد`: بناء وتصدير ألعاب HTML5 Canvas عربية (`أضف_لاعب`، `أضف_هدف`، `أضف_عقبة`، `بناء_html`، `حفظ`).
- `أداة_سطر_أوامر`: إطارات العناوين، جداول الطرفية، أشرطة التقدم النصية، والتلوين.
- `سجل_أحداث`: تسجيل منظم (`معلومات`، `تحذير`، `عطل`، `تصدير_json`).
- `مجدول_مهام`: جدولة وتنفيذ المهام حسب الأولوية، بالإضافة إلى `بصمة_crc32`، `ضغط_rle`، `نفّذ_أمر`."#,
    },
    DocTopic {
        id: "aliases",
        title: "الأسماء العربية الخالصة والتكامل مع بايثون وجافاسكريبت (Pure Arabic Aliases & Interop)",
        keywords: &["aliases", "interop", "عربي", "بايثون", "جافاسكريبت", "استورد_بايثون", "مرادفات"],
        body: r#"قاعدة العروبة الخالصة:
- جميع الكلمات المفتاحية والوحدات في لسان لها أسماء عربية خالصة: `"اردوينو"`، `"اي_اس_بي32"`، `"اس_تي_ام32"`، `"متحكمات"`، `"راسبيري"`، `"بيكو"`، `"روبوتات"`، `"ذكاء"`، `"إطار_ويب"`، `"موقع"`، `"هاتف"`، `"قاعدة_بيانات"`، `"ألعاب"`، `"أنظمة"`.
- عند استدعاء مكتبات خارجية من بايثون أو جافاسكريبت، يظهر اسم المكتبة الأجنبية حصراً داخل سلسلة نصية، مثل:
  `دع مصفوفات = استورد_بايثون("numpy")` أو `دع بيانات = استورد_بايثون("pandas")`.
- لا تكتب أبداً كلمات مفتاحية إنجليزية مثل `import numpy as np` داخل شيفرة لسان."#,
    },
    DocTopic {
        id: "python_table",
        title: "جدول التحويل الكامل من بايثون إلى لسان (Python-to-Lisan Table)",
        keywords: &["python", "بايثون", "تحويل", "جدول", "مقارنة", "table", "migration"],
        body: r#"جدول المطابقة السريع (بايثون -> لسان):
- `x = 1` -> `دع س = 1`
- `def f(a, b=2):` -> `دالة اسم(أ، ب = 2) {`
- `return v` -> `أرجع قيمة`
- `if c:` / `elif c2:` / `else:` -> `إذا (ش) { ... } وإلا { إذا (ش2) { ... } وإلا { ... } }`
- `for i in range(5):` -> `لكل (ت في مدى(5)) { ... }`
- `while c:` -> `طالما (ش) { ... }`
- `class C(Base):` / `def __init__(self, x):` -> `صنف ج يرث أساس { دالة بناء(س) { هذا.س = س } }`
- `print(x)` -> `اطبع(س)` (ولوسائط متعددة: `اطبع(نص(أ) + " " + نص(ب))`)
- `True / False / None` -> `صحيح / خطأ / لاشيء`
- `and / or / not / in` -> `و / أو / ليس / في`
- `try / except Exception as e / finally` -> `حاول { ... } التقط (رسالة) { ... } أخيرًا { ... }`
- `import math as m` -> `استورد رياضيات كـ ر`
- `f"hi {name}"` -> `ن"مرحبا {الاسم}"`
- `len / str / int / float / range / sum` -> `طول / نص / عدد_صحيح / عدد / مدى / مجموع`"#,
    },
];

fn tool_lisan_docs(args: &Value) -> (String, bool) {
    let query_raw = map_get_str(args, "query").unwrap_or_default();
    let query = query_raw.trim();

    if query.is_empty() || query.eq_ignore_ascii_case("all") || query == "الكل" {
        let mut overview = Vec::new();
        for t in DOC_TOPICS {
            overview.push(format!(
                r#"{{"id":"{}","title":"{}"}}"#,
                t.id,
                json_escape(t.title)
            ));
        }
        let json = format!(
            r#"{{"ok":true,"query":"{}","topics":[{}],"hint":"مرّر اسم الموضوع (مثل syntax, types, stdlib, web, mobile, ai, robotics, microcontrollers, systems_games, aliases, python_table) أو أي كلمة مفتاحية عربية للبحث التفصيلي."}}"#,
            json_escape(query),
            overview.join(",")
        );
        return (json, false);
    }

    let q_lower = query.to_ascii_lowercase();
    let q_norm = crate::diagnostics::normalize_arabic(&q_lower);

    let mut matches: Vec<(usize, String)> = Vec::new();

    // 1. فحص التوثيق المباشر للرمز من خادم LSP
    if let Some(hover) = crate::lsp::get_hover_documentation(query) {
        matches.push((
            100,
            format!(
                r#"{{"source":"symbol","id":"{}","title":"توثيق الرمز '{}'","content":"{}"}}"#,
                json_escape(query),
                json_escape(query),
                json_escape(hover)
            ),
        ));
    }

    // 2. البحث في المواضيع الشاملة المدمجة
    for topic in DOC_TOPICS {
        let mut score = 0usize;
        if topic.id == q_lower || topic.id == q_norm {
            score += 90;
        }
        for kw in topic.keywords {
            let kw_norm = crate::diagnostics::normalize_arabic(&kw.to_ascii_lowercase());
            if kw_norm == q_norm || kw.eq_ignore_ascii_case(&q_lower) {
                score += 70;
            } else if kw_norm.contains(&q_norm) || q_norm.contains(&kw_norm) {
                score += 35;
            }
        }
        let title_norm = crate::diagnostics::normalize_arabic(&topic.title.to_ascii_lowercase());
        if title_norm.contains(&q_norm) {
            score += 40;
        }
        let body_norm = crate::diagnostics::normalize_arabic(&topic.body.to_ascii_lowercase());
        if body_norm.contains(&q_norm) {
            score += 20;
        }
        if score > 0 {
            matches.push((
                score,
                format!(
                    r#"{{"source":"topic","id":"{}","title":"{}","content":"{}"}}"#,
                    topic.id,
                    json_escape(topic.title),
                    json_escape(topic.body)
                ),
            ));
        }
    }

    matches.sort_by(|a, b| b.0.cmp(&a.0));
    let results: Vec<String> = matches.into_iter().take(5).map(|(_, item)| item).collect();

    let json = format!(
        r#"{{"ok":true,"query":"{}","count":{},"results":[{}]}}"#,
        json_escape(query),
        results.len(),
        results.join(",")
    );
    (json, false)
}

// ============================================================================
// الأداة 6: lisan_from_python
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum BlockKind {
    Normal,
    Class,
    Function,
    IfBranch { extra_else_braces: usize },
}

#[derive(Debug, Clone)]
struct IndentBlock {
    indent: usize,
    kind: BlockKind,
}

/// تحويل حتمي لشيفرة بايثون الشائعة إلى شيفرة لسان عربية خالصة صالحة للتنفيذ.
pub fn convert_python_to_lisan(python_code: &str) -> (String, Vec<String>) {
    let mut out_lines: Vec<String> = Vec::new();
    let mut stack: Vec<IndentBlock> = Vec::new();
    let mut declared_vars: Vec<Vec<String>> = vec![Vec::new()];
    let mut notes: Vec<String> = Vec::new();

    for raw_line in python_code.lines() {
        let expanded = raw_line.replace('\t', "    ");
        let indent = expanded.chars().take_while(|c| *c == ' ').count();
        let trimmed = expanded.trim();

        if trimmed.is_empty() {
            out_lines.push(String::new());
            continue;
        }

        // سطر تعليق كامل
        if let Some(rest) = trimmed.strip_prefix('#') {
            out_lines.push(format!("//{}", rest));
            continue;
        }

        let (code_raw, comment_opt) = split_python_comment(trimmed);
        let code = code_raw.trim();
        let comment_suffix = match comment_opt {
            Some(c) => format!(" //{}", c.strip_prefix('#').unwrap_or(c)),
            None => String::new(),
        };

        // تحديد ما إذا كان السطر استمراراً لكتلة if/try سابقة (elif / else / except / finally)
        let is_elif = is_py_keyword_prefix(code, "elif");
        let is_else = code == "else:" || code.starts_with("else :");
        let is_except = is_py_keyword_prefix(code, "except") || code == "except:";
        let is_finally = code == "finally:" || code.starts_with("finally :");
        let is_continuation = is_elif || is_else || is_except || is_finally;

        // إغلاق الكتل الأعمق من مستوى الإزاحة الحالي
        while let Some(top) = stack.last() {
            if indent < top.indent || (!is_continuation && indent == top.indent) {
                let popped = stack.pop().unwrap();
                close_block(popped, &mut out_lines, &mut declared_vars);
            } else {
                break;
            }
        }

        let in_class = stack.last().map(|b| b.kind == BlockKind::Class).unwrap_or(false);

        // معالجة elif (في لسان: } وإلا { ثم إذا (...) { مع تتبع الأقواس الإضافية)
        if is_elif {
            let cond_raw = code["elif".len()..].trim().trim_end_matches(':').trim();
            let cond = translate_py_expr(cond_raw);
            let prev_extra = if let Some(top) = stack.last() {
                if top.indent == indent {
                    match stack.pop().unwrap().kind {
                        BlockKind::IfBranch { extra_else_braces } => extra_else_braces,
                        _ => 0,
                    }
                } else {
                    0
                }
            } else {
                0
            };
            out_lines.push("} وإلا {".into());
            out_lines.push(format!("إذا ({}) {{{}", cond, comment_suffix));
            stack.push(IndentBlock {
                indent,
                kind: BlockKind::IfBranch {
                    extra_else_braces: prev_extra + 1,
                },
            });
            continue;
        }

        // معالجة else:
        if is_else {
            let prev_extra = if let Some(top) = stack.last() {
                if top.indent == indent {
                    match stack.pop().unwrap().kind {
                        BlockKind::IfBranch { extra_else_braces } => extra_else_braces,
                        _ => 0,
                    }
                } else {
                    0
                }
            } else {
                0
            };
            out_lines.push(format!("}} وإلا {{{}", comment_suffix));
            stack.push(IndentBlock {
                indent,
                kind: BlockKind::IfBranch {
                    extra_else_braces: prev_extra,
                },
            });
            continue;
        }

        // معالجة except ...:
        if is_except {
            if let Some(top) = stack.last() {
                if top.indent == indent {
                    stack.pop();
                }
            }
            let header = code.trim_end_matches(':').trim();
            let var_name = extract_except_var(header);
            out_lines.push(format!("}} التقط ({}) {{{}", var_name, comment_suffix));
            stack.push(IndentBlock {
                indent,
                kind: BlockKind::Normal,
            });
            continue;
        }

        // معالجة finally:
        if is_finally {
            if let Some(top) = stack.last() {
                if top.indent == indent {
                    stack.pop();
                }
            }
            out_lines.push(format!("}} أخيرًا {{{}", comment_suffix));
            stack.push(IndentBlock {
                indent,
                kind: BlockKind::Normal,
            });
            continue;
        }

        // معالجة def ...:
        if is_py_keyword_prefix(code, "def") {
            let rest = code["def".len()..].trim().trim_end_matches(':').trim();
            if let Some((fname, params_str)) = split_name_and_parens(rest) {
                let mapped_name = if in_class && fname == "__init__" {
                    "بناء".to_string()
                } else {
                    sanitize_lisan_ident(fname)
                };
                let params = convert_py_params(params_str, in_class);
                out_lines.push(format!("دالة {}({}) {{{}", mapped_name, params, comment_suffix));
                stack.push(IndentBlock {
                    indent,
                    kind: BlockKind::Function,
                });
                declared_vars.push(extract_param_names(&params));
                continue;
            }
        }

        // معالجة class ...:
        if is_py_keyword_prefix(code, "class") {
            let rest = code["class".len()..].trim().trim_end_matches(':').trim();
            let header = if let Some((cname, base)) = split_name_and_parens(rest) {
                let base_clean = base.trim();
                if base_clean.is_empty() || base_clean == "object" {
                    format!("صنف {}", sanitize_lisan_ident(cname))
                } else {
                    let first_base = base_clean.split(',').next().unwrap_or(base_clean).trim();
                    format!(
                        "صنف {} يرث {}",
                        sanitize_lisan_ident(cname),
                        sanitize_lisan_ident(first_base)
                    )
                }
            } else {
                format!("صنف {}", sanitize_lisan_ident(rest))
            };
            out_lines.push(format!("{} {{{}", header, comment_suffix));
            stack.push(IndentBlock {
                indent,
                kind: BlockKind::Class,
            });
            continue;
        }

        // معالجة if ...:
        if is_py_keyword_prefix(code, "if") {
            let cond_raw = code["if".len()..].trim().trim_end_matches(':').trim();
            let cond = translate_py_expr(cond_raw);
            out_lines.push(format!("إذا ({}) {{{}", cond, comment_suffix));
            stack.push(IndentBlock {
                indent,
                kind: BlockKind::IfBranch {
                    extra_else_braces: 0,
                },
            });
            continue;
        }

        // معالجة while ...:
        if is_py_keyword_prefix(code, "while") {
            let cond_raw = code["while".len()..].trim().trim_end_matches(':').trim();
            let cond = translate_py_expr(cond_raw);
            out_lines.push(format!("طالما ({}) {{{}", cond, comment_suffix));
            stack.push(IndentBlock {
                indent,
                kind: BlockKind::Normal,
            });
            continue;
        }

        // معالجة for x in iterable:
        if is_py_keyword_prefix(code, "for") {
            let rest = code["for".len()..].trim().trim_end_matches(':').trim();
            if let Some((vars_part, iter_part)) = split_top_level_keyword(rest, "in") {
                let vars_clean = vars_part
                    .split(',')
                    .map(|v| sanitize_lisan_ident(v.trim()))
                    .collect::<Vec<_>>()
                    .join("، ");
                let iter_lisan = translate_py_expr(iter_part.trim());
                out_lines.push(format!("لكل ({} في {}) {{{}", vars_clean, iter_lisan, comment_suffix));
                stack.push(IndentBlock {
                    indent,
                    kind: BlockKind::Normal,
                });
                continue;
            }
        }

        // معالجة try:
        if code == "try:" || code.starts_with("try :") {
            out_lines.push(format!("حاول {{{}", comment_suffix));
            stack.push(IndentBlock {
                indent,
                kind: BlockKind::Normal,
            });
            continue;
        }

        // معالجة with expr as var:
        if is_py_keyword_prefix(code, "with") {
            let rest = code["with".len()..].trim().trim_end_matches(':').trim();
            if let Some((expr_part, alias_part)) = split_top_level_keyword(rest, "as") {
                let expr_lisan = translate_py_expr(expr_part.trim());
                let alias_clean = sanitize_lisan_ident(alias_part.trim());
                out_lines.push(format!("مع ({} كـ {}) {{{}", expr_lisan, alias_clean, comment_suffix));
            } else {
                let expr_lisan = translate_py_expr(rest);
                out_lines.push(format!("مع ({}) {{{}", expr_lisan, comment_suffix));
            }
            stack.push(IndentBlock {
                indent,
                kind: BlockKind::Normal,
            });
            continue;
        }

        // معالجة import / from ... import ...
        if is_py_keyword_prefix(code, "import") {
            let stmt = convert_py_import(code["import".len()..].trim(), &mut notes);
            out_lines.push(format!("{}{}", stmt, comment_suffix));
            continue;
        }
        if is_py_keyword_prefix(code, "from") {
            let stmt = convert_py_from_import(code["from".len()..].trim(), &mut notes);
            out_lines.push(format!("{}{}", stmt, comment_suffix));
            continue;
        }

        // عبارات مفردة
        if code == "pass" {
            out_lines.push(format!("// لا عملية{}", comment_suffix));
            continue;
        }
        if code == "break" {
            out_lines.push(format!("توقف{}", comment_suffix));
            continue;
        }
        if code == "continue" {
            out_lines.push(format!("تابع{}", comment_suffix));
            continue;
        }
        if code == "return" {
            out_lines.push(format!("أرجع{}", comment_suffix));
            continue;
        }
        if is_py_keyword_prefix(code, "return") {
            let expr = translate_py_expr(code["return".len()..].trim());
            out_lines.push(format!("أرجع {}{}", expr, comment_suffix));
            continue;
        }
        if is_py_keyword_prefix(code, "raise") {
            let expr = translate_py_expr(code["raise".len()..].trim());
            out_lines.push(format!("ارمِ {}{}", expr, comment_suffix));
            continue;
        }

        // معالجة print(...)
        if let Some(inner) = strip_call_args(code, "print") {
            let print_stmt = convert_py_print(inner);
            out_lines.push(format!("{}{}", print_stmt, comment_suffix));
            continue;
        }

        // معالجة الإسناد المركب (+=, -=, *=, /=, %=)
        if let Some((lhs, op, rhs)) = split_compound_assign(code) {
            let lhs_lisan = translate_py_expr(lhs.trim());
            let rhs_lisan = translate_py_expr(rhs.trim());
            out_lines.push(format!(
                "{} = {} {} ({}){}",
                lhs_lisan, lhs_lisan, op, rhs_lisan, comment_suffix
            ));
            continue;
        }

        // معالجة تعريف/إسناد المتغير البسيط x = expr
        if let Some((lhs, rhs)) = split_simple_assign(code) {
            let lhs_trim = lhs.trim();
            let rhs_lisan = translate_py_expr(rhs.trim());
            if is_simple_ident(lhs_trim) {
                let ident = sanitize_lisan_ident(lhs_trim);
                let scope = declared_vars.last_mut().unwrap();
                if !scope.contains(&ident) {
                    scope.push(ident.clone());
                    out_lines.push(format!("دع {} = {}{}", ident, rhs_lisan, comment_suffix));
                } else {
                    out_lines.push(format!("{} = {}{}", ident, rhs_lisan, comment_suffix));
                }
            } else {
                let lhs_lisan = translate_py_expr(lhs_trim);
                out_lines.push(format!("{} = {}{}", lhs_lisan, rhs_lisan, comment_suffix));
            }
            continue;
        }

        // أي تعبير عام (مثل استدعاء دالة أو append)
        let expr_lisan = translate_py_statement_expr(code);
        out_lines.push(format!("{}{}", expr_lisan, comment_suffix));
    }

    // إغلاق كافة الكتل المفتوحة عند نهاية الملف
    while let Some(popped) = stack.pop() {
        close_block(popped, &mut out_lines, &mut declared_vars);
    }

    let raw_lisan = out_lines.join("\n");
    let formatted = crate::fmt::format_source(&raw_lisan);
    (formatted, notes)
}

fn close_block(block: IndentBlock, out_lines: &mut Vec<String>, declared_vars: &mut Vec<Vec<String>>) {
    out_lines.push("}".into());
    match block.kind {
        BlockKind::Function => {
            if declared_vars.len() > 1 {
                declared_vars.pop();
            }
        }
        BlockKind::IfBranch { extra_else_braces } => {
            for _ in 0..extra_else_braces {
                out_lines.push("}".into());
            }
        }
        BlockKind::Normal | BlockKind::Class => {}
    }
}

fn is_py_keyword_prefix(code: &str, kw: &str) -> bool {
    if !code.starts_with(kw) {
        return false;
    }
    match code[kw.len()..].chars().next() {
        None => true,
        Some(c) => c.is_whitespace() || c == '(' || c == ':',
    }
}

fn split_python_comment(line: &str) -> (&str, Option<&str>) {
    let mut in_str = false;
    let mut quote = ' ';
    let mut escaped = false;
    for (byte_idx, c) in line.char_indices() {
        if in_str {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote {
                in_str = false;
            }
        } else if c == '"' || c == '\'' {
            in_str = true;
            quote = c;
        } else if c == '#' {
            return (&line[..byte_idx], Some(&line[byte_idx..]));
        }
    }
    (line, None)
}

fn split_name_and_parens(s: &str) -> Option<(&str, &str)> {
    let open = s.find('(')?;
    let close = s.rfind(')')?;
    if close <= open {
        return None;
    }
    Some((s[..open].trim(), &s[open + 1..close]))
}

fn strip_call_args<'a>(code: &'a str, fn_name: &str) -> Option<&'a str> {
    let rest = code.strip_prefix(fn_name)?.trim_start();
    if !rest.starts_with('(') || !rest.ends_with(')') {
        return None;
    }
    // تأكد أن القوس الأول يغلق عند آخر محرف بالضبط
    let chars: Vec<char> = rest.chars().collect();
    let mut depth = 0usize;
    let mut in_str = false;
    let mut quote = ' ';
    let mut escaped = false;
    for (i, &c) in chars.iter().enumerate() {
        if in_str {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote {
                in_str = false;
            }
        } else if c == '"' || c == '\'' {
            in_str = true;
            quote = c;
        } else if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth = depth.saturating_sub(1);
            if depth == 0 && i + 1 != chars.len() {
                return None;
            }
        }
    }
    Some(&rest[1..rest.len() - 1])
}

fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth_paren = 0usize;
    let mut depth_brack = 0usize;
    let mut depth_brace = 0usize;
    let mut in_str = false;
    let mut quote = ' ';
    let mut escaped = false;

    for c in s.chars() {
        if in_str {
            current.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                in_str = true;
                quote = c;
                current.push(c);
            }
            '(' => {
                depth_paren += 1;
                current.push(c);
            }
            ')' => {
                depth_paren = depth_paren.saturating_sub(1);
                current.push(c);
            }
            '[' => {
                depth_brack += 1;
                current.push(c);
            }
            ']' => {
                depth_brack = depth_brack.saturating_sub(1);
                current.push(c);
            }
            '{' => {
                depth_brace += 1;
                current.push(c);
            }
            '}' => {
                depth_brace = depth_brace.saturating_sub(1);
                current.push(c);
            }
            ',' if depth_paren == 0 && depth_brack == 0 && depth_brace == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn split_top_level_keyword<'a>(s: &'a str, kw: &str) -> Option<(&'a str, &'a str)> {
    let bytes = s.as_bytes();
    let kw_bytes = kw.as_bytes();
    let mut depth = 0usize;
    let mut in_str = false;
    let mut quote = b' ';
    let mut escaped = false;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == quote {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' | b'\'' => {
                in_str = true;
                quote = b;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            _ if depth == 0 && bytes[i..].starts_with(kw_bytes) => {
                let before_ok = i == 0 || bytes[i - 1].is_ascii_whitespace() || bytes[i - 1] == b')';
                let after_idx = i + kw_bytes.len();
                let after_ok = after_idx == bytes.len()
                    || bytes[after_idx].is_ascii_whitespace()
                    || bytes[after_idx] == b'(';
                if before_ok && after_ok {
                    return Some((&s[..i], &s[after_idx..]));
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn split_compound_assign(code: &str) -> Option<(&str, &'static str, &str)> {
    let ops = [("+=", "+"), ("-=", "-"), ("*=", "*"), ("/=", "/"), ("%=", "%")];
    let mut depth = 0usize;
    let mut in_str = false;
    let mut quote = ' ';
    let mut escaped = false;
    let chars: Vec<(usize, char)> = code.char_indices().collect();

    for idx in 0..chars.len() {
        let (byte_pos, c) = chars[idx];
        if in_str {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                in_str = true;
                quote = c;
            }
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 => {
                let slice = &code[byte_pos..];
                for (pat, op) in ops {
                    if slice.starts_with(pat) {
                        return Some((&code[..byte_pos], op, &code[byte_pos + pat.len()..]));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn split_simple_assign(code: &str) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    let mut in_str = false;
    let mut quote = ' ';
    let mut escaped = false;
    let chars: Vec<(usize, char)> = code.char_indices().collect();

    for idx in 0..chars.len() {
        let (byte_pos, c) = chars[idx];
        if in_str {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                in_str = true;
                quote = c;
            }
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            '=' if depth == 0 => {
                let prev = if idx > 0 { Some(chars[idx - 1].1) } else { None };
                let next = chars.get(idx + 1).map(|x| x.1);
                if matches!(prev, Some('=' | '!' | '<' | '>')) || matches!(next, Some('=' | '>')) {
                    continue;
                }
                return Some((&code[..byte_pos], &code[byte_pos + 1..]));
            }
            _ => {}
        }
    }
    None
}

fn is_simple_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// تجنّب تصادم أسماء المتغيرات مع الكلمات المحجوزة في لسان.
fn sanitize_lisan_ident(name: &str) -> String {
    let clean = name.trim();
    if english_keyword_suggestion(clean).is_some() {
        return format!("متغير_{}", clean);
    }
    match clean {
        "حالة" | "افتراضي" | "خطأ" | "صحيح" | "لاشيء" | "فارغ" | "و" | "أو" | "في" | "من"
        | "مع" | "فئة" | "هيكل" | "صنف" | "دع" | "دالة" | "إذا" | "وإلا" => {
            format!("قيمة_{}", clean)
        }
        _ => clean.to_string(),
    }
}

fn extract_except_var(header: &str) -> String {
    if let Some((_, alias)) = split_top_level_keyword(header, "as") {
        let candidate = alias.trim();
        if is_simple_ident(candidate) {
            return sanitize_lisan_ident(candidate);
        }
    }
    "رسالة".to_string()
}

fn convert_py_params(params_str: &str, strip_self: bool) -> String {
    let raw_params = split_top_level_commas(params_str);
    let mut result = Vec::new();
    for (idx, p) in raw_params.iter().enumerate() {
        let trimmed = p.trim();
        if trimmed.is_empty() {
            continue;
        }
        if strip_self && idx == 0 && (trimmed == "self" || trimmed.starts_with("self:")) {
            continue;
        }
        // إزالة تعليقات الأنواع البسيطة x: int = 5 -> x = 5
        if let Some((lhs, rhs)) = split_simple_assign(trimmed) {
            let pname = lhs.split(':').next().unwrap_or(lhs).trim();
            result.push(format!(
                "{} = {}",
                sanitize_lisan_ident(pname),
                translate_py_expr(rhs.trim())
            ));
        } else if let Some(vararg) = trimmed.strip_prefix('*') {
            let vname = vararg.trim_start_matches('*').split(':').next().unwrap_or(vararg).trim();
            result.push(format!("*{}", sanitize_lisan_ident(vname)));
        } else {
            let pname = trimmed.split(':').next().unwrap_or(trimmed).trim();
            result.push(sanitize_lisan_ident(pname));
        }
    }
    result.join("، ")
}

fn extract_param_names(params_lisan: &str) -> Vec<String> {
    params_lisan
        .split('،')
        .filter_map(|p| {
            let t = p.trim().trim_start_matches('*');
            let name = t.split('=').next()?.trim();
            if name.is_empty() { None } else { Some(name.to_string()) }
        })
        .collect()
}

fn map_py_stdlib_module(name: &str) -> Option<&'static str> {
    match name {
        "math" => Some("رياضيات"),
        "statistics" => Some("إحصاء"),
        "json" => Some("جيسون"),
        "random" => Some("عشوائي"),
        "time" => Some("وقت"),
        "datetime" => Some("تاريخ"),
        "os" | "sys" => Some("نظام"),
        _ => None,
    }
}

fn map_py_stdlib_symbol(name: &str) -> Option<&'static str> {
    match name {
        "sqrt" => Some("جذر"),
        "sin" => Some("جا"),
        "cos" => Some("جتا"),
        "tan" => Some("ظا"),
        "log" => Some("لوغاريتم"),
        "exp" => Some("أسي"),
        "pi" => Some("ط"),
        "e" => Some("هـ"),
        "factorial" => Some("مضروب"),
        "gcd" => Some("قاسم_مشترك"),
        "lcm" => Some("مضاعف_مشترك"),
        "mean" => Some("متوسط"),
        "median" => Some("وسيط"),
        "mode" => Some("منوال"),
        "variance" => Some("تباين"),
        "stdev" => Some("انحراف_معياري"),
        _ => None,
    }
}

fn convert_py_import(rest: &str, notes: &mut Vec<String>) -> String {
    if let Some((mod_part, alias_part)) = split_top_level_keyword(rest, "as") {
        let m = mod_part.trim();
        let a = sanitize_lisan_ident(alias_part.trim());
        if let Some(ar_mod) = map_py_stdlib_module(m) {
            format!("استورد {} كـ {}", ar_mod, a)
        } else {
            notes.push(format!("تم تحويل المكتبة الخارجية '{}' لاستخدام استورد_بايثون داخل نص.", m));
            format!("دع {} = استورد_بايثون(\"{}\")", a, json_escape(m))
        }
    } else {
        let m = rest.trim();
        if let Some(ar_mod) = map_py_stdlib_module(m) {
            format!("استورد {}", ar_mod)
        } else {
            let ident = sanitize_lisan_ident(m.split('.').last().unwrap_or(m));
            notes.push(format!("تم تحويل المكتبة الخارجية '{}' لاستخدام استورد_بايثون داخل نص.", m));
            format!("دع {} = استورد_بايثون(\"{}\")", ident, json_escape(m))
        }
    }
}

fn convert_py_from_import(rest: &str, notes: &mut Vec<String>) -> String {
    if let Some((mod_part, items_part)) = split_top_level_keyword(rest, "import") {
        let m = mod_part.trim();
        let mapped_items: Vec<String> = items_part
            .split(',')
            .map(|item| {
                let it = item.trim();
                if let Some(ar) = map_py_stdlib_symbol(it) {
                    ar.to_string()
                } else {
                    sanitize_lisan_ident(it)
                }
            })
            .collect();
        if let Some(ar_mod) = map_py_stdlib_module(m) {
            format!("من {} استورد {}", ar_mod, mapped_items.join("، "))
        } else {
            notes.push(format!("تم تحويل استيراد '{}' من بايثون عبر استورد_بايثون.", m));
            let ident = sanitize_lisan_ident(m.split('.').last().unwrap_or(m));
            format!("دع {} = استورد_بايثون(\"{}\")", ident, json_escape(m))
        }
    } else {
        format!("// {}", rest)
    }
}

fn convert_py_print(inner: &str) -> String {
    let args = split_top_level_commas(inner);
    if args.is_empty() {
        return "اطبع(\"\")".into();
    }
    if args.len() == 1 {
        return format!("اطبع({})", translate_py_expr(&args[0]));
    }
    let parts: Vec<String> = args
        .iter()
        .map(|arg| {
            let translated = translate_py_expr(arg);
            if translated.starts_with('"') || translated.starts_with("ن\"") {
                translated
            } else {
                format!("نص({})", translated)
            }
        })
        .collect();
    format!("اطبع({})", parts.join(" + \" \" + "))
}

fn translate_py_statement_expr(code: &str) -> String {
    // تحويل list.append(x) -> أضف(list، x)
    if let Some(dot_idx) = code.find(".append(") {
        if code.ends_with(')') {
            let target = translate_py_expr(&code[..dot_idx]);
            let arg = translate_py_expr(&code[dot_idx + ".append(".len()..code.len() - 1]);
            return format!("أضف({}، {})", target, arg);
        }
    }
    translate_py_expr(code)
}

/// تحويل تعبير بايثون إلى تعبير لسان عربي خالص مع الحفاظ التام على محتوى النصوص.
fn translate_py_expr(expr: &str) -> String {
    let chars: Vec<char> = expr.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(expr.len() + 16);
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // معالجة f"..." أو f'...' -> ن"..."
        if (c == 'f' || c == 'F') && matches!(chars.get(i + 1), Some('"' | '\'')) {
            let quote = chars[i + 1];
            i += 2;
            out.push_str("ن\"");
            while i < len {
                let sc = chars[i];
                if sc == '\\' && i + 1 < len {
                    out.push(sc);
                    out.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                if sc == quote {
                    i += 1;
                    break;
                }
                if sc == '"' && quote == '\'' {
                    out.push_str("\\\"");
                } else {
                    out.push(sc);
                }
                i += 1;
            }
            out.push('"');
            continue;
        }

        // معالجة النصوص المقتبسة بـ "..." أو '...' -> "..."
        if c == '"' || c == '\'' {
            let quote = c;
            i += 1;
            out.push('"');
            while i < len {
                let sc = chars[i];
                if sc == '\\' && i + 1 < len {
                    out.push(sc);
                    out.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                if sc == quote {
                    i += 1;
                    break;
                }
                if sc == '"' && quote == '\'' {
                    out.push_str("\\\"");
                } else {
                    out.push(sc);
                }
                i += 1;
            }
            out.push('"');
            continue;
        }

        // معالجة القسمة الصحيحة // في التعابير -> /
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            out.push('/');
            i += 2;
            continue;
        }

        // معالجة الكلمات والمعرّفات
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let next_non_ws = chars[i..].iter().find(|ch| !ch.is_whitespace()).copied();
            let is_call = next_non_ws == Some('(');

            let replaced: String = if is_call {
                match word.as_str() {
                    "range" => "مدى".into(),
                    "len" => "طول".into(),
                    "str" => "نص".into(),
                    "int" => "عدد_صحيح".into(),
                    "float" => "عدد".into(),
                    "abs" => "مطلق".into(),
                    "min" => "أدنى".into(),
                    "max" => "أعلى".into(),
                    "sum" => "مجموع".into(),
                    "round" => "قرّب".into(),
                    "sorted" => "فرز".into(),
                    "reversed" => "عكس".into(),
                    "enumerate" => "مرقم".into(),
                    "zip" => "سحاب".into(),
                    "any" => "أي".into(),
                    "all" => "كل".into(),
                    "type" => "نوع".into(),
                    "input" => "ادخل".into(),
                    "open" => "افتح".into(),
                    "print" => "اطبع".into(),
                    "set" => "مجموعة".into(),
                    "tuple" => "صف".into(),
                    _ => word,
                }
            } else {
                match word.as_str() {
                    "True" => "صحيح".into(),
                    "False" => "خطأ".into(),
                    "None" => "لاشيء".into(),
                    "and" => "و".into(),
                    "or" => "أو".into(),
                    "not" => "ليس".into(),
                    "in" => "في".into(),
                    "for" => "لكل".into(),
                    "if" => "إذا".into(),
                    "self" => "هذا".into(),
                    _ => word,
                }
            };
            out.push_str(&replaced);
            continue;
        }

        // الفواصل اللاتينية في القوائم والوسائط -> فاصلة عربية `،`
        if c == ',' {
            out.push('،');
            i += 1;
            continue;
        }

        out.push(c);
        i += 1;
    }

    out
}

fn tool_lisan_from_python(args: &Value) -> (String, bool) {
    let py_code = match map_get_str(args, "python_code") {
        Some(c) => c,
        None => {
            return (
                r#"{"ok":false,"error":"المعامل 'python_code' مطلوب لتحويل شيفرة بايثون"}"#.into(),
                true,
            );
        }
    };

    let (lisan_code, notes) = convert_python_to_lisan(&py_code);
    let diags = run_lisan_diagnostics(&lisan_code);
    let valid_syntax = diags.is_empty();
    let note_items: Vec<String> = notes
        .iter()
        .map(|n| format!("\"{}\"", json_escape(n)))
        .collect();
    let diag_items: Vec<String> = diags
        .iter()
        .map(|d| format!("\"{}\"", json_escape(&d.message)))
        .collect();

    let json = format!(
        r#"{{"ok":true,"valid_syntax":{},"lisan_code":"{}","notes":[{}],"diagnostics":[{}]}}"#,
        if valid_syntax { "true" } else { "false" },
        json_escape(&lisan_code),
        note_items.join(","),
        diag_items.join(",")
    );
    (json, false)
}

// ============================================================================
// أدوات JSON-RPC المساعدة (صفر اعتماديات خارجية)
// ============================================================================

fn jsonrpc_result(id_json: &str, result_json: &str) -> String {
    format!(r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#, id_json, result_json)
}

fn jsonrpc_error(id_json: &str, code: i32, message: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":{},"message":"{}"}}}}"#,
        id_json,
        code,
        json_escape(message)
    )
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            other => out.push(other),
        }
    }
    out
}

fn map_get(val: &Value, key: &str) -> Option<Value> {
    if let Value::Map(pairs) = val {
        pairs
            .borrow()
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    } else {
        None
    }
}

fn map_get_str(val: &Value, key: &str) -> Option<String> {
    match map_get(val, key)? {
        Value::Text(s) => Some(s),
        _ => None,
    }
}

fn map_get_usize(val: &Value, key: &str) -> Option<usize> {
    match map_get(val, key)? {
        Value::Number(n) if n.is_finite() && n >= 0.0 => Some(n as usize),
        _ => None,
    }
}

fn extract_id_json(val: &Value) -> Option<String> {
    match map_get(val, "id")? {
        Value::Number(n) => {
            if n.fract() == 0.0 {
                Some(format!("{}", n as i64))
            } else {
                Some(format!("{}", n))
            }
        }
        Value::Text(s) => Some(format!("\"{}\"", json_escape(&s))),
        Value::Null => Some("null".into()),
        _ => None,
    }
}

// ============================================================================
// الاختبارات الشاملة لخادم MCP
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_tool_text(response_json: &str) -> (String, bool) {
        let parsed = parse_json(response_json).expect("يجب أن يكون رد MCP JSON صالحاً");
        let result = map_get(&parsed, "result").expect("يجب أن يحتوي الرد على حقل result");
        let is_error = match map_get(&result, "isError") {
            Some(Value::Bool(b)) => b,
            _ => false,
        };
        let content = map_get(&result, "content").expect("يجب أن يحتوي result على content");
        let text = match content {
            Value::List(items) => {
                let first = items.borrow().first().cloned().expect("content غير فارغ");
                map_get_str(&first, "text").expect("يجب أن يحتوي العنصر على text")
            }
            _ => panic!("content يجب أن يكون قائمة"),
        };
        (text, is_error)
    }

    #[test]
    fn test_mcp_initialize_and_notifications() {
        let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
        let resp = handle_mcp_request(init_req).expect("initialize يجب أن يعيد استجابة");
        let parsed = parse_json(&resp).unwrap();
        let result = map_get(&parsed, "result").unwrap();
        assert_eq!(map_get_str(&result, "protocolVersion").unwrap(), "2024-11-05");

        let notif = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        assert!(handle_mcp_request(notif).is_none());

        let ping = r#"{"jsonrpc":"2.0","id":"p1","method":"ping"}"#;
        let ping_resp = handle_mcp_request(ping).unwrap();
        assert!(ping_resp.contains(r#""id":"p1""#));
        assert!(ping_resp.contains(r#""result":{}"#));
    }

    #[test]
    fn test_mcp_tools_list() {
        let req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
        let resp = handle_mcp_request(req).unwrap();
        let parsed = parse_json(&resp).unwrap();
        let result = map_get(&parsed, "result").unwrap();
        let tools = match map_get(&result, "tools").unwrap() {
            Value::List(l) => l.borrow().clone(),
            _ => panic!("tools يجب أن تكون قائمة"),
        };
        assert_eq!(tools.len(), 6);
        let names: Vec<String> = tools.iter().filter_map(|t| map_get_str(t, "name")).collect();
        assert!(names.contains(&"lisan_run".to_string()));
        assert!(names.contains(&"lisan_check".to_string()));
        assert!(names.contains(&"lisan_format".to_string()));
        assert!(names.contains(&"lisan_transpile".to_string()));
        assert!(names.contains(&"lisan_docs".to_string()));
        assert!(names.contains(&"lisan_from_python".to_string()));
    }

    #[test]
    fn test_mcp_tool_lisan_run_success_and_error() {
        let ok_req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lisan_run","arguments":{"code":"دع س = 7\nاطبع(س * 6)"}}}"#;
        let resp = handle_mcp_request(ok_req).unwrap();
        let (text, is_error) = extract_tool_text(&resp);
        assert!(!is_error);
        let payload = parse_json(&text).unwrap();
        assert_eq!(map_get(&payload, "ok"), Some(Value::Bool(true)));
        assert_eq!(map_get_str(&payload, "output").unwrap(), "42");

        let err_req = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"lisan_run","arguments":{"code":"اطبع(غير_موجود)"}}}"#;
        let err_resp = handle_mcp_request(err_req).unwrap();
        let (err_text, is_err) = extract_tool_text(&err_resp);
        assert!(is_err);
        let err_payload = parse_json(&err_text).unwrap();
        assert_eq!(map_get(&err_payload, "ok"), Some(Value::Bool(false)));
    }

    #[test]
    fn test_mcp_tool_lisan_check_detects_english_keywords() {
        let bad_req = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"lisan_check","arguments":{"code":"if (x > 0) {\n    print(x)\n}"}}}"#;
        let resp = handle_mcp_request(bad_req).unwrap();
        let (text, is_error) = extract_tool_text(&resp);
        assert!(!is_error);
        let payload = parse_json(&text).unwrap();
        assert_eq!(map_get(&payload, "ok"), Some(Value::Bool(false)));
        assert!(text.contains("إذا"));
        assert!(text.contains("اطبع"));

        let good_req = r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"lisan_check","arguments":{"code":"دع س = 10\nإذا (س > 5) {\n    اطبع(س)\n}"}}}"#;
        let good_resp = handle_mcp_request(good_req).unwrap();
        let (good_text, _) = extract_tool_text(&good_resp);
        let good_payload = parse_json(&good_text).unwrap();
        assert_eq!(map_get(&good_payload, "ok"), Some(Value::Bool(true)));
    }

    #[test]
    fn test_mcp_tool_lisan_format() {
        let req = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"lisan_format","arguments":{"code":"دع  س=5\nإذا(س>2){\nاطبع(س)\n}"}}}"#;
        let resp = handle_mcp_request(req).unwrap();
        let (text, is_error) = extract_tool_text(&resp);
        assert!(!is_error);
        let payload = parse_json(&text).unwrap();
        let formatted = map_get_str(&payload, "formatted").unwrap();
        assert!(formatted.contains("دع س = 5"));
        assert!(formatted.contains("    اطبع(س)"));
    }

    #[test]
    fn test_mcp_tool_lisan_transpile_all_targets() {
        for (target, expected_lang) in [
            ("arduino", "cpp"),
            ("esp32", "cpp"),
            ("stm32", "cpp"),
            ("raspberry", "python"),
            ("pico", "micropython"),
        ] {
            let req = format!(
                r#"{{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{{"name":"lisan_transpile","arguments":{{"code":"دع ع = 1\nاطبع(ع)","target":"{}"}}}}}}"#,
                target
            );
            let resp = handle_mcp_request(&req).unwrap();
            let (text, is_error) = extract_tool_text(&resp);
            assert!(!is_error, "فشلت ترجمة الهدف {}: {}", target, text);
            let payload = parse_json(&text).unwrap();
            assert_eq!(map_get(&payload, "ok"), Some(Value::Bool(true)));
            assert_eq!(map_get_str(&payload, "output_language").unwrap(), expected_lang);
            assert!(!map_get_str(&payload, "code").unwrap().is_empty());
        }
    }

    #[test]
    fn test_mcp_tool_lisan_docs() {
        let req = r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"lisan_docs","arguments":{"query":"robotics"}}}"#;
        let resp = handle_mcp_request(req).unwrap();
        let (text, is_error) = extract_tool_text(&resp);
        assert!(!is_error);
        assert!(text.contains("متحكم_pid"));
        assert!(text.contains("مرشح_كالمان"));

        let req_all = r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"lisan_docs","arguments":{}}}"#;
        let resp_all = handle_mcp_request(req_all).unwrap();
        let (text_all, _) = extract_tool_text(&resp_all);
        assert!(text_all.contains("microcontrollers"));
    }

    #[test]
    fn test_mcp_tool_lisan_from_python_and_execute() {
        let py = r#"# حساب مجموع الأعداد
def double_sum(n):
    total = 0
    for i in range(n):
        if i % 2 == 0:
            total += i
        elif i == 3:
            total += 10
        else:
            total += 1
    return total

print(double_sum(5))
"#;
        let req = format!(
            r#"{{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{{"name":"lisan_from_python","arguments":{{"python_code":"{}"}}}}}}"#,
            json_escape(py)
        );
        let resp = handle_mcp_request(&req).unwrap();
        let (text, is_error) = extract_tool_text(&resp);
        assert!(!is_error);
        let payload = parse_json(&text).unwrap();
        assert_eq!(map_get(&payload, "valid_syntax"), Some(Value::Bool(true)));
        let lisan_code = map_get_str(&payload, "lisan_code").unwrap();
        assert!(lisan_code.contains("دالة double_sum(n)"));
        assert!(lisan_code.contains("لكل (i في مدى(n))"));
        assert!(lisan_code.contains("أرجع total"));
        assert!(lisan_code.contains("// حساب مجموع الأعداد"));

        // التحقق الفعلي من تشغيل الشيفرة المحوّلة عبر مفسر لسان!
        // i=0 (+0), i=1 (+1), i=2 (+2), i=3 (+10), i=4 (+4) => 17
        let out = Interpreter::with_limits(Limits::sandbox())
            .run_source(&lisan_code)
            .expect("يجب أن تعمل الشيفرة المحوّلة من بايثون دون أي خطأ");
        assert_eq!(out, vec!["17"]);
    }

    #[test]
    fn test_mcp_resources_list_and_read() {
        let list_req = r#"{"jsonrpc":"2.0","id":12,"method":"resources/list"}"#;
        let list_resp = handle_mcp_request(list_req).unwrap();
        assert!(list_resp.contains("lisan://llms.txt"));

        let read_req = r#"{"jsonrpc":"2.0","id":13,"method":"resources/read","params":{"uri":"lisan://llms.txt"}}"#;
        let read_resp = handle_mcp_request(read_req).unwrap();
        let parsed = parse_json(&read_resp).unwrap();
        let result = map_get(&parsed, "result").unwrap();
        let contents = match map_get(&result, "contents").unwrap() {
            Value::List(l) => l.borrow().clone(),
            _ => panic!("contents يجب أن تكون قائمة"),
        };
        assert_eq!(contents.len(), 1);
        let txt = map_get_str(&contents[0], "text").unwrap();
        assert!(txt.contains("100% Arabic-Only Keywords"));
        assert!(txt.contains("lisan --mcp"));
    }

    #[test]
    fn test_mcp_tool_lisan_from_python_class_try_imports_and_logic() {
        let py = r#"import math as m
import numpy as np

class Counter:
    def __init__(self, start):
        self.value = start

    def step(self):
        self.value += 1
        return self.value

c = Counter(5)
active = True
if active and not False:
    print("val:", c.step())

try:
    x = 1
except Exception as err:
    print(err)
"#;
        let (lisan_code, notes) = convert_python_to_lisan(py);
        assert!(lisan_code.contains("استورد رياضيات كـ m"));
        assert!(lisan_code.contains(r#"دع np = استورد_بايثون("numpy")"#));
        assert!(lisan_code.contains("صنف Counter"));
        assert!(lisan_code.contains("دالة بناء(start)"));
        assert!(lisan_code.contains("هذا.value = start"));
        assert!(lisan_code.contains("دع active = صحيح"));
        assert!(lisan_code.contains("و ليس خطأ"));
        assert!(lisan_code.contains("حاول {"));
        assert!(lisan_code.contains("} التقط (err) {"));
        assert!(!notes.is_empty());

        // التحقق من صحة البناء النحوي للشيفرة المحوّلة بالكامل
        let diags = run_lisan_diagnostics(&lisan_code);
        assert!(diags.is_empty(), "أخطاء نحوية غير متوقعة: {:?}\n{}", diags, lisan_code);
    }

    #[test]
    fn test_mcp_check_catches_from_as_and_all_english_keywords() {
        let code = "from math import sqrt as s\nlet a = true\nvar b = false\n";
        let diags = run_lisan_diagnostics(code);
        let found: Vec<String> = diags.iter().filter_map(|d| d.found.clone()).collect();
        assert!(found.contains(&"from".to_string()));
        assert!(found.contains(&"import".to_string()));
        assert!(found.contains(&"as".to_string()));
        assert!(found.contains(&"let".to_string()));
        assert!(found.contains(&"true".to_string()));
        assert!(found.contains(&"var".to_string()));
        assert!(found.contains(&"false".to_string()));
    }

    #[test]
    fn test_mcp_jsonrpc_error_codes() {
        let parse_err = handle_mcp_request("{not valid json").unwrap();
        assert!(parse_err.contains("-32700"));

        let method_err = handle_mcp_request(r#"{"jsonrpc":"2.0","id":99,"method":"unknown/op"}"#).unwrap();
        assert!(method_err.contains("-32601"));
        assert!(method_err.contains(r#""id":99"#));

        let res_err = handle_mcp_request(r#"{"jsonrpc":"2.0","id":100,"method":"resources/read","params":{"uri":"lisan://missing"}}"#).unwrap();
        assert!(res_err.contains("-32002"));
    }
}
