use lisan::tester::{run_tests, run_test_file, discover_test_files};
use std::fs;

#[test]
fn test_tester_runs_multiple_test_cases() {
    let temp_dir = std::env::temp_dir().join(format!("lisan_suite_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let file_a = temp_dir.join("عمليات_اختبار.lsn");
    let code_a = r#"
دالة اختبار_جمع() {
    تحقق_يساوي(10 + 20، 30)
}

دالة اختبار_طرح() {
    تحقق_يساوي(50 - 20، 30)
}
"#;
    fs::write(&file_a, code_a).unwrap();

    let file_b = temp_dir.join("نصوص_اختبار.lsn");
    let code_b = r#"
دالة اختبار_دمج_نصوص() {
    تحقق_يساوي("أهلاً " + "بالعالم"، "أهلاً بالعالم")
}
"#;
    fs::write(&file_b, code_b).unwrap();

    let discovered = discover_test_files(&temp_dir);
    assert_eq!(discovered.len(), 2);

    let summary = run_tests(&[temp_dir.clone()]);
    assert_eq!(summary.total_files, 2);
    assert_eq!(summary.total_tests, 3);
    assert_eq!(summary.passed, 3);
    assert_eq!(summary.failed, 0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_tester_captures_failures_and_errors() {
    let temp_dir = std::env::temp_dir().join(format!("lisan_fail_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let test_file = temp_dir.join("فحص_اختبار.lsn");
    let code = r#"
دالة اختبار_صحيح() {
    تحقق(صحيح)
}

دالة اختبار_خاطئ() {
    تحقق_يساوي(5، 10)
}
"#;
    fs::write(&test_file, code).unwrap();

    let summary = run_tests(&[test_file]);
    assert_eq!(summary.total_tests, 2);
    assert_eq!(summary.passed, 1);
    assert_eq!(summary.failed, 1);
    assert!(summary.file_results[0].cases[1].error.as_ref().unwrap().contains("فشل التحقق"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_tester_runs_flat_assertion_files() {
    let temp_dir = std::env::temp_dir().join(format!("lisan_flat_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let test_file = temp_dir.join("ملف_اختبار.lsn");
    let code = r#"
دع س = 100
تحقق_يساوي(س / 2، 50)
"#;
    fs::write(&test_file, code).unwrap();

    let res = run_test_file(&test_file);
    assert_eq!(res.cases.len(), 1);
    assert!(res.cases[0].passed);

    let _ = fs::remove_dir_all(&temp_dir);
}
