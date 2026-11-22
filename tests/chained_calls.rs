use lisan::run;

#[test]
fn call_result_of_call() {
    let src = r#"
ليكن ف = دالة(أ) => دالة(ب) => أ + ب؛
اطبع(ف(1)(2))؛
"#;
    assert_eq!(run(src).unwrap(), ["3"]);
}

#[test]
fn call_list_element() {
    let src = r#"
ليكن ق = [دالة(س) => س * 2]؛
اطبع(ق[0](5))؛
"#;
    assert_eq!(run(src).unwrap(), ["10"]);
}

#[test]
fn call_non_function_errors() {
    let src = r#"
ليكن ق = [1، 2]؛
اطبع(ق[0](5))؛
"#;
    assert!(run(src).is_err());
}
