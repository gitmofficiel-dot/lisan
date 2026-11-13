use lisan::Interpreter;

fn run(source: &str) -> Result<Vec<String>, String> {
    Interpreter::default().run_source(source)
}

#[test]
fn runtime_error_reports_exact_line_and_column() {
    let src = "دع س = 10\nدع ص = 0\nاطبع(س / ص)";
    let err = run(src).unwrap_err();
    assert!(err.contains("السطر 3"), "Expected line 3 in error: {err}");
    assert!(err.contains("القسمة على صفر"), "Expected zero division in error: {err}");
    assert!(err.contains("خطأ تشغيل:"), "Expected runtime error prefix: {err}");
}

#[test]
fn multi_frame_call_stack_traceback() {
    let src = r#"
دالة ج() {
    أرجع 1 / 0
}

دالة ب() {
    أرجع ج()
}

دالة أ() {
    ب()
}

أ()
"#;
    let err = run(src).unwrap_err();
    assert!(err.contains("تتبع مسار التنفيذ (آخر استدعاء هو الأحدث):"), "Traceback header missing: {err}");
    assert!(err.contains("في «أ»"), "Frame أ missing: {err}");
    assert!(err.contains("في «ب»"), "Frame ب missing: {err}");
    assert!(err.contains("القسمة على صفر"), "Original error message missing: {err}");
}

#[test]
fn class_method_traceback() {
    let src = r#"
صنف حساب {
    دالة اقسم(س، ص) {
        أرجع س / ص
    }
}

دع ح = حساب()
ح.اقسم(10، 0)
"#;
    let err = run(src).unwrap_err();
    assert!(err.contains("تتبع مسار التنفيذ"), "Traceback header missing: {err}");
    assert!(err.contains("حساب.اقسم"), "Method frame missing: {err}");
    assert!(err.contains("القسمة على صفر"), "Error text missing: {err}");
}

#[test]
fn try_catch_handles_error_and_resets_call_stack() {
    let src = r#"
دالة قد_تخفق() {
    ارمِ "خطأ_مخصص"
}

دع تم_الالتقاط = خطأ
حاول {
    قد_تخفق()
} التقط (خ) {
    إذا (خ == "خطأ_مخصص") {
        تم_الالتقاط = صحيح
    }
}

اطبع(تم_الالتقاط)
"#;
    let out = run(src).unwrap();
    assert_eq!(out, ["صحيح"]);
}
