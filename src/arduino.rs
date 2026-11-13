//! مترجم ومحاكي الأردوينو والمتحكمات الدقيقة (Arduino / ESP32 / STM32) للغة لسان.
use crate::ast::{ClassDef, Expr, Function, Op, Stmt};
use std::path::{Path, PathBuf};

/// المكتبة القياسية المدمجة للأردوينو والمتحكمات الدقيقة بلغة لسان
pub const STDLIB_ARDUINO: &str = r#"
# مكتبة الأردوينو والمتحكمات الدقيقة القياسية في لغة لسان (Arduino / ESP32 / STM32)

دع مخرج = "OUTPUT"
دع مدخل = "INPUT"
دع مدخل_سحب = "INPUT_PULLUP"
دع عالي = 1
دع منخفض = 0

صنف محرك_سيرفو {
    دالة بناء(منفذ) {
        هذا.منفذ = منفذ
        هذا.الزاوية = 90
        وضع_المنفذ(منفذ، "OUTPUT")
    }

    دالة حرك(زاوية) {
        إذا (زاوية < 0) { زاوية = 0 }
        إذا (زاوية > 180) { زاوية = 180 }
        هذا.الزاوية = زاوية
        اكتب_تماثلي(هذا.منفذ، زاوية)
        أرجع هذا.الزاوية
    }

    دالة اقرأ_زاوية() {
        أرجع هذا.الزاوية
    }
}

صنف حساس_مسافة {
    دالة بناء(منفذ_إرسال، منفذ_استقبال) {
        هذا.إرسال = منفذ_إرسال
        هذا.استقبال = منفذ_استقبال
        هذا.آخر_قراءة = 25
        وضع_المنفذ(منفذ_إرسال، "OUTPUT")
        وضع_المنفذ(منفذ_استقبال، "INPUT")
    }

    دالة اقرأ_سم() {
        اكتب_رقمي(هذا.إرسال، 0)
        انتظر_ميكرو(2)
        اكتب_رقمي(هذا.إرسال، 1)
        انتظر_ميكرو(10)
        اكتب_رقمي(هذا.إرسال، 0)
        أرجع هذا.آخر_قراءة
    }

    دالة ضبط_محاكاة(مسافة_سم) {
        هذا.آخر_قراءة = مسافة_سم
        أرجع مسافة_سم
    }

    دالة هل_قريب(حد_سم) {
        أرجع هذا.اقرأ_سم() <= حد_سم
    }
}

صنف حساس_حرارة {
    دالة بناء(منفذ، نوع_الحساس) {
        هذا.منفذ = منفذ
        هذا.نوع_الحساس = نوع_الحساس
        هذا.حرارة = 26.5
        هذا.رطوبة = 55.0
        وضع_المنفذ(منفذ، "INPUT")
    }

    دالة اقرأ_حرارة() {
        أرجع هذا.حرارة
    }

    دالة اقرأ_رطوبة() {
        أرجع هذا.رطوبة
    }

    دالة ضبط_محاكاة(حرارة_جديدة، رطوبة_جديدة) {
        هذا.حرارة = حرارة_جديدة
        هذا.رطوبة = رطوبة_جديدة
    }
}

صنف شاشة_كريستال {
    دالة بناء(أعمدة، صفوف) {
        هذا.أعمدة = أعمدة
        هذا.صفوف = صفوف
        هذا.سطر0 = ""
        هذا.سطر1 = ""
        هذا.إضاءة_خلفية = صحيح
    }

    دالة امسح() {
        هذا.سطر0 = ""
        هذا.سطر1 = ""
        أرجع صحيح
    }

    دالة اطبع_في(عمود، صف، نص_الشاشة) {
        إذا (صف == 0) {
            هذا.سطر0 = نص(نص_الشاشة)
        } وإلا {
            هذا.سطر1 = نص(نص_الشاشة)
        }
        أرجع نص_الشاشة
    }

    دالة محتوى_السطر(صف) {
        إذا (صف == 0) {
            أرجع هذا.سطر0
        }
        أرجع هذا.سطر1
    }
}

صنف واي_فاي_متحكم {
    دالة بناء() {
        هذا.الشبكة = ""
        هذا.متصل = خطأ
        هذا.عنوان = "0.0.0.0"
    }

    دالة اتصل(اسم_الشبكة، كلمة_المرور) {
        هذا.الشبكة = اسم_الشبكة
        هذا.متصل = صحيح
        هذا.عنوان = "192.168.1.105"
        أرجع صحيح
    }

    دالة هل_متصل() {
        أرجع هذا.متصل
    }

    دالة عنوان_ip() {
        أرجع هذا.عنوان
    }
}

صنف بلوتوث_متحكم {
    دالة بناء(اسم_الجهاز) {
        هذا.اسم_الجهاز = اسم_الجهاز
        هذا.نشط = صحيح
        هذا.آخر_رسالة = ""
    }

    دالة أرسل(بيانات) {
        هذا.آخر_رسالة = نص(بيانات)
        أرجع صحيح
    }

    دالة استقبل() {
        أرجع هذا.آخر_رسالة
    }
}

صنف شريط_إضاءة {
    دالة بناء(منفذ، عدد_المصابيح) {
        هذا.منفذ = منفذ
        هذا.عدد = عدد_المصابيح
        وضع_المنفذ(منفذ، "OUTPUT")
    }

    دالة لون_مصباح(فهرس، أحمر، أخضر، أزرق) {
        أرجع [فهرس، أحمر، أخضر، أزرق]
    }

    دالة اعرض() {
        أرجع صحيح
    }
}
"#;

/// تحويل شيفرة لسان إلى كود C++ قياسي للأردوينو والمتحكمات الدقيقة (.ino)
pub fn transpile_arduino(source: &str) -> Result<String, String> {
    let tokens = crate::lexer::tokenize(source)?;
    let statements = crate::parser::parse(tokens)?;
    transpile_statements_to_ino(&statements, source)
}

/// بناء وحفظ ملف أردوينو (.ino) من ملف لسان (.lsn)
pub fn build_arduino_file(input: &Path, output: Option<&Path>) -> Result<PathBuf, String> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| format!("تعذرت قراءة ملف لسان '{}': {}", input.display(), e))?;
    let ino_code = transpile_arduino(&source)?;

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("sketch");
            PathBuf::from(format!("{}.ino", stem))
        }
    };

    std::fs::write(&out_path, ino_code)
        .map_err(|e| format!("تعذر حفظ ملف الأردوينو '{}': {}", out_path.display(), e))?;
    Ok(out_path)
}

fn transpile_statements_to_ino(statements: &[Stmt], raw_source: &str) -> Result<String, String> {
    let mut out = String::new();
    out.push_str("// ==================================================================\n");
    out.push_str("// تم توليد كود الأردوينو والمتحكمات تلقائياً بواسطة لغة لسان (Lisan)\n");
    out.push_str("// اللوحات المدعومة: Arduino Uno / Nano / Mega / ESP32 / ESP8266 / STM32\n");
    out.push_str("// ==================================================================\n\n");
    out.push_str("#include <Arduino.h>\n");

    if raw_source.contains("محرك_سيرفو") {
        out.push_str("#include <Servo.h>\n");
    }
    if raw_source.contains("شاشة_كريستال") {
        out.push_str("#include <Wire.h>\n#include <LiquidCrystal_I2C.h>\n");
    }
    if raw_source.contains("حساس_حرارة") {
        out.push_str("#include <DHT.h>\n");
    }
    if raw_source.contains("واي_فاي") {
        out.push_str("#if defined(ESP32)\n#include <WiFi.h>\n#elif defined(ESP8266)\n#include <ESP8266WiFi.h>\n#endif\n");
    }
    out.push('\n');

    let mut globals = Vec::new();
    let mut functions = Vec::new();
    let mut classes = Vec::new();
    let mut has_setup = false;
    let mut has_loop = false;

    for stmt in statements {
        match stmt.unwrap_spanned() {
            Stmt::Import(_) => {}
            Stmt::Function(func) => {
                if func.name == "إعداد" || func.name == "اعداد" || func.name == "setup" {
                    has_setup = true;
                }
                if func.name == "تكرار" || func.name == "حلقة" || func.name == "loop" {
                    has_loop = true;
                }
                functions.push(func);
            }
            Stmt::Class(cls) => {
                classes.push(cls);
            }
            other => {
                globals.push(other);
            }
        }
    }

    // ترجمة الأصناف إلى أصناف C++
    for cls in &classes {
        out.push_str(&emit_cpp_class(cls));
        out.push('\n');
    }

    // المتغيرات العامة
    let mut top_level_exec = Vec::new();
    for g in &globals {
        match g {
            Stmt::Let(name, expr) => {
                let c_name = sanitize_ident(name);
                let c_type = infer_cpp_type(expr);
                let c_val = emit_cpp_expr(expr);
                out.push_str(&format!("{} {} = {};\n", c_type, c_name, c_val));
            }
            other => {
                top_level_exec.push(other);
            }
        }
    }
    if !globals.is_empty() {
        out.push('\n');
    }

    // الدوال
    for func in &functions {
        out.push_str(&emit_cpp_function(func));
        out.push('\n');
    }

    // إذا لم يُعرّف المستخدم دالة إعداد() صراحة، ننشئ setup() من الجمل العليا
    if !has_setup {
        out.push_str("void setup() {\n");
        out.push_str("  Serial.begin(9600);\n");
        for stmt in &top_level_exec {
            out.push_str(&emit_cpp_stmt(stmt, 1));
        }
        out.push_str("}\n\n");
    }

    if !has_loop {
        out.push_str("void loop() {\n  // الحلقة الرئيسية للمتحكم\n}\n");
    }

    Ok(out)
}

fn emit_cpp_class(cls: &ClassDef) -> String {
    let mut s = String::new();
    let cname = sanitize_ident(&cls.name);
    if let Some(parent) = &cls.parent {
        s.push_str(&format!("class {} : public {} {{\npublic:\n", cname, sanitize_ident(parent)));
    } else {
        s.push_str(&format!("class {} {{\npublic:\n", cname));
    }
    for m in &cls.methods {
        if m.name == "بناء" || m.name == "تهيئة" {
            let params: Vec<String> = m.params.iter().map(|p| format!("auto {}", sanitize_ident(p))).collect();
            s.push_str(&format!("  {}({}) {{\n", cname, params.join(", ")));
            for stmt in &m.body {
                s.push_str(&emit_cpp_stmt(stmt, 2));
            }
            s.push_str("  }\n");
        } else {
            let ret_type = if body_has_return(&m.body) { "auto" } else { "void" };
            let params: Vec<String> = m.params.iter().map(|p| format!("auto {}", sanitize_ident(p))).collect();
            s.push_str(&format!("  {} {}({}) {{\n", ret_type, sanitize_ident(&m.name), params.join(", ")));
            for stmt in &m.body {
                s.push_str(&emit_cpp_stmt(stmt, 2));
            }
            s.push_str("  }\n");
        }
    }
    s.push_str("};\n");
    s
}

fn emit_cpp_function(func: &Function) -> String {
    let mut s = String::new();
    let (ret_type, fn_name) = match func.name.as_str() {
        "إعداد" | "اعداد" | "setup" => ("void", "setup".to_string()),
        "تكرار" | "حلقة" | "loop" => ("void", "loop".to_string()),
        other => {
            let rt = if body_has_return(&func.body) { "double" } else { "void" };
            (rt, sanitize_ident(other))
        }
    };
    let params: Vec<String> = func
        .params
        .iter()
        .map(|p| format!("double {}", sanitize_ident(p)))
        .collect();
    s.push_str(&format!("{} {}({}) {{\n", ret_type, fn_name, params.join(", ")));
    for stmt in &func.body {
        s.push_str(&emit_cpp_stmt(stmt, 1));
    }
    s.push_str("}\n");
    s
}

fn body_has_return(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        match stmt.unwrap_spanned() {
            Stmt::Return(Some(_)) => return true,
            Stmt::If(_, yes, no) => {
                if body_has_return(yes) {
                    return true;
                }
                if let Some(n) = no {
                    if body_has_return(n) {
                        return true;
                    }
                }
            }
            Stmt::While(_, body) | Stmt::ForEach(_, _, body) | Stmt::Block(body) => {
                if body_has_return(body) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn emit_cpp_stmt(stmt: &Stmt, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    match stmt {
        Stmt::Spanned(inner, _, _) => emit_cpp_stmt(inner, indent),
        Stmt::Let(name, expr) => {
            let c_type = infer_cpp_type(expr);
            format!("{}{} {} = {};\n", pad, c_type, sanitize_ident(name), emit_cpp_expr(expr))
        }
        Stmt::Print(expr) => {
            format!("{}Serial.println({});\n", pad, emit_cpp_expr(expr))
        }
        Stmt::Expr(expr) => {
            format!("{}{};\n", pad, emit_cpp_expr(expr))
        }
        Stmt::Return(Some(expr)) => {
            format!("{}return {};\n", pad, emit_cpp_expr(expr))
        }
        Stmt::Return(None) => format!("{}return;\n", pad),
        Stmt::Break => format!("{}break;\n", pad),
        Stmt::Continue => format!("{}continue;\n", pad),
        Stmt::Block(stmts) => {
            let mut s = format!("{}{{\n", pad);
            for inner in stmts {
                s.push_str(&emit_cpp_stmt(inner, indent + 1));
            }
            s.push_str(&format!("{}}}\n", pad));
            s
        }
        Stmt::If(cond, yes, no) => {
            let mut s = format!("{}if ({}) {{\n", pad, emit_cpp_expr(cond));
            for inner in yes {
                s.push_str(&emit_cpp_stmt(inner, indent + 1));
            }
            if let Some(no_body) = no {
                s.push_str(&format!("{}}} else {{\n", pad));
                for inner in no_body {
                    s.push_str(&emit_cpp_stmt(inner, indent + 1));
                }
            }
            s.push_str(&format!("{}}}\n", pad));
            s
        }
        Stmt::While(cond, body) => {
            let mut s = format!("{}while ({}) {{\n", pad, emit_cpp_expr(cond));
            for inner in body {
                s.push_str(&emit_cpp_stmt(inner, indent + 1));
            }
            s.push_str(&format!("{}}}\n", pad));
            s
        }
        Stmt::ForEach(var, iter, body) => {
            let mut s = format!(
                "{}for (auto {} : {}) {{\n",
                pad,
                sanitize_ident(var),
                emit_cpp_expr(iter)
            );
            for inner in body {
                s.push_str(&emit_cpp_stmt(inner, indent + 1));
            }
            s.push_str(&format!("{}}}\n", pad));
            s
        }
        _ => String::new(),
    }
}

fn infer_cpp_type(expr: &Expr) -> &'static str {
    match expr {
        Expr::Spanned(inner, _, _) => infer_cpp_type(inner),
        Expr::Number(n) => {
            if n.fract() == 0.0 {
                "int"
            } else {
                "double"
            }
        }
        Expr::Text(_) => "String",
        Expr::Bool(_) => "bool",
        _ => "auto",
    }
}

fn emit_cpp_expr(expr: &Expr) -> String {
    match expr {
        Expr::Spanned(inner, _, _) => emit_cpp_expr(inner),
        Expr::Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        Expr::Text(t) => format!("String({:?})", t),
        Expr::Bool(true) => "true".into(),
        Expr::Bool(false) => "false".into(),
        Expr::Null => "0".into(),
        Expr::Variable(name) => match name.as_str() {
            "عالي" | "مرتفع" | "HIGH" => "HIGH".into(),
            "منخفض" | "LOW" => "LOW".into(),
            "مخرج" | "OUTPUT" => "OUTPUT".into(),
            "مدخل" | "INPUT" => "INPUT".into(),
            "مدخل_سحب" | "INPUT_PULLUP" => "INPUT_PULLUP".into(),
            "هذا" | "ذاتي" => "(*this)".into(),
            other => sanitize_ident(other),
        },
        Expr::Assign(name, val) => {
            format!("{} = {}", sanitize_ident(name), emit_cpp_expr(val))
        }
        Expr::Binary(left, op, right) => {
            let l = emit_cpp_expr(left);
            let r = emit_cpp_expr(right);
            if matches!(op, Op::Power) {
                return format!("pow({}, {})", l, r);
            }
            let op_str = match op {
                Op::Add => "+",
                Op::Subtract => "-",
                Op::Multiply => "*",
                Op::Divide => "/",
                Op::Modulo => "%",
                Op::Power => "^",
                Op::Equal => "==",
                Op::NotEqual => "!=",
                Op::Less => "<",
                Op::LessEqual => "<=",
                Op::Greater => ">",
                Op::GreaterEqual => ">=",
                Op::And => "&&",
                Op::Or => "||",
                Op::Not => "!",
                Op::Negate => "-",
            };
            format!("({} {} {})", l, op_str, r)
        }
        Expr::Unary(op, inner) => {
            let v = emit_cpp_expr(inner);
            match op {
                Op::Negate => format!("(-{})", v),
                Op::Not => format!("(!{})", v),
                _ => v,
            }
        }
        Expr::Call(name, args) => emit_cpp_call(name, args),
        Expr::GetProperty(target, prop) => {
            format!("{}.{}", emit_cpp_expr(target), sanitize_ident(prop))
        }
        Expr::SetProperty(target, prop, val) => {
            format!(
                "{}.{} = {}",
                emit_cpp_expr(target),
                sanitize_ident(prop),
                emit_cpp_expr(val)
            )
        }
        Expr::MethodCall(target, method, args) => {
            let c_args: Vec<String> = args.iter().map(emit_cpp_expr).collect();
            format!(
                "{}.{}({})",
                emit_cpp_expr(target),
                sanitize_ident(method),
                c_args.join(", ")
            )
        }
        Expr::Index(target, idx) => {
            format!("{}[{}]", emit_cpp_expr(target), emit_cpp_expr(idx))
        }
        Expr::AssignIndex(name, idx, val) => {
            format!(
                "{}[{}] = {}",
                sanitize_ident(name),
                emit_cpp_expr(idx),
                emit_cpp_expr(val)
            )
        }
        Expr::List(items) => {
            let vals: Vec<String> = items.iter().map(emit_cpp_expr).collect();
            format!("{{{}}}", vals.join(", "))
        }
        _ => "0".into(),
    }
}

fn emit_cpp_call(name: &str, args: &[Expr]) -> String {
    let c_args: Vec<String> = args.iter().map(emit_cpp_expr).collect();
    match name {
        "وضع_المنفذ" if args.len() == 2 => {
            let mode = match args[1].unwrap_spanned() {
                Expr::Text(s) => match s.as_str() {
                    "مخرج" | "OUTPUT" => "OUTPUT".to_string(),
                    "مدخل" | "INPUT" => "INPUT".to_string(),
                    "مدخل_سحب" | "INPUT_PULLUP" => "INPUT_PULLUP".to_string(),
                    _ => c_args[1].clone(),
                },
                _ => c_args[1].clone(),
            };
            format!("pinMode({}, {})", c_args[0], mode)
        }
        "اكتب_رقمي" if args.len() == 2 => {
            let state = match args[1].unwrap_spanned() {
                Expr::Number(n) => {
                    if *n != 0.0 {
                        "HIGH".to_string()
                    } else {
                        "LOW".to_string()
                    }
                }
                Expr::Bool(b) => {
                    if *b {
                        "HIGH".to_string()
                    } else {
                        "LOW".to_string()
                    }
                }
                Expr::Text(s) if s == "عالي" || s == "HIGH" => "HIGH".to_string(),
                Expr::Text(s) if s == "منخفض" || s == "LOW" => "LOW".to_string(),
                _ => c_args[1].clone(),
            };
            format!("digitalWrite({}, {})", c_args[0], state)
        }
        "اقرأ_رقمي" if args.len() == 1 => format!("digitalRead({})", c_args[0]),
        "اكتب_تماثلي" if args.len() == 2 => format!("analogWrite({}, {})", c_args[0], c_args[1]),
        "اقرأ_تماثلي" if args.len() == 1 => format!("analogRead({})", c_args[0]),
        "انتظر_ملي" | "تأخير" | "تاخير" if args.len() == 1 => format!("delay({})", c_args[0]),
        "انتظر_ميكرو" if args.len() == 1 => format!("delayMicroseconds({})", c_args[0]),
        "وقت_التشغيل" if args.is_empty() => "millis()".into(),
        "نبضة_صوت" if args.len() >= 2 => {
            if args.len() == 3 {
                format!("tone({}, {}, {})", c_args[0], c_args[1], c_args[2])
            } else {
                format!("tone({}, {})", c_args[0], c_args[1])
            }
        }
        "أوقف_صوت" | "اوقف_صوت" if args.len() == 1 => format!("noTone({})", c_args[0]),
        "تسلسلي_ابدأ" | "تسلسلي_ابدا" if args.len() == 1 => format!("Serial.begin({})", c_args[0]),
        "تسلسلي_اطبع" if args.len() == 1 => format!("Serial.println({})", c_args[0]),
        "تسلسلي_متاح" if args.is_empty() => "Serial.available()".into(),
        "تسلسلي_اقرأ" | "تسلسلي_اقرا" if args.is_empty() => "Serial.readString()".into(),
        "خريطة_قيمة" if args.len() == 5 => format!(
            "map({}, {}, {}, {}, {})",
            c_args[0], c_args[1], c_args[2], c_args[3], c_args[4]
        ),
        "قيد_قيمة" if args.len() == 3 => {
            format!("constrain({}, {}, {})", c_args[0], c_args[1], c_args[2])
        }
        "نص" if args.len() == 1 => format!("String({})", c_args[0]),
        "مطلق" if args.len() == 1 => format!("abs({})", c_args[0]),
        "جذر" if args.len() == 1 => format!("sqrt({})", c_args[0]),
        other => format!("{}({})", sanitize_ident(other), c_args.join(", ")),
    }
}

fn sanitize_ident(name: &str) -> String {
    if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return name.to_string();
    }
    let mut out = String::from("lsn_");
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push_str(&format!("{:x}", c as u32));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arduino_transpiler_generates_valid_ino() {
        let src = r#"
دع منفذ_مصباح = 13

دالة إعداد() {
    تسلسلي_ابدأ(9600)
    وضع_المنفذ(منفذ_مصباح، "مخرج")
}

دالة تكرار() {
    اكتب_رقمي(منفذ_مصباح، 1)
    انتظر_ملي(1000)
    اكتب_رقمي(منفذ_مصباح، 0)
    انتظر_ملي(1000)
}
"#;
        let ino = transpile_arduino(src).unwrap();
        assert!(ino.contains("#include <Arduino.h>"));
        assert!(ino.contains("void setup()"));
        assert!(ino.contains("Serial.begin(9600);"));
        assert!(ino.contains("pinMode("));
        assert!(ino.contains("OUTPUT"));
        assert!(ino.contains("void loop()"));
        assert!(ino.contains("digitalWrite("));
        assert!(ino.contains("HIGH"));
        assert!(ino.contains("LOW"));
        assert!(ino.contains("delay(1000);"));
    }
}
