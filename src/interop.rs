//! طبقة التوافق البيني (Python 3، JavaScript/Node.js، HTTPS/TLS الآمن، عملاء الذكاء الاصطناعي)
//! والمترجم متعدد الأهداف (لسان ← Python 3 و لسان ← ES2022 JavaScript).
//!
//! مبنية بالكامل باستخدام مكتبة Rust القياسية (`std`) فقط دون أي حزم خارجية،
//! مع توافق كامل مع بيئة المتصفح WebAssembly (`wasm32`) للمترجمات (`transpile_to_python`, `transpile_to_js`).

use crate::ast::{ClassDef, Expr, Function, Op, Stmt};
use crate::runtime::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ============================================================================
// 1. المكتبة القياسية العربية للتوافق البيني والذكاء الاصطناعي (100% عربية)
// ============================================================================

/// المكتبة القياسية العربية المدمجة للربط مع بايثون وجافاسكربت ونماذج الذكاء الاصطناعي
pub const STDLIB_INTEROP: &str = r##"
# مكتبة التوافق البيني والذكاء الاصطناعي في لغة لسان (عربية بالكامل)
# أسماء المكتبات الخارجية والمزودين تظهر داخل النصوص فقط.

صنف مكتبة_بايثون {
    دالة بناء(اسم_الوحدة) {
        هذا.اسم_الوحدة = اسم_الوحدة
    }

    دالة استدعِ(اسم_الدالة، وسائط = []) {
        أرجع بايثون_استدعِ(هذا.اسم_الوحدة، اسم_الدالة، وسائط)
    }

    دالة استدع(اسم_الدالة، وسائط = []) {
        أرجع هذا.استدعِ(اسم_الدالة، وسائط)
    }

    دالة استدعِ_بمفاتيح(اسم_الدالة، وسائط = []، مفاتيح = {}) {
        أرجع بايثون_استدعِ(هذا.اسم_الوحدة، اسم_الدالة، وسائط، مفاتيح)
    }

    دالة استدع_بمفاتيح(اسم_الدالة، وسائط = []، مفاتيح = {}) {
        أرجع هذا.استدعِ_بمفاتيح(اسم_الدالة، وسائط، مفاتيح)
    }

    دالة سمة(اسم_السمة) {
        أرجع بايثون_سمة(هذا.اسم_الوحدة، اسم_السمة)
    }

    دالة متوفرة() {
        إذا (ليس بايثون_متوفر()) {
            أرجع خطأ
        }
        إذا (هذا.اسم_الوحدة == "") {
            أرجع صحيح
        }
        حاول {
            دع فحص = بايثون_نفّذ("import importlib; importlib.import_module(اسم); النتيجة = True"، {"اسم": هذا.اسم_الوحدة})
            أرجع فحص == صحيح
        } التقط (خلل) {
            أرجع خطأ
        }
    }

    دالة تعبير(نص_التعبير، مدخلات = {}) {
        دع شيفرة = "import importlib\n_م = importlib.import_module(_اسم_الوحدة)\n_بيئة = dict(vars(_م))\n_بيئة['وحدة'] = _م\n_بيئة['mod'] = _م\n_بيئة.update(_مدخلات)\nالنتيجة = eval(_تعبير, _بيئة, _بيئة)"
        أرجع بايثون_نفّذ(شيفرة، {
            "_اسم_الوحدة": هذا.اسم_الوحدة،
            "_تعبير": نص_التعبير،
            "_مدخلات": مدخلات
        })
    }
}

دالة استورد_بايثون(اسم_الوحدة) {
    أرجع جديد مكتبة_بايثون(اسم_الوحدة)
}

صنف مكتبة_جافاسكربت {
    دالة بناء(اسم_الوحدة = "") {
        هذا.اسم_الوحدة = اسم_الوحدة
    }

    دالة استدعِ(اسم_الدالة، وسائط = []) {
        أرجع جافاسكربت_استدعِ(هذا.اسم_الوحدة، اسم_الدالة، وسائط)
    }

    دالة استدع(اسم_الدالة، وسائط = []) {
        أرجع هذا.استدعِ(اسم_الدالة، وسائط)
    }

    دالة نفّذ(شيفرة، مدخلات = {}) {
        إذا (هذا.اسم_الوحدة == "") {
            أرجع جافاسكربت_نفّذ(شيفرة، مدخلات)
        }
        دع شيفرة_كاملة = "const وحدة = require(_اسم_الوحدة); const mod = وحدة;\n" + شيفرة
        دع بيانات = مدخلات.نسخ()
        بيانات.أضف("_اسم_الوحدة"، هذا.اسم_الوحدة)
        أرجع جافاسكربت_نفّذ(شيفرة_كاملة، بيانات)
    }

    دالة نفذ(شيفرة، مدخلات = {}) {
        أرجع هذا.نفّذ(شيفرة، مدخلات)
    }

    دالة متوفرة() {
        إذا (ليس جافاسكربت_متوفر()) {
            أرجع خطأ
        }
        إذا (هذا.اسم_الوحدة == "") {
            أرجع صحيح
        }
        حاول {
            دع فحص = جافاسكربت_نفّذ("require(_اسم_الوحدة); return true;"، {"_اسم_الوحدة": هذا.اسم_الوحدة})
            أرجع فحص == صحيح
        } التقط (خلل) {
            أرجع خطأ
        }
    }
}

دالة استورد_جافاسكربت(اسم_الوحدة) {
    أرجع جديد مكتبة_جافاسكربت(اسم_الوحدة)
}

صنف نموذج_ذكاء {
    دالة بناء(مزود = "جيميني"، مفتاح = ""، نموذج = ""، رابط_مخصص = "") {
        هذا.مزود = هذا.تطبيع_المزود(مزود)
        هذا.مفتاح = مفتاح
        إذا (نموذج == "") {
            هذا.نموذج = هذا.النموذج_الافتراضي(هذا.مزود)
        } وإلا {
            هذا.نموذج = نموذج
        }
        هذا.رابط_مخصص = رابط_مخصص
        هذا.رسالة_النظام = ""
        هذا.حرارة = 0.7
        هذا.أقصى_رموز = 1024
    }

    دالة تطبيع_المزود(مزود) {
        إذا (مزود == "جيميني" || مزود == "gemini") {
            أرجع "جيميني"
        }
        إذا (مزود == "أوبن_إيه_آي" || مزود == "اوبن_ايه_اي" || مزود == "openai") {
            أرجع "أوبن_إيه_آي"
        }
        إذا (مزود == "كلود" || مزود == "claude" || مزود == "أنثروبيك" || مزود == "anthropic") {
            أرجع "كلود"
        }
        إذا (مزود == "محلي" || مزود == "أولاما" || مزود == "اولاما" || مزود == "ollama") {
            أرجع "محلي"
        }
        أرجع مزود
    }

    دالة النموذج_الافتراضي(مزود_مطبع) {
        إذا (مزود_مطبع == "جيميني") {
            أرجع "gemini-2.0-flash"
        }
        إذا (مزود_مطبع == "أوبن_إيه_آي") {
            أرجع "gpt-4o-mini"
        }
        إذا (مزود_مطبع == "كلود") {
            أرجع "claude-3-5-sonnet-latest"
        }
        إذا (مزود_مطبع == "محلي") {
            أرجع "llama3.2"
        }
        أرجع "gemini-2.0-flash"
    }

    دالة ضبط_النظام(نص_النظام) {
        هذا.رسالة_النظام = نص_النظام
        أرجع هذا
    }

    دالة ضبط_الحرارة(قيمة_حرارة) {
        هذا.حرارة = قيمة_حرارة
        أرجع هذا
    }

    دالة تحويل_الدور(دور، للمزود) {
        إذا (دور == "مستخدم" || دور == "user") {
            أرجع "user"
        }
        إذا (دور == "مساعد" || دور == "assistant" || دور == "model") {
            إذا (للمزود == "جيميني") {
                أرجع "model"
            }
            أرجع "assistant"
        }
        إذا (دور == "نظام" || دور == "system") {
            أرجع "system"
        }
        أرجع "user"
    }

    دالة بناء_طلب(رسالة) {
        أرجع هذا.بناء_طلب_محادثة([{"دور": "مستخدم"، "محتوى": رسالة}])
    }

    دالة بناء_طلب_محادثة(رسائل) {
        دع ترويسات = {"Content-Type": "application/json; charset=utf-8"}

        إذا (هذا.مزود == "جيميني") {
            دع رابط = هذا.رابط_مخصص
            إذا (رابط == "") {
                رابط = "https://generativelanguage.googleapis.com/v1beta/models/" + هذا.نموذج + ":generateContent?key=" + هذا.مفتاح
            }
            دع محتويات = []
            لكل عنصر في رسائل {
                دع دور_خام = عنصر.احصل("دور"، عنصر.احصل("role"، "مستخدم"))
                دع نص_رسالة = عنصر.احصل("محتوى"، عنصر.احصل("content"، ""))
                محتويات.أضف({
                    "role": هذا.تحويل_الدور(دور_خام، "جيميني")،
                    "parts": [{"text": نص_رسالة}]
                })
            }
            دع جسم = {
                "contents": محتويات،
                "generationConfig": {"temperature": هذا.حرارة}
            }
            إذا (هذا.رسالة_النظام != "") {
                جسم.أضف("systemInstruction"، {"parts": [{"text": هذا.رسالة_النظام}]})
            }
            أرجع {
                "مزود": "جيميني"،
                "نموذج": هذا.نموذج،
                "طريقة": "POST"،
                "رابط": رابط،
                "ترويسات": ترويسات،
                "جسم": جسم
            }
        }

        إذا (هذا.مزود == "أوبن_إيه_آي") {
            دع رابط = هذا.رابط_مخصص
            إذا (رابط == "") {
                رابط = "https://api.openai.com/v1/chat/completions"
            }
            ترويسات.أضف("Authorization"، "Bearer " + هذا.مفتاح)
            دع قائمة_رسائل = []
            إذا (هذا.رسالة_النظام != "") {
                قائمة_رسائل.أضف({"role": "system"، "content": هذا.رسالة_النظام})
            }
            لكل عنصر في رسائل {
                دع دور_خام = عنصر.احصل("دور"، عنصر.احصل("role"، "مستخدم"))
                دع نص_رسالة = عنصر.احصل("محتوى"، عنصر.احصل("content"، ""))
                قائمة_رسائل.أضف({
                    "role": هذا.تحويل_الدور(دور_خام، "أوبن_إيه_آي")،
                    "content": نص_رسالة
                })
            }
            دع جسم = {
                "model": هذا.نموذج،
                "messages": قائمة_رسائل،
                "temperature": هذا.حرارة
            }
            أرجع {
                "مزود": "أوبن_إيه_آي"،
                "نموذج": هذا.نموذج،
                "طريقة": "POST"،
                "رابط": رابط،
                "ترويسات": ترويسات،
                "جسم": جسم
            }
        }

        إذا (هذا.مزود == "كلود") {
            دع رابط = هذا.رابط_مخصص
            إذا (رابط == "") {
                رابط = "https://api.anthropic.com/v1/messages"
            }
            ترويسات.أضف("x-api-key"، هذا.مفتاح)
            ترويسات.أضف("anthropic-version"، "2023-06-01")
            دع قائمة_رسائل = []
            لكل عنصر في رسائل {
                دع دور_خام = عنصر.احصل("دور"، عنصر.احصل("role"، "مستخدم"))
                دع نص_رسالة = عنصر.احصل("محتوى"، عنصر.احصل("content"، ""))
                قائمة_رسائل.أضف({
                    "role": هذا.تحويل_الدور(دور_خام، "كلود")،
                    "content": نص_رسالة
                })
            }
            دع جسم = {
                "model": هذا.نموذج،
                "max_tokens": هذا.أقصى_رموز،
                "messages": قائمة_رسائل،
                "temperature": هذا.حرارة
            }
            إذا (هذا.رسالة_النظام != "") {
                جسم.أضف("system"، هذا.رسالة_النظام)
            }
            أرجع {
                "مزود": "كلود"،
                "نموذج": هذا.نموذج،
                "طريقة": "POST"،
                "رابط": رابط،
                "ترويسات": ترويسات،
                "جسم": جسم
            }
        }

        # المزود المحلي (Ollama)
        دع رابط = هذا.رابط_مخصص
        إذا (رابط == "") {
            رابط = "http://127.0.0.1:11434/api/chat"
        }
        دع قائمة_رسائل = []
        إذا (هذا.رسالة_النظام != "") {
            قائمة_رسائل.أضف({"role": "system"، "content": هذا.رسالة_النظام})
        }
        لكل عنصر في رسائل {
            دع دور_خام = عنصر.احصل("دور"، عنصر.احصل("role"، "مستخدم"))
            دع نص_رسالة = عنصر.احصل("محتوى"، عنصر.احصل("content"، ""))
            قائمة_رسائل.أضف({
                "role": هذا.تحويل_الدور(دور_خام، "محلي")،
                "content": نص_رسالة
            })
        }
        دع جسم = {
            "model": هذا.نموذج،
            "messages": قائمة_رسائل،
            "stream": خطأ
        }
        أرجع {
            "مزود": "محلي"،
            "نموذج": هذا.نموذج،
            "طريقة": "POST"،
            "رابط": رابط،
            "ترويسات": ترويسات،
            "جسم": جسم
        }
    }

    دالة استخرج_الرد(بيانات) {
        حاول {
            إذا (هذا.مزود == "جيميني") {
                دع مرشحين = بيانات["candidates"]
                دع أجزاء = مرشحين[0]["content"]["parts"]
                أرجع أجزاء[0]["text"]
            }
            إذا (هذا.مزود == "أوبن_إيه_آي") {
                دع خيارات = بيانات["choices"]
                أرجع خيارات[0]["message"]["content"]
            }
            إذا (هذا.مزود == "كلود") {
                دع محتوى = بيانات["content"]
                أرجع محتوى[0]["text"]
            }
            إذا (هذا.مزود == "محلي") {
                أرجع بيانات["message"]["content"]
            }
            أرجع ""
        } التقط (خلل) {
            أرجع ""
        }
    }

    دالة محادثة(رسائل) {
        دع طلب = هذا.بناء_طلب_محادثة(رسائل)
        دع استجابة = طلب_شبكي_آمن(طلب["طريقة"]، طلب["رابط"]، طلب["ترويسات"]، طلب["جسم"])
        إذا (ليس استجابة["نجاح"]) {
            أرجع {
                "نجاح": خطأ،
                "حالة": استجابة["حالة"]،
                "نص": استجابة["نص"]،
                "رد": ""
            }
        }
        دع بيانات = تحليل_جيسون(استجابة["نص"])
        دع نص_الرد = هذا.استخرج_الرد(بيانات)
        أرجع {
            "نجاح": صحيح،
            "حالة": استجابة["حالة"]،
            "رد": نص_الرد،
            "خام": بيانات
        }
    }

    دالة اسأل(رسالة) {
        أرجع هذا.محادثة([{"دور": "مستخدم"، "محتوى": رسالة}])
    }

    دالة اسال(رسالة) {
        أرجع هذا.اسأل(رسالة)
    }
}

صنف عميل_ذكي يرث نموذج_ذكاء {
}
"##;

// ============================================================================
// 2. الدوال المدمجة الأصلية للتوافق البيني (بايثون، جافاسكربت، HTTPS/TLS)
// ============================================================================

/// جميع أسماء الدوال المدمجة العربية للتوافق البيني
pub const ALL_INTEROP_BUILTINS: &[&str] = &[
    "بايثون_متوفر",
    "بايثون_استدعِ",
    "بايثون_استدع",
    "بايثون_سمة",
    "بايثون_نفّذ",
    "بايثون_نفذ",
    "جافاسكربت_متوفر",
    "جافاسكربت_استدعِ",
    "جافاسكربت_استدع",
    "جافاسكربت_نفّذ",
    "جافاسكربت_نفذ",
    "طلب_شبكي_آمن",
    "طلب_شبكي_امن",
    "تحليل_جيسون",
    "فك_جيسون",
    "ترميز_جيسون",
    "إلى_جيسون",
    "الى_جيسون",
];

/// التحقق مما إذا كان الاسم يشير إلى دالة مدمجة في طبقة التوافق البيني
pub fn is_interop_builtin(name: &str) -> bool {
    ALL_INTEROP_BUILTINS.contains(&name)
}

/// موجّه تنفيذ الدوال المدمجة الخاصة بالتوافق البيني (بايثون، جافاسكربت، HTTPS/TLS، JSON)
pub fn try_call_interop_builtin(name: &str, args: &[Value]) -> Option<Result<Value, String>> {
    match name {
        "بايثون_متوفر" => Some(builtin_python_available(args)),
        "بايثون_استدعِ" | "بايثون_استدع" => Some(builtin_python_call(name, args)),
        "بايثون_سمة" => Some(builtin_python_attr(name, args)),
        "بايثون_نفّذ" | "بايثون_نفذ" => Some(builtin_python_exec(name, args)),
        "جافاسكربت_متوفر" => Some(builtin_js_available(args)),
        "جافاسكربت_استدعِ" | "جافاسكربت_استدع" => Some(builtin_js_call(name, args)),
        "جافاسكربت_نفّذ" | "جافاسكربت_نفذ" => Some(builtin_js_exec(name, args)),
        "طلب_شبكي_آمن" | "طلب_شبكي_امن" => Some(builtin_secure_http(name, args)),
        "تحليل_جيسون" | "فك_جيسون" => Some(builtin_json_parse(name, args)),
        "ترميز_جيسون" | "إلى_جيسون" | "الى_جيسون" => Some(builtin_json_stringify(name, args)),
        _ => None,
    }
}

fn expect_text<'a>(fn_name: &str, val: &'a Value) -> Result<&'a str, String> {
    match val {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("الدالة '{}' تتطلب نصًا", fn_name)),
    }
}

fn builtin_json_parse(name: &str, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "الدالة '{}' تتوقع معاملًا واحدًا (نص)، استلمت {}",
            name,
            args.len()
        ));
    }
    let raw = expect_text(name, &args[0])?;
    crate::json::parse_json(raw)
}

fn builtin_json_stringify(name: &str, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "الدالة '{}' تتوقع معاملًا واحدًا، استلمت {}",
            name,
            args.len()
        ));
    }
    let encoded = crate::json::to_json_string(&args[0])?;
    Ok(Value::Text(encoded))
}

fn builtin_python_available(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!(
            "الدالة 'بايثون_متوفر' لا تأخذ معاملات، استلمت {}",
            args.len()
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        Ok(Value::Bool(false))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(Value::Bool(find_python_command().is_some()))
    }
}

fn builtin_python_call(name: &str, args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 4 {
        return Err(format!(
            "الدالة '{}' تتوقع 2 إلى 4 معاملات (اسم_الوحدة، اسم_الدالة، [قائمة_وسائط]، [قاموس_مفاتيح])، استلمت {}",
            name,
            args.len()
        ));
    }
    let module = expect_text(name, &args[0])?;
    let func = expect_text(name, &args[1])?;
    let call_args = if args.len() >= 3 {
        match &args[2] {
            Value::List(_) | Value::Tuple(_) => args[2].clone(),
            Value::Null => Value::list(Vec::new()),
            _ => {
                return Err(format!(
                    "الدالة '{}' تتطلب قائمة وسائط في المعامل الثالث",
                    name
                ))
            }
        }
    } else {
        Value::list(Vec::new())
    };
    let kwargs = if args.len() == 4 {
        match &args[3] {
            Value::Map(_) => args[3].clone(),
            Value::Null => Value::map(Vec::new()),
            _ => {
                return Err(format!(
                    "الدالة '{}' تتطلب قاموس مفاتيح في المعامل الرابع",
                    name
                ))
            }
        }
    } else {
        Value::map(Vec::new())
    };

    let payload = Value::map(vec![
        ("mode".into(), Value::Text("call".into())),
        ("module".into(), Value::Text(module.into())),
        ("func".into(), Value::Text(func.into())),
        ("args".into(), call_args),
        ("kwargs".into(), kwargs),
    ]);
    invoke_python_bridge(&payload)
}

fn builtin_python_attr(name: &str, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!(
            "الدالة '{}' تتوقع معاملين (اسم_الوحدة، اسم_السمة)، استلمت {}",
            name,
            args.len()
        ));
    }
    let module = expect_text(name, &args[0])?;
    let attr = expect_text(name, &args[1])?;
    let payload = Value::map(vec![
        ("mode".into(), Value::Text("attr".into())),
        ("module".into(), Value::Text(module.into())),
        ("attr".into(), Value::Text(attr.into())),
    ]);
    invoke_python_bridge(&payload)
}

fn builtin_python_exec(name: &str, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(format!(
            "الدالة '{}' تتوقع معاملًا واحدًا أو معاملين (شيفرة_بايثون، [قاموس_مدخلات])، استلمت {}",
            name,
            args.len()
        ));
    }
    let code = expect_text(name, &args[0])?;
    let inputs = if args.len() == 2 {
        match &args[1] {
            Value::Map(_) => args[1].clone(),
            Value::Null => Value::map(Vec::new()),
            _ => {
                return Err(format!(
                    "الدالة '{}' تتطلب قاموس مدخلات في المعامل الثاني",
                    name
                ))
            }
        }
    } else {
        Value::map(Vec::new())
    };

    let payload = Value::map(vec![
        ("mode".into(), Value::Text("exec".into())),
        ("code".into(), Value::Text(code.into())),
        ("inputs".into(), inputs),
    ]);
    invoke_python_bridge(&payload)
}

fn builtin_js_available(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!(
            "الدالة 'جافاسكربت_متوفر' لا تأخذ معاملات، استلمت {}",
            args.len()
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        Ok(Value::Bool(false))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(Value::Bool(is_node_available()))
    }
}

fn builtin_js_call(name: &str, args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!(
            "الدالة '{}' تتوقع 2 أو 3 معاملات (اسم_الوحدة، اسم_الدالة، [قائمة_وسائط])، استلمت {}",
            name,
            args.len()
        ));
    }
    let module = expect_text(name, &args[0])?;
    let func = expect_text(name, &args[1])?;
    let call_args = if args.len() == 3 {
        match &args[2] {
            Value::List(_) | Value::Tuple(_) => args[2].clone(),
            Value::Null => Value::list(Vec::new()),
            _ => {
                return Err(format!(
                    "الدالة '{}' تتطلب قائمة وسائط في المعامل الثالث",
                    name
                ))
            }
        }
    } else {
        Value::list(Vec::new())
    };

    let payload = Value::map(vec![
        ("mode".into(), Value::Text("call".into())),
        ("module".into(), Value::Text(module.into())),
        ("func".into(), Value::Text(func.into())),
        ("args".into(), call_args),
    ]);
    invoke_node_bridge(&payload)
}

fn builtin_js_exec(name: &str, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(format!(
            "الدالة '{}' تتوقع معاملًا واحدًا أو معاملين (شيفرة_جافاسكربت، [قاموس_مدخلات])، استلمت {}",
            name,
            args.len()
        ));
    }
    let code = expect_text(name, &args[0])?;
    let inputs = if args.len() == 2 {
        match &args[1] {
            Value::Map(_) => args[1].clone(),
            Value::Null => Value::map(Vec::new()),
            _ => {
                return Err(format!(
                    "الدالة '{}' تتطلب قاموس مدخلات في المعامل الثاني",
                    name
                ))
            }
        }
    } else {
        Value::map(Vec::new())
    };

    let payload = Value::map(vec![
        ("mode".into(), Value::Text("exec".into())),
        ("code".into(), Value::Text(code.into())),
        ("inputs".into(), inputs),
    ]);
    invoke_node_bridge(&payload)
}

fn normalize_http_method(raw: &str) -> String {
    match raw.trim() {
        "جلب" => "GET".into(),
        "إرسال" | "ارسال" => "POST".into(),
        "تحديث" => "PUT".into(),
        "تعديل" => "PATCH".into(),
        "حذف" => "DELETE".into(),
        "رأس" | "راس" => "HEAD".into(),
        other => other.to_ascii_uppercase(),
    }
}

fn builtin_secure_http(name: &str, args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 4 {
        return Err(format!(
            "الدالة '{}' تتوقع 2 إلى 4 معاملات (طريقة، رابط، [ترويسات]، [جسم])، استلمت {}",
            name,
            args.len()
        ));
    }
    let method = normalize_http_method(expect_text(name, &args[0])?);
    if method.is_empty() || !method.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(format!("طريقة HTTP غير صالحة: '{}'", method));
    }
    let url = expect_text(name, &args[1])?.trim();
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("الرابط يجب أن يبدأ بـ https:// أو http://".into());
    }
    if url.contains('\r') || url.contains('\n') {
        return Err("الرابط يحتوي على أحرف سطر غير مسموح بها".into());
    }

    let mut headers: Vec<(String, String)> = Vec::new();
    if args.len() >= 3 {
        match &args[2] {
            Value::Map(m) => {
                for (k, v) in m.borrow().iter() {
                    let vs = v.to_string();
                    if k.contains('\r') || k.contains('\n') || vs.contains('\r') || vs.contains('\n') {
                        return Err("ترويسة HTTP تحتوي على أحرف سطر غير مسموح بها".into());
                    }
                    headers.push((k.clone(), vs));
                }
            }
            Value::Null => {}
            _ => return Err("الترويسات يجب أن تكون قاموسًا".into()),
        }
    }

    let mut body = String::new();
    let mut auto_json = false;
    if args.len() == 4 {
        match &args[3] {
            Value::Null => {}
            Value::Text(s) => body = s.clone(),
            Value::Map(_) | Value::List(_) | Value::Tuple(_) | Value::Set(_) | Value::Instance { .. } => {
                body = crate::json::to_json_string(&args[3])?;
                auto_json = true;
            }
            other => body = other.to_string(),
        }
    }
    if auto_json
        && !headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"))
    {
        headers.push(("Content-Type".into(), "application/json; charset=utf-8".into()));
    }

    secure_http_request_impl(&method, url, &headers, &body)
}

// ============================================================================
// 2.1. تنفيذ جسور بايثون و Node.js و HTTPS/TLS (محمي ضد wasm32)
// ============================================================================

#[cfg(target_arch = "wasm32")]
fn invoke_python_bridge(_payload: &Value) -> Result<Value, String> {
    Err("تشغيل بايثون غير مدعوم داخل بيئة المتصفح (WebAssembly)".into())
}

#[cfg(target_arch = "wasm32")]
fn invoke_node_bridge(_payload: &Value) -> Result<Value, String> {
    Err("تشغيل جافاسكربت عبر Node.js غير مدعوم داخل بيئة المتصفح (WebAssembly)".into())
}

#[cfg(target_arch = "wasm32")]
fn secure_http_request_impl(
    _method: &str,
    _url: &str,
    _headers: &[(String, String)],
    _body: &str,
) -> Result<Value, String> {
    Err("طلب_شبكي_آمن غير مدعوم مباشرة في بيئة WebAssembly".into())
}

#[cfg(not(target_arch = "wasm32"))]
const PYTHON_BRIDGE_RUNNER: &str = r#"
import importlib, json, math, sys

def _to_jsonable(v):
    if v is None or isinstance(v, (bool, int, str)):
        return v
    if isinstance(v, float):
        if math.isfinite(v):
            return v
        return str(v)
    if isinstance(v, (bytes, bytearray)):
        try:
            return v.decode("utf-8")
        except Exception:
            return list(v)
    if isinstance(v, (list, tuple, set, frozenset)):
        return [_to_jsonable(x) for x in v]
    if isinstance(v, dict):
        return {str(k): _to_jsonable(val) for k, val in v.items()}
    if hasattr(v, "tolist") and callable(v.tolist):
        try:
            return _to_jsonable(v.tolist())
        except Exception:
            pass
    if hasattr(v, "to_dict") and callable(v.to_dict):
        try:
            return _to_jsonable(v.to_dict())
        except Exception:
            pass
    return str(v)

def _resolve_attr(module_name, dotted_name):
    obj = importlib.import_module(module_name)
    parts = [p for p in dotted_name.split(".") if p]
    prefix = module_name
    for part in parts:
        prefix = f"{prefix}.{part}"
        if hasattr(obj, part):
            obj = getattr(obj, part)
        else:
            try:
                obj = importlib.import_module(prefix)
            except Exception:
                obj = getattr(obj, part)
    return obj

try:
    req = json.loads(sys.stdin.buffer.read().decode("utf-8") or "{}")
    mode = req.get("mode", "")
    if mode == "call":
        target = _resolve_attr(req["module"], req["func"])
        res = target(*req.get("args", []), **req.get("kwargs", {}))
    elif mode == "attr":
        res = _resolve_attr(req["module"], req["attr"])
    elif mode == "exec":
        code = req.get("code", "")
        ns = dict(req.get("inputs", {}))
        try:
            compiled = compile(code, "<lisan>", "eval")
            res = eval(compiled, ns, ns)
        except SyntaxError:
            exec(compile(code, "<lisan>", "exec"), ns, ns)
            if "النتيجة" in ns:
                res = ns["النتيجة"]
            elif "result" in ns:
                res = ns["result"]
            else:
                res = None
    else:
        raise ValueError(f"وضع غير معروف: {mode}")
    sys.stdout.write(json.dumps({"ok": True, "value": _to_jsonable(res)}, ensure_ascii=False))
except Exception as e:
    sys.stdout.write(json.dumps({"ok": False, "error": f"{type(e).__name__}: {e}"}, ensure_ascii=False))
"#;

#[cfg(not(target_arch = "wasm32"))]
const NODE_BRIDGE_RUNNER: &str = r#"
const fs = require("fs");

function toJsonable(v) {
    if (v === undefined || v === null) return null;
    if (typeof v === "boolean" || typeof v === "string") return v;
    if (typeof v === "number") return Number.isFinite(v) ? v : String(v);
    if (typeof v === "bigint") {
        const n = Number(v);
        return Number.isSafeInteger(n) ? n : v.toString();
    }
    if (Buffer.isBuffer(v)) return v.toString("utf8");
    if (Array.isArray(v)) return v.map(toJsonable);
    if (v instanceof Set) return Array.from(v).map(toJsonable);
    if (v instanceof Map) {
        const out = {};
        for (const [k, val] of v.entries()) out[String(k)] = toJsonable(val);
        return out;
    }
    if (v instanceof Date) return v.toISOString();
    if (typeof v === "object") {
        const out = {};
        for (const [k, val] of Object.entries(v)) out[k] = toJsonable(val);
        return out;
    }
    return String(v);
}

(async () => {
    try {
        const raw = fs.readFileSync(0, "utf8");
        const req = JSON.parse(raw || "{}");
        let res = null;
        if (req.mode === "call") {
            let parent = require(req.module);
            let target = parent;
            const parts = String(req.func || "").split(".").filter(Boolean);
            for (const p of parts) {
                parent = target;
                target = target[p];
                if (target === undefined) {
                    throw new Error(`الدالة أو الخاصية '${req.func}' غير موجودة في الوحدة '${req.module}'`);
                }
            }
            if (typeof target === "function") {
                res = await target.apply(parent, req.args || []);
            } else {
                res = target;
            }
        } else if (req.mode === "exec") {
            const code = String(req.code || "");
            const inputs = req.inputs || {};
            const keys = Object.keys(inputs);
            const vals = keys.map((k) => inputs[k]);
            const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
            let fn;
            try {
                fn = new AsyncFunction("require", ...keys, `return (${code});`);
            } catch (_syntaxErr) {
                fn = new AsyncFunction(
                    "require",
                    ...keys,
                    `${code}\n;if (typeof النتيجة !== "undefined") return النتيجة; if (typeof result !== "undefined") return result; return null;`
                );
            }
            res = await fn(require, ...vals);
        } else {
            throw new Error(`وضع غير معروف: ${req.mode}`);
        }
        process.stdout.write(JSON.stringify({ ok: true, value: toJsonable(res) }));
    } catch (e) {
        const msg = e && e.message ? `${e.name || "Error"}: ${e.message}` : String(e);
        process.stdout.write(JSON.stringify({ ok: false, error: msg }));
    }
})();
"#;

#[cfg(not(target_arch = "wasm32"))]
const PYTHON_HTTP_FALLBACK: &str = r#"
import json, ssl, sys, urllib.request, urllib.error

try:
    req_data = json.loads(sys.stdin.buffer.read().decode("utf-8") or "{}")
    url = req_data["url"]
    method = req_data["method"]
    headers = req_data.get("headers", {})
    body = req_data.get("body", "")
    data = body.encode("utf-8") if body else None
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    ctx = ssl.create_default_context()
    try:
        with urllib.request.urlopen(req, timeout=30, context=ctx) as resp:
            status = resp.getcode()
            resp_body = resp.read().decode("utf-8", errors="replace")
            resp_headers = dict(resp.headers.items())
    except urllib.error.HTTPError as he:
        status = he.code
        resp_body = he.read().decode("utf-8", errors="replace")
        resp_headers = dict(he.headers.items()) if he.headers else {}
    sys.stdout.write(json.dumps({
        "ok": True,
        "status": status,
        "body": resp_body,
        "headers": resp_headers
    }, ensure_ascii=False))
except Exception as e:
    sys.stdout.write(json.dumps({"ok": False, "error": f"{type(e).__name__}: {e}"}, ensure_ascii=False))
"#;

#[cfg(not(target_arch = "wasm32"))]
fn find_python_command() -> Option<&'static str> {
    use std::process::{Command, Stdio};
    for candidate in ["python3", "python"] {
        if let Ok(out) = Command::new(candidate)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            if out.status.success() {
                let text = format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                );
                if text.contains("Python 3") {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn is_node_available() -> bool {
    use std::process::{Command, Stdio};
    Command::new("node")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
fn run_process_with_stdin(program: &str, args: &[&str], stdin_data: &str) -> Result<std::process::Output, std::io::Error> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new(program)
        .args(args)
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(stdin_data.as_bytes());
    }
    child.wait_with_output()
}

#[cfg(not(target_arch = "wasm32"))]
fn unwrap_bridge_response(stdout: &str, stderr: &str, runtime_label: &str) -> Result<Value, String> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "فشل تنفيذ {}: {}",
            runtime_label,
            if stderr.trim().is_empty() {
                "لم يُرجع أي مخرجات"
            } else {
                stderr.trim()
            }
        ));
    }
    let parsed = crate::json::parse_json(trimmed).map_err(|e| {
        format!(
            "تعذر قراءة استجابة {} ({}): {}",
            runtime_label, e, trimmed
        )
    })?;
    if let Value::Map(pairs) = &parsed {
        let borrowed = pairs.borrow();
        let ok = borrowed
            .iter()
            .find(|(k, _)| k == "ok")
            .and_then(|(_, v)| match v {
                Value::Bool(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(false);
        if ok {
            let val = borrowed
                .iter()
                .find(|(k, _)| k == "value")
                .map(|(_, v)| v.clone())
                .unwrap_or(Value::Null);
            return Ok(val);
        }
        let err_msg = borrowed
            .iter()
            .find(|(k, _)| k == "error")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "خطأ غير معروف".into());
        return Err(format!("خطأ من {}: {}", runtime_label, err_msg));
    }
    Err(format!("صيغة استجابة غير متوقعة من {}", runtime_label))
}

#[cfg(not(target_arch = "wasm32"))]
fn invoke_python_bridge(payload: &Value) -> Result<Value, String> {
    let py = find_python_command()
        .ok_or_else(|| "مفسّر Python 3 غير متوفر على هذا النظام".to_string())?;
    let json_in = crate::json::to_json_string(payload)?;
    let out = run_process_with_stdin(py, &["-c", PYTHON_BRIDGE_RUNNER], &json_in)
        .map_err(|e| format!("تعذر تشغيل Python 3: {}", e))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    unwrap_bridge_response(&stdout, &stderr, "بايثون")
}

#[cfg(not(target_arch = "wasm32"))]
fn invoke_node_bridge(payload: &Value) -> Result<Value, String> {
    let json_in = crate::json::to_json_string(payload)?;
    let out = run_process_with_stdin("node", &["-e", NODE_BRIDGE_RUNNER], &json_in).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "بيئة Node.js غير متوفرة على هذا النظام".to_string()
        } else {
            format!("تعذر تشغيل Node.js: {}", e)
        }
    })?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    unwrap_bridge_response(&stdout, &stderr, "جافاسكربت")
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_curl_http_output(raw: &str) -> Result<(u16, Vec<(String, Value)>, String), String> {
    let mut rest = raw;
    let mut status: Option<u16> = None;
    let mut headers: Vec<(String, Value)> = Vec::new();

    while rest.starts_with("HTTP/") {
        let (header_block, after) = if let Some(pos) = rest.find("\r\n\r\n") {
            (&rest[..pos], &rest[pos + 4..])
        } else if let Some(pos) = rest.find("\n\n") {
            (&rest[..pos], &rest[pos + 2..])
        } else {
            (rest, "")
        };

        let mut lines = header_block.lines();
        if let Some(status_line) = lines.next() {
            let mut parts = status_line.split_whitespace();
            let _proto = parts.next();
            if let Some(code_str) = parts.next() {
                if let Ok(code) = code_str.parse::<u16>() {
                    status = Some(code);
                }
            }
        }
        headers.clear();
        for line in lines {
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim().to_string();
                let val = Value::Text(v.trim().to_string());
                if let Some(existing) = headers.iter_mut().find(|(ek, _)| ek.eq_ignore_ascii_case(&key)) {
                    existing.1 = val;
                } else {
                    headers.push((key, val));
                }
            }
        }
        rest = after;
    }

    let code = status.ok_or_else(|| "تعذر تحليل رمز حالة استجابة HTTP من الخادم".to_string())?;
    Ok((code, headers, rest.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn secure_http_via_curl(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: &str,
) -> Result<Value, std::io::ErrorKind> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut cmd = Command::new("curl");
    cmd.arg("-sS")
        .arg("-L")
        .arg("-i")
        .arg("--max-time")
        .arg("30")
        .arg("-X")
        .arg(method);

    for (k, v) in headers {
        cmd.arg("-H").arg(format!("{}: {}", k, v));
    }

    let has_body = !body.is_empty();
    if has_body {
        cmd.arg("--data-binary").arg("@-");
    }
    cmd.arg(url);

    cmd.stdin(if has_body { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| e.kind())?;
    if has_body {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(body.as_bytes());
        }
    }
    let out = child.wait_with_output().map_err(|e| e.kind())?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Ok(Value::map(vec![
            ("حالة".into(), Value::Number(0.0)),
            ("نص".into(), Value::Text(stderr)),
            ("نجاح".into(), Value::Bool(false)),
            ("ترويسات".into(), Value::map(Vec::new())),
        ]));
    }
    let raw = String::from_utf8_lossy(&out.stdout);
    match parse_curl_http_output(&raw) {
        Ok((status, resp_headers, resp_body)) => Ok(Value::map(vec![
            ("حالة".into(), Value::Number(status as f64)),
            ("نص".into(), Value::Text(resp_body)),
            ("نجاح".into(), Value::Bool((200..300).contains(&status))),
            ("ترويسات".into(), Value::map(resp_headers)),
        ])),
        Err(err) => Ok(Value::map(vec![
            ("حالة".into(), Value::Number(0.0)),
            ("نص".into(), Value::Text(err)),
            ("نجاح".into(), Value::Bool(false)),
            ("ترويسات".into(), Value::map(Vec::new())),
        ])),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn secure_http_via_python(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: &str,
) -> Result<Value, String> {
    let py = find_python_command()
        .ok_or_else(|| "لم يتم العثور على curl أو Python 3 لتنفيذ الطلب الشبكي الآمن".to_string())?;
    let hdr_map: Vec<(String, Value)> = headers
        .iter()
        .map(|(k, v)| (k.clone(), Value::Text(v.clone())))
        .collect();
    let payload = Value::map(vec![
        ("method".into(), Value::Text(method.into())),
        ("url".into(), Value::Text(url.into())),
        ("headers".into(), Value::map(hdr_map)),
        ("body".into(), Value::Text(body.into())),
    ]);
    let json_in = crate::json::to_json_string(&payload)?;
    let out = run_process_with_stdin(py, &["-c", PYTHON_HTTP_FALLBACK], &json_in)
        .map_err(|e| format!("فشل تنفيذ الطلب الشبكي عبر بايثون: {}", e))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed = crate::json::parse_json(stdout.trim())
        .map_err(|e| format!("تعذر قراءة استجابة الطلب الشبكي: {}", e))?;
    if let Value::Map(pairs) = &parsed {
        let borrowed = pairs.borrow();
        let ok = borrowed
            .iter()
            .find(|(k, _)| k == "ok")
            .and_then(|(_, v)| match v {
                Value::Bool(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(false);
        if !ok {
            let err_msg = borrowed
                .iter()
                .find(|(k, _)| k == "error")
                .map(|(_, v)| v.to_string())
                .unwrap_or_else(|| "فشل الاتصال الشبكي".into());
            return Err(format!("فشل الطلب الشبكي الآمن: {}", err_msg));
        }
        let status = borrowed
            .iter()
            .find(|(k, _)| k == "status")
            .and_then(|(_, v)| match v {
                Value::Number(n) => Some(*n as u16),
                _ => None,
            })
            .unwrap_or(0);
        let resp_body = borrowed
            .iter()
            .find(|(k, _)| k == "body")
            .map(|(_, v)| v.to_string())
            .unwrap_or_default();
        let resp_headers = borrowed
            .iter()
            .find(|(k, _)| k == "headers")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| Value::map(Vec::new()));
        return Ok(Value::map(vec![
            ("حالة".into(), Value::Number(status as f64)),
            ("نص".into(), Value::Text(resp_body)),
            ("نجاح".into(), Value::Bool((200..300).contains(&status))),
            ("ترويسات".into(), resp_headers),
        ]));
    }
    Err("استجابة غير صالحة من عميل الشبكة".into())
}

#[cfg(not(target_arch = "wasm32"))]
fn secure_http_request_impl(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: &str,
) -> Result<Value, String> {
    match secure_http_via_curl(method, url, headers, body) {
        Ok(val) => Ok(val),
        Err(std::io::ErrorKind::NotFound) => secure_http_via_python(method, url, headers, body),
        Err(_) => secure_http_via_python(method, url, headers, body),
    }
}

// ============================================================================
// 3. المترجم: لسان ← Python 3 (يعمل على Native و WebAssembly)
// ============================================================================

const PYTHON_PREAMBLE: &str = r#"#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# مُولَّد تلقائيًا بواسطة مترجم لغة لسان (Lisan → Python 3)
import importlib
import json
import math
import sys
import time


def _lsn_add(a, b):
    if isinstance(a, str) or isinstance(b, str):
        return str(a) + str(b)
    return a + b
"#;

/// تحويل شيفرة لسان إلى شيفرة Python 3 صالحة ونظيفة (متوافق مع Native و WASM)
pub fn transpile_to_python(source: &str) -> Result<String, String> {
    let tokens = crate::lexer::tokenize(source)?;
    let statements = crate::parser::parse(tokens)?;
    let mut ctx = PyEmitCtx { match_counter: 0 };
    let mut out = String::new();
    out.push_str(PYTHON_PREAMBLE);
    out.push('\n');
    for stmt in &statements {
        let rendered = ctx.emit_stmt(stmt, 0, false);
        if !rendered.is_empty() {
            out.push_str(&rendered);
        }
    }
    Ok(out)
}

/// بناء وحفظ ملف Python 3 (`<stem>.py`) من ملف لسان (`.lsn`)
#[cfg(not(target_arch = "wasm32"))]
pub fn build_python_file(input: &Path, output: Option<&Path>) -> Result<PathBuf, String> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| format!("تعذرت قراءة ملف لسان '{}': {}", input.display(), e))?;
    let py_code = transpile_to_python(&source)?;

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            input.with_file_name(format!("{}.py", stem))
        }
    };

    std::fs::write(&out_path, py_code)
        .map_err(|e| format!("تعذر حفظ ملف بايثون '{}': {}", out_path.display(), e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&out_path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&out_path, perms);
        }
    }

    Ok(out_path)
}

#[cfg(target_arch = "wasm32")]
pub fn build_python_file(_input: &Path, _output: Option<&Path>) -> Result<PathBuf, String> {
    Err("كتابة ملفات بايثون على القرص غير مدعومة في بيئة WebAssembly".into())
}

fn is_interop_stdlib_name(name: &str) -> bool {
    let trimmed = name.trim().strip_suffix(".lsn").unwrap_or(name.trim());
    let norm = crate::diagnostics::normalize_arabic(trimmed);
    matches!(
        norm.as_str(),
        "توافق"
            | "ترابط"
            | "بايثون"
            | "جافاسكربت"
            | "ذكاء_خارجي"
            | "interop"
            | "python"
            | "javascript"
            | "js"
    )
}

struct PyEmitCtx {
    match_counter: usize,
}

impl PyEmitCtx {
    fn emit_body(&mut self, stmts: &[Stmt], indent: usize, in_init: bool) -> String {
        let pad = "    ".repeat(indent);
        if stmts.is_empty() {
            return format!("{}pass\n", pad);
        }
        let mut s = String::new();
        for inner in stmts {
            s.push_str(&self.emit_stmt(inner, indent, in_init));
        }
        if s.trim().is_empty() {
            format!("{}pass\n", pad)
        } else {
            s
        }
    }

    fn emit_stmt(&mut self, stmt: &Stmt, indent: usize, in_init: bool) -> String {
        let pad = "    ".repeat(indent);
        match stmt {
            Stmt::Spanned(inner, _, _) => self.emit_stmt(inner, indent, in_init),
            Stmt::Import(module) => {
                if is_interop_stdlib_name(module) {
                    format!("{}# مكتبة التوافق البيني '{}'\n", pad, module)
                } else {
                    format!("{}# استورد {}\n", pad, module)
                }
            }
            Stmt::ImportModule { module, alias } => {
                if is_interop_stdlib_name(module) {
                    String::new()
                } else if let Some(al) = alias {
                    format!("{}import {} as {}\n", pad, py_ident(module), py_ident(al))
                } else {
                    format!("{}import {}\n", pad, py_ident(module))
                }
            }
            Stmt::FromImport { module, items } => {
                if is_interop_stdlib_name(module) {
                    String::new()
                } else {
                    let py_items: Vec<String> = items.iter().map(|i| py_ident(i)).collect();
                    format!(
                        "{}from {} import {}\n",
                        pad,
                        py_ident(module),
                        py_items.join(", ")
                    )
                }
            }
            Stmt::Let(name, expr) => {
                // إذا كانت دالة مجهولة متعددة الأسطر، نترجمها إلى def نظيفة
                if let Expr::Lambda(params, body) = expr.unwrap_spanned() {
                    if !is_single_return_lambda(body) {
                        let py_params: Vec<String> = params.iter().map(|p| py_ident(p)).collect();
                        let mut s = format!("{}def {}({}):\n", pad, py_ident(name), py_params.join(", "));
                        s.push_str(&self.emit_body(body, indent + 1, false));
                        return s;
                    }
                }
                format!("{}{} = {}\n", pad, py_ident(name), self.emit_expr(expr))
            }
            Stmt::LetUnpack(names, expr) => {
                let lhs: Vec<String> = names.iter().map(|n| py_ident(n)).collect();
                format!("{}{} = {}\n", pad, lhs.join(", "), self.emit_expr(expr))
            }
            Stmt::Print(expr) => format!("{}print({})\n", pad, self.emit_expr(expr)),
            Stmt::Expr(expr) => match expr.unwrap_spanned() {
                Expr::Assign(name, val) => {
                    format!("{}{} = {}\n", pad, py_ident(name), self.emit_expr(val))
                }
                Expr::SetProperty(target, prop, val) => format!(
                    "{}{}.{} = {}\n",
                    pad,
                    self.emit_expr(target),
                    py_ident(prop),
                    self.emit_expr(val)
                ),
                Expr::AssignIndex(name, idx, val) => format!(
                    "{}{}[{}] = {}\n",
                    pad,
                    py_ident(name),
                    self.emit_expr(idx),
                    self.emit_expr(val)
                ),
                Expr::AssignUnpack(names, val) => {
                    let lhs: Vec<String> = names.iter().map(|n| py_ident(n)).collect();
                    format!("{}{} = {}\n", pad, lhs.join(", "), self.emit_expr(val))
                }
                other => format!("{}{}\n", pad, self.emit_expr(other)),
            },
            Stmt::Block(stmts) => self.emit_body(stmts, indent, in_init),
            Stmt::If(cond, yes, no) => {
                self.emit_if(cond, yes, no.as_deref(), indent, in_init, "if")
            }
            Stmt::While(cond, body) => {
                let mut s = format!("{}while {}:\n", pad, self.emit_expr(cond));
                s.push_str(&self.emit_body(body, indent + 1, in_init));
                s
            }
            Stmt::ForEach(var, iter, body) => {
                let mut s = format!("{}for {} in {}:\n", pad, py_ident(var), self.emit_expr(iter));
                s.push_str(&self.emit_body(body, indent + 1, in_init));
                s
            }
            Stmt::ForEachUnpack(vars, iter, body) => {
                let py_vars: Vec<String> = vars.iter().map(|v| py_ident(v)).collect();
                let mut s = format!(
                    "{}for {} in {}:\n",
                    pad,
                    py_vars.join(", "),
                    self.emit_expr(iter)
                );
                s.push_str(&self.emit_body(body, indent + 1, in_init));
                s
            }
            Stmt::With { expr, alias, body } => {
                let mut s = if let Some(al) = alias {
                    format!("{}with {} as {}:\n", pad, self.emit_expr(expr), py_ident(al))
                } else {
                    format!("{}with {}:\n", pad, self.emit_expr(expr))
                };
                s.push_str(&self.emit_body(body, indent + 1, in_init));
                s
            }
            Stmt::Function(func) => self.emit_function(func, indent, false),
            Stmt::Class(cls) => self.emit_class(cls, indent),
            Stmt::Return(Some(expr)) => {
                if in_init {
                    format!("{}return\n", pad)
                } else {
                    format!("{}return {}\n", pad, self.emit_expr(expr))
                }
            }
            Stmt::Return(None) => format!("{}return\n", pad),
            Stmt::Break => format!("{}break\n", pad),
            Stmt::Continue => format!("{}continue\n", pad),
            Stmt::TryCatch {
                try_body,
                catch_var,
                catch_body,
                finally_body,
            } => {
                let mut s = format!("{}try:\n", pad);
                s.push_str(&self.emit_body(try_body, indent + 1, in_init));
                if let Some(var) = catch_var {
                    s.push_str(&format!("{}except Exception as {}:\n", pad, py_ident(var)));
                } else {
                    s.push_str(&format!("{}except Exception:\n", pad));
                }
                s.push_str(&self.emit_body(catch_body, indent + 1, in_init));
                if let Some(fin) = finally_body {
                    s.push_str(&format!("{}finally:\n", pad));
                    s.push_str(&self.emit_body(fin, indent + 1, in_init));
                }
                s
            }
            Stmt::Throw(expr) => format!("{}raise RuntimeError({})\n", pad, self.emit_expr(expr)),
            Stmt::Match {
                target,
                cases,
                default_case,
            } => {
                self.match_counter += 1;
                let mvar = format!("_lsn_match_{}", self.match_counter);
                let mut s = format!("{}{} = {}\n", pad, mvar, self.emit_expr(target));
                if cases.is_empty() {
                    if let Some(def_stmts) = default_case {
                        s.push_str(&self.emit_body(def_stmts, indent, in_init));
                    }
                    return s;
                }
                for (idx, (case_expr, case_body)) in cases.iter().enumerate() {
                    let kw = if idx == 0 { "if" } else { "elif" };
                    s.push_str(&format!(
                        "{}{} {} == {}:\n",
                        pad,
                        kw,
                        mvar,
                        self.emit_expr(case_expr)
                    ));
                    s.push_str(&self.emit_body(case_body, indent + 1, in_init));
                }
                if let Some(def_stmts) = default_case {
                    s.push_str(&format!("{}else:\n", pad));
                    s.push_str(&self.emit_body(def_stmts, indent + 1, in_init));
                }
                s
            }
        }
    }

    fn emit_if(
        &mut self,
        cond: &Expr,
        yes: &[Stmt],
        no: Option<&[Stmt]>,
        indent: usize,
        in_init: bool,
        kw: &str,
    ) -> String {
        let pad = "    ".repeat(indent);
        let mut s = format!("{}{} {}:\n", pad, kw, self.emit_expr(cond));
        s.push_str(&self.emit_body(yes, indent + 1, in_init));
        if let Some(no_body) = no {
            if no_body.len() == 1 {
                if let Stmt::If(elif_cond, elif_yes, elif_no) = no_body[0].unwrap_spanned() {
                    s.push_str(&self.emit_if(
                        elif_cond,
                        elif_yes,
                        elif_no.as_deref(),
                        indent,
                        in_init,
                        "elif",
                    ));
                    return s;
                }
            }
            s.push_str(&format!("{}else:\n", pad));
            s.push_str(&self.emit_body(no_body, indent + 1, in_init));
        }
        s
    }

    fn emit_function(&mut self, func: &Function, indent: usize, is_method: bool) -> String {
        let pad = "    ".repeat(indent);
        let is_init = is_method && (func.name == "بناء" || func.name == "تهيئة");
        let fn_name = if is_init {
            "__init__".to_string()
        } else if is_method && func.name == "ادخل" {
            "__enter__".to_string()
        } else if is_method && func.name == "اخرج" {
            "__exit__".to_string()
        } else {
            py_ident(&func.name)
        };

        let mut params = Vec::new();
        if is_method {
            params.push("self".to_string());
        }
        for (idx, param) in func.params.iter().enumerate() {
            let p_ident = py_ident(param);
            if let Some(Some(def_expr)) = func.defaults.get(idx) {
                params.push(format!("{}={}", p_ident, self.emit_expr(def_expr)));
            } else {
                params.push(p_ident);
            }
        }
        if is_method && func.name == "اخرج" && func.params.is_empty() {
            params.push("*_exc".to_string());
        } else if let Some(vararg) = &func.vararg {
            params.push(format!("*{}", py_ident(vararg)));
        }

        let mut s = format!("{}def {}({}):\n", pad, fn_name, params.join(", "));
        s.push_str(&self.emit_body(&func.body, indent + 1, is_init));
        s.push('\n');
        s
    }

    fn emit_class(&mut self, cls: &ClassDef, indent: usize) -> String {
        let pad = "    ".repeat(indent);
        let cname = py_ident(&cls.name);
        let mut s = if let Some(parent) = &cls.parent {
            format!("{}class {}({}):\n", pad, cname, py_ident(parent))
        } else {
            format!("{}class {}:\n", pad, cname)
        };
        if cls.methods.is_empty() {
            s.push_str(&format!("{}    pass\n\n", pad));
        } else {
            for m in &cls.methods {
                s.push_str(&self.emit_function(m, indent + 1, true));
            }
        }
        s
    }

    fn emit_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Spanned(inner, _, _) => self.emit_expr(inner),
            Expr::Number(n) => {
                if n.fract() == 0.0 && n.abs() < (i64::MAX as f64) {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Expr::Text(t) => py_string_literal(t),
            Expr::Bool(true) => "True".into(),
            Expr::Bool(false) => "False".into(),
            Expr::Null => "None".into(),
            Expr::List(items) => {
                let vals: Vec<String> = items.iter().map(|e| self.emit_expr(e)).collect();
                format!("[{}]", vals.join(", "))
            }
            Expr::Map(pairs) => {
                let entries: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", py_string_literal(k), self.emit_expr(v)))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            }
            Expr::Variable(name) => match name.as_str() {
                "هذا" | "ذاتي" => "self".into(),
                "ط" | "قيمة_ط" => "math.pi".into(),
                "هـ" | "قيمة_هـ" => "math.e".into(),
                other => py_ident(other),
            },
            Expr::Assign(name, val) => format!("({} := {})", py_ident(name), self.emit_expr(val)),
            Expr::AssignIndex(name, idx, val) => format!(
                "{}.__setitem__({}, {})",
                py_ident(name),
                self.emit_expr(idx),
                self.emit_expr(val)
            ),
            Expr::Index(target, idx) => format!("{}[{}]", self.emit_expr(target), self.emit_expr(idx)),
            Expr::Unary(op, inner) => {
                let v = self.emit_expr(inner);
                match op {
                    Op::Negate => format!("(-{})", v),
                    Op::Not => format!("(not {})", v),
                    _ => v,
                }
            }
            Expr::Binary(left, op, right) => {
                let l = self.emit_expr(left);
                let r = self.emit_expr(right);
                if matches!(op, Op::Add) {
                    let l_str = is_known_str_expr(left);
                    let r_str = is_known_str_expr(right);
                    if l_str && r_str {
                        return format!("({} + {})", l, r);
                    } else if l_str {
                        return format!("({} + str({}))", l, r);
                    } else if r_str {
                        return format!("(str({}) + {})", l, r);
                    } else {
                        return format!("_lsn_add({}, {})", l, r);
                    }
                }
                let op_str = match op {
                    Op::Add => "+",
                    Op::Subtract => "-",
                    Op::Multiply => "*",
                    Op::Divide => "/",
                    Op::Modulo => "%",
                    Op::Power => "**",
                    Op::Equal => "==",
                    Op::NotEqual => "!=",
                    Op::Less => "<",
                    Op::LessEqual => "<=",
                    Op::Greater => ">",
                    Op::GreaterEqual => ">=",
                    Op::And => "and",
                    Op::Or => "or",
                    Op::In => "in",
                    Op::NotIn => "not in",
                    Op::Not | Op::Negate => "==",
                };
                format!("({} {} {})", l, op_str, r)
            }
            Expr::Call(name, args) => self.emit_call(name, args),
            Expr::GetProperty(target, prop) => {
                format!("{}.{}", self.emit_expr(target), py_ident(prop))
            }
            Expr::SetProperty(target, prop, val) => format!(
                "setattr({}, {}, {})",
                self.emit_expr(target),
                py_string_literal(prop),
                self.emit_expr(val)
            ),
            Expr::MethodCall(target, method, args) => self.emit_method_call(target, method, args),
            Expr::Lambda(params, body) => {
                let py_params: Vec<String> = params.iter().map(|p| py_ident(p)).collect();
                if body.len() == 1 {
                    if let Stmt::Return(Some(ret_expr)) = body[0].unwrap_spanned() {
                        return format!(
                            "(lambda {}: {})",
                            py_params.join(", "),
                            self.emit_expr(ret_expr)
                        );
                    }
                    if let Stmt::Expr(e) = body[0].unwrap_spanned() {
                        return format!("(lambda {}: {})", py_params.join(", "), self.emit_expr(e));
                    }
                }
                format!("(lambda {}: None)", py_params.join(", "))
            }
            Expr::Slice(target, start, end, step) => {
                let t = self.emit_expr(target);
                let s = start
                    .as_ref()
                    .map(|e| self.emit_int_expr(e))
                    .unwrap_or_default();
                let e = end
                    .as_ref()
                    .map(|e| self.emit_int_expr(e))
                    .unwrap_or_default();
                if let Some(st) = step {
                    format!("{}[{}:{}:{}]", t, s, e, self.emit_int_expr(st))
                } else {
                    format!("{}[{}:{}]", t, s, e)
                }
            }
            Expr::ListComprehension {
                expr,
                var,
                iterable,
                condition,
            } => {
                let e = self.emit_expr(expr);
                let v = py_ident(var);
                let it = self.emit_expr(iterable);
                if let Some(cond) = condition {
                    format!("[{} for {} in {} if {}]", e, v, it, self.emit_expr(cond))
                } else {
                    format!("[{} for {} in {}]", e, v, it)
                }
            }
            Expr::AssignUnpack(names, val) => {
                let lhs: Vec<String> = names.iter().map(|n| py_ident(n)).collect();
                format!("({} := {})", lhs.join(", "), self.emit_expr(val))
            }
            Expr::FString(segments) => self.emit_fstring(segments),
        }
    }

    fn emit_int_expr(&self, expr: &Expr) -> String {
        match expr.unwrap_spanned() {
            Expr::Number(n) if n.fract() == 0.0 => format!("{}", *n as i64),
            other => format!("int({})", self.emit_expr(other)),
        }
    }

    fn emit_fstring(&self, segments: &[Expr]) -> String {
        if segments.is_empty() {
            return "\"\"".into();
        }
        let mut can_use_fstr = true;
        let mut rendered_exprs: Vec<Option<String>> = Vec::with_capacity(segments.len());
        for seg in segments {
            match seg.unwrap_spanned() {
                Expr::Text(_) => rendered_exprs.push(None),
                other => {
                    let s = self.emit_expr(other);
                    if s.contains('"') || s.contains('\\') || s.contains('\n') {
                        can_use_fstr = false;
                    }
                    rendered_exprs.push(Some(s));
                }
            }
        }
        if can_use_fstr {
            let mut out = String::from("f\"");
            for (seg, rendered) in segments.iter().zip(rendered_exprs.iter()) {
                match seg.unwrap_spanned() {
                    Expr::Text(t) => {
                        for ch in t.chars() {
                            match ch {
                                '\\' => out.push_str("\\\\"),
                                '"' => out.push_str("\\\""),
                                '\n' => out.push_str("\\n"),
                                '\r' => out.push_str("\\r"),
                                '\t' => out.push_str("\\t"),
                                '{' => out.push_str("{{"),
                                '}' => out.push_str("}}"),
                                c => out.push(c),
                            }
                        }
                    }
                    _ => {
                        if let Some(expr_s) = rendered {
                            out.push('{');
                            out.push_str(expr_s);
                            out.push('}');
                        }
                    }
                }
            }
            out.push('"');
            return out;
        }
        let parts: Vec<String> = segments
            .iter()
            .map(|seg| match seg.unwrap_spanned() {
                Expr::Text(t) => py_string_literal(t),
                other => format!("str({})", self.emit_expr(other)),
            })
            .collect();
        if parts.len() == 1 {
            parts[0].clone()
        } else {
            format!("({})", parts.join(" + "))
        }
    }

    fn emit_call(&self, name: &str, args: &[Expr]) -> String {
        let py_args: Vec<String> = args.iter().map(|e| self.emit_expr(e)).collect();
        match name {
            "طول" if args.len() == 1 => format!("len({})", py_args[0]),
            "مدى" | "نطاق" if !args.is_empty() && args.len() <= 3 => {
                let int_args: Vec<String> = args.iter().map(|e| self.emit_int_expr(e)).collect();
                format!("range({})", int_args.join(", "))
            }
            "نوع" if args.len() == 1 => format!("type({})", py_args[0]),
            "نص" if args.len() == 1 => format!("str({})", py_args[0]),
            "عدد" if args.len() == 1 => format!("float({})", py_args[0]),
            "عدد_صحيح" if args.len() == 1 => format!("int({})", py_args[0]),
            "مجموع" if args.len() == 1 => format!("sum({})", py_args[0]),
            "متوسط" if args.len() == 1 => format!("(sum({}) / len({}))", py_args[0], py_args[0]),
            "أدنى" | "ادنى" if !args.is_empty() => format!("min({})", py_args.join(", ")),
            "أعلى" | "اعلى" if !args.is_empty() => format!("max({})", py_args.join(", ")),
            "مطلق" if args.len() == 1 => format!("abs({})", py_args[0]),
            "فرز" if args.len() == 1 => format!("sorted({})", py_args[0]),
            "عكس" | "اعكس" if args.len() == 1 => format!("list(reversed({}))", py_args[0]),
            "سحاب" if !args.is_empty() => format!("list(zip({}))", py_args.join(", ")),
            "مرقم" if args.len() == 1 => format!("list(enumerate({}))", py_args[0]),
            "مرقم" if args.len() == 2 => {
                format!("list(enumerate({}, {}))", py_args[0], self.emit_int_expr(&args[1]))
            }
            "أي" | "اي" if args.len() == 1 => format!("any({})", py_args[0]),
            "كل" if args.len() == 1 => format!("all({})", py_args[0]),
            "استورد_بايثون" if args.len() == 1 => format!("__import__({})", py_args[0]),
            "جذر" if args.len() == 1 => format!("math.sqrt({})", py_args[0]),
            "جا" if args.len() == 1 => format!("math.sin({})", py_args[0]),
            "جتا" if args.len() == 1 => format!("math.cos({})", py_args[0]),
            "ظا" if args.len() == 1 => format!("math.tan({})", py_args[0]),
            "لوغاريتم" | "لوغ" if args.len() == 1 => format!("math.log({})", py_args[0]),
            "أسي" | "اسي" if args.len() == 1 => format!("math.exp({})", py_args[0]),
            "قرّب" | "قرب" if args.len() == 1 => format!("round({})", py_args[0]),
            "قرّب" | "قرب" if args.len() == 2 => {
                format!("round({}, {})", py_args[0], self.emit_int_expr(&args[1]))
            }
            "أرضية" | "ارضية" if args.len() == 1 => format!("math.floor({})", py_args[0]),
            "سقف" if args.len() == 1 => format!("math.ceil({})", py_args[0]),
            "قوة" if args.len() == 2 => format!("pow({}, {})", py_args[0], py_args[1]),
            "أضف" | "اضف" | "ألحق" | "الحق" if args.len() == 2 => {
                format!("{}.append({})", py_args[0], py_args[1])
            }
            "مفاتيح" if args.len() == 1 => format!("list({}.keys())", py_args[0]),
            "قيم" if args.len() == 1 => format!("list({}.values())", py_args[0]),
            "عناصر" if args.len() == 1 => format!("list({}.items())", py_args[0]),
            "مجموعة" if args.len() == 1 => format!("set({})", py_args[0]),
            "صف" if !args.is_empty() => format!("tuple([{}])", py_args.join(", ")),
            "تحليل_json" | "تحليل_جيسون" | "فك_جيسون" if args.len() == 1 => {
                format!("json.loads({})", py_args[0])
            }
            "ترميز_json" | "ترميز_جيسون" | "إلى_جيسون" | "الى_جيسون" if args.len() == 1 => {
                format!("json.dumps({}, ensure_ascii=False)", py_args[0])
            }
            "تحقق" if args.len() == 1 => format!("assert ({})", py_args[0]),
            "تحقق_يساوي" if args.len() == 2 => {
                format!("assert ({} == {})", py_args[0], py_args[1])
            }
            "تحقق_صحيح" if args.len() == 1 => format!("assert bool({})", py_args[0]),
            "تحقق_خطأ" | "تحقق_خطا" if args.len() == 1 => {
                format!("assert (not bool({}))", py_args[0])
            }
            other => format!("{}({})", py_ident(other), py_args.join(", ")),
        }
    }

    fn emit_method_call(&self, target: &Expr, method: &str, args: &[Expr]) -> String {
        let t = self.emit_expr(target);
        let py_args: Vec<String> = args.iter().map(|e| self.emit_expr(e)).collect();
        if method == "\u{0}استدعاء" {
            return format!("{}({})", t, py_args.join(", "));
        }
        match method {
            "أضف" | "اضف" | "ألحق" | "الحق" if args.len() == 1 => {
                format!("{}.append({})", t, py_args[0])
            }
            "أضف" | "عيّن" if args.len() == 2 => {
                format!("{}.__setitem__({}, {})", t, py_args[0], py_args[1])
            }
            "طول" | "حجم" if args.is_empty() => format!("len({})", t),
            "مسح" if args.is_empty() => format!("{}.clear()", t),
            "نسخ" if args.is_empty() => format!("{}.copy()", t),
            "مجموع" if args.is_empty() => format!("sum({})", t),
            "تحتوي" | "يحتوي" if args.len() == 1 => format!("({} in {})", py_args[0], t),
            "تقسيم" if args.len() == 1 => format!("{}.split({})", t, py_args[0]),
            "أحرف_كبيرة" | "احرف_كبيرة" if args.is_empty() => format!("{}.upper()", t),
            "أحرف_صغيرة" | "احرف_صغيرة" if args.is_empty() => format!("{}.lower()", t),
            "استبدل" | "أعد" if args.len() == 2 => {
                format!("{}.replace({}, {})", t, py_args[0], py_args[1])
            }
            "مفاتيح" if args.is_empty() => format!("list({}.keys())", t),
            "قيم" if args.is_empty() => format!("list({}.values())", t),
            "عناصر" if args.is_empty() => format!("list({}.items())", t),
            "احصل" if args.len() == 1 => format!("{}.get({})", t, py_args[0]),
            "احصل" if args.len() == 2 => format!("{}.get({}, {})", t, py_args[0], py_args[1]),
            "عكس" if args.is_empty() => format!("list(reversed({}))", t),
            "فرز" if args.is_empty() => format!("sorted({})", t),
            other => format!("{}.{}({})", t, py_ident(other), py_args.join(", ")),
        }
    }
}

fn is_single_return_lambda(body: &[Stmt]) -> bool {
    if body.len() != 1 {
        return false;
    }
    matches!(
        body[0].unwrap_spanned(),
        Stmt::Return(Some(_)) | Stmt::Expr(_)
    )
}

fn is_known_str_expr(expr: &Expr) -> bool {
    match expr.unwrap_spanned() {
        Expr::Text(_) | Expr::FString(_) => true,
        Expr::Call(name, _) if name == "نص" => true,
        Expr::Binary(l, Op::Add, r) => is_known_str_expr(l) || is_known_str_expr(r),
        _ => false,
    }
}

fn py_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn py_ident(name: &str) -> String {
    if is_python_keyword(name) {
        return format!("{}_", name);
    }
    let mut out = String::with_capacity(name.len());
    for (idx, c) in name.chars().enumerate() {
        let valid = if idx == 0 {
            c == '_' || c.is_alphabetic()
        } else {
            c == '_'
                || c.is_alphanumeric()
                || ('\u{064B}'..='\u{065F}').contains(&c)
                || c == '\u{0670}'
                || c == '\u{0640}'
        };
        if valid {
            out.push(c);
        } else {
            if idx == 0 {
                out.push('_');
            }
            out.push_str(&format!("_u{:x}_", c as u32));
        }
    }
    if out.is_empty() {
        "_lsn_id".into()
    } else {
        out
    }
}

fn is_python_keyword(name: &str) -> bool {
    matches!(
        name,
        "False"
            | "None"
            | "True"
            | "and"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "del"
            | "elif"
            | "else"
            | "except"
            | "finally"
            | "for"
            | "from"
            | "global"
            | "if"
            | "import"
            | "in"
            | "is"
            | "lambda"
            | "nonlocal"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
}

// ============================================================================
// 4. المترجم: لسان ← ES2022 JavaScript (يعمل على Native و WebAssembly)
// ============================================================================

const JS_RUNTIME_HELPERS: &str = r#"// مُولَّد تلقائيًا بواسطة مترجم لغة لسان (Lisan → ES2022 JavaScript)
"use strict";

function _lsn_print(v) {
  console.log(v);
}

function _lsn_len(x) {
  if (x === null || x === undefined) return 0;
  if (typeof x === "string" || Array.isArray(x)) return x.length;
  if (x instanceof Set || x instanceof Map) return x.size;
  if (typeof x === "object") return Object.keys(x).length;
  return 0;
}

function _lsn_range(start, stop, step = 1) {
  let a = start;
  let b = stop;
  if (b === undefined) {
    b = a;
    a = 0;
  }
  const s = step === 0 ? 1 : step;
  const out = [];
  if (s > 0) {
    for (let i = a; i < b; i += s) out.push(i);
  } else {
    for (let i = a; i > b; i += s) out.push(i);
  }
  return out;
}

function _lsn_iter(x) {
  if (Array.isArray(x) || typeof x === "string") return x;
  if (x && typeof x[Symbol.iterator] === "function") return Array.from(x);
  if (x && typeof x === "object") return Object.keys(x);
  return [];
}

function _lsn_in(item, container) {
  if (typeof container === "string") return container.includes(String(item));
  if (Array.isArray(container)) return container.includes(item);
  if (container instanceof Set) return container.has(item);
  if (container && typeof container === "object") {
    return Object.prototype.hasOwnProperty.call(container, String(item));
  }
  return false;
}

function _lsn_slice(seq, start, end, step = 1) {
  const arr = typeof seq === "string" ? Array.from(seq) : Array.from(seq);
  const len = arr.length;
  const s = step === null || step === undefined ? 1 : Math.trunc(step);
  if (s === 0) throw new Error("خطوة التقطيع لا يمكن أن تكون صفرًا");
  const norm = (idx, def) => {
    if (idx === null || idx === undefined) return def;
    let i = Math.trunc(idx);
    if (i < 0) i += len;
    return i;
  };
  const out = [];
  if (s > 0) {
    const a = Math.max(0, Math.min(len, norm(start, 0)));
    const b = Math.max(0, Math.min(len, norm(end, len)));
    for (let i = a; i < b; i += s) out.push(arr[i]);
  } else {
    const a = Math.max(-1, Math.min(len - 1, norm(start, len - 1)));
    const b = Math.max(-1, Math.min(len - 1, norm(end, -1)));
    for (let i = a; i > b; i += s) out.push(arr[i]);
  }
  return typeof seq === "string" ? out.join("") : out;
}

function _lsn_zip(...lists) {
  if (lists.length === 0) return [];
  const arrs = lists.map(_lsn_iter);
  const minLen = Math.min(...arrs.map((a) => a.length));
  const out = [];
  for (let i = 0; i < minLen; i++) {
    out.push(arrs.map((a) => a[i]));
  }
  return out;
}

function _lsn_enumerate(seq, start = 0) {
  return Array.from(_lsn_iter(seq)).map((v, i) => [i + start, v]);
}

function _lsn_type(v) {
  if (v === null || v === undefined) return "لاشيء";
  if (Array.isArray(v)) return "قائمة";
  if (typeof v === "boolean") return "منطقي";
  if (typeof v === "number") return "عدد";
  if (typeof v === "string") return "نص";
  if (typeof v === "function") return "دالة";
  return "قاموس";
}

function _lsn_eq(a, b) {
  if (a === b) return true;
  if (Array.isArray(a) && Array.isArray(b)) {
    return a.length === b.length && a.every((v, i) => _lsn_eq(v, b[i]));
  }
  if (a && b && typeof a === "object" && typeof b === "object") {
    const ka = Object.keys(a);
    const kb = Object.keys(b);
    return ka.length === kb.length && ka.every((k) => _lsn_eq(a[k], b[k]));
  }
  return false;
}

function _lsn_assert(cond, msg = "فشل التحقق") {
  if (!cond) throw new Error(msg);
}
"#;

/// تحويل شيفرة لسان إلى ES2022 JavaScript نظيف وصالح لبيئة Node.js والمتصفح (متوافق مع Native و WASM)
pub fn transpile_to_js(source: &str) -> Result<String, String> {
    let tokens = crate::lexer::tokenize(source)?;
    let statements = crate::parser::parse(tokens)?;

    let mut class_names = HashSet::new();
    collect_class_names(&statements, &mut class_names);

    let mut ctx = JsEmitCtx {
        class_names,
        scopes: vec![HashSet::new()],
        with_counter: 0,
    };

    let mut out = String::new();
    out.push_str(JS_RUNTIME_HELPERS);
    out.push('\n');

    for stmt in &statements {
        let rendered = ctx.emit_stmt(stmt, 0, false);
        if !rendered.is_empty() {
            out.push_str(&rendered);
        }
    }
    Ok(out)
}

/// بناء وحفظ ملف JavaScript (`<stem>.js`) من ملف لسان (`.lsn`)
#[cfg(not(target_arch = "wasm32"))]
pub fn build_js_file(input: &Path, output: Option<&Path>) -> Result<PathBuf, String> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| format!("تعذرت قراءة ملف لسان '{}': {}", input.display(), e))?;
    let js_code = transpile_to_js(&source)?;

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            input.with_file_name(format!("{}.js", stem))
        }
    };

    std::fs::write(&out_path, js_code)
        .map_err(|e| format!("تعذر حفظ ملف جافاسكربت '{}': {}", out_path.display(), e))?;
    Ok(out_path)
}

#[cfg(target_arch = "wasm32")]
pub fn build_js_file(_input: &Path, _output: Option<&Path>) -> Result<PathBuf, String> {
    Err("كتابة ملفات جافاسكربت على القرص غير مدعومة في بيئة WebAssembly".into())
}

fn collect_class_names(stmts: &[Stmt], out: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt.unwrap_spanned() {
            Stmt::Class(cls) => {
                out.insert(cls.name.clone());
            }
            Stmt::Block(body) => collect_class_names(body, out),
            Stmt::If(_, yes, no) => {
                collect_class_names(yes, out);
                if let Some(n) = no {
                    collect_class_names(n, out);
                }
            }
            Stmt::While(_, body)
            | Stmt::ForEach(_, _, body)
            | Stmt::ForEachUnpack(_, _, body)
            | Stmt::With { body, .. } => collect_class_names(body, out),
            Stmt::Function(f) => collect_class_names(&f.body, out),
            Stmt::TryCatch {
                try_body,
                catch_body,
                finally_body,
                ..
            } => {
                collect_class_names(try_body, out);
                collect_class_names(catch_body, out);
                if let Some(fin) = finally_body {
                    collect_class_names(fin, out);
                }
            }
            Stmt::Match {
                cases,
                default_case,
                ..
            } => {
                for (_, body) in cases {
                    collect_class_names(body, out);
                }
                if let Some(d) = default_case {
                    collect_class_names(d, out);
                }
            }
            _ => {}
        }
    }
}

struct JsEmitCtx {
    class_names: HashSet<String>,
    scopes: Vec<HashSet<String>>,
    with_counter: usize,
}

impl JsEmitCtx {
    fn push_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_in_current_scope(&mut self, name: &str) -> bool {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string())
        } else {
            true
        }
    }

    fn emit_block_stmts(&mut self, stmts: &[Stmt], indent: usize, in_ctor: bool) -> String {
        self.push_scope();
        let mut s = String::new();
        for inner in stmts {
            s.push_str(&self.emit_stmt(inner, indent, in_ctor));
        }
        self.pop_scope();
        s
    }

    fn emit_stmt(&mut self, stmt: &Stmt, indent: usize, in_ctor: bool) -> String {
        let pad = "  ".repeat(indent);
        match stmt {
            Stmt::Spanned(inner, _, _) => self.emit_stmt(inner, indent, in_ctor),
            Stmt::Import(module) => format!("{}// استورد {}\n", pad, module),
            Stmt::ImportModule { module, alias } => {
                if is_interop_stdlib_name(module) {
                    format!("{}// وحدة توافق '{}'\n", pad, module)
                } else {
                    let local = js_ident(alias.as_deref().unwrap_or(module));
                    self.declare_in_current_scope(&local);
                    format!(
                        "{}const {} = require({});\n",
                        pad,
                        local,
                        js_string_literal(module)
                    )
                }
            }
            Stmt::FromImport { module, items } => {
                if is_interop_stdlib_name(module) {
                    String::new()
                } else {
                    let js_items: Vec<String> = items.iter().map(|i| js_ident(i)).collect();
                    for it in &js_items {
                        self.declare_in_current_scope(it);
                    }
                    format!(
                        "{}const {{ {} }} = require({});\n",
                        pad,
                        js_items.join(", "),
                        js_string_literal(module)
                    )
                }
            }
            Stmt::Let(name, expr) => {
                let ident = js_ident(name);
                let val = self.emit_expr(expr, indent);
                if self.declare_in_current_scope(&ident) {
                    format!("{}let {} = {};\n", pad, ident, val)
                } else {
                    format!("{}{} = {};\n", pad, ident, val)
                }
            }
            Stmt::LetUnpack(names, expr) => {
                let idents: Vec<String> = names.iter().map(|n| js_ident(n)).collect();
                let all_new = idents
                    .iter()
                    .all(|id| self.declare_in_current_scope(id));
                let val = self.emit_expr(expr, indent);
                if all_new {
                    format!("{}let [{}] = {};\n", pad, idents.join(", "), val)
                } else {
                    format!("{}[{}] = {};\n", pad, idents.join(", "), val)
                }
            }
            Stmt::Print(expr) => format!("{}_lsn_print({});\n", pad, self.emit_expr(expr, indent)),
            Stmt::Expr(expr) => match expr.unwrap_spanned() {
                Expr::Assign(name, val) => {
                    format!("{}{} = {};\n", pad, js_ident(name), self.emit_expr(val, indent))
                }
                Expr::SetProperty(target, prop, val) => format!(
                    "{}{}.{} = {};\n",
                    pad,
                    self.emit_expr(target, indent),
                    js_ident(prop),
                    self.emit_expr(val, indent)
                ),
                Expr::AssignIndex(name, idx, val) => format!(
                    "{}{}[{}] = {};\n",
                    pad,
                    js_ident(name),
                    self.emit_expr(idx, indent),
                    self.emit_expr(val, indent)
                ),
                Expr::AssignUnpack(names, val) => {
                    let lhs: Vec<String> = names.iter().map(|n| js_ident(n)).collect();
                    format!("{}[{}] = {};\n", pad, lhs.join(", "), self.emit_expr(val, indent))
                }
                other => format!("{}{};\n", pad, self.emit_expr(other, indent)),
            },
            Stmt::Block(stmts) => {
                let mut s = format!("{}{{\n", pad);
                s.push_str(&self.emit_block_stmts(stmts, indent + 1, in_ctor));
                s.push_str(&format!("{}}}\n", pad));
                s
            }
            Stmt::If(cond, yes, no) => self.emit_if(cond, yes, no.as_deref(), indent, in_ctor, false),
            Stmt::While(cond, body) => {
                let mut s = format!("{}while ({}) {{\n", pad, self.emit_expr(cond, indent));
                s.push_str(&self.emit_block_stmts(body, indent + 1, in_ctor));
                s.push_str(&format!("{}}}\n", pad));
                s
            }
            Stmt::ForEach(var, iter, body) => {
                let v = js_ident(var);
                let mut s = format!(
                    "{}for (const {} of _lsn_iter({})) {{\n",
                    pad,
                    v,
                    self.emit_expr(iter, indent)
                );
                self.push_scope();
                self.declare_in_current_scope(&v);
                for inner in body {
                    s.push_str(&self.emit_stmt(inner, indent + 1, in_ctor));
                }
                self.pop_scope();
                s.push_str(&format!("{}}}\n", pad));
                s
            }
            Stmt::ForEachUnpack(vars, iter, body) => {
                let js_vars: Vec<String> = vars.iter().map(|v| js_ident(v)).collect();
                let mut s = format!(
                    "{}for (const [{}] of _lsn_iter({})) {{\n",
                    pad,
                    js_vars.join(", "),
                    self.emit_expr(iter, indent)
                );
                self.push_scope();
                for v in &js_vars {
                    self.declare_in_current_scope(v);
                }
                for inner in body {
                    s.push_str(&self.emit_stmt(inner, indent + 1, in_ctor));
                }
                self.pop_scope();
                s.push_str(&format!("{}}}\n", pad));
                s
            }
            Stmt::With { expr, alias, body } => {
                self.with_counter += 1;
                let ctx_var = format!("_lsn_ctx_{}", self.with_counter);
                let inner_pad = "  ".repeat(indent + 1);
                let mut s = format!("{}{{\n", pad);
                s.push_str(&format!(
                    "{}const {} = {};\n",
                    inner_pad,
                    ctx_var,
                    self.emit_expr(expr, indent + 1)
                ));
                self.push_scope();
                if let Some(al) = alias {
                    let a_ident = js_ident(al);
                    self.declare_in_current_scope(&a_ident);
                    s.push_str(&format!(
                        "{}const {} = (typeof {}?.ادخل === \"function\") ? {}.ادخل() : {};\n",
                        inner_pad, a_ident, ctx_var, ctx_var, ctx_var
                    ));
                } else {
                    s.push_str(&format!(
                        "{}if (typeof {}?.ادخل === \"function\") {}.ادخل();\n",
                        inner_pad, ctx_var, ctx_var
                    ));
                }
                s.push_str(&format!("{}try {{\n", inner_pad));
                for inner in body {
                    s.push_str(&self.emit_stmt(inner, indent + 2, in_ctor));
                }
                s.push_str(&format!("{}}} finally {{\n", inner_pad));
                s.push_str(&format!(
                    "{}  if (typeof {}?.اخرج === \"function\") {}.اخرج();\n",
                    inner_pad, ctx_var, ctx_var
                ));
                s.push_str(&format!("{}}}\n", inner_pad));
                self.pop_scope();
                s.push_str(&format!("{}}}\n", pad));
                s
            }
            Stmt::Function(func) => self.emit_function(func, indent, false, false),
            Stmt::Class(cls) => self.emit_class(cls, indent),
            Stmt::Return(Some(expr)) => {
                if in_ctor {
                    format!("{}return;\n", pad)
                } else {
                    format!("{}return {};\n", pad, self.emit_expr(expr, indent))
                }
            }
            Stmt::Return(None) => format!("{}return;\n", pad),
            Stmt::Break => format!("{}break;\n", pad),
            Stmt::Continue => format!("{}continue;\n", pad),
            Stmt::TryCatch {
                try_body,
                catch_var,
                catch_body,
                finally_body,
            } => {
                let mut s = format!("{}try {{\n", pad);
                s.push_str(&self.emit_block_stmts(try_body, indent + 1, in_ctor));
                if let Some(var) = catch_var {
                    let cv = js_ident(var);
                    s.push_str(&format!("{}}} catch ({}) {{\n", pad, cv));
                    self.push_scope();
                    self.declare_in_current_scope(&cv);
                    for inner in catch_body {
                        s.push_str(&self.emit_stmt(inner, indent + 1, in_ctor));
                    }
                    self.pop_scope();
                } else {
                    s.push_str(&format!("{}}} catch (_err) {{\n", pad));
                    s.push_str(&self.emit_block_stmts(catch_body, indent + 1, in_ctor));
                }
                if let Some(fin) = finally_body {
                    s.push_str(&format!("{}}} finally {{\n", pad));
                    s.push_str(&self.emit_block_stmts(fin, indent + 1, in_ctor));
                }
                s.push_str(&format!("{}}}\n", pad));
                s
            }
            Stmt::Throw(expr) => format!(
                "{}throw new Error(String({}));\n",
                pad,
                self.emit_expr(expr, indent)
            ),
            Stmt::Match {
                target,
                cases,
                default_case,
            } => {
                let mut s = format!("{}switch ({}) {{\n", pad, self.emit_expr(target, indent));
                let case_pad = "  ".repeat(indent + 1);
                let break_pad = "  ".repeat(indent + 2);
                for (case_expr, case_body) in cases {
                    s.push_str(&format!(
                        "{}case {}: {{\n",
                        case_pad,
                        self.emit_expr(case_expr, indent + 1)
                    ));
                    s.push_str(&self.emit_block_stmts(case_body, indent + 2, in_ctor));
                    s.push_str(&format!("{}break;\n", break_pad));
                    s.push_str(&format!("{}}}\n", case_pad));
                }
                if let Some(def_body) = default_case {
                    s.push_str(&format!("{}default: {{\n", case_pad));
                    s.push_str(&self.emit_block_stmts(def_body, indent + 2, in_ctor));
                    s.push_str(&format!("{}break;\n", break_pad));
                    s.push_str(&format!("{}}}\n", case_pad));
                }
                s.push_str(&format!("{}}}\n", pad));
                s
            }
        }
    }

    fn emit_if(
        &mut self,
        cond: &Expr,
        yes: &[Stmt],
        no: Option<&[Stmt]>,
        indent: usize,
        in_ctor: bool,
        is_else_if: bool,
    ) -> String {
        let pad = "  ".repeat(indent);
        let prefix = if is_else_if { " else if" } else { "if" };
        let mut s = if is_else_if {
            format!("{} ({}) {{\n", prefix, self.emit_expr(cond, indent))
        } else {
            format!("{}{} ({}) {{\n", pad, prefix, self.emit_expr(cond, indent))
        };
        s.push_str(&self.emit_block_stmts(yes, indent + 1, in_ctor));
        s.push_str(&format!("{}}}", pad));
        if let Some(no_body) = no {
            if no_body.len() == 1 {
                if let Stmt::If(elif_cond, elif_yes, elif_no) = no_body[0].unwrap_spanned() {
                    s.push_str(&self.emit_if(
                        elif_cond,
                        elif_yes,
                        elif_no.as_deref(),
                        indent,
                        in_ctor,
                        true,
                    ));
                    return s;
                }
            }
            s.push_str(" else {\n");
            s.push_str(&self.emit_block_stmts(no_body, indent + 1, in_ctor));
            s.push_str(&format!("{}}}\n", pad));
        } else {
            s.push('\n');
        }
        s
    }

    fn emit_function(
        &mut self,
        func: &Function,
        indent: usize,
        is_method: bool,
        has_parent: bool,
    ) -> String {
        let pad = "  ".repeat(indent);
        let is_ctor = is_method && (func.name == "بناء" || func.name == "تهيئة");
        let fn_name = if is_ctor {
            "constructor".to_string()
        } else {
            js_ident(&func.name)
        };

        let mut params = Vec::new();
        let mut param_names = Vec::new();
        for (idx, param) in func.params.iter().enumerate() {
            let p_ident = js_ident(param);
            param_names.push(p_ident.clone());
            if let Some(Some(def_expr)) = func.defaults.get(idx) {
                params.push(format!("{} = {}", p_ident, self.emit_expr(def_expr, indent)));
            } else {
                params.push(p_ident);
            }
        }
        if let Some(vararg) = &func.vararg {
            let v_ident = js_ident(vararg);
            param_names.push(v_ident.clone());
            params.push(format!("...{}", v_ident));
        }

        let mut s = if is_method {
            format!("{}{}({}) {{\n", pad, fn_name, params.join(", "))
        } else {
            format!("{}function {}({}) {{\n", pad, fn_name, params.join(", "))
        };
        if is_ctor && has_parent {
            s.push_str(&format!("{}  super();\n", pad));
        }
        self.push_scope();
        for p in &param_names {
            self.declare_in_current_scope(p);
        }
        for inner in &func.body {
            s.push_str(&self.emit_stmt(inner, indent + 1, is_ctor));
        }
        self.pop_scope();
        s.push_str(&format!("{}}}\n\n", pad));
        s
    }

    fn emit_class(&mut self, cls: &ClassDef, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        let cname = js_ident(&cls.name);
        let has_parent = cls.parent.is_some();
        let mut s = if let Some(parent) = &cls.parent {
            format!("{}class {} extends {} {{\n", pad, cname, js_ident(parent))
        } else {
            format!("{}class {} {{\n", pad, cname)
        };
        for m in &cls.methods {
            s.push_str(&self.emit_function(m, indent + 1, true, has_parent));
        }
        s.push_str(&format!("{}}}\n\n", pad));
        s
    }

    fn emit_expr(&mut self, expr: &Expr, indent: usize) -> String {
        match expr {
            Expr::Spanned(inner, _, _) => self.emit_expr(inner, indent),
            Expr::Number(n) => {
                if n.fract() == 0.0 && n.abs() < (i64::MAX as f64) {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Expr::Text(t) => js_string_literal(t),
            Expr::Bool(true) => "true".into(),
            Expr::Bool(false) => "false".into(),
            Expr::Null => "null".into(),
            Expr::List(items) => {
                let vals: Vec<String> = items.iter().map(|e| self.emit_expr(e, indent)).collect();
                format!("[{}]", vals.join(", "))
            }
            Expr::Map(pairs) => {
                let entries: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", js_string_literal(k), self.emit_expr(v, indent)))
                    .collect();
                format!("({{ {} }})", entries.join(", "))
            }
            Expr::Variable(name) => match name.as_str() {
                "هذا" | "ذاتي" => "this".into(),
                "ط" | "قيمة_ط" => "Math.PI".into(),
                "هـ" | "قيمة_هـ" => "Math.E".into(),
                other => js_ident(other),
            },
            Expr::Assign(name, val) => {
                format!("({} = {})", js_ident(name), self.emit_expr(val, indent))
            }
            Expr::AssignIndex(name, idx, val) => format!(
                "({}[{}] = {})",
                js_ident(name),
                self.emit_expr(idx, indent),
                self.emit_expr(val, indent)
            ),
            Expr::Index(target, idx) => {
                format!("{}[{}]", self.emit_expr(target, indent), self.emit_expr(idx, indent))
            }
            Expr::Unary(op, inner) => {
                let v = self.emit_expr(inner, indent);
                match op {
                    Op::Negate => format!("(-{})", v),
                    Op::Not => format!("(!{})", v),
                    _ => v,
                }
            }
            Expr::Binary(left, op, right) => {
                let l = self.emit_expr(left, indent);
                let r = self.emit_expr(right, indent);
                match op {
                    Op::In => format!("_lsn_in({}, {})", l, r),
                    Op::NotIn => format!("(!_lsn_in({}, {}))", l, r),
                    _ => {
                        let op_str = match op {
                            Op::Add => "+",
                            Op::Subtract => "-",
                            Op::Multiply => "*",
                            Op::Divide => "/",
                            Op::Modulo => "%",
                            Op::Power => "**",
                            Op::Equal => "===",
                            Op::NotEqual => "!==",
                            Op::Less => "<",
                            Op::LessEqual => "<=",
                            Op::Greater => ">",
                            Op::GreaterEqual => ">=",
                            Op::And => "&&",
                            Op::Or => "||",
                            Op::In | Op::NotIn | Op::Not | Op::Negate => "===",
                        };
                        format!("({} {} {})", l, op_str, r)
                    }
                }
            }
            Expr::Call(name, args) => self.emit_call(name, args, indent),
            Expr::GetProperty(target, prop) => {
                format!("{}.{}", self.emit_expr(target, indent), js_ident(prop))
            }
            Expr::SetProperty(target, prop, val) => format!(
                "({}.{} = {})",
                self.emit_expr(target, indent),
                js_ident(prop),
                self.emit_expr(val, indent)
            ),
            Expr::MethodCall(target, method, args) => {
                self.emit_method_call(target, method, args, indent)
            }
            Expr::Lambda(params, body) => {
                let js_params: Vec<String> = params.iter().map(|p| js_ident(p)).collect();
                if body.len() == 1 {
                    if let Stmt::Return(Some(ret_expr)) = body[0].unwrap_spanned() {
                        return format!(
                            "(({}) => ({}))",
                            js_params.join(", "),
                            self.emit_expr(ret_expr, indent)
                        );
                    }
                }
                let mut s = format!("(({}) => {{\n", js_params.join(", "));
                self.push_scope();
                for p in &js_params {
                    self.declare_in_current_scope(p);
                }
                for inner in body {
                    s.push_str(&self.emit_stmt(inner, indent + 1, false));
                }
                self.pop_scope();
                s.push_str(&format!("{}}})", "  ".repeat(indent)));
                s
            }
            Expr::Slice(target, start, end, step) => {
                let t = self.emit_expr(target, indent);
                let s = start
                    .as_ref()
                    .map(|e| self.emit_expr(e, indent))
                    .unwrap_or_else(|| "null".into());
                let e = end
                    .as_ref()
                    .map(|e| self.emit_expr(e, indent))
                    .unwrap_or_else(|| "null".into());
                let st = step
                    .as_ref()
                    .map(|e| self.emit_expr(e, indent))
                    .unwrap_or_else(|| "1".into());
                format!("_lsn_slice({}, {}, {}, {})", t, s, e, st)
            }
            Expr::ListComprehension {
                expr,
                var,
                iterable,
                condition,
            } => {
                let v = js_ident(var);
                let it = self.emit_expr(iterable, indent);
                let mapped = self.emit_expr(expr, indent);
                if let Some(cond) = condition {
                    let c = self.emit_expr(cond, indent);
                    format!(
                        "Array.from(_lsn_iter({})).filter(({}) => ({})).map(({}) => ({}))",
                        it, v, c, v, mapped
                    )
                } else {
                    format!(
                        "Array.from(_lsn_iter({})).map(({}) => ({}))",
                        it, v, mapped
                    )
                }
            }
            Expr::AssignUnpack(names, val) => {
                let lhs: Vec<String> = names.iter().map(|n| js_ident(n)).collect();
                format!("([{}] = {})", lhs.join(", "), self.emit_expr(val, indent))
            }
            Expr::FString(segments) => self.emit_fstring(segments, indent),
        }
    }

    fn emit_fstring(&mut self, segments: &[Expr], indent: usize) -> String {
        let mut out = String::from("`");
        for seg in segments {
            match seg.unwrap_spanned() {
                Expr::Text(t) => {
                    let chars: Vec<char> = t.chars().collect();
                    let mut i = 0;
                    while i < chars.len() {
                        match chars[i] {
                            '\\' => out.push_str("\\\\"),
                            '`' => out.push_str("\\`"),
                            '$' if chars.get(i + 1) == Some(&'{') => {
                                out.push_str("\\${");
                                i += 1;
                            }
                            '\r' => out.push_str("\\r"),
                            c => out.push(c),
                        }
                        i += 1;
                    }
                }
                other => {
                    out.push_str("${");
                    out.push_str(&self.emit_expr(other, indent));
                    out.push('}');
                }
            }
        }
        out.push('`');
        out
    }

    fn emit_call(&mut self, name: &str, args: &[Expr], indent: usize) -> String {
        let js_args: Vec<String> = args.iter().map(|e| self.emit_expr(e, indent)).collect();
        if self.class_names.contains(name) {
            return format!("new {}({})", js_ident(name), js_args.join(", "));
        }
        match name {
            "اطبع" if args.len() == 1 => format!("_lsn_print({})", js_args[0]),
            "طول" if args.len() == 1 => format!("_lsn_len({})", js_args[0]),
            "مدى" | "نطاق" if !args.is_empty() && args.len() <= 3 => {
                format!("_lsn_range({})", js_args.join(", "))
            }
            "نوع" if args.len() == 1 => format!("_lsn_type({})", js_args[0]),
            "نص" if args.len() == 1 => format!("String({})", js_args[0]),
            "عدد" if args.len() == 1 => format!("Number({})", js_args[0]),
            "عدد_صحيح" if args.len() == 1 => format!("Math.trunc(Number({}))", js_args[0]),
            "مجموع" if args.len() == 1 => {
                format!("({}).reduce((_a, _b) => _a + _b, 0)", js_args[0])
            }
            "متوسط" if args.len() == 1 => format!(
                "(({}).reduce((_a, _b) => _a + _b, 0) / ({}).length)",
                js_args[0], js_args[0]
            ),
            "أدنى" | "ادنى" if args.len() == 1 => format!(
                "(Array.isArray({0}) ? Math.min(...{0}) : {0})",
                js_args[0]
            ),
            "أدنى" | "ادنى" if args.len() > 1 => format!("Math.min({})", js_args.join(", ")),
            "أعلى" | "اعلى" if args.len() == 1 => format!(
                "(Array.isArray({0}) ? Math.max(...{0}) : {0})",
                js_args[0]
            ),
            "أعلى" | "اعلى" if args.len() > 1 => format!("Math.max({})", js_args.join(", ")),
            "مطلق" if args.len() == 1 => format!("Math.abs({})", js_args[0]),
            "جذر" if args.len() == 1 => format!("Math.sqrt({})", js_args[0]),
            "جا" if args.len() == 1 => format!("Math.sin({})", js_args[0]),
            "جتا" if args.len() == 1 => format!("Math.cos({})", js_args[0]),
            "ظا" if args.len() == 1 => format!("Math.tan({})", js_args[0]),
            "لوغاريتم" | "لوغ" if args.len() == 1 => format!("Math.log({})", js_args[0]),
            "أسي" | "اسي" if args.len() == 1 => format!("Math.exp({})", js_args[0]),
            "قرّب" | "قرب" if args.len() == 1 => format!("Math.round({})", js_args[0]),
            "قرّب" | "قرب" if args.len() == 2 => {
                format!("Number(({}).toFixed({}))", js_args[0], js_args[1])
            }
            "أرضية" | "ارضية" if args.len() == 1 => format!("Math.floor({})", js_args[0]),
            "سقف" if args.len() == 1 => format!("Math.ceil({})", js_args[0]),
            "قوة" if args.len() == 2 => format!("Math.pow({}, {})", js_args[0], js_args[1]),
            "فرز" if args.len() == 1 => format!(
                "[...{}].sort((_a, _b) => (typeof _a === \"number\" && typeof _b === \"number\") ? _a - _b : String(_a).localeCompare(String(_b)))",
                js_args[0]
            ),
            "عكس" | "اعكس" if args.len() == 1 => format!(
                "(typeof {0} === \"string\" ? Array.from({0}).reverse().join(\"\") : [...{0}].reverse())",
                js_args[0]
            ),
            "سحاب" if !args.is_empty() => format!("_lsn_zip({})", js_args.join(", ")),
            "مرقم" if !args.is_empty() && args.len() <= 2 => {
                format!("_lsn_enumerate({})", js_args.join(", "))
            }
            "أي" | "اي" if args.len() == 1 => {
                format!("Array.from(_lsn_iter({})).some(Boolean)", js_args[0])
            }
            "كل" if args.len() == 1 => {
                format!("Array.from(_lsn_iter({})).every(Boolean)", js_args[0])
            }
            "أضف" | "اضف" | "ألحق" | "الحق" if args.len() == 2 => {
                format!("({}.push({}), {})", js_args[0], js_args[1], js_args[0])
            }
            "مفاتيح" if args.len() == 1 => format!("Object.keys({})", js_args[0]),
            "قيم" if args.len() == 1 => format!("Object.values({})", js_args[0]),
            "عناصر" if args.len() == 1 => format!("Object.entries({})", js_args[0]),
            "استورد_جافاسكربت" if args.len() == 1 => format!("require({})", js_args[0]),
            "تحليل_json" | "تحليل_جيسون" | "فك_جيسون" if args.len() == 1 => {
                format!("JSON.parse({})", js_args[0])
            }
            "ترميز_json" | "ترميز_جيسون" | "إلى_جيسون" | "الى_جيسون" if args.len() == 1 => {
                format!("JSON.stringify({})", js_args[0])
            }
            "تحقق" if args.len() == 1 => format!("_lsn_assert({})", js_args[0]),
            "تحقق_يساوي" if args.len() == 2 => {
                format!("_lsn_assert(_lsn_eq({}, {}), \"القيمتان غير متساويتين\")", js_args[0], js_args[1])
            }
            "تحقق_صحيح" if args.len() == 1 => format!("_lsn_assert(Boolean({}))", js_args[0]),
            "تحقق_خطأ" | "تحقق_خطا" if args.len() == 1 => {
                format!("_lsn_assert(!Boolean({}))", js_args[0])
            }
            other => format!("{}({})", js_ident(other), js_args.join(", ")),
        }
    }

    fn emit_method_call(
        &mut self,
        target: &Expr,
        method: &str,
        args: &[Expr],
        indent: usize,
    ) -> String {
        let t = self.emit_expr(target, indent);
        let js_args: Vec<String> = args.iter().map(|e| self.emit_expr(e, indent)).collect();
        if method == "\u{0}استدعاء" {
            return format!("{}({})", t, js_args.join(", "));
        }
        match method {
            "أضف" | "اضف" | "ألحق" | "الحق" if args.len() == 1 => {
                format!("{}.push({})", t, js_args[0])
            }
            "أضف" | "عيّن" if args.len() == 2 => {
                format!("({}[{}] = {})", t, js_args[0], js_args[1])
            }
            "طول" | "حجم" if args.is_empty() => format!("_lsn_len({})", t),
            "تحتوي" | "يحتوي" if args.len() == 1 => format!("_lsn_in({}, {})", js_args[0], t),
            "تقسيم" if args.len() == 1 => format!("{}.split({})", t, js_args[0]),
            "أحرف_كبيرة" | "احرف_كبيرة" if args.is_empty() => format!("{}.toUpperCase()", t),
            "أحرف_صغيرة" | "احرف_صغيرة" if args.is_empty() => format!("{}.toLowerCase()", t),
            "استبدل" | "أعد" if args.len() == 2 => {
                format!("{}.replaceAll({}, {})", t, js_args[0], js_args[1])
            }
            "مفاتيح" if args.is_empty() => format!("Object.keys({})", t),
            "قيم" if args.is_empty() => format!("Object.values({})", t),
            "عناصر" if args.is_empty() => format!("Object.entries({})", t),
            "احصل" if args.len() == 1 => format!("({}[{}] ?? null)", t, js_args[0]),
            "احصل" if args.len() == 2 => format!("({}[{}] ?? {})", t, js_args[0], js_args[1]),
            "نسخ" if args.is_empty() => {
                format!("(Array.isArray({0}) ? [...{0}] : ({{ ...{0} }}))", t)
            }
            "عكس" if args.is_empty() => format!(
                "(typeof {0} === \"string\" ? Array.from({0}).reverse().join(\"\") : [...{0}].reverse())",
                t
            ),
            "فرز" if args.is_empty() => format!(
                "[...{}].sort((_a, _b) => (typeof _a === \"number\" && typeof _b === \"number\") ? _a - _b : String(_a).localeCompare(String(_b)))",
                t
            ),
            other => format!("{}.{}({})", t, js_ident(other), js_args.join(", ")),
        }
    }
}

fn js_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn js_ident(name: &str) -> String {
    if is_js_keyword(name) {
        return format!("{}_", name);
    }
    let mut out = String::with_capacity(name.len());
    for (idx, c) in name.chars().enumerate() {
        let valid = if idx == 0 {
            c == '_' || c == '$' || c.is_alphabetic()
        } else {
            c == '_'
                || c == '$'
                || c.is_alphanumeric()
                || ('\u{064B}'..='\u{065F}').contains(&c)
                || c == '\u{0670}'
                || c == '\u{0640}'
        };
        if valid {
            out.push(c);
        } else {
            if idx == 0 {
                out.push('_');
            }
            out.push_str(&format!("_u{:x}_", c as u32));
        }
    }
    if out.is_empty() {
        "_lsn_id".into()
    } else {
        out
    }
}

fn is_js_keyword(name: &str) -> bool {
    matches!(
        name,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "let"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

// ============================================================================
// 5. الاختبارات الشاملة
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_interop_tokenizes_parses_and_is_pure_arabic() {
        let tokens = crate::lexer::tokenize(STDLIB_INTEROP)
            .expect("يجب تحليل STDLIB_INTEROP لفظيًا بنجاح");
        // التأكد من عدم وجود أي معرّف لاتيني خارج السلاسل النصية
        for tok in &tokens {
            if let crate::lexer::TokenKind::Identifier(ref name) = tok.kind {
                assert!(
                    !name.chars().any(|c| c.is_ascii_alphabetic()),
                    "وُجد معرّف إنجليزي غير مسموح به في STDLIB_INTEROP: '{}'",
                    name
                );
            }
        }
        let stmts = crate::parser::parse(tokens)
            .expect("يجب تحليل STDLIB_INTEROP نحويًا بنجاح");
        assert!(stmts.len() >= 5);
    }

    #[test]
    fn test_ai_client_offline_payload_builders() {
        let user_code = r##"
دع جيميني = جديد نموذج_ذكاء("جيميني"، "مفتاح_تجريبي")
جيميني.ضبط_النظام("أنت مساعد برمجي عربي")
دع ط_ج = جيميني.بناء_طلب("مرحباً بلسان")
تحقق_يساوي(ط_ج["مزود"]، "جيميني")
تحقق_يساوي(ط_ج["طريقة"]، "POST")
تحقق(ط_ج["رابط"].تحتوي("generativelanguage.googleapis.com"))
تحقق_يساوي(ط_ج["جسم"]["contents"][0]["parts"][0]["text"]، "مرحباً بلسان")

دع أوبن = جديد نموذج_ذكاء("أوبن_إيه_آي"، "sk-test"، "gpt-4o-mini")
دع ط_أ = أوبن.بناء_طلب("احسب 2+2")
تحقق_يساوي(ط_أ["مزود"]، "أوبن_إيه_آي")
تحقق_يساوي(ط_أ["ترويسات"]["Authorization"]، "Bearer sk-test")
تحقق_يساوي(ط_أ["جسم"]["model"]، "gpt-4o-mini")

دع كلود = جديد عميل_ذكي("كلود"، "claude-key")
دع ط_ك = كلود.بناء_طلب("اشرح التوافق")
تحقق_يساوي(ط_ك["مزود"]، "كلود")
تحقق_يساوي(ط_ك["ترويسات"]["x-api-key"]، "claude-key")

دع محلي = جديد نموذج_ذكاء("محلي"، ""، "llama3.2")
دع ط_م = محلي.بناء_طلب("أهلاً")
تحقق_يساوي(ط_م["مزود"]، "محلي")
تحقق(ط_م["رابط"].تحتوي("11434/api/chat"))
اطبع("اكتمل_اختبار_الذكاء")
"##;
        let full = format!("{}\n{}", STDLIB_INTEROP, user_code);
        let out = crate::runtime::Interpreter::default()
            .run_source(&full)
            .expect("يجب أن يعمل بناء طلبات نموذج_ذكاء دون اتصال");
        assert_eq!(out, vec!["اكتمل_اختبار_الذكاء"]);
    }

    #[test]
    fn test_transpile_to_python_comprehensive_and_execute() {
        let lisan_src = r##"
صنف حاسبة {
    دالة بناء(بداية = 0) {
        هذا.قيمة = بداية
    }
    دالة أضف_قيمة(مقدار) {
        هذا.قيمة = هذا.قيمة + مقدار
        أرجع هذا.قيمة
    }
}

دالة جمع_متغير(أساس، *أرقام) {
    دع ناتج = أساس
    لكل ر في أرقام {
        ناتج = ناتج + ر
    }
    أرجع ناتج
}

دع ح = جديد حاسبة(10)
ح.أضف_قيمة(5)
دع أعداد = [1، 2، 3، 4، 5]
دع مربعات_زوجية = [س * س لكل س في أعداد إذا س % 2 == 0]
دع مضاعف = دالة(س) => س * 2
دع شريحة = أعداد[1:4]
دع أ، ب = [7، 8]

اختر (ب) {
    حالة 8:
        دع تصنيف = "ثمانية"
    افتراضي:
        دع تصنيف = "غير_ذلك"
}

حاول {
    ارمِ "اختبار"
} التقط (سبب) {
    دع التقط_نجح = صحيح
}

دع مج = جمع_متغير(1، 2، 3، 4)
اطبع(ن"{ح.قيمة}|{طول(مربعات_زوجية)}|{مضاعف(6)}|{مجموع(شريحة)}|{أ + ب}|{تصنيف}|{مج}")
"##;

        let py = transpile_to_python(lisan_src).expect("يجب أن ينجح التحويل إلى Python 3");
        assert!(py.contains("class حاسبة:"));
        assert!(py.contains("def __init__(self, بداية=0):"));
        assert!(py.contains("def جمع_متغير(أساس, *أرقام):"));
        assert!(py.contains("[((س * س))") || py.contains("[(س * س) for س in أعداد if ((س % 2) == 0)]"));
        assert!(py.contains("len(مربعات_زوجية)"));
        assert!(py.contains("sum(شريحة)"));

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(py_cmd) = find_python_command() {
            // التحقق النحوي وتنفيذ الشيفرة المولدة
            let out = run_process_with_stdin(
                py_cmd,
                &[
                    "-c",
                    "import sys; sys.stdout.reconfigure(encoding='utf-8'); code = sys.stdin.buffer.read().decode('utf-8'); compile(code, '<gen>', 'exec'); exec(code, {})",
                ],
                &py,
            )
            .expect("تشغيل python3 للتحقق من الشيفرة المولدة");
            assert!(
                out.status.success(),
                "فشل تشغيل شيفرة Python المولدة:\nstderr: {}\ncode:\n{}",
                String::from_utf8_lossy(&out.stderr),
                py
            );
            assert_eq!(
                String::from_utf8_lossy(&out.stdout).trim(),
                "15|2|12|9|15|ثمانية|10"
            );
        }
    }

    #[test]
    fn test_transpile_to_python_import_python_and_builtins() {
        let src = r##"
دع م = استورد_بايثون("math")
دع قائمة = [3، 1، 2]
دع مفروزة = فرز(قائمة)
دع معكوسة = عكس(مفروزة)
دع مزدوج = سحاب([1، 2]، ["أ"، "ب"])
دع مرقمة = مرقم(["س"، "ص"])
اطبع(أدنى(قائمة))
اطبع(أعلى(قائمة))
اطبع(مطلق(-9))
اطبع(أي([خطأ، صحيح]))
اطبع(كل([صحيح، صحيح]))
اطبع(عدد_صحيح(م.sqrt(25)))
اطبع(طول(مزدوج) + طول(مرقمة))
"##;
        let py = transpile_to_python(src).expect("تحويل الدوال المدمجة إلى Python 3");
        assert!(py.contains("__import__(\"math\")"));
        assert!(py.contains("sorted(قائمة)"));
        assert!(py.contains("list(reversed(مفروزة))"));
        assert!(py.contains("list(zip("));
        assert!(py.contains("list(enumerate("));

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(py_cmd) = find_python_command() {
            let out = run_process_with_stdin(py_cmd, &["-"], &py).expect("تشغيل python3");
            assert!(
                out.status.success(),
                "فشل تنفيذ بايثون: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            let lines: Vec<&str> = std::str::from_utf8(&out.stdout)
                .unwrap()
                .lines()
                .collect();
            assert_eq!(lines, vec!["1", "3", "9", "True", "True", "5", "4"]);
        }
    }

    #[test]
    fn test_transpile_to_js_comprehensive_and_execute() {
        let lisan_src = r##"
صنف حيوان {
    دالة بناء(اسم) {
        هذا.اسم = اسم
    }
    دالة تحية() {
        أرجع ن"أهلاً {هذا.اسم}"
    }
}

صنف قطة يرث حيوان {
    دالة بناء(اسم) {
        هذا.اسم = اسم
        هذا.صوت = "مواء"
    }
}

دالة جمع_أرقام(أساس، *بقية) {
    دع كلي = أساس
    لكل ع في بقية {
        كلي = كلي + ع
    }
    أرجع كلي
}

دع ق = جديد قطة("لولو")
دع أرقام = [1، 2، 3، 4، 5]
دع زوجية = [س * 10 لكل س في أرقام إذا س % 2 == 0]
دع مضاعف = دالة(س) => س * 3
دع شريحة = أرقام[1:4]

دع وصف = ""
اختر (طول(زوجية)) {
    حالة 2:
        وصف = "اثنان"
    افتراضي:
        وصف = "أخرى"
}

حاول {
    ارمِ "خطأ_تجريبي"
} التقط (سبب) {
    وصف = وصف + "_التقط"
}

اطبع(ن"{ق.تحية()}|{ق.صوت}|{مضاعف(4)}|{مجموع(شريحة)}|{جمع_أرقام(10، 1، 2)}|{وصف}")
"##;
        let js = transpile_to_js(lisan_src).expect("يجب أن ينجح التحويل إلى JavaScript");
        assert!(js.contains("class حيوان"));
        assert!(js.contains("class قطة extends حيوان"));
        assert!(js.contains("new قطة(\"لولو\")"));
        assert!(js.contains(".filter("));
        assert!(js.contains(".map("));
        assert!(js.contains("switch ("));

        #[cfg(not(target_arch = "wasm32"))]
        if is_node_available() {
            let out = run_process_with_stdin("node", &[], &js)
                .expect("تشغيل node على شيفرة JS المولدة");
            assert!(
                out.status.success(),
                "فشل تشغيل شيفرة JS المولدة:\nstderr: {}\ncode:\n{}",
                String::from_utf8_lossy(&out.stderr),
                js
            );
            assert_eq!(
                String::from_utf8_lossy(&out.stdout).trim(),
                "أهلاً لولو|مواء|12|9|13|اثنان_التقط"
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_live_python_interop_builtins() {
        if find_python_command().is_none() {
            return;
        }
        // بايثون_متوفر
        let avail = try_call_interop_builtin("بايثون_متوفر", &[])
            .unwrap()
            .unwrap();
        assert_eq!(avail, Value::Bool(true));

        // بايثون_استدعِ("math", "sqrt", [16])
        let sqrt_res = try_call_interop_builtin(
            "بايثون_استدعِ",
            &[
                Value::Text("math".into()),
                Value::Text("sqrt".into()),
                Value::list(vec![Value::Number(16.0)]),
            ],
        )
        .unwrap()
        .unwrap();
        assert_eq!(sqrt_res, Value::Number(4.0));

        // دالة متداخلة بالنقطة: os.path.join
        let join_res = try_call_interop_builtin(
            "بايثون_استدعِ",
            &[
                Value::Text("os".into()),
                Value::Text("path.join".into()),
                Value::list(vec![Value::Text("مجلد".into()), Value::Text("ملف.lsn".into())]),
            ],
        )
        .unwrap()
        .unwrap();
        assert!(join_res.to_string().contains("مجلد"));
        assert!(join_res.to_string().contains("ملف.lsn"));

        // استدعاء مع مفاتيح kwargs: json.dumps({"أ": 1}, ensure_ascii=False)
        let dumps_res = try_call_interop_builtin(
            "بايثون_استدعِ",
            &[
                Value::Text("json".into()),
                Value::Text("dumps".into()),
                Value::list(vec![Value::map(vec![("أ".into(), Value::Number(1.0))])]),
                Value::map(vec![("ensure_ascii".into(), Value::Bool(false))]),
            ],
        )
        .unwrap()
        .unwrap();
        assert_eq!(dumps_res, Value::Text("{\"أ\": 1}".into()));

        // بايثون_سمة("math", "pi")
        let pi_res = try_call_interop_builtin(
            "بايثون_سمة",
            &[Value::Text("math".into()), Value::Text("pi".into())],
        )
        .unwrap()
        .unwrap();
        match pi_res {
            Value::Number(n) => assert!((n - std::f64::consts::PI).abs() < 1e-10),
            other => panic!("متوقع رقم لقيمة pi، وُجد {:?}", other),
        }

        // بايثون_نفّذ تعبير ومدخلات
        let eval_res = try_call_interop_builtin(
            "بايثون_نفّذ",
            &[
                Value::Text("x * 3 + 2".into()),
                Value::map(vec![("x".into(), Value::Number(10.0))]),
            ],
        )
        .unwrap()
        .unwrap();
        assert_eq!(eval_res, Value::Number(32.0));

        // بايثون_نفّذ كتلة ترجع متغير النتيجة
        let exec_res = try_call_interop_builtin(
            "بايثون_نفّذ",
            &[Value::Text("مجموع = sum([10, 20, 30])\nالنتيجة = مجموع".into())],
        )
        .unwrap()
        .unwrap();
        assert_eq!(exec_res, Value::Number(60.0));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_live_js_interop_builtins() {
        if !is_node_available() {
            return;
        }
        let avail = try_call_interop_builtin("جافاسكربت_متوفر", &[])
            .unwrap()
            .unwrap();
        assert_eq!(avail, Value::Bool(true));

        let joined = try_call_interop_builtin(
            "جافاسكربت_استدعِ",
            &[
                Value::Text("path".into()),
                Value::Text("join".into()),
                Value::list(vec![Value::Text("أ".into()), Value::Text("ب".into())]),
            ],
        )
        .unwrap()
        .unwrap();
        assert!(joined.to_string().contains("أ"));
        assert!(joined.to_string().contains("ب"));

        let eval_js = try_call_interop_builtin(
            "جافاسكربت_نفّذ",
            &[Value::Text("[1, 2, 3, 4].reduce((a, b) => a + b, 0)".into())],
        )
        .unwrap()
        .unwrap();
        assert_eq!(eval_js, Value::Number(10.0));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_secure_http_local_roundtrip_and_validation() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        // التحقق من رفض الروابط غير الصالحة
        let bad = try_call_interop_builtin(
            "طلب_شبكي_آمن",
            &[Value::Text("GET".into()), Value::Text("ftp://example.com".into())],
        )
        .unwrap();
        assert!(bad.is_err());

        // خادم محلي مؤقت لاختبار طلب_شبكي_آمن بالكامل دون الاعتماد على الإنترنت
        let listener = TcpListener::bind("127.0.0.1:0").expect("ربط منفذ محلي للاختبار");
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let body = r#"{"رسالة":"مرحباً من لسان"}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nX-Lisan: ok\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
            }
        });

        let url = format!("http://127.0.0.1:{}/اختبار", port);
        let res = try_call_interop_builtin(
            "طلب_شبكي_آمن",
            &[
                Value::Text("POST".into()),
                Value::Text(url),
                Value::map(vec![("X-Test".into(), Value::Text("1".into()))]),
                Value::map(vec![("تحية".into(), Value::Text("سلام".into()))]),
            ],
        )
        .unwrap()
        .expect("يجب أن ينجح الطلب المحلي");
        let _ = handle.join();

        if let Value::Map(pairs) = res {
            let b = pairs.borrow();
            let status = b.iter().find(|(k, _)| k == "حالة").map(|(_, v)| v.clone());
            let ok = b.iter().find(|(k, _)| k == "نجاح").map(|(_, v)| v.clone());
            let text = b.iter().find(|(k, _)| k == "نص").map(|(_, v)| v.to_string()).unwrap_or_default();
            assert_eq!(status, Some(Value::Number(200.0)));
            assert_eq!(ok, Some(Value::Bool(true)));
            assert!(text.contains("مرحباً من لسان"));
        } else {
            panic!("متوقع قاموس استجابة من طلب_شبكي_آمن");
        }
    }
}
