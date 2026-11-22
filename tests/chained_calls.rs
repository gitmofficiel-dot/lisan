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

#[test]
fn inner_named_function_captures_outer_variables() {
    let src = r#"
دالة اصنع_مضاعف(عامل) {
    دالة اضرب(س) {
        أرجع س * عامل
    }
    أرجع اضرب
}
دع ثلاثي = اصنع_مضاعف(3)
اطبع(ثلاثي(7))
اطبع(اصنع_مضاعف(10)(4))
"#;
    assert_eq!(run(src).unwrap(), ["21", "40"]);
}

#[test]
fn named_function_stored_in_list_and_map_can_be_called() {
    let src = r#"
دالة مربع(ن) { أرجع ن * ن }
دع عمليات = {"تربيع": مربع}
دع قائمة = [مربع]
اطبع(عمليات["تربيع"](6))
اطبع(قائمة[0](9))
"#;
    assert_eq!(run(src).unwrap(), ["36", "81"]);
}
