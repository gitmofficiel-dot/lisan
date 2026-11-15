use lisan::run;

#[test]
fn test_advanced_math_functions() {
    let src = r#"
استورد رياضيات كـ ر

اطبع(ر.مضروب(5))
اطبع(ر.قاسم_مشترك(48، 18))
اطبع(ر.مضاعف_مشترك(4، 6))
اطبع(ر.توافيق(5، 2))
اطبع(ر.تباديل(5، 2))
اطبع(ر.لوغاريتم10(1000))
اطبع(ر.لوغاريتم2(32))
اطبع(ر.تقارب(0.1 + 0.2، 0.3))
"#;
    let out = run(src).unwrap();
    assert_eq!(out[0], "120");
    assert_eq!(out[1], "6");
    assert_eq!(out[2], "12");
    assert_eq!(out[3], "10");
    assert_eq!(out[4], "20");
    assert_eq!(out[5], "3");
    assert_eq!(out[6], "5");
    assert_eq!(out[7], "صحيح");
}

#[test]
fn test_statistics_module_functions() {
    let src = r#"
استورد إحصاء كـ ح

دع عينة = [10، 20، 20، 30، 40]
اطبع(ح.متوسط(عينة))
اطبع(ح.وسيط(عينة))
اطبع(ح.منوال(عينة))
اطبع(ح.مدى(عينة))

دع عينة_زوجية = [1، 2، 5، 6]
اطبع(ح.وسيط(عينة_زوجية))

دع متساوية = [10، 10، 10]
اطبع(ح.انحراف_معياري(متساوية))

دع أرقام = [2، 8]
اطبع(ح.متوسط_هندسي(أرقام))

دع توافقي = [2، 6]
اطبع(ح.متوسط_توافقي(توافقي))

دع س = [1، 2، 3]
دع ص = [2، 4، 6]
اطبع(ح.ارتباط(س، ص))
"#;
    let out = run(src).unwrap();
    assert_eq!(out[0], "24");
    assert_eq!(out[1], "20");
    assert_eq!(out[2], "20");
    assert_eq!(out[3], "30");
    assert_eq!(out[4], "3.5");
    assert_eq!(out[5], "0");
    assert_eq!(out[6], "4");
    assert_eq!(out[7], "3");
    assert_eq!(out[8], "1");
}

#[test]
fn test_from_stats_import() {
    let src = r#"
من إحصاء استورد وسيط، منوال

دع قائمة = [5، 1، 5، 2، 9]
اطبع(وسيط(قائمة))
اطبع(منوال(قائمة))
"#;
    let out = run(src).unwrap();
    assert_eq!(out[0], "5");
    assert_eq!(out[1], "5");
}

#[test]
fn test_vm_executes_math_and_stats() {
    let mut vm = lisan::vm::VM::new();
    let src = r#"
دع قائمة = [10، 20، 30]
اطبع(مجموع(قائمة))
اطبع(متوسط(قائمة))
اطبع(مضروب(4))
اطبع(توافيق(6، 2))
"#;
    let out = vm.run_source(src).unwrap();
    assert_eq!(out, ["60", "20", "24", "15"]);
}
