use lisan::run;

#[test]
fn arabic_fusha_and_ascii_numbers() {
    assert_eq!(run("ليكن عدد = 120؛ متغير نتيجة = عدد / 3؛ بينما (نتيجة > 10) { نتيجة = نتيجة - 10؛ } اطبع(نتيجة)؛").unwrap(), ["10"]);
}

#[test]
fn reject_arabic_digits_and_persian_digits() {
    for input in ["اطبع(١٢)؛", "اطبع(۱۲)؛", "اطبع(٠.١)؛", "اطبع(1٫5)؛"] {
        assert!(run(input).is_err(), "input unexpectedly accepted: {input}");
    }
}

#[test]
fn dictionaries_roundtrip_and_mutation() {
    let source = r#"دع الشخص = {"الاسم": "سلمى", "السن": 25}؛ اطبع(الشخص["الاسم"])؛ الشخص["السن"] = 26؛ الشخص["المدينة"] = "الرباط"؛ اطبع(طول(الشخص))؛ اطبع(الشخص["السن"])؛ اطبع(نوع(الشخص))؛"#;
    assert_eq!(run(source).unwrap(), ["سلمى", "3", "26", "قاموس"]);
}

#[test]
fn dictionary_keys_values_and_membership() {
    assert_eq!(run(r#"دع م = {"أ": 1، "ب": 2}؛ اطبع(مفاتيح(م)[0])؛ اطبع(قيم(م)[1])؛ اطبع(يحتوي(م، "أ"))؛ اطبع(يحتوي(م، "س"))؛"#).unwrap(), ["أ", "2", "صحيح", "خطأ"]);
}

#[test]
fn duplicate_key_and_missing_key_errors() {
    assert!(run(r#"دع م = {"أ": 1، "أ": 2}؛"#).unwrap_err().contains("مكرر"));
    assert!(run(r#"دع م = {"أ": 1}؛ اطبع(م["ب"])؛"#).unwrap_err().contains("مفتاح"));
    assert!(run(r#"دع م = {"أ": 1}؛ اطبع(م[0])؛"#).unwrap_err().contains("نصًا"));
}

#[test]
fn dictionary_deletion_and_list_deletion() {
    assert_eq!(run(r#"دع م = احذف({"أ": 1، "ب": 2}، "أ")؛ اطبع(طول(م))؛ دع ل = احذف([10، 20، 30]، 1)؛ اطبع(ل[1])؛"#).unwrap(), ["1", "30"]);
}

#[test]
fn string_functions_are_unicode_aware() {
    assert_eq!(run(r#"اطبع(اقتطع("مرحبا بالعالم"، 0، 5))؛ اطبع(نظّف("  أهلاً  "))؛ اطبع(استبدل("كتاب جديد"، "جديد"، "مفيد"))؛"#).unwrap(), ["مرحبا", "أهلاً", "كتاب مفيد"]);
}

#[test]
fn split_and_join() {
    assert_eq!(run(r#"دع أسماء = قسّم("سلمى,علي,منى"، ",")؛ اطبع(طول(أسماء))؛ اطبع(ادمج(أسماء، " | "))؛"#).unwrap(), ["3", "سلمى | علي | منى"]);
}

#[test]
fn math_functions_and_reject_negative_sqrt() {
    assert_eq!(run("اطبع(مطلق(-7))؛ اطبع(جذر(81))؛ اطبع(قرّب(3.6))؛ اطبع(أدنى(7، 4))؛ اطبع(أعلى(7، 4))؛").unwrap(), ["7", "9", "4", "4", "7"]);
    assert!(run("اطبع(جذر(-1))؛").unwrap_err().contains("سالب"));
}

#[test]
fn assignments_stay_in_existing_scopes() {
    assert_eq!(run(r#"دع م = {"عدد": 1}؛ { م["عدد"] = 5؛ } اطبع(م["عدد"])؛"#).unwrap(), ["5"]);
}

#[test]
fn source_with_arabic_comma_and_latin_decimal() {
    assert_eq!(run("اطبع([0، 1.25، 3][1])؛").unwrap(), ["1.25"]);
}

#[test]
fn reject_invalid_map_literal() {
    assert!(run("دع م = {عدد: 1}؛").unwrap_err().contains("مفتاح"));
}

#[test]
fn latin_digits_allowed_in_identifiers_but_not_arabic_indic_digits() {
    assert_eq!(run("ليكن عدد2 = 7؛ اطبع(عدد2)؛").unwrap(), ["7"]);
    for input in ["ليكن عدد٢ = 7؛", "ليكن عدد۲ = 7؛", "اطبع(12٣)؛"] {
        let err = run(input).expect_err("non Latin source digit must fail");
        assert!(err.contains("0-9"), "{input} returned {err}");
    }
}

#[test]
fn list_reverse_and_sort() {
    assert_eq!(run("دع ل = [3, 1, 4, 1, 5]؛ اطبع(عكس(ل))؛ اطبع(فرز(ل))؛").unwrap(), ["[5، 1، 4، 1، 3]", "[1، 1، 3، 4، 5]"]);
    assert_eq!(run(r#"دع ك = ["موز", "تفاح", "برتقال"]؛ اطبع(فرز(ك))؛ اطبع(عكس("لسان"))؛"#).unwrap(), [r#"["برتقال"، "تفاح"، "موز"]"#, "ناسل"]);
}

#[test]
fn list_sum_and_mean() {
    assert_eq!(run("دع أ = [10, 20, 30, 40]؛ اطبع(مجموع(أ))؛ اطبع(متوسط(أ))؛").unwrap(), ["100", "25"]);
    assert_eq!(run("اطبع(مجموع([]))؛").unwrap(), ["0"]);
    assert!(run("متوسط([])؛").unwrap_err().contains("فارغة"));
}

#[test]
fn search_and_string_features() {
    assert_eq!(run(r#"دع ن = "مرحبا بلغة لسان"؛ اطبع(ابحث(ن، "لسان"))؛ اطبع(يبدأ_ب(ن، "مرحبا"))؛ اطبع(ينتهي_ب(ن، "لسان"))؛ اطبع(كرر("أهلاً "، 3))؛"#).unwrap(), ["11", "صحيح", "صحيح", "أهلاً أهلاً أهلاً "]);
    assert_eq!(run("دع ل = [10, 20, 30]؛ اطبع(ابحث(ل، 20))؛ اطبع(ابحث(ل، 99))؛").unwrap(), ["1", "-1"]);
    assert_eq!(run(r#"اطبع(أحرف_كبيرة("lisan"))؛ اطبع(أحرف_صغيرة("LISAN"))؛"#).unwrap(), ["LISAN", "lisan"]);
}

#[test]
fn extended_math_builtins() {
    assert_eq!(run("اطبع(أرضية(4.9))؛ اطبع(سقف(4.1))؛ اطبع(قوة(2، 10))؛").unwrap(), ["4", "5", "1024"]);
}

