use lisan::run;

#[test]
fn arabic_indic_digits_arithmetic() {
    let src = r#"
دع أ = ١٠
دع ب = ٢٠
دع ج = ٣
اطبع(أ + ب * ج)
"#;
    assert_eq!(run(src).unwrap(), vec!["70"]);
}

#[test]
fn persian_digits_arithmetic() {
    let src = r#"
دع س = ۴۵
دع ص = ۱۵
اطبع(س - ص)
"#;
    assert_eq!(run(src).unwrap(), vec!["30"]);
}

#[test]
fn arabic_decimal_separator_and_mixed_numerals() {
    let src = r#"
دع سعر = ٣٫٥
دع إضافة = 1.5
اطبع(سعر + إضافة)
"#;
    assert_eq!(run(src).unwrap(), vec!["5"]);
}

#[test]
fn scientific_notation_with_arabic_digits() {
    let src = r#"
اطبع(٢e٣)
اطبع(١٫٥e2)
"#;
    assert_eq!(run(src).unwrap(), vec!["2000", "150"]);
}

#[test]
fn arabic_digits_inside_identifiers() {
    let src = r#"
دع متغير١ = ٧
دع متغير٢ = ٨
اطبع(متغير١ + متغير٢)
"#;
    assert_eq!(run(src).unwrap(), vec!["15"]);
}

#[test]
fn normalized_arabic_keywords_without_hamza_and_taa_marbuta() {
    let src = r#"
داله جمع(س، ص) {
    ارجع س + ص
}

اذا (صحيح) {
    اطبع(جمع(٤، ٦))
} والا {
    اطبع(٠)
}
"#;
    assert_eq!(run(src).unwrap(), vec!["10"]);
}

#[test]
fn normalized_arabic_keywords_in_loops_and_collections() {
    let src = r#"
متغير مجموع = ٠
لكل رقم في مدى(١، ٥) {
    مجموع = مجموع + رقم
}
اطبع(مجموع)
"#;
    assert_eq!(run(src).unwrap(), vec!["10"]);
}

#[test]
fn invalid_scientific_and_overflow_numbers_return_error() {
    assert!(run("اطبع(1e)").is_err());
    assert!(run("اطبع(1e9999)").is_err());
}
