use lisan::dap::DapServer;
use lisan::diagnostics::{diagnose_source, diagnostics_to_json};
use lisan::interop::{transpile_to_js, transpile_to_python};
use lisan::lsp::handle_json_rpc;
use lisan::mcp::handle_mcp_request;
use lisan::runtime::{Interpreter, Limits};

#[test]
fn test_arabic_only_core_and_structured_diagnostics() {
    // 1) English keywords 'as' and 'from' are no longer accepted by the lexer/parser
    let bad_as = r#"استورد رياضيات as ر"#;
    assert!(lisan::run(bad_as).is_err(), "English 'as' must be rejected");
    let diags_as = diagnose_source(bad_as);
    assert!(
        diags_as.iter().any(|d| d.code == "ل0001" && d.replacement.as_deref() == Some("كـ")),
        "Expected ل0001 suggesting كـ, got: {:?}",
        diags_as
    );

    // 2) Accidental English keywords (if/def/return) produce structured JSON with exact Arabic fixes
    let bad_py = "def foo():\n    if true:\n        return 1\n";
    let diags = diagnose_source(bad_py);
    assert!(diags.iter().any(|d| d.code == "ل0001" && d.replacement.as_deref() == Some("دالة")));
    assert!(diags.iter().any(|d| d.code == "ل0001" && d.replacement.as_deref() == Some("إذا")));
    assert!(diags.iter().any(|d| d.code == "ل0001" && d.replacement.as_deref() == Some("أرجع")));
    let json = diagnostics_to_json(&diags);
    assert!(json.contains("\"code\":\"ل0001\""));
    assert!(json.contains("\"replacement\":\"إذا\""));

    // 3) Legacy Latin-segment builtin produces polite warning ل0004 suggesting pure Arabic name
    let legacy = r#"دع ب = بصمة_crc32("مرحبا")"#;
    let diags_legacy = diagnose_source(legacy);
    assert!(
        diags_legacy.iter().any(|d| d.code == "ل0004" && d.severity == "warning" && d.replacement.as_deref() == Some("بصمة_تحقق")),
        "Expected ل0004 warning suggesting بصمة_تحقق, got: {:?}",
        diags_legacy
    );
}

#[test]
fn test_pure_arabic_aliases_execute_at_runtime() {
    let code = r#"
استورد "ذكاء"
استورد "روبوت"

دع ب = بصمة_تحقق("لسان")
تحقق(طول(ب) == 8)

دع مضغوط = ضغط_تكراري("أأأأبب")
تحقق_يساوي(فك_ضغط_تكراري(مضغوط)، "أأأأبب")

دع ت = تجزئة_آمنة("لسان")
تحقق(طول(ت) == 64)

دع نص_ج = ترميز_جيسون({"لغة": "لسان"، "اصدار": 2})
دع كائن_ج = تحليل_جيسون(نص_ج)
تحقق_يساوي(كائن_ج["لغة"]، "لسان")

دع مصنف = جديد مصنف_الجيران_الأقرب(1)
مصنف.درّب([[0، 0]، [10، 10]]، ["أ"، "ب"])
تحقق_يساوي(مصنف.توقّع([1، 1])، "أ")

دع متحكم = جديد متحكم_تناسبي_تكاملي(2.0، 0.0، 0.0)
متحكم.الهدف = 10
دع خرج = متحكم.احسب(5، 0.1)
تحقق(خرج > 0)

اطبع("نجحت الأسماء العربية الخالصة")
"#;
    let out = Interpreter::with_limits(Limits::full()).run_source(code).expect("pure Arabic aliases must run");
    assert_eq!(out, vec!["نجحت الأسماء العربية الخالصة"]);
}

#[test]
fn test_gradual_type_annotations_and_match_sugar() {
    let valid = r#"
دع اسم: نص = "لسان"
دع رقم_الإصدار: عدد = 2
دع جاهز: منطقي = صحيح
دع أرقام: قائمة[عدد] = [1، 2، 3]

دالة جمع(أ: عدد، ب: عدد = 5) -> عدد {
    أرجع أ + ب
}

دالة تحية(شخص: نص) ← نص {
    أرجع "أهلاً " + شخص
}

طابق اسم {
    حالة "لسان": اطبع(تحية(اسم))
    افتراضي: اطبع("غير معروف")
}
اطبع(جمع(10، 20))
"#;
    let diags = diagnose_source(valid);
    assert!(diags.is_empty(), "Expected zero diagnostics for valid typed code, got: {:?}", diags);
    let out = lisan::run(valid).expect("typed Lisan code must run");
    assert_eq!(out, vec!["أهلاً لسان", "30"]);

    // Type mismatch detection (ل0003)
    let mismatch = r#"
دع س: عدد = "هذا نص وليس عدداً"
دالة ف(أ: عدد) -> عدد {
    أرجع "نص خاطئ"
}
"#;
    let err_diags = diagnose_source(mismatch);
    let type_errors: Vec<_> = err_diags.iter().filter(|d| d.code == "ل0003").collect();
    assert_eq!(type_errors.len(), 2, "Expected 2 ل0003 type errors, got: {:?}", err_diags);
}

#[test]
fn test_pure_arabic_python_import_and_transpilers() {
    let code = r#"
استورد بايثون "math" كـ رياضيات_ب
دع جذر_25 = رياضيات_ب.استدعِ("sqrt"، [25])
اطبع(جذر_25)
"#;
    // Only assert live Python execution if python3 is installed on the host
    let py_check = lisan::run("اطبع(بايثون_متوفر())").unwrap();
    if py_check.first().map(|s| s.as_str()) == Some("صحيح") {
        let out = Interpreter::with_limits(Limits::full()).run_source(code).expect("live Python import must run");
        assert_eq!(out, vec!["5"]);
    }

    // Verify Python & JS transpilers work on pure Arabic typed code
    let sample = r#"
دالة مربع(س: عدد) -> عدد {
    أرجع س * س
}
اطبع(مربع(7))
"#;
    let py = transpile_to_python(sample).expect("transpile_to_python must succeed");
    assert!(py.contains("def مربع"));
    let js = transpile_to_js(sample).expect("transpile_to_js must succeed");
    assert!(js.contains("function مربع"));
}

#[test]
fn test_mcp_dap_and_enhanced_lsp_end_to_end() {
    // 1) MCP: transpile to python & js + resources/read
    let req_py = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"lisan_transpile","arguments":{"code":"دع س = 5\nاطبع(س)","target":"python"}}}"#;
    let resp_py = handle_mcp_request(req_py).expect("MCP response expected");
    assert!(resp_py.contains("\"ok\":true") || resp_py.contains("\\\"ok\\\":true"));
    assert!(resp_py.contains("python"));

    let req_js = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"lisan_transpile","arguments":{"code":"دع س = 5\nاطبع(س)","target":"js"}}}"#;
    let resp_js = handle_mcp_request(req_js).expect("MCP JS response expected");
    assert!(resp_js.contains("javascript"));

    let req_res = r#"{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"lisan://llms.txt"}}"#;
    let resp_res = handle_mcp_request(req_res).expect("MCP resource expected");
    assert!(resp_res.contains("لسان"));

    // 2) Enhanced LSP: didOpen -> hover, documentSymbol, codeAction (English keyword quick-fix), formatting
    let open_req = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.lsn","languageId":"lisan","version":1,"text":"دالة جمع(أ، ب) {\nأرجع أ + ب\n}\nif (1) { اطبع(1) }"}}}"#;
    let diag_notif = handle_json_rpc(open_req).expect("diagnostics notification");
    assert!(diag_notif.contains("ل0001"));

    let sym_req = r#"{"jsonrpc":"2.0","id":10,"method":"textDocument/documentSymbol","params":{"textDocument":{"uri":"file:///test.lsn"}}}"#;
    let sym_resp = handle_json_rpc(sym_req).expect("documentSymbol response");
    assert!(sym_resp.contains("جمع"));

    let action_req = r#"{"jsonrpc":"2.0","id":11,"method":"textDocument/codeAction","params":{"textDocument":{"uri":"file:///test.lsn"}}}"#;
    let action_resp = handle_json_rpc(action_req).expect("codeAction response");
    assert!(action_resp.contains("إذا"));

    // 3) DAP: initialize -> disconnect
    let mut dap = DapServer::new();
    let init_msgs = dap.handle_message(r#"{"seq":1,"type":"request","command":"initialize","arguments":{}}"#);
    assert_eq!(init_msgs.len(), 2);
    let init_resp = lisan::json::parse_json(&init_msgs[0]).expect("valid DAP JSON response");
    let init_evt = lisan::json::parse_json(&init_msgs[1]).expect("valid DAP JSON event");
    assert!(init_resp.to_string().contains("initialize"));
    assert!(init_evt.to_string().contains("initialized"));
}
