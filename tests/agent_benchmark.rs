//! مجموعة الاختبار الآلي لمعيار وكلاء الذكاء الاصطناعي بلغة لسان (30 مهمة عربية خالصة 100%).
//!
//! تحمّل هذه المجموعة ملف `benchmarks/agent_benchmark.json` وتنفّذ جميع المهام الـ 30
//! عبر المفسّر ومدقق العربية فقط (`ل0001`/`ل0003`/`ل0004`) ومحوّل `lisan_from_python`
//! والمترجمات متعددة الأهداف، وتتحقق من تحقيق نسبة نجاح 100% (30/30).

use lisan::arduino::{transpile_arduino, transpile_esp32, transpile_stm32};
use lisan::diagnostics::{diagnose_source, lint_arabic_only};
use lisan::interop::{transpile_to_js, transpile_to_python};
use lisan::json::parse_json;
use lisan::mcp::{convert_python_to_lisan, handle_mcp_request};
use lisan::pico::transpile_pico_micropython;
use lisan::raspberry::transpile_raspberry_python;
use lisan::runtime::{Interpreter, Limits, Value};
use std::collections::HashSet;

fn map_get<'a>(val: &'a Value, key: &str) -> Option<Value> {
    if let Value::Map(m) = val {
        let borrowed = m.borrow();
        for (k, v) in borrowed.iter() {
            if k == key {
                return Some(v.clone());
            }
        }
    }
    None
}

fn map_get_str(val: &Value, key: &str) -> Option<String> {
    match map_get(val, key) {
        Some(Value::Text(s)) => Some(s),
        _ => None,
    }
}

fn map_get_str_list(val: &Value, key: &str) -> Vec<String> {
    match map_get(val, key) {
        Some(Value::List(items)) => items
            .borrow()
            .iter()
            .filter_map(|v| match v {
                Value::Text(s) => Some(s.clone()),
                other => Some(other.to_string()),
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn escape_json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn run_single_task(task: &Value) -> Result<(), String> {
    let id = map_get_str(task, "id").ok_or("المهمة تفتقد الحقل id")?;
    let mode = map_get_str(task, "mode").ok_or_else(|| format!("{}: يفتقد الحقل mode", id))?;

    match mode.as_str() {
        "run" => {
            let code = map_get_str(task, "code").ok_or_else(|| format!("{}: يفتقد code", id))?;
            let expected = map_get_str_list(task, "expected_output");

            // التحقق من قاعدة العربية فقط (صفر أخطاء ل0001)
            let lints = lint_arabic_only(&code);
            let eng_errs: Vec<_> = lints.iter().filter(|d| d.code == "ل0001").collect();
            if !eng_errs.is_empty() {
                return Err(format!("{}: يحتوي كلمات إنجليزية (ل0001): {:?}", id, eng_errs));
            }

            let actual = Interpreter::with_limits(Limits::full())
                .run_source(&code)
                .map_err(|e| format!("{}: فشل التشغيل: {}", id, e))?;
            if actual != expected {
                return Err(format!(
                    "{}: ناتج غير مطابق. المتوقع {:?}، الفعلي {:?}",
                    id, expected, actual
                ));
            }
            Ok(())
        }
        "lint_clean" => {
            let code = map_get_str(task, "code").ok_or_else(|| format!("{}: يفتقد code", id))?;
            let expected = map_get_str_list(task, "expected_output");

            let diags = diagnose_source(&code);
            if !diags.is_empty() {
                return Err(format!(
                    "{}: متوقع صفر تشخيصات لكن وُجدت: {:?}",
                    id, diags
                ));
            }

            let actual = Interpreter::with_limits(Limits::full())
                .run_source(&code)
                .map_err(|e| format!("{}: فشل التشغيل: {}", id, e))?;
            if actual != expected {
                return Err(format!(
                    "{}: ناتج غير مطابق. المتوقع {:?}، الفعلي {:?}",
                    id, expected, actual
                ));
            }
            Ok(())
        }
        "lint_error" => {
            let code = map_get_str(task, "code").ok_or_else(|| format!("{}: يفتقد code", id))?;
            let expected_codes = map_get_str_list(task, "expected_codes");
            let expected_repls = map_get_str_list(task, "expected_replacements");

            let diags = diagnose_source(&code);
            let actual_codes: Vec<String> = diags.iter().map(|d| d.code.clone()).collect();
            for code_tag in &expected_codes {
                let want_count = expected_codes.iter().filter(|c| *c == code_tag).count();
                let got_count = actual_codes.iter().filter(|c| *c == code_tag).count();
                if got_count < want_count {
                    return Err(format!(
                        "{}: عدد التشخيصات للرمز '{}' أقل من المتوقع (متوقع {}، فعلي {}): {:?}",
                        id, code_tag, want_count, got_count, diags
                    ));
                }
            }
            for repl in &expected_repls {
                if !diags.iter().any(|d| d.replacement.as_deref() == Some(repl.as_str())) {
                    return Err(format!(
                        "{}: لم يُقترح البديل '{}' في التشخيصات: {:?}",
                        id, repl, diags
                    ));
                }
            }
            Ok(())
        }
        "from_python" => {
            let py_code = map_get_str(task, "python_code")
                .ok_or_else(|| format!("{}: يفتقد python_code", id))?;
            let expected = map_get_str_list(task, "expected_output");

            // 1) اختبار الدالة المباشرة convert_python_to_lisan
            let (lisan_code, _notes) = convert_python_to_lisan(&py_code);
            let lints = lint_arabic_only(&lisan_code);
            let eng_errs: Vec<_> = lints.iter().filter(|d| d.code == "ل0001").collect();
            if !eng_errs.is_empty() {
                return Err(format!(
                    "{}: الشيفرة المحوّلة من بايثون ما زالت تحوي كلمات إنجليزية: {:?}\nالكود:\n{}",
                    id, eng_errs, lisan_code
                ));
            }

            let actual = Interpreter::with_limits(Limits::full())
                .run_source(&lisan_code)
                .map_err(|e| format!("{}: فشل تشغيل الشيفرة المحوّلة:\n{}\nالخطأ: {}", id, lisan_code, e))?;
            if actual != expected {
                return Err(format!(
                    "{}: ناتج المحوّل غير مطابق. المتوقع {:?}، الفعلي {:?}\nالكود:\n{}",
                    id, expected, actual, lisan_code
                ));
            }

            // 2) اختبار أداة MCP JSON-RPC `lisan_from_python`
            let mcp_req = format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"lisan_from_python","arguments":{{"python_code":"{}"}}}}}}"#,
                escape_json_str(&py_code)
            );
            let mcp_resp = handle_mcp_request(&mcp_req)
                .ok_or_else(|| format!("{}: لم يُرجع خادم MCP استجابة لـ lisan_from_python", id))?;
            if !(mcp_resp.contains("\"ok\":true") || mcp_resp.contains("\\\"ok\\\":true")) {
                return Err(format!(
                    "{}: استجابة MCP لأداة lisan_from_python غير ناجحة: {}",
                    id, mcp_resp
                ));
            }
            Ok(())
        }
        "transpile" => {
            let code = map_get_str(task, "code").ok_or_else(|| format!("{}: يفتقد code", id))?;
            let target =
                map_get_str(task, "target").ok_or_else(|| format!("{}: يفتقد target", id))?;
            let expected_contains = map_get_str_list(task, "expected_contains");

            let lints = lint_arabic_only(&code);
            let eng_errs: Vec<_> = lints.iter().filter(|d| d.code == "ل0001").collect();
            if !eng_errs.is_empty() {
                return Err(format!("{}: مصدر الترجمة يحتوي كلمات إنجليزية: {:?}", id, eng_errs));
            }

            let out = match target.as_str() {
                "python" => transpile_to_python(&code),
                "js" => transpile_to_js(&code),
                "arduino" => transpile_arduino(&code),
                "esp32" => transpile_esp32(&code),
                "stm32" => transpile_stm32(&code),
                "raspberry" => transpile_raspberry_python(&code),
                "pico" => transpile_pico_micropython(&code),
                other => Err(format!("{}: هدف ترجمة غير معروف '{}'", id, other)),
            }
            .map_err(|e| format!("{}: فشلت الترجمة إلى {}: {}", id, target, e))?;

            for needle in &expected_contains {
                if !out.contains(needle.as_str()) {
                    return Err(format!(
                        "{}: مخرج الترجمة إلى {} لا يحتوي '{}'\nالمخرج:\n{}",
                        id, target, needle, out
                    ));
                }
            }
            Ok(())
        }
        other => Err(format!("{}: نمط مهمة غير معروف '{}'", id, other)),
    }
}

#[test]
fn test_agent_benchmark_30_tasks_100_percent_pass_rate() {
    let raw = std::fs::read_to_string("benchmarks/agent_benchmark.json")
        .expect("يجب أن يتوفر ملف benchmarks/agent_benchmark.json");
    let root = parse_json(&raw).expect("يجب أن يكون ملف benchmarks/agent_benchmark.json بصيغة JSON صالحة");

    let total_declared = match map_get(&root, "total_tasks") {
        Some(Value::Number(n)) => n as usize,
        other => panic!("الحقل total_tasks غير صالح: {:?}", other),
    };
    assert_eq!(total_declared, 30, "المعيار يجب أن يعلن عن 30 مهمة");

    let tasks = match map_get(&root, "tasks") {
        Some(Value::List(list)) => list.borrow().clone(),
        other => panic!("الحقل tasks يجب أن يكون قائمة: {:?}", other),
    };
    assert_eq!(tasks.len(), 30, "يجب أن يحتوي المعيار على 30 مهمة بالضبط");

    let mut seen_ids = HashSet::new();
    let mut seen_categories = HashSet::new();
    let mut passed = 0usize;
    let mut failures = Vec::new();

    for task in &tasks {
        let id = map_get_str(task, "id").expect("كل مهمة يجب أن تملك id");
        let cat = map_get_str(task, "category").expect("كل مهمة يجب أن تملك category");
        assert!(seen_ids.insert(id.clone()), "معرّف مهمة مكرر: {}", id);
        seen_categories.insert(cat);

        match run_single_task(task) {
            Ok(()) => passed += 1,
            Err(err) => failures.push(err),
        }
    }

    assert!(
        seen_categories.len() >= 10,
        "يجب أن يغطي المعيار طيفًا واسعًا من المجالات، وُجد: {:?}",
        seen_categories
    );

    assert!(
        failures.is_empty(),
        "فشلت {} مهمة من أصل 30 (نجح {}/30):\n{}",
        failures.len(),
        passed,
        failures.join("\n---\n")
    );
    assert_eq!(passed, 30, "يجب تحقيق نسبة نجاح 100% (30/30)");
}

#[test]
fn test_reactive_web_and_capacitor_example_is_pure_arabic_and_parses() {
    let source = std::fs::read_to_string("examples/reactive_web_and_capacitor.lsn")
        .expect("يجب قراءة ملف examples/reactive_web_and_capacitor.lsn");

    let tokens = lisan::lexer::tokenize(&source)
        .expect("يجب أن ينجح التحليل المعجمي لمثال الويب التفاعلي");
    let stmts = lisan::parser::parse(tokens)
        .expect("يجب أن ينجح التحليل النحوي لمثال الويب التفاعلي");
    assert!(!stmts.is_empty());

    let lints = lint_arabic_only(&source);
    assert!(
        lints.is_empty(),
        "مثال الويب التفاعلي يجب أن يكون عربيًا خالصًا بلا أي تشخيصات ل0001 أو ل0004، وُجد: {:?}",
        lints
    );
}
