//! وقت تشغيل التزامن والمهام غير المتزامنة والتوازي الحقيقي للغة لسان (Lisan).
//!
//! مبني بالكامل باستخدام مكتبة Rust القياسية (`std`) فقط دون أي حزم خارجية،
//! مع توافق كامل مع أنظمة التشغيل الأصلية (Linux و macOS و Windows) وبيئة WebAssembly (`wasm32`).

use crate::runtime::{Interpreter, Limits, Value};

/// الحد الأقصى لعدد المهام المتوازية أو الطلبات الشبكية المتزامنة في استدعاء واحد
const MAX_PARALLEL_TASKS: usize = 16;

/// المكتبة القياسية العربية 100% للبرمجة غير المتزامنة (المهام، الوعود، القنوات، حلقة الأحداث)
pub const STDLIB_ASYNC: &str = r##"
# مكتبة التزامن والمهام غير المتزامنة في لغة لسان (عربية بالكامل)

صنف مهمة {
    دالة بناء(دالة_العمل = لاشيء) {
        هذا.القيمة = لاشيء
        هذا.الوضع = "معلقة"
        هذا.سبب_الرفض = لاشيء
        هذا.دالة_العمل = دالة_العمل
    }

    دالة أنجز(قيمة) {
        إذا (هذا.الوضع == "معلقة") {
            هذا.القيمة = قيمة
            هذا.الوضع = "منجزة"
            هذا.سبب_الرفض = لاشيء
        }
        أرجع هذا
    }

    دالة انجز(قيمة) {
        أرجع هذا.أنجز(قيمة)
    }

    دالة ارفض(سبب) {
        إذا (هذا.الوضع == "معلقة") {
            هذا.سبب_الرفض = سبب
            هذا.الوضع = "مرفوضة"
        }
        أرجع هذا
    }

    دالة شغّل() {
        إذا (هذا.الوضع == "معلقة" و هذا.دالة_العمل != لاشيء) {
            حاول {
                دع عمل = هذا.دالة_العمل
                دع ناتج = عمل()
                إذا (نوع(ناتج) == "مهمة" أو نوع(ناتج) == "وعد") {
                    ناتج = ناتج.انتظر()
                }
                هذا.أنجز(ناتج)
            } التقط (عطل) {
                هذا.ارفض(عطل)
            }
        }
        أرجع هذا
    }

    دالة شغل() {
        أرجع هذا.شغّل()
    }

    دالة انتظر() {
        إذا (هذا.الوضع == "معلقة") {
            هذا.شغّل()
        }
        إذا (هذا.الوضع == "مرفوضة") {
            ارمِ نص(هذا.سبب_الرفض)
        }
        إذا (هذا.الوضع == "معلقة") {
            ارمِ "المهمة لا تزال معلقة ولم تُنجز بعد"
        }
        أرجع هذا.القيمة
    }

    دالة ثم(متابعة) {
        دع المهمة_الحالية = هذا
        أرجع جديد مهمة(دالة() {
            دع قيمة_سابقة = المهمة_الحالية.انتظر()
            أرجع متابعة(قيمة_سابقة)
        })
    }

    دالة عند_الخطأ(معالج) {
        دع المهمة_الحالية = هذا
        أرجع جديد مهمة(دالة() {
            حاول {
                أرجع المهمة_الحالية.انتظر()
            } التقط (عطل) {
                أرجع معالج(عطل)
            }
        })
    }

    دالة عند_الخطا(معالج) {
        أرجع هذا.عند_الخطأ(معالج)
    }

    دالة منجزة() {
        أرجع هذا.الوضع == "منجزة"
    }

    دالة مرفوضة() {
        أرجع هذا.الوضع == "مرفوضة"
    }

    دالة معلقة() {
        أرجع هذا.الوضع == "معلقة"
    }
}

صنف وعد يرث مهمة {
}

صنف قناة {
    دالة بناء() {
        هذا.العناصر = []
    }

    دالة أرسل(رسالة) {
        هذا.العناصر.أضف(رسالة)
        أرجع هذا
    }

    دالة ارسل(رسالة) {
        أرجع هذا.أرسل(رسالة)
    }

    دالة استقبل() {
        إذا (طول(هذا.العناصر) == 0) {
            أرجع لاشيء
        }
        أرجع هذا.العناصر.احذف(0)
    }

    دالة معاينة() {
        إذا (طول(هذا.العناصر) == 0) {
            أرجع لاشيء
        }
        أرجع هذا.العناصر[0]
    }

    دالة حجم() {
        أرجع طول(هذا.العناصر)
    }

    دالة فارغة() {
        أرجع طول(هذا.العناصر) == 0
    }
}

صنف حلقة_أحداث {
    دالة بناء() {
        هذا.المهام = []
        هذا.النتائج = []
    }

    دالة أضف_مهمة(م) {
        دع عنصر = م
        إذا (نوع(م) == "دالة") {
            عنصر = جديد مهمة(م)
        }
        هذا.المهام.أضف(عنصر)
        أرجع عنصر
    }

    دالة اضف_مهمة(م) {
        أرجع هذا.أضف_مهمة(م)
    }

    دالة شغّل_الكل() {
        دع مخرجات = []
        طالما (طول(هذا.المهام) > 0) {
            دع م = هذا.المهام.احذف(0)
            م.شغّل()
            إذا (م.الوضع == "منجزة") {
                مخرجات.أضف(م.القيمة)
            } وإلا {
                مخرجات.أضف(لاشيء)
            }
        }
        هذا.النتائج = مخرجات
        أرجع مخرجات
    }

    دالة شغل_الكل() {
        أرجع هذا.شغّل_الكل()
    }

    دالة انتظر_الكل(قائمة_مهام) {
        دع مخرجات = []
        لكل عنصر في قائمة_مهام {
            إذا (نوع(عنصر) == "مهمة" أو نوع(عنصر) == "وعد") {
                مخرجات.أضف(عنصر.انتظر())
            } وإلا {
                إذا (نوع(عنصر) == "دالة") {
                    دع م = جديد مهمة(عنصر)
                    مخرجات.أضف(م.انتظر())
                } وإلا {
                    مخرجات.أضف(عنصر)
                }
            }
        }
        أرجع مخرجات
    }

    دالة انتظر_أول(قائمة_مهام) {
        إذا (طول(قائمة_مهام) == 0) {
            ارمِ "قائمة المهام فارغة في انتظر_أول"
        }
        لكل عنصر في قائمة_مهام {
            إذا ((نوع(عنصر) == "مهمة" أو نوع(عنصر) == "وعد") و عنصر.الوضع == "منجزة") {
                أرجع عنصر.القيمة
            }
        }
        دع آخر_سبب = "لم تنجح أي مهمة في انتظر_أول"
        لكل عنصر في قائمة_مهام {
            إذا (نوع(عنصر) == "مهمة" أو نوع(عنصر) == "وعد") {
                عنصر.شغّل()
                إذا (عنصر.الوضع == "منجزة") {
                    أرجع عنصر.القيمة
                }
                إذا (عنصر.الوضع == "مرفوضة") {
                    آخر_سبب = نص(عنصر.سبب_الرفض)
                }
            } وإلا {
                إذا (نوع(عنصر) == "دالة") {
                    دع م = جديد مهمة(عنصر)
                    م.شغّل()
                    إذا (م.الوضع == "منجزة") {
                        أرجع م.القيمة
                    }
                    آخر_سبب = نص(م.سبب_الرفض)
                } وإلا {
                    أرجع عنصر
                }
            }
        }
        ارمِ آخر_سبب
    }

    دالة انتظر_اول(قائمة_مهام) {
        أرجع هذا.انتظر_أول(قائمة_مهام)
    }

    دالة عدد_المهام() {
        أرجع طول(هذا.المهام)
    }
}

دالة مهمة_منجزة(قيمة) {
    دع م = جديد مهمة()
    م.أنجز(قيمة)
    أرجع م
}

دالة مهمة_مرفوضة(سبب) {
    دع م = جديد مهمة()
    م.ارفض(سبب)
    أرجع م
}

دالة انتظر_الكل(قائمة_مهام) {
    دع حلقة = جديد حلقة_أحداث()
    أرجع حلقة.انتظر_الكل(قائمة_مهام)
}

دالة انتظر_أول(قائمة_مهام) {
    دع حلقة = جديد حلقة_أحداث()
    أرجع حلقة.انتظر_أول(قائمة_مهام)
}

دالة انتظر_اول(قائمة_مهام) {
    أرجع انتظر_أول(قائمة_مهام)
}
"##;

/// جميع أسماء الدوال المدمجة في وحدة التزامن والتوازي (عربية خالصة)
pub const ALL_ASYNC_BUILTINS: &[&str] = &[
    "انتظر_قيمة",
    "انتظر_قيمه",
    "تشغيل_متوازٍ",
    "تشغيل_متوازي",
    "طلب_شبكي_متوازي",
    "طلب_شبكي_متوازٍ",
];

/// التحقق مما إذا كان الاسم يشير إلى دالة مدمجة في وحدة التزامن والتوازي
pub fn is_async_builtin(name: &str) -> bool {
    ALL_ASYNC_BUILTINS.contains(&name)
}

fn require_async_args(name: &str, args: &[Value], expected: usize) -> Result<(), String> {
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

/// تمثيل آمن للإرسال بين خيوط نظام التشغيل (`Send`) لقيم لسان الأساسية
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
enum SendValue {
    Number(f64),
    Text(String),
    Bool(bool),
    Null,
    List(Vec<SendValue>),
    Map(Vec<(String, SendValue)>),
}

#[cfg(not(target_arch = "wasm32"))]
impl SendValue {
    fn from_value(v: &Value) -> Self {
        match v {
            Value::Number(n) => SendValue::Number(*n),
            Value::Text(s) => SendValue::Text(s.clone()),
            Value::Bool(b) => SendValue::Bool(*b),
            Value::Null => SendValue::Null,
            Value::List(items) => {
                let converted = items.borrow().iter().map(Self::from_value).collect();
                SendValue::List(converted)
            }
            Value::Tuple(items) => {
                let converted = items.iter().map(Self::from_value).collect();
                SendValue::List(converted)
            }
            Value::Set(items) => {
                let converted = items.borrow().iter().map(Self::from_value).collect();
                SendValue::List(converted)
            }
            Value::Map(pairs) => {
                let converted = pairs
                    .borrow()
                    .iter()
                    .map(|(k, val)| (k.clone(), Self::from_value(val)))
                    .collect();
                SendValue::Map(converted)
            }
            other => SendValue::Text(other.to_string()),
        }
    }

    fn into_value(self) -> Value {
        match self {
            SendValue::Number(n) => Value::Number(n),
            SendValue::Text(s) => Value::Text(s),
            SendValue::Bool(b) => Value::Bool(b),
            SendValue::Null => Value::Null,
            SendValue::List(items) => {
                Value::list(items.into_iter().map(Self::into_value).collect())
            }
            SendValue::Map(pairs) => Value::map(
                pairs
                    .into_iter()
                    .map(|(k, v)| (k, v.into_value()))
                    .collect(),
            ),
        }
    }
}

/// تنفيذ دالة مدمجة من وحدة التزامن والتوازي
pub fn eval_async_builtin(name: &str, args: Vec<Value>) -> Result<Value, String> {
    match name {
        "انتظر_قيمة" | "انتظر_قيمه" => {
            require_async_args(name, &args, 1)?;
            eval_await_value(args.into_iter().next().unwrap())
        }
        "تشغيل_متوازٍ" | "تشغيل_متوازي" => {
            require_async_args(name, &args, 1)?;
            eval_parallel_run(name, &args[0])
        }
        "طلب_شبكي_متوازي" | "طلب_شبكي_متوازٍ" => {
            require_async_args(name, &args, 1)?;
            eval_parallel_http(name, &args[0])
        }
        _ => Err(format!("دالة التزامن '{}' غير معروفة", name)),
    }
}

/// فك قيمة `مهمة` أو `وعد` إن كانت منجزة (أو إرجاع سبب الرفض إن كانت مرفوضة)،
/// وإرجاع القيمة مباشرة لأي نوع آخر (تُستخدم في تفكيك تعبير `انتظر`).
fn eval_await_value(val: Value) -> Result<Value, String> {
    if let Value::Instance {
        ref class_name,
        ref fields,
    } = val
    {
        if class_name == "مهمة" || class_name == "وعد" {
            let borrowed = fields.borrow();
            let status = borrowed
                .iter()
                .find(|(k, _)| k == "الوضع")
                .and_then(|(_, v)| match v {
                    Value::Text(s) => Some(s.as_str()),
                    _ => None,
                });
            if status == Some("مرفوضة") {
                let reason = borrowed
                    .iter()
                    .find(|(k, _)| k == "سبب_الرفض")
                    .map(|(_, v)| v.to_string())
                    .unwrap_or_else(|| "المهمة مرفوضة".to_string());
                return Err(reason);
            }
            if let Some((_, inner)) = borrowed.iter().find(|(k, _)| k == "القيمة") {
                return Ok(inner.clone());
            }
            return Ok(Value::Null);
        }
    }
    Ok(val)
}

fn build_parallel_task_result(outcome: Result<Vec<String>, String>) -> Value {
    match outcome {
        Ok(outputs) => Value::map(vec![
            ("نجاح".to_string(), Value::Bool(true)),
            (
                "مخرجات".to_string(),
                Value::list(outputs.into_iter().map(Value::Text).collect()),
            ),
            ("خطأ".to_string(), Value::Null),
        ]),
        Err(err) => Value::map(vec![
            ("نجاح".to_string(), Value::Bool(false)),
            ("مخرجات".to_string(), Value::list(Vec::new())),
            ("خطأ".to_string(), Value::Text(err)),
        ]),
    }
}

fn eval_parallel_run(name: &str, arg: &Value) -> Result<Value, String> {
    let snippets: Vec<String> = match arg {
        Value::List(items) => {
            let borrowed = items.borrow();
            if borrowed.len() > MAX_PARALLEL_TASKS {
                return Err(format!(
                    "الدالة '{}' تجاوزت الحد الأقصى للمهام المتوازية ({} مهمة كحد أقصى)",
                    name, MAX_PARALLEL_TASKS
                ));
            }
            let mut out = Vec::with_capacity(borrowed.len());
            for (idx, item) in borrowed.iter().enumerate() {
                match item {
                    Value::Text(code) => out.push(code.clone()),
                    _ => {
                        return Err(format!(
                            "العنصر رقم {} في '{}' يجب أن يكون نص شيفرة لسان",
                            idx, name
                        ))
                    }
                }
            }
            out
        }
        _ => {
            return Err(format!(
                "الدالة '{}' تتطلب قائمة نصوص شيفرة لسان",
                name
            ))
        }
    };

    run_snippets_parallel(snippets)
}

#[cfg(not(target_arch = "wasm32"))]
fn run_snippets_parallel(snippets: Vec<String>) -> Result<Value, String> {
    let mut handles = Vec::with_capacity(snippets.len());
    for code in snippets {
        let handle = std::thread::spawn(move || {
            let mut interp = Interpreter::with_limits(Limits::sandbox());
            interp.run_source(&code)
        });
        handles.push(handle);
    }

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        let outcome = match handle.join() {
            Ok(res) => res,
            Err(_) => Err("توقف خيط التنفيذ المتوازي بشكل غير متوقع".to_string()),
        };
        results.push(build_parallel_task_result(outcome));
    }
    Ok(Value::list(results))
}

#[cfg(target_arch = "wasm32")]
fn run_snippets_parallel(snippets: Vec<String>) -> Result<Value, String> {
    let mut results = Vec::with_capacity(snippets.len());
    for code in snippets {
        let mut interp = Interpreter::with_limits(Limits::sandbox());
        let outcome = interp.run_source(&code);
        results.push(build_parallel_task_result(outcome));
    }
    Ok(Value::list(results))
}

#[cfg(not(target_arch = "wasm32"))]
struct HttpRequestSpec {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_http_specs(name: &str, arg: &Value) -> Result<Vec<HttpRequestSpec>, String> {
    let items = match arg {
        Value::List(list) => list.borrow(),
        _ => {
            return Err(format!(
                "الدالة '{}' تتطلب قائمة روابط نصية أو قواميس طلبات",
                name
            ))
        }
    };
    if items.len() > MAX_PARALLEL_TASKS {
        return Err(format!(
            "الدالة '{}' تجاوزت الحد الأقصى للطلبات المتوازية ({} طلبًا كحد أقصى)",
            name, MAX_PARALLEL_TASKS
        ));
    }

    let mut specs = Vec::with_capacity(items.len());
    for (idx, item) in items.iter().enumerate() {
        match item {
            Value::Text(url) => specs.push(HttpRequestSpec {
                method: "GET".to_string(),
                url: url.clone(),
                headers: Vec::new(),
                body: None,
            }),
            Value::Map(m) => {
                let map_ref = m.borrow();
                let url = map_ref
                    .iter()
                    .find(|(k, _)| k == "رابط")
                    .and_then(|(_, v)| match v {
                        Value::Text(s) => Some(s.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        format!("الطلب رقم {} في '{}' يفتقد حقل 'رابط' النصي", idx, name)
                    })?;
                let method = map_ref
                    .iter()
                    .find(|(k, _)| k == "طريقة")
                    .and_then(|(_, v)| match v {
                        Value::Text(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "GET".to_string());
                let mut headers = Vec::new();
                if let Some((_, Value::Map(hm))) =
                    map_ref.iter().find(|(k, _)| k == "ترويسات")
                {
                    for (hk, hv) in hm.borrow().iter() {
                        headers.push((hk.clone(), hv.to_string()));
                    }
                }
                let body = map_ref
                    .iter()
                    .find(|(k, _)| k == "جسم")
                    .and_then(|(_, v)| match v {
                        Value::Null => None,
                        Value::Text(s) => Some(s.clone()),
                        other => Some(other.to_string()),
                    });
                specs.push(HttpRequestSpec {
                    method,
                    url,
                    headers,
                    body,
                });
            }
            _ => {
                return Err(format!(
                    "العنصر رقم {} في '{}' يجب أن يكون رابطًا نصيًا أو قاموس طلب",
                    idx, name
                ))
            }
        }
    }
    Ok(specs)
}

#[cfg(not(target_arch = "wasm32"))]
fn eval_parallel_http(name: &str, arg: &Value) -> Result<Value, String> {
    let specs = parse_http_specs(name, arg)?;
    let mut handles = Vec::with_capacity(specs.len());

    for spec in specs {
        let handle = std::thread::spawn(move || -> (String, Result<SendValue, String>) {
            let url_copy = spec.url.clone();
            let mut call_args = vec![Value::Text(spec.method), Value::Text(spec.url)];
            if !spec.headers.is_empty() || spec.body.is_some() {
                let header_pairs = spec
                    .headers
                    .into_iter()
                    .map(|(k, v)| (k, Value::Text(v)))
                    .collect();
                call_args.push(Value::map(header_pairs));
            }
            if let Some(body_str) = spec.body {
                call_args.push(Value::Text(body_str));
            }
            let outcome = match crate::interop::try_call_interop_builtin("طلب_شبكي_آمن", &call_args)
            {
                Some(Ok(val)) => Ok(SendValue::from_value(&val)),
                Some(Err(err)) => Err(err),
                None => Err("الدالة المدمجة 'طلب_شبكي_آمن' غير متاحة".to_string()),
            };
            (url_copy, outcome)
        });
        handles.push(handle);
    }

    let mut responses = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.join() {
            Ok((_url, Ok(sv))) => responses.push(sv.into_value()),
            Ok((url, Err(err))) => {
                responses.push(Value::map(vec![
                    ("نجاح".to_string(), Value::Bool(false)),
                    ("رابط".to_string(), Value::Text(url)),
                    ("خطأ".to_string(), Value::Text(err)),
                ]));
            }
            Err(_) => {
                responses.push(Value::map(vec![
                    ("نجاح".to_string(), Value::Bool(false)),
                    (
                        "خطأ".to_string(),
                        Value::Text("توقف خيط الطلب الشبكي بشكل غير متوقع".to_string()),
                    ),
                ]));
            }
        }
    }
    Ok(Value::list(responses))
}

#[cfg(target_arch = "wasm32")]
fn eval_parallel_http(_name: &str, _arg: &Value) -> Result<Value, String> {
    Err("طلب_شبكي_متوازي غير مدعوم مباشرة في بيئة WebAssembly".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_with_async_stdlib(user_code: &str) -> Result<Vec<String>, String> {
        let full = format!("{}\n{}", STDLIB_ASYNC, user_code);
        let mut interp = Interpreter::with_limits(Limits::sandbox());
        interp.run_source(&full)
    }

    #[test]
    fn test_stdlib_async_parses_cleanly() {
        let tokens = crate::lexer::tokenize(STDLIB_ASYNC).expect("تعذّر تحليل رموز STDLIB_ASYNC");
        let stmts = crate::parser::parse(tokens).expect("تعذّر تحليل بنية STDLIB_ASYNC");
        assert!(!stmts.is_empty());
    }

    #[test]
    fn test_task_promise_chaining_and_error_recovery() {
        let out = run_with_async_stdlib(
            r##"
            دع م1 = جديد مهمة(دالة() { أرجع 21 })
            دع م2 = م1.ثم(دالة(ق) => ق * 2)
            اطبع(م2.انتظر())
            اطبع(م1.منجزة())

            دع و1 = جديد وعد(دالة() { ارمِ "فشل مؤقت" })
            دع و2 = و1.عند_الخطأ(دالة(ع) => "تم التعافي: " + نص(ع))
            اطبع(و2.انتظر())

            دع جاهزة = مهمة_منجزة(99)
            اطبع(جاهزة.انتظر())

            دع مرفوضة = مهمة_مرفوضة("انقطاع")
            حاول {
                مرفوضة.انتظر()
                اطبع("لم يُرمَ")
            } التقط (رسالة) {
                اطبع(رسالة)
            }
            "##,
        )
        .expect("فشل تنفيذ اختبار مهمة/وعد");

        assert_eq!(out, vec!["42", "صحيح", "تم التعافي: فشل مؤقت", "99", "انقطاع"]);
    }

    #[test]
    fn test_channel_fifo_operations() {
        let out = run_with_async_stdlib(
            r##"
            دع ق = جديد قناة()
            اطبع(ق.فارغة())
            ق.أرسل("أول").أرسل("ثاني").أرسل("ثالث")
            اطبع(ق.حجم())
            اطبع(ق.معاينة())
            اطبع(ق.استقبل())
            اطبع(ق.استقبل())
            اطبع(ق.حجم())
            اطبع(ق.استقبل())
            اطبع(ق.فارغة())
            اطبع(ق.استقبل())
            "##,
        )
        .expect("فشل تنفيذ اختبار قناة");

        assert_eq!(
            out,
            vec!["صحيح", "3", "أول", "أول", "ثاني", "1", "ثالث", "صحيح", "لاشيء"]
        );
    }

    #[test]
    fn test_event_loop_and_await_all_and_first() {
        let out = run_with_async_stdlib(
            r##"
            دع حلقة = جديد حلقة_أحداث()
            حلقة.أضف_مهمة(جديد مهمة(دالة() => 10))
            حلقة.أضف_مهمة(دالة() => 20)
            حلقة.أضف_مهمة(مهمة_منجزة(30))
            اطبع(حلقة.عدد_المهام())
            دع نواتج = حلقة.شغّل_الكل()
            اطبع(نواتج)
            اطبع(حلقة.عدد_المهام())

            دع الكل = انتظر_الكل([
                جديد مهمة(دالة() => "أ")،
                مهمة_منجزة("ب")،
                جديد وعد(دالة() => "ج")
            ])
            اطبع(الكل)

            دع أول_ناجحة = حلقة.انتظر_أول([
                مهمة_مرفوضة("تجاهل")،
                جديد مهمة(دالة() => "الفائز")،
                جديد مهمة(دالة() => "متأخر")
            ])
            اطبع(أول_ناجحة)
            "##,
        )
        .expect("فشل تنفيذ اختبار حلقة_أحداث");

        assert_eq!(out[0], "3");
        assert_eq!(out[1], "[10، 20، 30]");
        assert_eq!(out[2], "0");
        assert_eq!(out[3], "[\"أ\"، \"ب\"، \"ج\"]");
        assert_eq!(out[4], "الفائز");
    }

    #[test]
    fn test_await_value_builtin_unwraps_task_and_passes_plain() {
        assert!(is_async_builtin("انتظر_قيمة"));
        assert!(is_async_builtin("تشغيل_متوازٍ"));
        assert!(is_async_builtin("تشغيل_متوازي"));
        assert!(is_async_builtin("طلب_شبكي_متوازي"));

        // قيمة عادية تُعاد كما هي
        let plain = eval_async_builtin("انتظر_قيمة", vec![Value::Number(7.5)]).unwrap();
        assert_eq!(plain, Value::Number(7.5));

        // كائن مهمة منجزة يُفك إلى قيمته
        let ready_task = Value::Instance {
            class_name: "مهمة".to_string(),
            fields: std::rc::Rc::new(std::cell::RefCell::new(vec![
                ("الوضع".to_string(), Value::Text("منجزة".to_string())),
                ("القيمة".to_string(), Value::Text("تم".to_string())),
                ("سبب_الرفض".to_string(), Value::Null),
            ])),
        };
        let unwrapped = eval_async_builtin("انتظر_قيمة", vec![ready_task]).unwrap();
        assert_eq!(unwrapped, Value::Text("تم".to_string()));

        // كائن وعد مرفوض يُرجع خطأً
        let rejected_promise = Value::Instance {
            class_name: "وعد".to_string(),
            fields: std::rc::Rc::new(std::cell::RefCell::new(vec![
                ("الوضع".to_string(), Value::Text("مرفوضة".to_string())),
                ("القيمة".to_string(), Value::Null),
                ("سبب_الرفض".to_string(), Value::Text("خطأ شبكة".to_string())),
            ])),
        };
        let err = eval_async_builtin("انتظر_قيمة", vec![rejected_promise]).unwrap_err();
        assert!(err.contains("خطأ شبكة"));
    }

    #[test]
    fn test_parallel_run_multithreaded_order_and_isolation() {
        let snippets = Value::list(vec![
            Value::Text("دع س = 6 * 7\nاطبع(س)".to_string()),
            Value::Text("دع مجموع = 0\nلكل ع في مدى(1، 6) { مجموع = مجموع + ع }\nاطبع(مجموع)".to_string()),
            Value::Text("ارمِ \"عطل مقصود\"".to_string()),
            Value::Text("اطبع(\"مهمة رابعة\")".to_string()),
        ]);

        let result = eval_async_builtin("تشغيل_متوازٍ", vec![snippets])
            .expect("فشل تشغيل المهام المتوازية");

        let list = match result {
            Value::List(l) => l.borrow().clone(),
            other => panic!("متوقع قائمة نتائج، وصل: {:?}", other),
        };
        assert_eq!(list.len(), 4);

        let get_field = |val: &Value, key: &str| -> Value {
            match val {
                Value::Map(m) => m
                    .borrow()
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone())
                    .unwrap_or(Value::Null),
                _ => panic!("متوقع قاموس نتيجة"),
            }
        };

        assert_eq!(get_field(&list[0], "نجاح"), Value::Bool(true));
        assert_eq!(
            get_field(&list[0], "مخرجات"),
            Value::list(vec![Value::Text("42".to_string())])
        );
        assert_eq!(get_field(&list[0], "خطأ"), Value::Null);

        assert_eq!(get_field(&list[1], "نجاح"), Value::Bool(true));
        assert_eq!(
            get_field(&list[1], "مخرجات"),
            Value::list(vec![Value::Text("15".to_string())])
        );

        assert_eq!(get_field(&list[2], "نجاح"), Value::Bool(false));
        assert_eq!(get_field(&list[2], "مخرجات"), Value::list(vec![]));
        match get_field(&list[2], "خطأ") {
            Value::Text(msg) => assert!(msg.contains("عطل مقصود")),
            other => panic!("متوقع رسالة خطأ نصية، وصل: {:?}", other),
        }

        assert_eq!(get_field(&list[3], "نجاح"), Value::Bool(true));
        assert_eq!(
            get_field(&list[3], "مخرجات"),
            Value::list(vec![Value::Text("مهمة رابعة".to_string())])
        );
    }

    #[test]
    fn test_parallel_run_rejects_excessive_tasks() {
        let too_many: Vec<Value> = (0..=MAX_PARALLEL_TASKS)
            .map(|i| Value::Text(format!("اطبع({})", i)))
            .collect();
        let err = eval_async_builtin("تشغيل_متوازي", vec![Value::list(too_many)]).unwrap_err();
        assert!(err.contains("16"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_parallel_http_local_server() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("تعذّر ربط منفذ محلي للاختبار");
        let addr = listener.local_addr().unwrap();

        let server_thread = std::thread::spawn(move || {
            for _ in 0..2 {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = [0u8; 1024];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let body = if req.contains("GET /a ") {
                        "{\"مفتاح\":\"أ\"}"
                    } else {
                        "{\"مفتاح\":\"ب\"}"
                    };
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                }
            }
        });

        let urls = Value::list(vec![
            Value::Text(format!("http://{}/a", addr)),
            Value::Text(format!("http://{}/b", addr)),
        ]);

        let result = eval_async_builtin("طلب_شبكي_متوازي", vec![urls])
            .expect("فشل تنفيذ طلب_شبكي_متوازي");
        let _ = server_thread.join();

        let list = match result {
            Value::List(l) => l.borrow().clone(),
            other => panic!("متوقع قائمة استجابات، وصل: {:?}", other),
        };
        assert_eq!(list.len(), 2);
        assert!(matches!(list[0], Value::Map(_)));
        assert!(matches!(list[1], Value::Map(_)));
    }
}
