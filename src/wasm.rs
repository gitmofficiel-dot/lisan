use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// تنفيذ شفرة لسان داخل بيئة WebAssembly
/// تستقبل مؤشراً لنص UTF-8، وتعيد مؤشراً لنص المخرجات المفصولة بأسطر جديدة
#[no_mangle]
pub extern "C" fn lisan_eval_wasm(ptr: *const c_char) -> *mut c_char {
    if ptr.is_null() {
        return CString::new("خطأ: مؤشر الإدخال فارغ").unwrap().into_raw();
    }
    let c_str = unsafe { CStr::from_ptr(ptr) };
    let source = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new("خطأ: نص غير متوافق مع UTF-8").unwrap().into_raw(),
    };

    let output = match crate::run(source) {
        Ok(lines) => lines.join("\n"),
        Err(err) => format!("خطأ: {}", err),
    };

    CString::new(output).unwrap_or_else(|_| CString::new("خطأ غير متوقع").unwrap()).into_raw()
}

/// تحرير الذاكرة المخصصة للنص بعد استلامه في JavaScript
#[no_mangle]
pub extern "C" fn lisan_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// تخصيص كتلة ذاكرة من طرف JavaScript لإرسال الكود
#[no_mangle]
pub extern "C" fn lisan_alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// تحرير كتلة الذاكرة
#[no_mangle]
pub extern "C" fn lisan_dealloc(ptr: *mut u8, size: usize) {
    if !ptr.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, 0, size);
        }
    }
}
