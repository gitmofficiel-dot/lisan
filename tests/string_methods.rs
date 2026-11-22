use lisan::run;

#[test]
fn split_method_on_text() {
    assert_eq!(run("اطبع(\"أ-ب-ج\".قسّم(\"-\")[1])؛").unwrap(), ["ب"]);
}

#[test]
fn trim_method_on_text() {
    assert_eq!(run("اطبع(\"  لسان  \".نظّف())؛").unwrap(), ["لسان"]);
}
