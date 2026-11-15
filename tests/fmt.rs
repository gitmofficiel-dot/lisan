use lisan::fmt::format_source;

#[test]
fn test_fmt_basic_assignment_and_conditionals() {
    let unformatted = r#"
دع    س=5
دع ص=س*2
إذا(س<10){
اطبع(ن"العدد:{ص}")
}إلا{
اطبع("كبير")
}
"#;

    let formatted = format_source(unformatted);
    let expected = r#"دع س = 5
دع ص = س * 2
إذا (س < 10) {
    اطبع(ن"العدد:{ص}")
} إلا {
    اطبع("كبير")
}
"#;
    assert_eq!(formatted, expected);
}

#[test]
fn test_fmt_collections_and_loops() {
    let unformatted = r#"
دع قائمة=[1،2،3،4]
لكل عنصر في قائمة{
إذا(عنصر%2==0){
اطبع(عنصر)
}
}
"#;

    let formatted = format_source(unformatted);
    assert!(formatted.contains("دع قائمة = [1، 2، 3، 4]"));
    assert!(formatted.contains("لكل عنصر في قائمة {"));
    assert!(formatted.contains("    إذا (عنصر % 2 == 0) {"));
    assert!(formatted.contains("        اطبع(عنصر)"));
}

#[test]
fn test_fmt_preserves_internal_strings() {
    let code = r#"دع رسالة = "  نص مع مسافات  + = # محفوظة  ""#;
    let formatted = format_source(code);
    assert!(formatted.contains(r#"دع رسالة = "  نص مع مسافات  + = # محفوظة  ""#));
}
