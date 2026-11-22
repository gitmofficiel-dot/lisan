use lisan::run;

#[test]
fn inner_named_function_sees_outer_variable() {
    let src = r#"
دالة خارجية(أ) {
    دالة داخلية(ب) {
        ارجع أ + ب؛
    }
    ارجع داخلية(10)؛
}
اطبع(خارجية(5))؛
"#;
    assert_eq!(run(src).unwrap(), ["15"]);
}

#[test]
fn returned_lambda_keeps_captured_value() {
    let src = r#"
دالة صانع(ن) {
    ارجع دالة(س) => س + ن؛
}
ليكن ز = صانع(3)؛
اطبع(ز(4))؛
"#;
    assert_eq!(run(src).unwrap(), ["7"]);
}
