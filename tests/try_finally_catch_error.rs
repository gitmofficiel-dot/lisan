use lisan::{run, Interpreter};

#[test]
fn finally_runs_when_catch_rethrows_and_propagates_error() {
    let nested_src = r#"
دع تم_التنظيف = خطأ
حاول {
    حاول {
        ارمِ "خطأ داخلي"
    } التقط (رسالة) {
        ارمِ "إعادة رمي: " + رسالة
    } أخيرًا {
        تم_التنظيف = صحيح
    }
} التقط (خارجي) {
    اطبع(خارجي)
}
اطبع(تم_التنظيف)
"#;
    assert_eq!(run(nested_src).unwrap(), ["إعادة رمي: خطأ داخلي", "صحيح"]);

    let mut interp = Interpreter::default();
    let err = interp
        .run_source(
            r#"
دع تنظيف_خارجي = خطأ
حاول {
    ارمِ "أول"
} التقط (ر) {
    ارمِ "ثاني"
} أخيرًا {
    تنظيف_خارجي = صحيح
}
"#,
        )
        .unwrap_err();
    assert!(err.contains("ثاني"), "expected rethrown error, got: {err}");
    assert_eq!(interp.run_source("اطبع(تنظيف_خارجي)").unwrap(), ["صحيح"]);
}

#[test]
fn finally_error_supersedes_pending_catch_error() {
    let src = r#"
حاول {
    حاول {
        ارمِ "خطأ_أول"
    } التقط (ر) {
        اطبع(ر)
        ارمِ "خطأ_ثاني"
    } أخيرًا {
        اطبع("تنفيذ_أخير")
        ارمِ "خطأ_نهائي"
    }
} التقط (نهائي) {
    اطبع(نهائي)
}
"#;
    assert_eq!(run(src).unwrap(), ["خطأ_أول", "تنفيذ_أخير", "خطأ_نهائي"]);
}

#[test]
fn finally_return_overrides_pending_catch_error() {
    let src = r#"
دالة تجربة() {
    حاول {
        ارمِ "خطأ_أول"
    } التقط (ر) {
        ارمِ "خطأ_ثاني"
    } أخيرًا {
        أرجع "قيمة_نهائية"
    }
}
اطبع(تجربة())
"#;
    assert_eq!(run(src).unwrap(), ["قيمة_نهائية"]);
}
