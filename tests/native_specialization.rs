use lisan::{lexer, native, parser};

fn compile_ir(source: &str) -> Result<String, String> {
    native::generate_llvm(&parser::parse(lexer::tokenize(source)?)?)
}

#[test]
fn native_specializes_function_with_text_parameter_and_return() {
    let src = r#"
دالة ترحيب(اسم) {
    أرجع "مرحبا " + اسم
}
اطبع(ترحيب("لسان"))
"#;
    let ir = compile_ir(src).unwrap();
    assert!(ir.contains("define ptr @lisan_fn_0_txt(ptr %arg0)"));
    assert!(ir.contains("call ptr @lisan_fn_0_txt("));
}

#[test]
fn native_specializes_function_with_number_list_parameter() {
    let src = r#"
دالة مجموع_القائمة(أرقام) {
    أرجع مجموع(أرقام)
}
دع قائمة = [10, 20, 30]
اطبع(مجموع_القائمة(قائمة))
"#;
    let ir = compile_ir(src).unwrap();
    assert!(ir.contains("define double @lisan_fn_0_list({ i64, ptr } %arg0)"));
    assert!(ir.contains("call double @lisan_fn_0_list("));
}

#[test]
fn native_specializes_same_function_for_multiple_types() {
    let src = r#"
دالة هوية(قيمة) {
    أرجع قيمة
}
اطبع(هوية(42))
اطبع(هوية("عربي"))
"#;
    let ir = compile_ir(src).unwrap();
    assert!(ir.contains("define double @lisan_fn_0(double %arg0)"));
    assert!(ir.contains("define ptr @lisan_fn_0_txt(ptr %arg0)"));
}

#[test]
fn native_accepts_arabic_numerals_and_normalized_keywords() {
    let src = r#"
داله مضاعف(س) {
    اذا (س > ٠) {
        ارجع س * ٢
    }
    ارجع ٠
}
اطبع(مضاعف(١٥))
"#;
    let ir = compile_ir(src).unwrap();
    assert!(ir.contains("define i32 @main()"));
    assert!(ir.contains("define double @lisan_fn_0(double %arg0)"));
}

#[test]
fn native_rejects_nested_function_and_printing_number_list() {
    let nested = r#"
دالة خارجية() {
    دالة داخلية() { أرجع 1 }
    أرجع 0
}
اطبع(خارجية())
"#;
    assert!(compile_ir(nested).is_err());

    let print_list = r#"
دع أرقام = [1, 2, 3]
اطبع(أرقام)
"#;
    assert!(compile_ir(print_list).is_err());
}
