//! أدوات الأنظمة وأتمتة نظام التشغيل وسطر الأوامر ومحرك الألعاب ثنائي الأبعاد للغة لسان.
//! مبنية بالكامل بلغة Rust القياسية (`std` فقط) دون أي حزم خارجية، مع أمان كامل لبيئة WebAssembly.

use crate::runtime::Value;
use std::path::{Path, PathBuf};

/// الحد الأقصى لطول النص بعد فك ضغط RLE لحماية الذاكرة
const MAX_RLE_OUTPUT_CHARS: usize = 2_000_000;

/// جميع أسماء الدوال المدمجة للأنظمة والمسارات والملفات والأوامر والضغط (بالعربية والإنجليزية)
pub const ALL_SYS_BUILTINS: &[&str] = &[
    // المسارات (Path)
    "مسار_ادمج",
    "path_join",
    "اسم_ملف",
    "path_basename",
    "امتداد_ملف",
    "path_ext",
    "مجلد_أب",
    "مجلد_اب",
    "path_dirname",
    // عمليات الملفات والمجلدات
    "انسخ_ملف",
    "copy_file",
    "انقل_ملف",
    "move_file",
    "حجم_ملف",
    "file_size",
    "احذف_مجلد",
    "remove_dir",
    // تنفيذ أوامر نظام التشغيل
    "نفّذ_أمر",
    "نفذ_أمر",
    "نفذ_امر",
    "run_command",
    // التحقق من السلامة والضغط (CRC32 & RLE)
    "بصمة_crc32",
    "crc32",
    "ضغط_rle",
    "rle_compress",
    "فك_ضغط_rle",
    "rle_decompress",
];

/// التحقق مما إذا كان الاسم يشير إلى دالة مدمجة في وحدة الأنظمة والألعاب
pub fn is_sys_builtin(name: &str) -> bool {
    ALL_SYS_BUILTINS.contains(&name)
}

fn require_sys_args(name: &str, args: &[Value], expected: usize) -> Result<(), String> {
    if args.len() != expected {
        Err(format!(
            "الدالة '{}' تتوقع {} معاملات، استلمت {}",
            name,
            expected,
            args.len()
        ))
    } else {
        Ok(())
    }
}

fn expect_sys_text<'a>(name: &str, val: &'a Value) -> Result<&'a str, String> {
    match val {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("الدالة '{}' تتطلب قيمة نصية", name)),
    }
}

/// تنفيذ دالة مدمجة من وحدة الأنظمة والمسارات والأوامر والضغط
pub fn eval_sys_builtin(name: &str, args: Vec<Value>) -> Result<Value, String> {
    match name {
        // ==================== 1. أدوات المسارات ====================
        "مسار_ادمج" | "path_join" => {
            if args.is_empty() {
                return Err(format!(
                    "الدالة '{}' تتطلب قائمة أجزاء نصية أو مسارات نصية",
                    name
                ));
            }
            let mut path_buf = PathBuf::new();
            if args.len() == 1 {
                match &args[0] {
                    Value::List(items) => {
                        let borrowed = items.borrow();
                        for item in borrowed.iter() {
                            match item {
                                Value::Text(part) => path_buf.push(part),
                                _ => {
                                    return Err(format!(
                                        "جميع عناصر القائمة في '{}' يجب أن تكون نصوصًا",
                                        name
                                    ))
                                }
                            }
                        }
                    }
                    Value::Text(single) => {
                        path_buf.push(single);
                    }
                    _ => {
                        return Err(format!(
                            "الدالة '{}' تتطلب قائمة أجزاء نصية",
                            name
                        ))
                    }
                }
            } else {
                for item in &args {
                    match item {
                        Value::Text(part) => path_buf.push(part),
                        _ => {
                            return Err(format!(
                                "جميع معاملات '{}' يجب أن تكون نصوصًا",
                                name
                            ))
                        }
                    }
                }
            }
            Ok(Value::Text(path_buf.to_string_lossy().into_owned()))
        }

        "اسم_ملف" | "path_basename" => {
            require_sys_args(name, &args, 1)?;
            let raw = expect_sys_text(name, &args[0])?;
            let basename = Path::new(raw)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            Ok(Value::Text(basename))
        }

        "امتداد_ملف" | "path_ext" => {
            require_sys_args(name, &args, 1)?;
            let raw = expect_sys_text(name, &args[0])?;
            let ext = Path::new(raw)
                .extension()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            Ok(Value::Text(ext))
        }

        "مجلد_أب" | "مجلد_اب" | "path_dirname" => {
            require_sys_args(name, &args, 1)?;
            let raw = expect_sys_text(name, &args[0])?;
            let parent = Path::new(raw)
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            Ok(Value::Text(parent))
        }

        // ==================== 2. عمليات نظام الملفات ====================
        "انسخ_ملف" | "copy_file" => {
            require_sys_args(name, &args, 2)?;
            let src = expect_sys_text(name, &args[0])?;
            let dst = expect_sys_text(name, &args[1])?;
            #[cfg(not(target_arch = "wasm32"))]
            {
                let bytes = std::fs::copy(src, dst)
                    .map_err(|e| format!("تعذّر نسخ الملف من '{}' إلى '{}': {}", src, dst, e))?;
                Ok(Value::Number(bytes as f64))
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = (src, dst);
                Err("نسخ الملفات غير متاح في بيئة المتصفح".into())
            }
        }

        "انقل_ملف" | "move_file" => {
            require_sys_args(name, &args, 2)?;
            let src = expect_sys_text(name, &args[0])?;
            let dst = expect_sys_text(name, &args[1])?;
            #[cfg(not(target_arch = "wasm32"))]
            {
                std::fs::rename(src, dst)
                    .map_err(|e| format!("تعذّر نقل الملف من '{}' إلى '{}': {}", src, dst, e))?;
                Ok(Value::Bool(true))
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = (src, dst);
                Err("نقل الملفات غير متاح في بيئة المتصفح".into())
            }
        }

        "حجم_ملف" | "file_size" => {
            require_sys_args(name, &args, 1)?;
            let path = expect_sys_text(name, &args[0])?;
            #[cfg(not(target_arch = "wasm32"))]
            {
                let meta = std::fs::metadata(path)
                    .map_err(|e| format!("تعذّرت قراءة حجم الملف '{}': {}", path, e))?;
                Ok(Value::Number(meta.len() as f64))
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = path;
                Err("قراءة حجم الملف غير متاحة في بيئة المتصفح".into())
            }
        }

        "احذف_مجلد" | "remove_dir" => {
            require_sys_args(name, &args, 1)?;
            let path = expect_sys_text(name, &args[0])?;
            let trimmed = path.trim();
            if trimmed.is_empty() || trimmed == "/" || trimmed == "\\" {
                return Err("مسار غير آمن لحذف المجلد".into());
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                std::fs::remove_dir_all(trimmed)
                    .map_err(|e| format!("تعذّر حذف المجلد '{}': {}", trimmed, e))?;
                Ok(Value::Bool(true))
            }
            #[cfg(target_arch = "wasm32")]
            {
                Err("حذف المجلدات غير متاح في بيئة المتصفح".into())
            }
        }

        // ==================== 3. تنفيذ أوامر النظام ====================
        "نفّذ_أمر" | "نفذ_أمر" | "نفذ_امر" | "run_command" => {
            if !(1..=2).contains(&args.len()) {
                return Err(format!(
                    "الدالة '{}' تتوقع معاملًا واحدًا أو معاملين (الأمر، وقائمة الوسائط الاختيارية)",
                    name
                ));
            }
            let cmd = expect_sys_text(name, &args[0])?;
            if cmd.trim().is_empty() {
                return Err("اسم الأمر المراد تنفيذه لا يمكن أن يكون فارغًا".into());
            }
            let mut cmd_args: Vec<String> = Vec::new();
            if args.len() == 2 {
                match &args[1] {
                    Value::List(items) => {
                        let borrowed = items.borrow();
                        for item in borrowed.iter() {
                            match item {
                                Value::Text(s) => cmd_args.push(s.clone()),
                                _ => {
                                    return Err(format!(
                                        "جميع وسائط الأمر في '{}' يجب أن تكون نصوصًا",
                                        name
                                    ))
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(format!(
                            "المعامل الثاني للدالة '{}' يجب أن يكون قائمة نصوص للوسائط",
                            name
                        ))
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let output = std::process::Command::new(cmd)
                    .args(&cmd_args)
                    .output()
                    .map_err(|e| format!("تعذّر تنفيذ الأمر '{}': {}", cmd, e))?;
                let success = output.status.success();
                let exit_code = output.status.code().unwrap_or(-1) as f64;
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                Ok(Value::map(vec![
                    ("نجاح".to_string(), Value::Bool(success)),
                    ("كود".to_string(), Value::Number(exit_code)),
                    ("مخرج".to_string(), Value::Text(stdout)),
                    ("خطأ_مخرج".to_string(), Value::Text(stderr)),
                ]))
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = (cmd, cmd_args);
                Err("تنفيذ أوامر النظام غير متاح في بيئة المتصفح".into())
            }
        }

        // ==================== 4. بصمة CRC-32 وضغط RLE ====================
        "بصمة_crc32" | "crc32" => {
            require_sys_args(name, &args, 1)?;
            let text = expect_sys_text(name, &args[0])?;
            Ok(Value::Text(crc32_hex(text.as_bytes())))
        }

        "ضغط_rle" | "rle_compress" => {
            require_sys_args(name, &args, 1)?;
            let text = expect_sys_text(name, &args[0])?;
            Ok(Value::Text(rle_compress(text)))
        }

        "فك_ضغط_rle" | "rle_decompress" => {
            require_sys_args(name, &args, 1)?;
            let encoded = expect_sys_text(name, &args[0])?;
            Ok(Value::Text(rle_decompress(encoded)?))
        }

        _ => Err(format!("دالة أنظمة غير معروفة: '{}'", name)),
    }
}

// ============================================================================
// خوارزمية IEEE 802.3 CRC-32 الصافية
// ============================================================================

/// حساب بصمة CRC-32 القياسية (IEEE 802.3 — متعدد الحدود المعكوس 0xEDB88320)
pub fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// إرجاع بصمة CRC-32 كنص ست عشري صغير بطول 8 أحرف
pub fn crc32_hex(data: &[u8]) -> String {
    format!("{:08x}", crc32_ieee(data))
}

// ============================================================================
// ضغط وفك ضغط RLE غير الفاقد لنصوص UTF-8 الكاملة (عربي + لاتيني + أرقام + رموز)
// ============================================================================

/// ضغط نص UTF-8 بترميز طول التكرار (Run-Length Encoding) غير الفاقد.
/// - يُرمّز التكرار (4 فأكثر) بصيغة `~<العدد>:<الحرف>`.
/// - يُهرّب الحرف الخاص `~` كـ `~~` (للظهور المفرد) أو `~<العدد>:~` (للتكرار).
/// - يضمن استرجاعًا مطابقًا 100% لجميع محارف Unicode والأرقام العربية واللاتينية والرموز.
pub fn rle_compress(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        let mut count: usize = 1;
        while chars.peek() == Some(&ch) {
            chars.next();
            count += 1;
        }

        if ch == '~' {
            if count == 1 {
                out.push_str("~~");
            } else {
                out.push_str(&format!("~{}:~", count));
            }
        } else if count >= 4 {
            out.push_str(&format!("~{}:{}", count, ch));
        } else {
            for _ in 0..count {
                out.push(ch);
            }
        }
    }

    out
}

/// فك ضغط نص مضغوط بصيغة RLE غير الفاقدة الخاصة بلغة لسان
pub fn rle_decompress(encoded: &str) -> Result<String, String> {
    let chars: Vec<char> = encoded.chars().collect();
    let mut out = String::with_capacity(encoded.len());
    let mut out_chars: usize = 0;
    let mut i: usize = 0;

    while i < chars.len() {
        let c = chars[i];
        if c != '~' {
            out_chars += 1;
            if out_chars > MAX_RLE_OUTPUT_CHARS {
                return Err("تجاوز النص المفكوك الحد الأقصى المسموح في RLE".into());
            }
            out.push(c);
            i += 1;
            continue;
        }

        i += 1;
        if i >= chars.len() {
            return Err("بيانات ضغط RLE غير مكتملة بعد علامة '~'".into());
        }

        if chars[i] == '~' {
            out_chars += 1;
            if out_chars > MAX_RLE_OUTPUT_CHARS {
                return Err("تجاوز النص المفكوك الحد الأقصى المسموح في RLE".into());
            }
            out.push('~');
            i += 1;
            continue;
        }

        let start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        if i == start || i >= chars.len() || chars[i] != ':' {
            return Err("صيغة ضغط RLE غير صالحة: متوقع عدد متبوع بنقطتين ':'".into());
        }
        let num_str: String = chars[start..i].iter().collect();
        let count: usize = num_str
            .parse()
            .map_err(|_| "عدد التكرار في بيانات RLE خارج النطاق المسموح".to_string())?;
        if count == 0 {
            return Err("عدد التكرار في بيانات RLE يجب أن يكون أكبر من صفر".into());
        }

        i += 1; // تخطي ':'
        if i >= chars.len() {
            return Err("بيانات ضغط RLE غير مكتملة: الحرف المكرر مفقود".into());
        }
        let repeated = chars[i];
        i += 1;

        out_chars = out_chars
            .checked_add(count)
            .ok_or_else(|| "تجاوز النص المفكوك الحد الأقصى المسموح في RLE".to_string())?;
        if out_chars > MAX_RLE_OUTPUT_CHARS {
            return Err("تجاوز النص المفكوك الحد الأقصى المسموح في RLE".into());
        }
        for _ in 0..count {
            out.push(repeated);
        }
    }

    Ok(out)
}

// ============================================================================
// المكتبة القياسية الكائنية بلغة لسان: أدوات سطر الأوامر، السجلات، المجدول، والألعاب 2D
// ============================================================================

/// المكتبة القياسية الكائنية بلغة لسان للأنظمة وسطر الأوامر والسجلات وجدولة المهام ومحرك الألعاب ثنائي الأبعاد
pub const STDLIB_SYSTEMS_AND_GAMES: &str = concat!(
    r##"
# المكتبة القياسية للأنظمة وسطر الأوامر ومحرك الألعاب ثنائي الأبعاد في لغة لسان

صنف أداة_سطر_أوامر {
    دالة بناء(اسم_الأداة، إصدار = "1.0.0") {
        هذا.اسم_الأداة = اسم_الأداة
        هذا.إصدار = إصدار
    }

    دالة إطار_عنوان(عنوان) {
        دع سطر_أول = "  " + نص(عنوان) + "  "
        دع سطر_ثان = "  " + نص(هذا.اسم_الأداة) + " — الإصدار " + نص(هذا.إصدار) + "  "
        دع عرض_داخلي = طول(سطر_أول)
        إذا (طول(سطر_ثان) > عرض_داخلي) {
            عرض_داخلي = طول(سطر_ثان)
        }
        إذا (عرض_داخلي < 28) {
            عرض_داخلي = 28
        }
        دع حشو_أول = عرض_داخلي - طول(سطر_أول)
        دع حشو_ثان = عرض_داخلي - طول(سطر_ثان)
        دع حد_علوي = "╔" + كرر("═"، عرض_داخلي) + "╗"
        دع حد_وسط = "╟" + كرر("─"، عرض_داخلي) + "╢"
        دع حد_سفلي = "╚" + كرر("═"، عرض_داخلي) + "╝"
        دع نتيجة = حد_علوي + "\n"
        نتيجة = نتيجة + "║" + سطر_أول + كرر(" "، حشو_أول) + "║\n"
        نتيجة = نتيجة + حد_وسط + "\n"
        نتيجة = نتيجة + "║" + سطر_ثان + كرر(" "، حشو_ثان) + "║\n"
        نتيجة = نتيجة + حد_سفلي
        أرجع نتيجة
    }

    دالة جدول_طرفية(ترويسات، صفوف) {
        دع عدد_أعمدة = طول(ترويسات)
        إذا (عدد_أعمدة == 0) {
            أرجع ""
        }
        دع عروض = []
        دع مؤشر_ع = 0
        طالما (مؤشر_ع < عدد_أعمدة) {
            دع أقصى = طول(نص(ترويسات[مؤشر_ع]))
            لكل صف_بيانات في صفوف {
                إذا (مؤشر_ع < طول(صف_بيانات)) {
                    دع عرض_خلية = طول(نص(صف_بيانات[مؤشر_ع]))
                    إذا (عرض_خلية > أقصى) {
                        أقصى = عرض_خلية
                    }
                }
            }
            عروض = أضف(عروض، أقصى)
            مؤشر_ع = مؤشر_ع + 1
        }

        دع خط_علوي = "┌"
        دع خط_فاصل = "├"
        دع خط_سفلي = "└"
        دع ف = 0
        طالما (ف < عدد_أعمدة) {
            دع شرطة = كرر("─"، عروض[ف] + 2)
            خط_علوي = خط_علوي + شرطة
            خط_فاصل = خط_فاصل + شرطة
            خط_سفلي = خط_سفلي + شرطة
            إذا (ف + 1 < عدد_أعمدة) {
                خط_علوي = خط_علوي + "┬"
                خط_فاصل = خط_فاصل + "┼"
                خط_سفلي = خط_سفلي + "┴"
            } وإلا {
                خط_علوي = خط_علوي + "┐"
                خط_فاصل = خط_فاصل + "┤"
                خط_سفلي = خط_سفلي + "┘"
            }
            ف = ف + 1
        }

        دع سطر_ترويسة = "│"
        ف = 0
        طالما (ف < عدد_أعمدة) {
            دع نص_ترويسة = نص(ترويسات[ف])
            دع مسافات = عروض[ف] - طول(نص_ترويسة)
            سطر_ترويسة = سطر_ترويسة + " " + نص_ترويسة + كرر(" "، مسافات) + " │"
            ف = ف + 1
        }

        دع مخرج_جدول = خط_علوي + "\n" + سطر_ترويسة + "\n" + خط_فاصل + "\n"
        لكل صف_بيانات في صفوف {
            دع سطر_صف = "│"
            ف = 0
            طالما (ف < عدد_أعمدة) {
                دع نص_خلية = ""
                إذا (ف < طول(صف_بيانات)) {
                    نص_خلية = نص(صف_بيانات[ف])
                }
                دع مسافات_خلية = عروض[ف] - طول(نص_خلية)
                سطر_صف = سطر_صف + " " + نص_خلية + كرر(" "، مسافات_خلية) + " │"
                ف = ف + 1
            }
            مخرج_جدول = مخرج_جدول + سطر_صف + "\n"
        }
        مخرج_جدول = مخرج_جدول + خط_سفلي
        أرجع مخرج_جدول
    }

    دالة شريط_تقدم_نصي(نسبة، طول_الشريط = 20) {
        دع نسبة_فعلي = نسبة
        إذا (نسبة_فعلي < 0) {
            نسبة_فعلي = 0
        }
        إذا (نسبة_فعلي > 100) {
            نسبة_فعلي = 100
        }
        دع طول_فعلي = عدد_صحيح(طول_الشريط)
        إذا (طول_فعلي < 1) {
            طول_فعلي = 1
        }
        دع مملوء = عدد_صحيح(قرّب((نسبة_فعلي / 100) * طول_فعلي))
        إذا (مملوء < 0) {
            مملوء = 0
        }
        إذا (مملوء > طول_فعلي) {
            مملوء = طول_فعلي
        }
        دع متبقي = طول_فعلي - مملوء
        أرجع "[" + كرر("█"، مملوء) + كرر("░"، متبقي) + "] " + نص(عدد_صحيح(قرّب(نسبة_فعلي))) + "%"
    }

    دالة ملوّن(نص_رسالة، لون = "أخضر") {
        دع رمز_هروب = ""##,
    "\x1b",
    r##""
        دع كود_لون = "32"
        إذا (لون == "أحمر" أو لون == "احمر" أو لون == "red") {
            كود_لون = "31"
        }
        إذا (لون == "أخضر" أو لون == "اخضر" أو لون == "green") {
            كود_لون = "32"
        }
        إذا (لون == "أصفر" أو لون == "اصفر" أو لون == "yellow") {
            كود_لون = "33"
        }
        إذا (لون == "أزرق" أو لون == "ازرق" أو لون == "blue") {
            كود_لون = "34"
        }
        إذا (لون == "بنفسجي" أو لون == "أرجواني" أو لون == "magenta") {
            كود_لون = "35"
        }
        إذا (لون == "سماوي" أو لون == "cyan") {
            كود_لون = "36"
        }
        إذا (لون == "أبيض" أو لون == "ابيض" أو لون == "white") {
            كود_لون = "37"
        }
        إذا (لون == "رمادي" أو لون == "gray") {
            كود_لون = "90"
        }
        إذا (لون == "عريض" أو لون == "bold") {
            كود_لون = "1"
        }
        أرجع رمز_هروب + "[" + كود_لون + "m" + نص(نص_رسالة) + رمز_هروب + "[0m"
    }
}

صنف سجل_أحداث {
    دالة بناء(اسم_الخدمة = "تطبيق") {
        هذا.اسم_الخدمة = اسم_الخدمة
        هذا.السجلات = []
        هذا.العداد = 0
    }

    دالة أضف_حدث(مستوى، رسالة) {
        هذا.العداد = هذا.العداد + 1
        دع وقت_الحدث = عدد_صحيح(وقت_دقيق_ملي())
        دع سجل_جديد = {"رقم": هذا.العداد، "الخدمة": هذا.اسم_الخدمة، "المستوى": مستوى، "الرسالة": نص(رسالة)، "الوقت_ملي": وقت_الحدث}
        هذا.السجلات = أضف(هذا.السجلات، سجل_جديد)
        أرجع سجل_جديد
    }

    دالة معلومات(رسالة) {
        أرجع هذا.أضف_حدث("معلومات"، رسالة)
    }

    دالة تحذير(رسالة) {
        أرجع هذا.أضف_حدث("تحذير"، رسالة)
    }

    دالة عطل(رسالة) {
        أرجع هذا.أضف_حدث("عطل"، رسالة)
    }

    دالة عدد_السجلات() {
        أرجع طول(هذا.السجلات)
    }

    دالة الكل() {
        أرجع هذا.السجلات
    }

    دالة تصدير_json() {
        أرجع ترميز_json(هذا.السجلات)
    }
}

صنف مجدول_مهام {
    دالة بناء(اسم_المسار) {
        هذا.اسم_المسار = اسم_المسار
        هذا.المهام = []
        هذا.سجل_التنفيذ = []
        هذا.عدد_المنفذة = 0
    }

    دالة أضف_مهمة(اسم، أولوية = 1) {
        دع معرف_جديد = طول(هذا.المهام) + 1
        دع مهمة_جديدة = {"معرف": معرف_جديد، "اسم": نص(اسم)، "أولوية": أولوية، "الوضع": "قيد_الانتظار"}
        هذا.المهام = أضف(هذا.المهام، مهمة_جديدة)
        أرجع مهمة_جديدة
    }

    دالة نفّذ_الكل() {
        دع متبقية = []
        لكل عنصر_مهمة في هذا.المهام {
            متبقية = أضف(متبقية، عنصر_مهمة)
        }
        دع مرتبة = []
        طالما (طول(متبقية) > 0) {
            دع أفضل_فهرس = 0
            دع فهرس_بحث = 1
            طالما (فهرس_بحث < طول(متبقية)) {
                دع مهمة_حالية = متبقية[فهرس_بحث]
                دع مهمة_أفضل = متبقية[أفضل_فهرس]
                إذا (مهمة_حالية["أولوية"] > مهمة_أفضل["أولوية"]) {
                    أفضل_فهرس = فهرس_بحث
                }
                فهرس_بحث = فهرس_بحث + 1
            }
            مرتبة = أضف(مرتبة، متبقية[أفضل_فهرس])
            دع باقي_المهام = []
            دع فهرس_نسخ = 0
            طالما (فهرس_نسخ < طول(متبقية)) {
                إذا (فهرس_نسخ != أفضل_فهرس) {
                    باقي_المهام = أضف(باقي_المهام، متبقية[فهرس_نسخ])
                }
                فهرس_نسخ = فهرس_نسخ + 1
            }
            متبقية = باقي_المهام
        }

        دع سجل_تنفيذ_جديد = []
        دع مهام_محدثة = []
        دع ترتيب = 0
        لكل مهمة_عنصر في مرتبة {
            ترتيب = ترتيب + 1
            دع وقت_تنفيذ = عدد_صحيح(وقت_دقيق_ملي())
            دع نسخة_مهمة = {"معرف": مهمة_عنصر["معرف"]، "اسم": مهمة_عنصر["اسم"]، "أولوية": مهمة_عنصر["أولوية"]، "الوضع": "مكتملة"}
            مهام_محدثة = أضف(مهام_محدثة، نسخة_مهمة)
            دع نتيجة_خطوة = {"ترتيب": ترتيب، "معرف": مهمة_عنصر["معرف"]، "اسم": مهمة_عنصر["اسم"]، "أولوية": مهمة_عنصر["أولوية"]، "الوضع": "مكتملة"، "الوقت_ملي": وقت_تنفيذ}
            سجل_تنفيذ_جديد = أضف(سجل_تنفيذ_جديد، نتيجة_خطوة)
        }

        هذا.المهام = مهام_محدثة
        هذا.سجل_التنفيذ = سجل_تنفيذ_جديد
        هذا.عدد_المنفذة = طول(سجل_تنفيذ_جديد)
        أرجع سجل_تنفيذ_جديد
    }

    دالة تقرير() {
        دع نص_تقرير = "=== تقرير مسار المهام: " + نص(هذا.اسم_المسار) + " ===\n"
        نص_تقرير = نص_تقرير + "إجمالي المهام: " + نص(طول(هذا.المهام)) + " | المهام المكتملة: " + نص(هذا.عدد_المنفذة) + "\n"
        دع رقم_سطر = 0
        لكل مهمة_عنصر في هذا.المهام {
            رقم_سطر = رقم_سطر + 1
            نص_تقرير = نص_تقرير + نص(رقم_سطر) + ". [" + نص(مهمة_عنصر["الوضع"]) + "] " + نص(مهمة_عنصر["اسم"]) + " (أولوية: " + نص(مهمة_عنصر["أولوية"]) + ")\n"
        }
        أرجع نص_تقرير
    }
}

صنف لعبة_ثنائية_الأبعاد {
    دالة بناء(عنوان_اللعبة، عرض = 800، ارتفاع = 500، خلفية = "#091d37") {
        هذا.عنوان_اللعبة = عنوان_اللعبة
        هذا.عرض = عرض
        هذا.ارتفاع = ارتفاع
        هذا.خلفية = خلفية
        هذا.اللاعبون = []
        هذا.الأهداف = []
        هذا.العقبات = []
        هذا.نوع_التحكم = "أسهم_ولمس"
    }

    دالة أضف_لاعب(اسم، س، ص، عرض_ك، ارتفاع_ك، لون = "#e65c1a"، سرعة = 6) {
        دع لاعب_جديد = {"name": نص(اسم)، "x": س، "y": ص، "w": عرض_ك، "h": ارتفاع_ك، "color": نص(لون)، "speed": سرعة}
        هذا.اللاعبون = أضف(هذا.اللاعبون، لاعب_جديد)
        أرجع لاعب_جديد
    }

    دالة أضف_هدف(اسم، لون = "#10b981"، نقاط = 10) {
        دع هدف_جديد = {"name": نص(اسم)، "color": نص(لون)، "points": نقاط}
        هذا.الأهداف = أضف(هذا.الأهداف، هدف_جديد)
        أرجع هدف_جديد
    }

    دالة أضف_عقبة(اسم، لون = "#ef4444"، سرعة = 3) {
        دع عقبة_جديدة = {"name": نص(اسم)، "color": نص(لون)، "speed": سرعة}
        هذا.العقبات = أضف(هذا.العقبات، عقبة_جديدة)
        أرجع عقبة_جديدة
    }

    دالة اضبط_تحكم(نوع_التحكم = "أسهم_ولمس") {
        هذا.نوع_التحكم = نص(نوع_التحكم)
        أرجع هذا.نوع_التحكم
    }

    دالة بناء_لعبة_html() {
        دع عنوان_آمن = استبدل(استبدل(استبدل(نص(هذا.عنوان_اللعبة)، "&"، "&amp;")، "<"، "&lt;")، ">"، "&gt;")
        دع إعدادات = {"title": نص(هذا.عنوان_اللعبة)، "width": هذا.عرض، "height": هذا.ارتفاع، "bg": نص(هذا.خلفية)، "control": نص(هذا.نوع_التحكم)، "players": هذا.اللاعبون، "targets": هذا.الأهداف، "obstacles": هذا.العقبات}
        دع json_آمن = استبدل(ترميز_json(إعدادات)، "</"، "<\\/")

        دع كود = "<!DOCTYPE html>\n<html lang=\"ar\" dir=\"rtl\">\n<head>\n<meta charset=\"UTF-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no\">\n"
        كود = كود + "<title>" + عنوان_آمن + " — محرك ألعاب لسان 2D</title>\n<style>\n"
        كود = كود + "* { box-sizing: border-box; margin: 0; padding: 0; user-select: none; -webkit-user-select: none; }\n"
        كود = كود + "body { font-family: system-ui, -apple-system, 'Segoe UI', Tahoma, sans-serif; background: radial-gradient(circle at top, #132c4f 0%, #050d1a 100%); color: #f8fafc; min-height: 100vh; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 14px; gap: 12px; }\n"
        كود = كود + ".game-shell { width: 100%; max-width: 920px; background: rgba(15, 23, 42, 0.85); border: 1px solid rgba(148, 163, 184, 0.25); border-radius: 20px; padding: 16px; box-shadow: 0 20px 60px rgba(0, 0, 0, 0.55); display: flex; flex-direction: column; gap: 12px; }\n"
        كود = كود + ".hud { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 10px; padding: 10px 14px; background: rgba(30, 41, 59, 0.85); border-radius: 14px; border: 1px solid rgba(148, 163, 184, 0.18); }\n"
        كود = كود + ".hud-title { font-size: 20px; font-weight: 800; color: #f8fafc; display: flex; align-items: center; gap: 8px; }\n"
        كود = كود + ".hud-stats { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }\n"
        كود = كود + ".pill { background: rgba(15, 23, 42, 0.9); border: 1px solid rgba(148, 163, 184, 0.25); border-radius: 10px; padding: 6px 12px; font-size: 14px; font-weight: 700; display: inline-flex; align-items: center; gap: 6px; }\n"
        كود = كود + ".pill span { color: #38bdf8; }\n"
        كود = كود + ".pill.score span { color: #10b981; }\n"
        كود = كود + ".pill.best span { color: #f59e0b; }\n"
        كود = كود + ".pill.lives span { color: #f43f5e; }\n"
        كود = كود + ".actions { display: flex; gap: 8px; flex-wrap: wrap; }\n"
        كود = كود + ".btn { background: linear-gradient(135deg, #e65c1a, #f97316); color: #fff; border: none; border-radius: 10px; padding: 8px 14px; font-size: 14px; font-weight: 700; cursor: pointer; transition: transform 0.12s, opacity 0.15s; }\n"
        كود = كود + ".btn:hover { transform: translateY(-1px); opacity: 0.95; }\n"
        كود = كود + ".btn:active { transform: scale(0.97); }\n"
        كود = كود + ".btn.sec { background: rgba(51, 65, 85, 0.95); border: 1px solid rgba(148, 163, 184, 0.3); }\n"
        كود = كود + ".stage-wrap { position: relative; width: 100%; display: flex; justify-content: center; border-radius: 14px; overflow: hidden; border: 2px solid rgba(56, 189, 248, 0.3); background: #020617; }\n"
        كود = كود + "canvas { width: 100%; height: auto; display: block; touch-action: none; }\n"
        كود = كود + ".overlay { position: absolute; inset: 0; background: rgba(2, 6, 23, 0.78); backdrop-filter: blur(4px); display: none; flex-direction: column; align-items: center; justify-content: center; gap: 14px; text-align: center; padding: 20px; }\n"
        كود = كود + ".overlay.visible { display: flex; }\n"
        كود = كود + ".overlay h2 { font-size: 30px; font-weight: 900; color: #f8fafc; }\n"
        كود = كود + ".overlay p { font-size: 16px; color: #cbd5e1; }\n"
        كود = كود + ".touch-pad { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; padding: 4px 2px; }\n"
        كود = كود + ".dpad { display: grid; grid-template-columns: repeat(3, 52px); grid-template-rows: repeat(2, 46px); gap: 6px; direction: ltr; }\n"
        كود = كود + ".tbtn { background: rgba(30, 41, 59, 0.92); color: #f8fafc; border: 1px solid rgba(148, 163, 184, 0.35); border-radius: 12px; font-size: 18px; font-weight: 800; cursor: pointer; display: flex; align-items: center; justify-content: center; touch-action: none; }\n"
        كود = كود + ".tbtn:active, .tbtn.active { background: #0284c7; border-color: #38bdf8; transform: scale(0.95); }\n"
        كود = كود + ".legend { display: flex; flex-wrap: wrap; gap: 10px; align-items: center; font-size: 13px; color: #cbd5e1; }\n"
        كود = كود + ".dot { width: 11px; height: 11px; border-radius: 50%; display: inline-block; margin-left: 5px; vertical-align: middle; }\n"
        كود = كود + ".footer-note { text-align: center; font-size: 12px; color: #94a3b8; }\n"
        كود = كود + "</style>\n</head>\n<body>\n<div class=\"game-shell\">\n"
        كود = كود + "<div class=\"hud\">\n  <div class=\"hud-title\">🚀 " + عنوان_آمن + "</div>\n"
        كود = كود + "  <div class=\"hud-stats\">\n"
        كود = كود + "    <div class=\"pill score\">النقاط: <span id=\"scoreVal\">0</span></div>\n"
        كود = كود + "    <div class=\"pill best\">أعلى نتيجة: <span id=\"bestVal\">0</span></div>\n"
        كود = كود + "    <div class=\"pill lives\">الحياة: <span id=\"livesVal\">❤️❤️❤️</span></div>\n"
        كود = كود + "    <div class=\"pill\">المستوى: <span id=\"levelVal\">1</span></div>\n"
        كود = كود + "  </div>\n"
        كود = كود + "  <div class=\"actions\">\n"
        كود = كود + "    <button class=\"btn sec\" id=\"muteBtn\" type=\"button\">🔊 الصوت</button>\n"
        كود = كود + "    <button class=\"btn sec\" id=\"pauseBtn\" type=\"button\">⏸ إيقاف مؤقت</button>\n"
        كود = كود + "    <button class=\"btn\" id=\"restartTopBtn\" type=\"button\">🔄 إعادة اللعب</button>\n"
        كود = كود + "  </div>\n</div>\n"
        كود = كود + "<div class=\"stage-wrap\">\n"
        كود = كود + "  <canvas id=\"gameCanvas\" width=\"" + نص(هذا.عرض) + "\" height=\"" + نص(هذا.ارتفاع) + "\"></canvas>\n"
        كود = كود + "  <div class=\"overlay\" id=\"gameOverlay\">\n"
        كود = كود + "    <h2 id=\"overlayTitle\">انتهت اللعبة!</h2>\n"
        كود = كود + "    <p id=\"overlayMsg\">نتيجتك النهائية: 0</p>\n"
        كود = كود + "    <button class=\"btn\" id=\"restartOverlayBtn\" type=\"button\">🔄 إعادة اللعب الآن</button>\n"
        كود = كود + "  </div>\n</div>\n"
        كود = كود + "<div class=\"touch-pad\" id=\"touchPad\">\n"
        كود = كود + "  <div class=\"legend\" id=\"legendBox\"></div>\n"
        كود = كود + "  <div class=\"dpad\">\n"
        كود = كود + "    <div></div><button class=\"tbtn\" data-dir=\"up\" type=\"button\">▲</button><div></div>\n"
        كود = كود + "    <button class=\"tbtn\" data-dir=\"left\" type=\"button\">◀</button><button class=\"tbtn\" data-dir=\"down\" type=\"button\">▼</button><button class=\"tbtn\" data-dir=\"right\" type=\"button\">▶</button>\n"
        كود = كود + "  </div>\n</div>\n"
        كود = كود + "<div class=\"footer-note\">التحكم: مفاتيح الأسهم أو WASD أو أزرار اللمس أو السحب باللمس/الفأرة — صُنع بمحرك ألعاب لغة لسان العربية 2D</div>\n"
        كود = كود + "</div>\n<script>\n"
        كود = كود + "const GAME_CONFIG = " + json_آمن + ";\n"
        كود = كود + "const canvas = document.getElementById('gameCanvas');\n"
        كود = كود + "const ctx = canvas.getContext('2d');\n"
        كود = كود + "const W = Number(GAME_CONFIG.width) || 800;\n"
        كود = كود + "const H = Number(GAME_CONFIG.height) || 500;\n"
        كود = كود + "canvas.width = W; canvas.height = H;\n"
        كود = كود + "const STORAGE_KEY = 'lisan_game_best_' + encodeURIComponent(GAME_CONFIG.title || 'arcade');\n"
        كود = كود + "const scoreEl = document.getElementById('scoreVal');\n"
        كود = كود + "const bestEl = document.getElementById('bestVal');\n"
        كود = كود + "const livesEl = document.getElementById('livesVal');\n"
        كود = كود + "const levelEl = document.getElementById('levelVal');\n"
        كود = كود + "const overlayEl = document.getElementById('gameOverlay');\n"
        كود = كود + "const overlayTitle = document.getElementById('overlayTitle');\n"
        كود = كود + "const overlayMsg = document.getElementById('overlayMsg');\n"
        كود = كود + "const pauseBtn = document.getElementById('pauseBtn');\n"
        كود = كود + "const muteBtn = document.getElementById('muteBtn');\n"
        كود = كود + "const legendBox = document.getElementById('legendBox');\n"
        كود = كود + "const basePlayer = (GAME_CONFIG.players && GAME_CONFIG.players.length > 0) ? GAME_CONFIG.players[0] : { name: 'البطل', x: W/2 - 22, y: H - 72, w: 44, h: 44, color: '#e65c1a', speed: 6 };\n"
        كود = كود + "const targetDefs = (GAME_CONFIG.targets && GAME_CONFIG.targets.length > 0) ? GAME_CONFIG.targets : [{ name: 'نجمة الطاقة', color: '#10b981', points: 10 }];\n"
        كود = كود + "const obstacleDefs = (GAME_CONFIG.obstacles && GAME_CONFIG.obstacles.length > 0) ? GAME_CONFIG.obstacles : [{ name: 'نيزك', color: '#ef4444', speed: 3 }];\n"
        كود = كود + "function escHtml(v) { return String(v).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/\"/g, '&quot;'); }\n"
        كود = كود + "function renderLegend() {\n"
        كود = كود + "  let html = '<span><i class=\"dot\" style=\"background:' + escHtml(basePlayer.color) + '\"></i>اللاعب: ' + escHtml(basePlayer.name) + '</span>';\n"
        كود = كود + "  targetDefs.forEach(t => { html += '<span><i class=\"dot\" style=\"background:' + escHtml(t.color) + '\"></i>' + escHtml(t.name) + ' (+' + Number(t.points) + ')</span>'; });\n"
        كود = كود + "  obstacleDefs.forEach(o => { html += '<span><i class=\"dot\" style=\"background:' + escHtml(o.color) + '\"></i>' + escHtml(o.name) + '</span>'; });\n"
        كود = كود + "  legendBox.innerHTML = html;\n"
        كود = كود + "}\nrenderLegend();\n"
        كود = كود + "if (GAME_CONFIG.control === 'لوحة_مفاتيح' || GAME_CONFIG.control === 'keyboard') { const dp = document.querySelector('.dpad'); if (dp) dp.style.display = 'none'; }\n"
        كود = كود + "let audioCtx = null;\nlet soundMuted = false;\n"
        كود = كود + "function playTone(freq, duration, type, gainVal) {\n"
        كود = كود + "  if (soundMuted) return;\n"
        كود = كود + "  try {\n"
        كود = كود + "    const AC = window.AudioContext || window.webkitAudioContext;\n"
        كود = كود + "    if (!AC) return;\n"
        كود = كود + "    if (!audioCtx) audioCtx = new AC();\n"
        كود = كود + "    if (audioCtx.state === 'suspended') audioCtx.resume();\n"
        كود = كود + "    const osc = audioCtx.createOscillator();\n"
        كود = كود + "    const gain = audioCtx.createGain();\n"
        كود = كود + "    osc.type = type || 'sine';\n"
        كود = كود + "    osc.frequency.setValueAtTime(freq, audioCtx.currentTime);\n"
        كود = كود + "    osc.frequency.exponentialRampToValueAtTime(Math.max(60, freq * 1.35), audioCtx.currentTime + duration);\n"
        كود = كود + "    gain.gain.setValueAtTime(gainVal || 0.12, audioCtx.currentTime);\n"
        كود = كود + "    gain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + duration);\n"
        كود = كود + "    osc.connect(gain); gain.connect(audioCtx.destination);\n"
        كود = كود + "    osc.start(); osc.stop(audioCtx.currentTime + duration);\n"
        كود = كود + "  } catch (_) {}\n"
        كود = كود + "}\n"
        كود = كود + "const stars = Array.from({ length: 75 }, () => ({ x: Math.random() * W, y: Math.random() * H, r: Math.random() * 1.8 + 0.5, s: Math.random() * 1.4 + 0.3 }));\n"
        كود = كود + "let player, targets, obstacles, particles, popups, score, bestScore, lives, level, paused, gameOver, invuln, shakeTime, lastTime;\n"
        كود = كود + "try { bestScore = Number(localStorage.getItem(STORAGE_KEY) || 0); } catch (_) { bestScore = 0; }\n"
        كود = كود + "bestEl.textContent = String(bestScore);\n"
        كود = كود + "const keys = { up: false, down: false, left: false, right: false };\n"
        كود = كود + "let pointerTarget = null;\n"
        كود = كود + "function spawnTarget(def) {\n"
        كود = كود + "  const r = 15;\n"
        كود = كود + "  return { name: def.name, color: def.color, points: Number(def.points) || 10, x: r + Math.random() * (W - r * 2), y: -20 - Math.random() * (H * 0.6), r: r, vy: 1.6 + Math.random() * 1.6, angle: Math.random() * Math.PI * 2 };\n"
        كود = كود + "}\n"
        كود = كود + "function spawnObstacle(def) {\n"
        كود = كود + "  const size = 28 + Math.random() * 18;\n"
        كود = كود + "  const spd = Number(def.speed) || 3;\n"
        كود = كود + "  return { name: def.name, color: def.color, baseSpeed: spd, x: Math.random() * (W - size), y: -size - Math.random() * (H * 0.7), w: size, h: size, vx: (Math.random() - 0.5) * 1.6, vy: spd + Math.random() * 1.5, angle: 0, spin: (Math.random() - 0.5) * 0.08 };\n"
        كود = كود + "}\n"
        كود = كود + "function addBurst(x, y, color, count) {\n"
        كود = كود + "  for (let i = 0; i < count; i++) {\n"
        كود = كود + "    const a = Math.random() * Math.PI * 2;\n"
        كود = كود + "    const sp = 1.5 + Math.random() * 4.5;\n"
        كود = كود + "    particles.push({ x, y, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp, life: 1, decay: 0.02 + Math.random() * 0.025, r: 2 + Math.random() * 3.5, color });\n"
        كود = كود + "  }\n"
        كود = كود + "}\n"
        كود = كود + "function updateHud() {\n"
        كود = كود + "  scoreEl.textContent = String(score);\n"
        كود = كود + "  bestEl.textContent = String(bestScore);\n"
        كود = كود + "  livesEl.textContent = lives > 0 ? '❤️'.repeat(lives) : '💔';\n"
        كود = كود + "  levelEl.textContent = String(level);\n"
        كود = كود + "}\n"
        كود = كود + "function resetGame() {\n"
        كود = كود + "  player = { name: basePlayer.name, x: Number(basePlayer.x) || W/2 - 22, y: Number(basePlayer.y) || H - 72, w: Number(basePlayer.w) || 44, h: Number(basePlayer.h) || 44, color: basePlayer.color || '#e65c1a', speed: Number(basePlayer.speed) || 6 };\n"
        كود = كود + "  targets = [];\n"
        كود = كود + "  for (let i = 0; i < Math.max(3, targetDefs.length * 2); i++) targets.push(spawnTarget(targetDefs[i % targetDefs.length]));\n"
        كود = كود + "  obstacles = [];\n"
        كود = كود + "  for (let i = 0; i < Math.max(4, obstacleDefs.length * 2); i++) obstacles.push(spawnObstacle(obstacleDefs[i % obstacleDefs.length]));\n"
        كود = كود + "  particles = []; popups = []; score = 0; lives = 3; level = 1; paused = false; gameOver = false; invuln = 0; shakeTime = 0; pointerTarget = null;\n"
        كود = كود + "  overlayEl.classList.remove('visible');\n"
        كود = كود + "  pauseBtn.textContent = '⏸ إيقاف مؤقت';\n"
        كود = كود + "  updateHud();\n"
        كود = كود + "}\n"
        كود = كود + "function setKey(code, pressed) {\n"
        كود = كود + "  if (code === 'ArrowUp' || code === 'KeyW') keys.up = pressed;\n"
        كود = كود + "  if (code === 'ArrowDown' || code === 'KeyS') keys.down = pressed;\n"
        كود = كود + "  if (code === 'ArrowLeft' || code === 'KeyA') keys.left = pressed;\n"
        كود = كود + "  if (code === 'ArrowRight' || code === 'KeyD') keys.right = pressed;\n"
        كود = كود + "}\n"
        كود = كود + "window.addEventListener('keydown', e => {\n"
        كود = كود + "  if (['ArrowUp','ArrowDown','ArrowLeft','ArrowRight','Space'].includes(e.code)) e.preventDefault();\n"
        كود = كود + "  if (e.code === 'KeyP') { togglePause(); return; }\n"
        كود = كود + "  if (e.code === 'KeyR' && gameOver) { resetGame(); return; }\n"
        كود = كود + "  setKey(e.code, true);\n"
        كود = كود + "});\n"
        كود = كود + "window.addEventListener('keyup', e => setKey(e.code, false));\n"
        كود = كود + "document.querySelectorAll('.tbtn').forEach(btn => {\n"
        كود = كود + "  const dir = btn.getAttribute('data-dir');\n"
        كود = كود + "  const on = ev => { ev.preventDefault(); keys[dir] = true; btn.classList.add('active'); };\n"
        كود = كود + "  const off = ev => { ev.preventDefault(); keys[dir] = false; btn.classList.remove('active'); };\n"
        كود = كود + "  btn.addEventListener('pointerdown', on); btn.addEventListener('pointerup', off); btn.addEventListener('pointerleave', off); btn.addEventListener('pointercancel', off);\n"
        كود = كود + "});\n"
        كود = كود + "canvas.addEventListener('pointerdown', e => {\n"
        كود = كود + "  const r = canvas.getBoundingClientRect();\n"
        كود = كود + "  pointerTarget = { x: (e.clientX - r.left) * (W / r.width), y: (e.clientY - r.top) * (H / r.height) };\n"
        كود = كود + "});\n"
        كود = كود + "canvas.addEventListener('pointermove', e => {\n"
        كود = كود + "  if (!pointerTarget) return;\n"
        كود = كود + "  const r = canvas.getBoundingClientRect();\n"
        كود = كود + "  pointerTarget = { x: (e.clientX - r.left) * (W / r.width), y: (e.clientY - r.top) * (H / r.height) };\n"
        كود = كود + "});\n"
        كود = كود + "window.addEventListener('pointerup', () => { pointerTarget = null; });\n"
        كود = كود + "function togglePause() {\n"
        كود = كود + "  if (gameOver) return;\n"
        كود = كود + "  paused = !paused;\n"
        كود = كود + "  pauseBtn.textContent = paused ? '▶ استئناف' : '⏸ إيقاف مؤقت';\n"
        كود = كود + "}\n"
        كود = كود + "pauseBtn.addEventListener('click', togglePause);\n"
        كود = كود + "muteBtn.addEventListener('click', () => { soundMuted = !soundMuted; muteBtn.textContent = soundMuted ? '🔇 صامت' : '🔊 الصوت'; });\n"
        كود = كود + "document.getElementById('restartTopBtn').addEventListener('click', resetGame);\n"
        كود = كود + "document.getElementById('restartOverlayBtn').addEventListener('click', resetGame);\n"
        كود = كود + "function rectHit(ax, ay, aw, ah, bx, by, bw, bh) { return ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by; }\n"
        كود = كود + "function update(dt) {\n"
        كود = كود + "  const step = Math.min(dt, 0.05) * 60;\n"
        كود = كود + "  stars.forEach(s => { s.y += s.s * step; if (s.y > H) { s.y = 0; s.x = Math.random() * W; } });\n"
        كود = كود + "  if (paused || gameOver) return;\n"
        كود = كود + "  let dx = 0, dy = 0;\n"
        كود = كود + "  if (keys.left) dx -= 1; if (keys.right) dx += 1; if (keys.up) dy -= 1; if (keys.down) dy += 1;\n"
        كود = كود + "  if (dx !== 0 || dy !== 0) {\n"
        كود = كود + "    const len = Math.hypot(dx, dy);\n"
        كود = كود + "    player.x += (dx / len) * player.speed * step;\n"
        كود = كود + "    player.y += (dy / len) * player.speed * step;\n"
        كود = كود + "  } else if (pointerTarget) {\n"
        كود = كود + "    const tx = pointerTarget.x - player.w / 2, ty = pointerTarget.y - player.h / 2;\n"
        كود = كود + "    const ddx = tx - player.x, ddy = ty - player.y, dist = Math.hypot(ddx, ddy);\n"
        كود = كود + "    if (dist > 3) { const mv = Math.min(dist, player.speed * 1.15 * step); player.x += (ddx / dist) * mv; player.y += (ddy / dist) * mv; }\n"
        كود = كود + "  }\n"
        كود = كود + "  player.x = Math.max(6, Math.min(W - player.w - 6, player.x));\n"
        كود = كود + "  player.y = Math.max(6, Math.min(H - player.h - 6, player.y));\n"
        كود = كود + "  if (invuln > 0) invuln = Math.max(0, invuln - dt);\n"
        كود = كود + "  if (shakeTime > 0) shakeTime = Math.max(0, shakeTime - dt);\n"
        كود = كود + "  const speedBoost = 1 + (level - 1) * 0.14;\n"
        كود = كود + "  for (let i = 0; i < targets.length; i++) {\n"
        كود = كود + "    const t = targets[i]; t.y += t.vy * speedBoost * step; t.angle += 0.05 * step;\n"
        كود = كود + "    if (t.y - t.r > H + 10) { targets[i] = spawnTarget(targetDefs[i % targetDefs.length]); continue; }\n"
        كود = كود + "    if (rectHit(player.x, player.y, player.w, player.h, t.x - t.r, t.y - t.r, t.r * 2, t.r * 2)) {\n"
        كود = كود + "      score += t.points;\n"
        كود = كود + "      const newLevel = 1 + Math.floor(score / 80);\n"
        كود = كود + "      if (newLevel > level) { level = newLevel; playTone(660, 0.22, 'triangle', 0.14); }\n"
        كود = كود + "      else { playTone(520 + (t.points * 5), 0.12, 'sine', 0.11); }\n"
        كود = كود + "      if (score > bestScore) { bestScore = score; try { localStorage.setItem(STORAGE_KEY, String(bestScore)); } catch (_) {} }\n"
        كود = كود + "      addBurst(t.x, t.y, t.color, 18);\n"
        كود = كود + "      popups.push({ x: t.x, y: t.y, text: '+' + t.points, color: t.color, life: 1 });\n"
        كود = كود + "      targets[i] = spawnTarget(targetDefs[i % targetDefs.length]);\n"
        كود = كود + "      updateHud();\n"
        كود = كود + "    }\n"
        كود = كود + "  }\n"
        كود = كود + "  for (let i = 0; i < obstacles.length; i++) {\n"
        كود = كود + "    const o = obstacles[i]; o.x += o.vx * step; o.y += o.vy * speedBoost * step; o.angle += o.spin * step;\n"
        كود = كود + "    if (o.x < 0 || o.x + o.w > W) o.vx *= -1;\n"
        كود = كود + "    if (o.y > H + o.h) { obstacles[i] = spawnObstacle(obstacleDefs[i % obstacleDefs.length]); continue; }\n"
        كود = كود + "    if (invuln <= 0 && rectHit(player.x + 5, player.y + 5, player.w - 10, player.h - 10, o.x + 3, o.y + 3, o.w - 6, o.h - 6)) {\n"
        كود = كود + "      lives -= 1; invuln = 1.4; shakeTime = 0.28;\n"
        كود = كود + "      addBurst(player.x + player.w / 2, player.y + player.h / 2, o.color, 26);\n"
        كود = كود + "      playTone(140, 0.28, 'sawtooth', 0.15);\n"
        كود = كود + "      obstacles[i] = spawnObstacle(obstacleDefs[i % obstacleDefs.length]);\n"
        كود = كود + "      updateHud();\n"
        كود = كود + "      if (lives <= 0) {\n"
        كود = كود + "        gameOver = true;\n"
        كود = كود + "        overlayTitle.textContent = score >= bestScore && score > 0 ? '🏆 رقم قياسي جديد!' : 'انتهت المهمة!';\n"
        كود = كود + "        overlayMsg.textContent = 'النقاط المحققة: ' + score + ' — أعلى نتيجة: ' + bestScore;\n"
        كود = كود + "        overlayEl.classList.add('visible');\n"
        كود = كود + "      }\n"
        كود = كود + "    }\n"
        كود = كود + "  }\n"
        كود = كود + "  particles = particles.filter(p => { p.x += p.vx * step; p.y += p.vy * step; p.life -= p.decay * step; return p.life > 0; });\n"
        كود = كود + "  popups = popups.filter(p => { p.y -= 1.2 * step; p.life -= 0.022 * step; return p.life > 0; });\n"
        كود = كود + "}\n"
        كود = كود + "function draw() {\n"
        كود = كود + "  ctx.save();\n"
        كود = كود + "  if (shakeTime > 0) ctx.translate((Math.random() - 0.5) * 8, (Math.random() - 0.5) * 8);\n"
        كود = كود + "  ctx.fillStyle = GAME_CONFIG.bg || '#091d37';\n"
        كود = كود + "  ctx.fillRect(0, 0, W, H);\n"
        كود = كود + "  ctx.fillStyle = 'rgba(226, 232, 240, 0.78)';\n"
        كود = كود + "  stars.forEach(s => { ctx.beginPath(); ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2); ctx.fill(); });\n"
        كود = كود + "  targets.forEach(t => {\n"
        كود = كود + "    ctx.save(); ctx.translate(t.x, t.y); ctx.rotate(t.angle);\n"
        كود = كود + "    ctx.shadowColor = t.color; ctx.shadowBlur = 14; ctx.fillStyle = t.color;\n"
        كود = كود + "    ctx.beginPath(); ctx.moveTo(0, -t.r); ctx.lineTo(t.r * 0.75, 0); ctx.lineTo(0, t.r); ctx.lineTo(-t.r * 0.75, 0); ctx.closePath(); ctx.fill();\n"
        كود = كود + "    ctx.restore();\n"
        كود = كود + "  });\n"
        كود = كود + "  obstacles.forEach(o => {\n"
        كود = كود + "    ctx.save(); ctx.translate(o.x + o.w / 2, o.y + o.h / 2); ctx.rotate(o.angle);\n"
        كود = كود + "    ctx.shadowColor = o.color; ctx.shadowBlur = 12; ctx.fillStyle = o.color;\n"
        كود = كود + "    const rx = o.w / 2, ry = o.h / 2;\n"
        كود = كود + "    ctx.beginPath(); ctx.moveTo(-rx, -ry * 0.4); ctx.lineTo(-rx * 0.3, -ry); ctx.lineTo(rx * 0.6, -ry * 0.8); ctx.lineTo(rx, 0); ctx.lineTo(rx * 0.5, ry); ctx.lineTo(-rx * 0.6, ry * 0.85); ctx.closePath(); ctx.fill();\n"
        كود = كود + "    ctx.restore();\n"
        كود = كود + "  });\n"
        كود = كود + "  const blink = invuln > 0 && Math.floor(invuln * 14) % 2 === 0;\n"
        كود = كود + "  if (!blink) {\n"
        كود = كود + "    const cx = player.x + player.w / 2, cy = player.y + player.h / 2;\n"
        كود = كود + "    ctx.save(); ctx.shadowColor = player.color; ctx.shadowBlur = 18; ctx.fillStyle = player.color;\n"
        كود = كود + "    ctx.beginPath(); ctx.moveTo(cx, player.y); ctx.lineTo(player.x + player.w, player.y + player.h * 0.88); ctx.lineTo(cx, player.y + player.h * 0.7); ctx.lineTo(player.x, player.y + player.h * 0.88); ctx.closePath(); ctx.fill();\n"
        كود = كود + "    ctx.fillStyle = '#38bdf8'; ctx.beginPath(); ctx.arc(cx, cy, player.w * 0.16, 0, Math.PI * 2); ctx.fill();\n"
        كود = كود + "    ctx.fillStyle = '#facc15'; ctx.beginPath(); ctx.moveTo(cx - 6, player.y + player.h * 0.76); ctx.lineTo(cx + 6, player.y + player.h * 0.76); ctx.lineTo(cx, player.y + player.h + 6 + Math.random() * 6); ctx.closePath(); ctx.fill();\n"
        كود = كود + "    ctx.restore();\n"
        كود = كود + "  }\n"
        كود = كود + "  particles.forEach(p => { ctx.globalAlpha = Math.max(0, p.life); ctx.fillStyle = p.color; ctx.beginPath(); ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2); ctx.fill(); });\n"
        كود = كود + "  ctx.globalAlpha = 1;\n"
        كود = كود + "  ctx.font = 'bold 16px system-ui, sans-serif'; ctx.textAlign = 'center';\n"
        كود = كود + "  popups.forEach(p => { ctx.globalAlpha = Math.max(0, p.life); ctx.fillStyle = p.color; ctx.fillText(p.text, p.x, p.y); });\n"
        كود = كود + "  ctx.globalAlpha = 1;\n"
        كود = كود + "  if (paused && !gameOver) {\n"
        كود = كود + "    ctx.fillStyle = 'rgba(2, 6, 23, 0.55)'; ctx.fillRect(0, 0, W, H);\n"
        كود = كود + "    ctx.fillStyle = '#f8fafc'; ctx.font = 'bold 28px system-ui, sans-serif'; ctx.fillText('اللعبة متوقفة مؤقتاً', W / 2, H / 2);\n"
        كود = كود + "  }\n"
        كود = كود + "  ctx.restore();\n"
        كود = كود + "}\n"
        كود = كود + "function loop(ts) {\n"
        كود = كود + "  if (!lastTime) lastTime = ts;\n"
        كود = كود + "  const dt = (ts - lastTime) / 1000; lastTime = ts;\n"
        كود = كود + "  update(dt); draw(); requestAnimationFrame(loop);\n"
        كود = كود + "}\n"
        كود = كود + "resetGame(); requestAnimationFrame(loop);\n"
        كود = كود + "</script>\n</body>\n</html>"
        أرجع كود
    }

    دالة حفظ_لعبة(مسار_الملف) {
        دع محتوى_html = هذا.بناء_لعبة_html()
        اكتب_ملف(مسار_الملف، محتوى_html)
        أرجع مسار_الملف
    }
}
"##
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_ieee_standard_vectors() {
        // متجه اختبار IEEE 802.3 القياسي الرسمي
        assert_eq!(crc32_hex(b"123456789"), "cbf43926");
        assert_eq!(crc32_hex(b""), "00000000");

        let res_ar = eval_sys_builtin("بصمة_crc32", vec![Value::Text("123456789".into())])
            .expect("crc32 Arabic builtin must succeed");
        assert_eq!(res_ar, Value::Text("cbf43926".into()));

        let res_en = eval_sys_builtin("crc32", vec![Value::Text("123456789".into())])
            .expect("crc32 English builtin must succeed");
        assert_eq!(res_en, Value::Text("cbf43926".into()));
    }

    #[test]
    fn test_rle_compress_decompress_arabic_and_latin() {
        let samples = [
            "",
            "a",
            "AAAAAAAAAAbbbbbbcccccc",
            "أأأأأأأأأأبببببببببب — لغة لسان العربية 2026!",
            "اختبار الأرقام 111111 و 2026 وعلامات ~ و ~~~ و ~~~~ و ::: بدقة تامة",
            "Hello   World!!!! 12345\nسطر جديد\tتبويب",
        ];

        for sample in samples {
            let compressed = rle_compress(sample);
            let restored = rle_decompress(&compressed)
                .unwrap_or_else(|e| panic!("RLE decompress failed for {:?}: {}", sample, e));
            assert_eq!(restored, sample);
        }

        // التحقق من فعالية الضغط على النصوص المتكررة
        let repeated_ar = "أأأأأأأأأأأأأأأأأأأأ";
        let compressed_ar = rle_compress(repeated_ar);
        assert!(compressed_ar.len() < repeated_ar.len());
        assert_eq!(rle_decompress(&compressed_ar).unwrap(), repeated_ar);

        // اختبار عبر eval_sys_builtin
        let enc = eval_sys_builtin(
            "ضغط_rle",
            vec![Value::Text("LLLLLLLLLLلساننننننننن".into())],
        )
        .unwrap();
        let dec = eval_sys_builtin("فك_ضغط_rle", vec![enc]).unwrap();
        assert_eq!(dec, Value::Text("LLLLLLLLLLلساننننننننن".into()));

        // مدخلات غير صالحة يجب أن ترفض بأمان
        assert!(rle_decompress("~").is_err());
        assert!(rle_decompress("~0:a").is_err());
        assert!(rle_decompress("~abc:a").is_err());
        assert!(rle_decompress("~5:").is_err());
    }

    #[test]
    fn test_path_helpers_and_file_ops_in_temp_dir() {
        // اختبار دوال المسارات
        let joined = eval_sys_builtin(
            "مسار_ادمج",
            vec![Value::list(vec![
                Value::Text("مشروع".into()),
                Value::Text("بيانات".into()),
                Value::Text("تقرير.json".into()),
            ])],
        )
        .unwrap();
        let joined_str = match joined {
            Value::Text(s) => s,
            other => panic!("expected Text from path_join, got {:?}", other),
        };
        assert_eq!(
            eval_sys_builtin("اسم_ملف", vec![Value::Text(joined_str.clone())]).unwrap(),
            Value::Text("تقرير.json".into())
        );
        assert_eq!(
            eval_sys_builtin("امتداد_ملف", vec![Value::Text(joined_str.clone())]).unwrap(),
            Value::Text("json".into())
        );
        let parent = eval_sys_builtin("مجلد_أب", vec![Value::Text(joined_str)]).unwrap();
        match parent {
            Value::Text(p) => assert!(p.ends_with("بيانات")),
            other => panic!("expected Text from path_dirname, got {:?}", other),
        }

        // اختبار نسخ ونقل وحجم وحذف مجلد في مجلد مؤقت معزول
        let unique_name = format!(
            "lisan_sys_games_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let temp_dir = std::env::temp_dir().join(unique_name);
        std::fs::create_dir_all(&temp_dir).expect("must create temp test dir");

        let src_file = temp_dir.join("source.txt");
        let copied_file = temp_dir.join("copied.txt");
        let moved_file = temp_dir.join("moved.txt");
        let content = "محتوى اختبار نظام الملفات في لغة لسان 2026";
        std::fs::write(&src_file, content).expect("must write source file");

        let size_val = eval_sys_builtin(
            "حجم_ملف",
            vec![Value::Text(src_file.to_string_lossy().into_owned())],
        )
        .unwrap();
        assert_eq!(size_val, Value::Number(content.len() as f64));

        let copied_bytes = eval_sys_builtin(
            "انسخ_ملف",
            vec![
                Value::Text(src_file.to_string_lossy().into_owned()),
                Value::Text(copied_file.to_string_lossy().into_owned()),
            ],
        )
        .unwrap();
        assert_eq!(copied_bytes, Value::Number(content.len() as f64));
        assert!(copied_file.is_file());

        let moved_ok = eval_sys_builtin(
            "انقل_ملف",
            vec![
                Value::Text(copied_file.to_string_lossy().into_owned()),
                Value::Text(moved_file.to_string_lossy().into_owned()),
            ],
        )
        .unwrap();
        assert_eq!(moved_ok, Value::Bool(true));
        assert!(!copied_file.exists());
        assert_eq!(std::fs::read_to_string(&moved_file).unwrap(), content);

        // حذف المجلد المؤقت بالكامل
        let removed_ok = eval_sys_builtin(
            "احذف_مجلد",
            vec![Value::Text(temp_dir.to_string_lossy().into_owned())],
        )
        .unwrap();
        assert_eq!(removed_ok, Value::Bool(true));
        assert!(!temp_dir.exists());

        // اختبار تنفيذ أمر نظام التشغيل
        #[cfg(unix)]
        {
            let cmd_res = eval_sys_builtin(
                "نفّذ_أمر",
                vec![
                    Value::Text("echo".into()),
                    Value::list(vec![Value::Text("lisan_ok".into())]),
                ],
            )
            .expect("run_command echo must succeed on unix");
            match cmd_res {
                Value::Map(pairs) => {
                    let map = pairs.borrow();
                    assert!(map.iter().any(|(k, v)| k == "نجاح" && *v == Value::Bool(true)));
                    assert!(map.iter().any(|(k, v)| k == "كود" && *v == Value::Number(0.0)));
                    assert!(map.iter().any(|(k, v)| {
                        k == "مخرج" && matches!(v, Value::Text(s) if s.trim() == "lisan_ok")
                    }));
                }
                other => panic!("expected Map from run_command, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_stdlib_systems_and_games_parse_and_execute() {
        let tokens = crate::lexer::tokenize(STDLIB_SYSTEMS_AND_GAMES)
            .expect("STDLIB_SYSTEMS_AND_GAMES must tokenize cleanly");
        let stmts = crate::parser::parse(tokens)
            .expect("STDLIB_SYSTEMS_AND_GAMES must parse cleanly");
        assert_eq!(stmts.len(), 4);

        let test_script = r##"
دع أداة = جديد أداة_سطر_أوامر("حزمة_لسان"، "2.0.0")
دع إطار = أداة.إطار_عنوان("لوحة الأنظمة العربية")
دع جدول = أداة.جدول_طرفية(["الأداة"، "الوضع"]، [["ضغط"، "جاهز"]، ["بصمة"، "نشط"]])
دع شريط = أداة.شريط_تقدم_نصي(50، 20)
دع ملون = أداة.ملوّن("نجاح باهر"، "أخضر")
اطبع(إطار)
اطبع(جدول)
اطبع(شريط)
اطبع(ملون)

دع مسجل = جديد سجل_أحداث("خادم_لسان")
مسجل.معلومات("بدء التشغيل")
مسجل.تحذير("استهلاك الذاكرة مرتفع")
مسجل.عطل("انقطاع اتصال مؤقت")
اطبع("عدد_السجلات=" + نص(مسجل.عدد_السجلات()) + "|الكل=" + نص(طول(مسجل.الكل())))
اطبع(مسجل.تصدير_json())

دع مجدول = جديد مجدول_مهام("نشر_الإصدار")
مجدول.أضف_مهمة("تنظيف"، 1)
مجدول.أضف_مهمة("بناء_المشروع"، 3)
مجدول.أضف_مهمة("تشغيل_الاختبارات"، 2)
دع نتائج = مجدول.نفّذ_الكل()
اطبع("أول_مهمة=" + نتائج[0]["اسم"])
اطبع(مجدول.تقرير())

دع لعبة = جديد لعبة_ثنائية_الأبعاد("مغامرة الفضاء العربية"، 800، 500، "#091d37")
لعبة.أضف_لاعب("المسبار_العربي"، 378، 420، 44، 44، "#e65c1a"، 7)
لعبة.أضف_هدف("بلورة_طاقة"، "#10b981"، 15)
لعبة.أضف_عقبة("نيزك_ملتهب"، "#ef4444"، 4)
لعبة.اضبط_تحكم("أسهم_ولمس")
دع صفحة_اللعبة = لعبة.بناء_لعبة_html()
اطبع(صفحة_اللعبة)
"##;

        let full_source = format!("{}\n{}", STDLIB_SYSTEMS_AND_GAMES, test_script);
        let outputs = crate::Interpreter::default()
            .run_source(&full_source)
            .expect("STDLIB_SYSTEMS_AND_GAMES execution must succeed");

        assert!(outputs[0].contains("لوحة الأنظمة العربية"));
        assert!(outputs[0].contains("╔"));
        assert!(outputs[1].contains("┌"));
        assert!(outputs[1].contains("ضغط"));
        assert_eq!(outputs[2], "[██████████░░░░░░░░░░] 50%");
        assert!(outputs[3].contains("\x1b[32mنجاح باهر\x1b[0m"));
        assert_eq!(outputs[4], "عدد_السجلات=3|الكل=3");
        assert!(outputs[5].contains("خادم_لسان"));
        assert!(outputs[5].contains("تحذير"));
        assert!(outputs[5].contains("عطل"));
        assert_eq!(outputs[6], "أول_مهمة=بناء_المشروع");
        assert!(outputs[7].contains("نشر_الإصدار"));
        assert!(outputs[7].contains("مكتملة"));

        let html = &outputs[8];
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("dir=\"rtl\""));
        assert!(html.contains("<canvas id=\"gameCanvas\""));
        assert!(html.contains("requestAnimationFrame"));
        assert!(html.contains("localStorage"));
        assert!(html.contains("AudioContext"));
        assert!(html.contains("مغامرة الفضاء العربية"));
        assert!(html.contains("المسبار_العربي"));
        assert!(html.contains("بلورة_طاقة"));
        assert!(html.contains("نيزك_ملتهب"));
        assert!(html.contains("إعادة اللعب"));
    }
}
