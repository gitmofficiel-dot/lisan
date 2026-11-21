use lisan::{lexer,parser,native};
use std::path::PathBuf;

fn compile_ir(source:&str)->Result<String,String>{
    native::generate_llvm(&parser::parse(lexer::tokenize(source)?)?)
}

#[test]
fn native_lists_index_and_loop_have_real_llvm(){
    let ir=compile_ir("ليكن س = [1، 2، 3]؛ س[0] = 7؛ لكل (ق في س) { اطبع(ق)؛ }").unwrap();
    assert!(ir.contains("extractvalue { i64, ptr }"));
    assert!(ir.contains("getelementptr double"));
    assert!(ir.contains("foreach.cond"));
    assert!(!ir.contains("Interpreter"));
}
#[test]
fn native_strings_concatenate_compare_and_unicode_length(){
    let ir=compile_ir("ليكن ج = \"لغة \" + \"لسان\"؛ تحقق(ج == \"لغة لسان\")؛ اطبع(طول(\"سلام\"))؛").unwrap();
    assert!(ir.contains("@strcat"));
    assert!(ir.contains("@strcmp"));
    assert!(ir.contains("@lisan_utf8_len"));
}
#[test]
fn native_range_and_math_functions(){
    let ir=compile_ir("لكل (س في مدى(5، 0، -2)) { اطبع(مطلق(س))؛ }").unwrap();
    assert!(ir.contains("range.cond"));
    assert!(ir.contains("@llvm.fabs.f64"));
}
#[test]
fn native_sum_avg_and_extended_math(){
    let ir=compile_ir("ليكن أ = [10, 20, 30]؛ اطبع(مجموع(أ))؛ اطبع(متوسط(أ))؛ اطبع(أرضية(3.8))؛ اطبع(سقف(3.2))؛ اطبع(قوة(2, 4))؛").unwrap();
    assert!(ir.contains("sum.cond"));
    assert!(ir.contains("avg.cond"));
    assert!(ir.contains("@llvm.floor.f64"));
    assert!(ir.contains("@llvm.ceil.f64"));
    assert!(ir.contains("@llvm.pow.f64"));
}
#[test]
fn native_match_statement(){
    let ir=compile_ir(r#"
اختر (2) {
    حالة 1: {
        اطبع("واحد")؛
    }
    حالة 2: {
        اطبع("اثنان")؛
    }
    افتراضي: {
        اطبع("غير معروف")؛
    }
}
"#).unwrap();
    assert!(ir.contains("match.case"));
    assert!(ir.contains("match.next"));
    assert!(ir.contains("match.end"));
}
#[test]
fn native_rejects_unsupported_nested_maps(){
    let err=compile_ir("ليكن س = {\"مفتاح\": 5}؛").unwrap_err();
    assert!(err.contains("لم تُنفّذ"));
}
#[cfg(target_os="linux")]
#[test]
fn native_collections_binary_executes_without_interpreter(){
    if std::process::Command::new("clang").arg("--version").output().is_err(){return;}
    let source=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/native_collections.lsn");
    let target=std::env::temp_dir().join(format!("lisan-collections-{}",std::process::id()));
    native::build_native(&source,&target).unwrap();
    let process=std::process::Command::new(&target).output().unwrap();
    let _=std::fs::remove_file(&target);
    assert!(process.status.success(),"{}",String::from_utf8_lossy(&process.stderr));
    assert_eq!(String::from_utf8_lossy(&process.stdout),"مجموع القيم = 65\nعدد الحروف = 4\n5\n3\n1\nنجحت الاختبارات الأصلية\n");
}
#[test]
fn native_functions_accept_and_return_strings() {
    let source = r#"
دالة تحية(اسم) {
    أرجع "مرحبا " + اسم؛
}
دالة طول_نص(ن) {
    أرجع طول(ن)؛
}
دع رسالة = تحية("لسان")؛
اطبع(رسالة)؛
اطبع(طول_نص(رسالة))؛
"#;
    let ir = compile_ir(source).expect("string function args/return should compile");
    assert!(ir.contains("define ptr @lisan_fn_0_txt(ptr %arg0)"));
    assert!(ir.contains("define double @lisan_fn_1_txt(ptr %arg0)"));
    assert!(ir.contains("call ptr @lisan_fn_0_txt(ptr "));
    assert!(ir.contains("call double @lisan_fn_1_txt(ptr "));
}

#[test]
fn native_functions_accept_and_return_number_lists() {
    let source = r#"
دالة اجمع_قائمة(ق) {
    دع ناتج = 0؛
    لكل عنصر في ق {
        ناتج = ناتج + عنصر؛
    }
    أرجع ناتج؛
}
دالة مرر_قائمة(ق) {
    أرجع ق؛
}
دع أرقام = [10, 20, 30]؛
دع نسخة = مرر_قائمة(أرقام)؛
اطبع(اجمع_قائمة(نسخة))؛
"#;
    let ir = compile_ir(source).expect("list function args/return should compile");
    assert!(ir.contains("define double @lisan_fn_0_list({ i64, ptr } %arg0)"));
    assert!(ir.contains("define { i64, ptr } @lisan_fn_1_list({ i64, ptr } %arg0)"));
    assert!(ir.contains("call { i64, ptr } @lisan_fn_1_list({ i64, ptr } "));
    assert!(ir.contains("call double @lisan_fn_0_list({ i64, ptr } "));
}

#[test]
fn native_functions_support_mixed_params_and_recursion() {
    let source = r#"
دالة وصف(عنوان, قيم, معامل) {
    أرجع طول(عنوان) + مجموع(قيم) * معامل؛
}
دالة مجموع_تنازلي(ن) {
    إذا (ن <= 1) {
        أرجع 1؛
    }
    أرجع ن + مجموع_تنازلي(ن - 1)؛
}
اطبع(وصف("لسان", [1, 2, 3], 2))؛
اطبع(مجموع_تنازلي(4))؛
"#;
    let ir = compile_ir(source).expect("mixed params and recursion should compile");
    assert!(ir.contains("define double @lisan_fn_0_txt_list_num(ptr %arg0, { i64, ptr } %arg1, double %arg2)"));
    assert!(ir.contains("define double @lisan_fn_1(double %arg0)"));
    assert!(ir.contains("call double @lisan_fn_1(double "));
}

#[test]
fn native_arena_memory_cleanup_ir() {
    let source = r#"
دالة كوّن_قائمة(س) {
    أرجع [س, س + 1, س + 2]؛
}
دع ق = كوّن_قائمة(5)؛
اطبع("النتيجة: " + نص(مجموع(ق)))؛
"#;
    let ir = compile_ir(source).expect("arena IR should compile");
    assert!(ir.contains("declare void @free(ptr)"));
    assert!(ir.contains("define internal ptr @lisan_alloc(i64 %size)"));
    assert!(ir.contains("define internal void @lisan_free_all()"));
    assert!(ir.contains("call ptr @lisan_alloc("));
    assert!(ir.contains("call void @lisan_free_all()"));
}

#[cfg(unix)]
#[test]
fn native_functions_strings_and_lists_binary_executes() {
    if std::process::Command::new("clang").arg("--version").output().is_err() {
        return;
    }
    let source_code = r#"
دالة تحية(اسم) {
    أرجع "مرحبا " + اسم؛
}
دالة أنشئ_قائمة(أ, ب) {
    أرجع [أ, ب, أ + ب]؛
}
دالة مجموع_قائمة(ق) {
    دع إجمالي = 0؛
    لكل س في ق {
        إجمالي = إجمالي + س؛
    }
    أرجع إجمالي؛
}
دع رسالة = تحية("لسان")؛
دع قائمة = أنشئ_قائمة(4, 6)؛
اطبع(رسالة)؛
اطبع(مجموع_قائمة(قائمة))؛
"#;
    let tmp = std::env::temp_dir();
    let src_path = tmp.join("lisan_native_fn_strings_lists.lsn");
    let bin_path = tmp.join("lisan_native_fn_strings_lists_bin");
    std::fs::write(&src_path, source_code).expect("write temp lsn source");
    let built = native::build_native(&src_path, &bin_path).expect("build native binary");
    let output = std::process::Command::new(&built).output().expect("execute native binary");
    let _ = std::fs::remove_file(&src_path);
    let _ = std::fs::remove_file(&built);
    assert!(output.status.success(), "binary failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["مرحبا لسان", "20"]);
}
