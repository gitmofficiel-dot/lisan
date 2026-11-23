use lisan::diagnostics::diagnose_source;
use lisan::runtime::{Interpreter, Limits};
use lisan::vm::VM;

#[test]
fn test_async_keywords_and_await_desugaring() {
    let code = r#"
استورد "تزامن"

دالة_غير_متزامنة احسب_مربع(س: عدد) -> عدد {
    أرجع مهمة_منجزة(س * س)
}

غير_متزامنة دالة اجمع(أ: عدد، ب: عدد) -> عدد {
    أرجع أ + ب
}

دع ن1 = انتظر احسب_مربع(9)
دع ن2 = انتظر اجمع(10، 20)
اطبع(ن"{ن1}|{ن2}")
"#;
    let diags = diagnose_source(code);
    assert!(
        diags.iter().all(|d| d.severity != "error"),
        "Async code must have zero errors: {:?}",
        diags
    );
    let out = Interpreter::with_limits(Limits::full())
        .run_source(code)
        .expect("async/await must execute cleanly");
    assert_eq!(out, vec!["81|30"]);
}

#[test]
fn test_vm_match_and_pure_arabic_aliases() {
    let code = r#"
دع اليوم = "الجمعة"
طابق اليوم {
    حالة "السبت": اطبع("بداية")
    حالة "الجمعة": اطبع("عطلة سعيدة")
    افتراضي: اطبع("يوم عمل")
}
دع ب = بصمة_تحقق("لسان")
اطبع(طول(ب))
"#;
    let mut vm = VM::new();
    let out = vm.run_source(code).expect("VM must execute طابق and pure Arabic alias");
    assert_eq!(out, vec!["عطلة سعيدة", "8"]);
}

#[test]
fn test_all_wave2_examples_execute_cleanly() {
    let examples = [
        "examples/async_and_ffi.lsn",
        "examples/rag_agent_arabic.lsn",
        "examples/reactive_web_and_capacitor.lsn",
    ];
    for path in examples {
        let src = std::fs::read_to_string(path).expect(path);
        let diags = diagnose_source(&src);
        let errors: Vec<_> = diags.iter().filter(|d| d.severity == "error").collect();
        assert!(errors.is_empty(), "Example {} had diagnostics errors: {:?}", path, errors);

        let out = Interpreter::with_limits(Limits::full())
            .run_file(path)
            .unwrap_or_else(|e| panic!("Example {} failed to run: {}", path, e));
        assert!(!out.is_empty(), "Example {} produced no output", path);
    }
    let _ = std::fs::remove_dir_all("مخرجات_كاباسيتور");
    let _ = std::fs::remove_dir_all("مخرجات_تاوري");
}
