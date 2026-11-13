//! اختبارات حدود التنفيذ: وضع الصندوق الآمن مقابل الوضع الكامل للبرامج الحقيقية.
use lisan::{Interpreter, Limits};

fn run_full(source: &'static str) -> Result<Vec<String>, String> {
    lisan::runtime::run_with_large_stack(move || {
        Interpreter::with_limits(Limits::full()).run_source(source)
    })
}

#[test]
fn sandbox_still_stops_infinite_loops() {
    let err = Interpreter::default()
        .run_source("طالما (صحيح) { }")
        .unwrap_err();
    assert!(err.contains("خطوات"));
}

#[test]
fn full_mode_runs_long_loops_beyond_sandbox_step_limit() {
    let out = run_full(
        "دع مجموع = 0\nدع ع = 0\nطالما (ع < 300000) { مجموع = مجموع + ع؛ ع = ع + 1 }\nاطبع(مجموع)",
    )
    .unwrap();
    assert_eq!(out, ["44999850000"]);
}

#[test]
fn full_mode_allows_python_like_recursion_depth() {
    let out = run_full(
        "دالة عمق(ن) { إذا (ن == 0) { أرجع 0 } أرجع 1 + عمق(ن - 1) }\nاطبع(عمق(900))",
    )
    .unwrap();
    assert_eq!(out, ["900"]);
}

#[test]
fn full_mode_still_reports_runaway_recursion() {
    let err = run_full("دالة لانهائي() { أرجع لانهائي() } لانهائي()").unwrap_err();
    assert!(err.contains("عمق"));
}

#[test]
fn full_mode_allows_large_lists() {
    let out = run_full("دع ك = مدى(200000)\nاطبع(طول(ك))").unwrap();
    assert_eq!(out, ["200000"]);
}

#[test]
fn sandbox_list_limit_is_restored_after_full_run() {
    run_full("دع ك = مدى(20000)").unwrap();
    let err = Interpreter::default().run_source("دع س = مدى(100001)").unwrap_err();
    assert!(err.contains("الحد الأقصى"));
}
