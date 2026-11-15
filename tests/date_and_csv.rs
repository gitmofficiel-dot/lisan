use lisan::run;

#[test]
fn test_date_module_and_functions() {
    let code = r#"
استورد تاريخ كـ ت
دع د1 = "2026-01-01"
دع د2 = "2026-01-11"
دع فرق = ت.فرق_أيام(د2، د1)
اطبع(فرق)
دع بعد_خمسة = ت.أضف_أيام(د1، 5)
اطبع(بعد_خمسة)
دع يوم = ت.يوم_الأسبوع("2026-11-15")
اطبع(يوم)
"#;
    let output = run(code).expect("يجب تنفيذ كود التاريخ بنجاح");
    assert_eq!(output[0], "10");
    assert_eq!(output[1], "2026-01-06");
    assert_eq!(output[2], "الأحد");
}

#[test]
fn test_from_date_import() {
    let code = r#"
من تاريخ استورد فرق_أيام، أضف_أيام، يوم_الأسبوع
دع فرق = فرق_أيام("2026-05-10"، "2026-05-01")
اطبع(فرق)
دع بعد = أضف_أيام("2025-12-30"، 3)
اطبع(بعد)
"#;
    let output = run(code).expect("يجب تنفيذ الاستيراد الجزئي بنجاح");
    assert_eq!(output[0], "9");
    assert_eq!(output[1], "2026-01-02");
}

#[test]
fn test_csv_module_parse_and_encode() {
    let code = r#"
استورد csv كـ سي_إس_في
دع نص_جدول = "الاسم,العمر,المدينة\nأحمد,25,الرياض\nسارة,28,دبي"
دع جدول = سي_إس_في.تحليل_csv(نص_جدول)
اطبع(طول(جدول))
اطبع(جدول[1][0])
اطبع(جدول[1][1])

دع صفوف = [
    ["رقم", "وصف"],
    [101, "عنصر أ"],
    [102, "عنصر ب"]
]
دع نص_ناتج = سي_إس_في.ترميز_csv(صفوف)
اطبع(نص_ناتج)
"#;
    let output = run(code).expect("يجب تنفيذ وحدة CSV بنجاح");
    assert_eq!(output[0], "3");
    assert_eq!(output[1], "أحمد");
    assert_eq!(output[2], "25");
    assert!(output[3].contains("رقم,وصف"));
    assert!(output[3].contains("101,عنصر أ"));
}

#[test]
fn test_vm_loop_control_and_modules() {
    let mut vm = lisan::vm::VM::new();
    let source = r#"
استورد رياضيات كـ رض
دع س = 0
طالما (س < 10) {
    س = س + 1
    إذا (س == 5) {
        توقف
    }
}
اطبع(س)
دع جذر_16 = رض.جذر(16)
اطبع(جذر_16)
"#;
    let output = vm.run_source(source).unwrap();
    assert_eq!(output[0], "5");
    assert_eq!(output[1], "4");
}

#[test]
fn test_vm_foreach_unpacking() {
    let mut vm = lisan::vm::VM::new();
    let source = r#"
دع أزواج = [[1, 10], [2, 20], [3, 30]]
دع مجموع = 0
لكل أ، ب في أزواج {
    مجموع = مجموع + أ + ب
}
اطبع(مجموع)
"#;
    let output = vm.run_source(source).unwrap();
    assert_eq!(output[0], "66");
}
