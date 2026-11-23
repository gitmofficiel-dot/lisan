//! جسر الربط الأصلي بواجهة C ABI (FFI) للغة لسان (Lisan).
//!
//! مبني بالكامل باستخدام مكتبة Rust القياسية (`std`) فقط دون أي حزم خارجية:
//! - على أنظمة Unix (Linux و macOS و BSD): يستخدم `dlopen` / `dlsym` / `dlclose` / `dlerror`.
//! - على نظام Windows: يستخدم `LoadLibraryA` / `GetProcAddress` / `FreeLibrary` من `kernel32`.
//! - على بيئة WebAssembly (`wasm32`): يُعيد خطأً عربيًا واضحًا وآمنًا دون أي تعطل.

use crate::runtime::Value;

/// المكتبة القياسية العربية 100% للربط مع المكتبات الأصلية عبر C ABI (FFI)
pub const STDLIB_FFI: &str = r##"
# مكتبة الربط الأصلي (C ABI FFI) في لغة لسان — عربية بالكامل
# أسماء المكتبات والدوال الأجنبية تظهر داخل النصوص فقط.

صنف مكتبة_خارجية {
    دالة بناء(اسم_المكتبة) {
        هذا.اسم_المكتبة = اسم_المكتبة
    }

    دالة متوفرة() {
        أرجع مكتبة_نظام_متوفرة(هذا.اسم_المكتبة)
    }

    دالة استدعِ(اسم_الدالة، التوقيع، وسائط = []) {
        أرجع استدعِ_دالة_نظام(هذا.اسم_المكتبة، اسم_الدالة، التوقيع، وسائط)
    }

    دالة استدع(اسم_الدالة، التوقيع، وسائط = []) {
        أرجع هذا.استدعِ(اسم_الدالة، التوقيع، وسائط)
    }
}

دالة حمّل_مكتبة(اسم_المكتبة) {
    أرجع جديد مكتبة_خارجية(اسم_المكتبة)
}

دالة حمل_مكتبة(اسم_المكتبة) {
    أرجع جديد مكتبة_خارجية(اسم_المكتبة)
}
"##;

/// جميع أسماء الدوال المدمجة في جسر الربط الأصلي C ABI (عربية خالصة)
pub const ALL_FFI_BUILTINS: &[&str] = &[
    "مكتبة_نظام_متوفرة",
    "مكتبه_نظام_متوفره",
    "استدعِ_دالة_نظام",
    "استدع_دالة_نظام",
    "استدع_داله_نظام",
];

/// التحقق مما إذا كان الاسم يشير إلى دالة مدمجة في جسر الربط الأصلي (FFI)
pub fn is_ffi_builtin(name: &str) -> bool {
    ALL_FFI_BUILTINS.contains(&name)
}

fn expect_ffi_text<'a>(fn_name: &str, arg_label: &str, val: &'a Value) -> Result<&'a str, String> {
    match val {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!(
            "الدالة '{}' تتطلب نصًا في معامل '{}'",
            fn_name, arg_label
        )),
    }
}

/// تنفيذ دالة مدمجة من جسر الربط الأصلي (FFI)
pub fn eval_ffi_builtin(name: &str, args: Vec<Value>) -> Result<Value, String> {
    match name {
        "مكتبة_نظام_متوفرة" | "مكتبه_نظام_متوفره" => {
            if args.len() != 1 {
                return Err(format!(
                    "الدالة '{}' تتوقع معاملًا واحدًا (اسم المكتبة)، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let lib_name = expect_ffi_text(name, "اسم_المكتبة", &args[0])?;
            Ok(Value::Bool(is_library_available(lib_name)))
        }
        "استدعِ_دالة_نظام" | "استدع_دالة_نظام" | "استدع_داله_نظام" => {
            if args.len() != 3 && args.len() != 4 {
                return Err(format!(
                    "الدالة '{}' تتوقع 3 أو 4 معاملات (اسم_المكتبة، اسم_الدالة، التوقيع، [الوسائط])، استلمت {}",
                    name,
                    args.len()
                ));
            }
            let lib_name = expect_ffi_text(name, "اسم_المكتبة", &args[0])?;
            let sym_name = expect_ffi_text(name, "اسم_الدالة", &args[1])?;
            let signature = expect_ffi_text(name, "التوقيع", &args[2])?;
            let call_args: Vec<Value> = if args.len() == 4 {
                match &args[3] {
                    Value::List(items) => items.borrow().clone(),
                    Value::Null => Vec::new(),
                    _ => {
                        return Err(format!(
                            "المعامل الرابع (الوسائط) في '{}' يجب أن يكون قائمة",
                            name
                        ))
                    }
                }
            } else {
                Vec::new()
            };
            call_c_symbol(lib_name, sym_name, signature, &call_args)
        }
        _ => Err(format!("دالة الربط الأصلي '{}' غير معروفة", name)),
    }
}

/// تطبيع التوقيع (إزالة الفراغات وتحويل الفاصلة العربية `،` إلى `,` والسهم `→` إلى `->`)
fn normalize_signature(sig: &str) -> String {
    let mut out = String::with_capacity(sig.len());
    for ch in sig.chars() {
        if ch.is_whitespace() {
            continue;
        }
        match ch {
            '،' => out.push(','),
            '→' => out.push_str("->"),
            other => out.push(other),
        }
    }
    out
}

/// تحويل الأسماء العربية أو المختصرة للمكتبات القياسية إلى أسمائها الفعلية حسب نظام التشغيل
pub fn resolve_library_candidates(lib_name: &str) -> Vec<String> {
    let trimmed = lib_name.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let norm_ar = crate::diagnostics::normalize_arabic(trimmed);
    let lower = trimmed.to_ascii_lowercase();

    let is_math_alias = matches!(
        norm_ar.as_str(),
        "مكتبه_الرياضيات" | "الرياضيات_الاصليه"
    ) || matches!(lower.as_str(), "libm" | "m");

    let is_libc_alias = matches!(
        norm_ar.as_str(),
        "مكتبه_النظام" | "النظام_الاصلي"
    ) || matches!(lower.as_str(), "libc" | "c");

    if is_math_alias {
        return os_math_library_candidates();
    }
    if is_libc_alias {
        return os_libc_library_candidates();
    }
    vec![trimmed.to_string()]
}

fn os_math_library_candidates() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        vec![
            "libSystem.B.dylib".to_string(),
            "/usr/lib/libSystem.B.dylib".to_string(),
            "libm.dylib".to_string(),
        ]
    }
    #[cfg(target_os = "windows")]
    {
        vec!["ucrtbase.dll".to_string(), "msvcrt.dll".to_string()]
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        vec![
            "libm.so.6".to_string(),
            "libm.so".to_string(),
            "libm.so.5".to_string(),
        ]
    }
    #[cfg(target_arch = "wasm32")]
    {
        vec!["libm.so.6".to_string()]
    }
}

fn os_libc_library_candidates() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        vec![
            "libSystem.B.dylib".to_string(),
            "/usr/lib/libSystem.B.dylib".to_string(),
        ]
    }
    #[cfg(target_os = "windows")]
    {
        vec![
            "ucrtbase.dll".to_string(),
            "msvcrt.dll".to_string(),
            "kernel32.dll".to_string(),
        ]
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        vec![
            "libc.so.6".to_string(),
            "libc.so".to_string(),
            "libc.so.7".to_string(),
        ]
    }
    #[cfg(target_arch = "wasm32")]
    {
        vec!["libc.so.6".to_string()]
    }
}

// ============================================================================
// طبقة التحميل الديناميكي لأنظمة Unix (Linux / macOS / BSD)
// ============================================================================

#[cfg(all(unix, not(target_arch = "wasm32")))]
mod platform {
    use std::ffi::{c_char, c_int, c_void, CStr, CString};

    const RTLD_LAZY: c_int = 1;

    extern "C" {
        fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        fn dlclose(handle: *mut c_void) -> c_int;
        fn dlerror() -> *const c_char;
    }

    pub struct DynamicLibrary {
        handle: *mut c_void,
    }

    impl DynamicLibrary {
        pub fn open(path: &str) -> Result<Self, String> {
            let c_path = CString::new(path)
                .map_err(|_| format!("اسم المكتبة '{}' يحتوي على بايت صفري غير صالح", path))?;
            // SAFETY: c_path is a valid NUL-terminated C string; RTLD_LAZY is standard on Unix.
            let handle = unsafe { dlopen(c_path.as_ptr(), RTLD_LAZY) };
            if handle.is_null() {
                let reason = last_dlerror().unwrap_or_else(|| "سبب غير معروف".to_string());
                return Err(format!("تعذّر فتح المكتبة '{}': {}", path, reason));
            }
            Ok(Self { handle })
        }

        pub fn symbol(&self, name: &str) -> Result<*mut c_void, String> {
            let c_name = CString::new(name)
                .map_err(|_| format!("اسم الدالة '{}' يحتوي على بايت صفري غير صالح", name))?;
            // SAFETY: clearing any stale dlerror state before dlsym per POSIX specification.
            unsafe {
                let _ = dlerror();
            }
            // SAFETY: self.handle is a valid open library handle and c_name is NUL-terminated.
            let sym = unsafe { dlsym(self.handle, c_name.as_ptr()) };
            if sym.is_null() {
                let reason = last_dlerror().unwrap_or_else(|| "الرمز غير موجود".to_string());
                return Err(format!("تعذّر العثور على الرمز '{}': {}", name, reason));
            }
            Ok(sym)
        }
    }

    impl Drop for DynamicLibrary {
        fn drop(&mut self) {
            if !self.handle.is_null() {
                // SAFETY: self.handle was returned by dlopen and has not yet been closed.
                unsafe {
                    let _ = dlclose(self.handle);
                }
                self.handle = std::ptr::null_mut();
            }
        }
    }

    fn last_dlerror() -> Option<String> {
        // SAFETY: dlerror returns either NULL or a valid NUL-terminated static/thread-local C string.
        let ptr = unsafe { dlerror() };
        if ptr.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(ptr) }
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }
}

// ============================================================================
// طبقة التحميل الديناميكي لنظام Windows (kernel32)
// ============================================================================

#[cfg(all(windows, not(target_arch = "wasm32")))]
mod platform {
    use std::ffi::{c_char, c_int, c_void, CString};

    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryA(lp_lib_file_name: *const c_char) -> *mut c_void;
        fn GetProcAddress(h_module: *mut c_void, lp_proc_name: *const c_char) -> *mut c_void;
        fn FreeLibrary(h_lib_module: *mut c_void) -> c_int;
    }

    pub struct DynamicLibrary {
        handle: *mut c_void,
    }

    impl DynamicLibrary {
        pub fn open(path: &str) -> Result<Self, String> {
            let c_path = CString::new(path)
                .map_err(|_| format!("اسم المكتبة '{}' يحتوي على بايت صفري غير صالح", path))?;
            // SAFETY: c_path is a valid NUL-terminated ANSI/UTF-8 string.
            let handle = unsafe { LoadLibraryA(c_path.as_ptr()) };
            if handle.is_null() {
                return Err(format!("تعذّر تحميل المكتبة '{}' عبر LoadLibraryA", path));
            }
            Ok(Self { handle })
        }

        pub fn symbol(&self, name: &str) -> Result<*mut c_void, String> {
            let c_name = CString::new(name)
                .map_err(|_| format!("اسم الدالة '{}' يحتوي على بايت صفري غير صالح", name))?;
            // SAFETY: self.handle is a valid module handle and c_name is NUL-terminated.
            let sym = unsafe { GetProcAddress(self.handle, c_name.as_ptr()) };
            if sym.is_null() {
                return Err(format!(
                    "تعذّر العثور على الرمز '{}' عبر GetProcAddress",
                    name
                ));
            }
            Ok(sym)
        }
    }

    impl Drop for DynamicLibrary {
        fn drop(&mut self) {
            if !self.handle.is_null() {
                // SAFETY: self.handle was returned by LoadLibraryA and has not yet been freed.
                unsafe {
                    let _ = FreeLibrary(self.handle);
                }
                self.handle = std::ptr::null_mut();
            }
        }
    }
}

// ============================================================================
// واجهة التحقق والاستدعاء الموحّدة (Native + WASM)
// ============================================================================

/// التحقق من توفّر مكتبة نظام أصلية على الجهاز الحالي
#[cfg(not(target_arch = "wasm32"))]
pub fn is_library_available(lib_name: &str) -> bool {
    let candidates = resolve_library_candidates(lib_name);
    for candidate in candidates {
        if platform::DynamicLibrary::open(&candidate).is_ok() {
            return true;
        }
    }
    false
}

/// التحقق من توفّر مكتبة نظام في بيئة WebAssembly (غير مدعوم دائمًا -> خطأ منطقي `false`)
#[cfg(target_arch = "wasm32")]
pub fn is_library_available(_lib_name: &str) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
fn open_library_with_fallback(lib_name: &str) -> Result<platform::DynamicLibrary, String> {
    let candidates = resolve_library_candidates(lib_name);
    if candidates.is_empty() {
        return Err("اسم المكتبة الخارجية لا يمكن أن يكون فارغًا".to_string());
    }
    let mut last_err = String::new();
    for candidate in &candidates {
        match platform::DynamicLibrary::open(candidate) {
            Ok(lib) => return Ok(lib),
            Err(err) => last_err = err,
        }
    }
    Err(format!(
        "تعذّر فتح المكتبة '{}' (المرشحات: [{}]): {}",
        lib_name,
        candidates.join("، "),
        last_err
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn expect_f64_arg(sig: &str, args: &[Value], idx: usize) -> Result<f64, String> {
    match args.get(idx) {
        Some(Value::Number(n)) if n.is_finite() => Ok(*n),
        Some(Value::Number(_)) => Err(format!(
            "المعامل رقم {} للتوقيع '{}' يجب أن يكون رقمًا منتهيًا",
            idx, sig
        )),
        _ => Err(format!(
            "المعامل رقم {} للتوقيع '{}' يجب أن يكون رقمًا عشريًا",
            idx, sig
        )),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn expect_i64_arg(sig: &str, args: &[Value], idx: usize) -> Result<i64, String> {
    match args.get(idx) {
        Some(Value::Number(n)) if n.is_finite() => Ok(*n as i64),
        Some(Value::Number(_)) => Err(format!(
            "المعامل رقم {} للتوقيع '{}' يجب أن يكون عددًا منتهيًا",
            idx, sig
        )),
        _ => Err(format!(
            "المعامل رقم {} للتوقيع '{}' يجب أن يكون عددًا صحيحًا",
            idx, sig
        )),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn expect_c_string_arg(sig: &str, args: &[Value], idx: usize) -> Result<std::ffi::CString, String> {
    match args.get(idx) {
        Some(Value::Text(s)) => std::ffi::CString::new(s.as_str()).map_err(|_| {
            format!(
                "المعامل النصي رقم {} للتوقيع '{}' يحتوي على بايت صفري غير مسموح",
                idx, sig
            )
        }),
        _ => Err(format!(
            "المعامل رقم {} للتوقيع '{}' يجب أن يكون نصًا",
            idx, sig
        )),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn require_sig_arity(sig: &str, args: &[Value], expected: usize) -> Result<(), String> {
    if args.len() != expected {
        Err(format!(
            "التوقيع '{}' يتوقع {} معاملات، استلم {}",
            sig,
            expected,
            args.len()
        ))
    } else {
        Ok(())
    }
}

/// استدعاء دالة C أصلية بأمان تام وفق التوقيع النوعي المحدّد
#[cfg(not(target_arch = "wasm32"))]
pub fn call_c_symbol(
    lib_name: &str,
    sym_name: &str,
    signature: &str,
    args: &[Value],
) -> Result<Value, String> {
    use std::ffi::c_char;

    let norm_sig = normalize_signature(signature);

    // التحقق المسبق من التوقيع والوسائط قبل تحميل المكتبة أو استدعاء الرمز
    match norm_sig.as_str() {
        "عشري->عشري" => require_sig_arity(signature, args, 1)?,
        "عشري,عشري->عشري" => require_sig_arity(signature, args, 2)?,
        "صحيح->صحيح" => require_sig_arity(signature, args, 1)?,
        "صحيح,صحيح->صحيح" => require_sig_arity(signature, args, 2)?,
        "نص->صحيح" => require_sig_arity(signature, args, 1)?,
        "->صحيح" => require_sig_arity(signature, args, 0)?,
        _ => {
            return Err(format!(
                "التوقيع '{}' غير مدعوم. التوقيعات المدعومة: 'عشري->عشري'، 'عشري،عشري->عشري'، 'صحيح->صحيح'، 'صحيح،صحيح->صحيح'، 'نص->صحيح'، '->صحيح'",
                signature
            ));
        }
    }

    let lib = open_library_with_fallback(lib_name)?;
    let sym = lib.symbol(sym_name)?;

    match norm_sig.as_str() {
        "عشري->عشري" => {
            let a = expect_f64_arg(signature, args, 0)?;
            // SAFETY: caller selected C ABI signature `extern "C" fn(f64) -> f64` for `sym`.
            let func: extern "C" fn(f64) -> f64 = unsafe { std::mem::transmute(sym) };
            Ok(Value::Number(func(a)))
        }
        "عشري,عشري->عشري" => {
            let a = expect_f64_arg(signature, args, 0)?;
            let b = expect_f64_arg(signature, args, 1)?;
            // SAFETY: caller selected C ABI signature `extern "C" fn(f64, f64) -> f64` for `sym`.
            let func: extern "C" fn(f64, f64) -> f64 = unsafe { std::mem::transmute(sym) };
            Ok(Value::Number(func(a, b)))
        }
        "صحيح->صحيح" => {
            let a = expect_i64_arg(signature, args, 0)?;
            // SAFETY: caller selected C ABI signature `extern "C" fn(i64) -> i64` for `sym`.
            let func: extern "C" fn(i64) -> i64 = unsafe { std::mem::transmute(sym) };
            Ok(Value::Number(func(a) as f64))
        }
        "صحيح,صحيح->صحيح" => {
            let a = expect_i64_arg(signature, args, 0)?;
            let b = expect_i64_arg(signature, args, 1)?;
            // SAFETY: caller selected C ABI signature `extern "C" fn(i64, i64) -> i64` for `sym`.
            let func: extern "C" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(sym) };
            Ok(Value::Number(func(a, b) as f64))
        }
        "نص->صحيح" => {
            let c_str = expect_c_string_arg(signature, args, 0)?;
            // SAFETY: caller selected C ABI signature `extern "C" fn(*const c_char) -> usize`
            // and `c_str` remains alive for the full duration of the call.
            let func: extern "C" fn(*const c_char) -> usize = unsafe { std::mem::transmute(sym) };
            Ok(Value::Number(func(c_str.as_ptr()) as f64))
        }
        "->صحيح" => {
            // SAFETY: caller selected C ABI signature `extern "C" fn() -> i64` for `sym`.
            let func: extern "C" fn() -> i64 = unsafe { std::mem::transmute(sym) };
            Ok(Value::Number(func() as f64))
        }
        _ => unreachable!(),
    }
}

/// في بيئة WebAssembly لا يُسمح بالربط الديناميكي لمكتبات نظام التشغيل
#[cfg(target_arch = "wasm32")]
pub fn call_c_symbol(
    _lib_name: &str,
    _sym_name: &str,
    _signature: &str,
    _args: &[Value],
) -> Result<Value, String> {
    let _ = normalize_signature(_signature);
    Err("استدعاء المكتبات الأصلية (C ABI FFI) غير مدعوم داخل بيئة WebAssembly".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_ffi_parses_cleanly() {
        let tokens = crate::lexer::tokenize(STDLIB_FFI).expect("تعذّر تحليل رموز STDLIB_FFI");
        let stmts = crate::parser::parse(tokens).expect("تعذّر تحليل بنية STDLIB_FFI");
        assert!(!stmts.is_empty());
    }

    #[test]
    fn test_builtin_registration_and_aliases() {
        assert!(is_ffi_builtin("مكتبة_نظام_متوفرة"));
        assert!(is_ffi_builtin("استدعِ_دالة_نظام"));
        assert!(is_ffi_builtin("استدع_دالة_نظام"));
        assert!(!is_ffi_builtin("دالة_غير_موجودة"));

        let math_cands = resolve_library_candidates("مكتبة_الرياضيات");
        assert!(!math_cands.is_empty());
        let libc_cands = resolve_library_candidates("مكتبة_النظام");
        assert!(!libc_cands.is_empty());
        assert_eq!(resolve_library_candidates("custom.so"), vec!["custom.so"]);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_native_c_math_cos_and_pow() {
        let avail = eval_ffi_builtin(
            "مكتبة_نظام_متوفرة",
            vec![Value::Text("مكتبة_الرياضيات".to_string())],
        )
        .unwrap();
        assert_eq!(avail, Value::Bool(true));

        let missing = eval_ffi_builtin(
            "مكتبة_نظام_متوفرة",
            vec![Value::Text("lib_nonexistent_lisan_test_999.so".to_string())],
        )
        .unwrap();
        assert_eq!(missing, Value::Bool(false));

        // cos(0.0) == 1.0
        let cos_res = eval_ffi_builtin(
            "استدعِ_دالة_نظام",
            vec![
                Value::Text("مكتبة_الرياضيات".to_string()),
                Value::Text("cos".to_string()),
                Value::Text("عشري->عشري".to_string()),
                Value::list(vec![Value::Number(0.0)]),
            ],
        )
        .expect("فشل استدعاء cos من مكتبة_الرياضيات");
        match cos_res {
            Value::Number(n) => assert!((n - 1.0).abs() < 1e-12, "cos(0.0) = {}", n),
            other => panic!("متوقع رقم، وصل: {:?}", other),
        }

        // pow(2.0, 10.0) == 1024.0 (باستخدام الفاصلة العربية ،)
        let pow_res = eval_ffi_builtin(
            "استدع_دالة_نظام",
            vec![
                Value::Text("libm".to_string()),
                Value::Text("pow".to_string()),
                Value::Text("عشري،عشري->عشري".to_string()),
                Value::list(vec![Value::Number(2.0), Value::Number(10.0)]),
            ],
        )
        .expect("فشل استدعاء pow من libm");
        match pow_res {
            Value::Number(n) => assert!((n - 1024.0).abs() < 1e-9, "pow(2,10) = {}", n),
            other => panic!("متوقع رقم، وصل: {:?}", other),
        }

        // pow(3.0, 4.0) == 81.0 (باستخدام الفاصلة اللاتينية ,)
        let pow_ascii = call_c_symbol(
            "مكتبة_الرياضيات",
            "pow",
            "عشري,عشري->عشري",
            &[Value::Number(3.0), Value::Number(4.0)],
        )
        .expect("فشل استدعاء pow بالفاصلة اللاتينية");
        assert_eq!(pow_ascii, Value::Number(81.0));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_native_c_libc_strlen_and_llabs() {
        // "مرحبا" في UTF-8 تتكون من 5 أحرف عربية × 2 بايت = 10 بايت
        let len_res = eval_ffi_builtin(
            "استدعِ_دالة_نظام",
            vec![
                Value::Text("مكتبة_النظام".to_string()),
                Value::Text("strlen".to_string()),
                Value::Text("نص->صحيح".to_string()),
                Value::list(vec![Value::Text("مرحبا".to_string())]),
            ],
        )
        .expect("فشل استدعاء strlen من مكتبة_النظام");
        assert_eq!(len_res, Value::Number(10.0));

        // llabs(-42) == 42
        let abs_res = call_c_symbol(
            "libc",
            "llabs",
            "صحيح->صحيح",
            &[Value::Number(-42.0)],
        )
        .expect("فشل استدعاء llabs من libc");
        assert_eq!(abs_res, Value::Number(42.0));
    }

    #[cfg(all(unix, not(target_arch = "wasm32")))]
    #[test]
    fn test_native_c_unix_getpid() {
        let pid_res = eval_ffi_builtin(
            "استدعِ_دالة_نظام",
            vec![
                Value::Text("مكتبة_النظام".to_string()),
                Value::Text("getpid".to_string()),
                Value::Text("->صحيح".to_string()),
            ],
        )
        .expect("فشل استدعاء getpid");
        match pid_res {
            Value::Number(pid) => assert!(pid > 0.0, "pid غير صالح: {}", pid),
            other => panic!("متوقع رقم pid، وصل: {:?}", other),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_ffi_validation_errors() {
        let bad_sig = call_c_symbol("مكتبة_الرياضيات", "cos", "غير_معروف->نص", &[]);
        assert!(bad_sig.is_err());

        let bad_arity = call_c_symbol("مكتبة_الرياضيات", "cos", "عشري->عشري", &[]);
        assert!(bad_arity.is_err());

        let bad_type = call_c_symbol(
            "مكتبة_الرياضيات",
            "cos",
            "عشري->عشري",
            &[Value::Text("نص".to_string())],
        );
        assert!(bad_type.is_err());

        let missing_sym = call_c_symbol(
            "مكتبة_الرياضيات",
            "symbol_does_not_exist_lisan_xyz",
            "عشري->عشري",
            &[Value::Number(1.0)],
        );
        assert!(missing_sym.is_err());
    }
}
