//! File-based imports with relative paths, deduplication, and cycle detection.
use crate::ast::Stmt;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_IMPORT_DEPTH: usize = 32;
const MAX_SOURCE_BYTES: u64 = 2_000_000;

/// التحقق مما إذا كان اسم الوحدة يشير إلى مكتبة قياسية مدمجة في لسان
pub fn resolve_builtin_module(name: &str) -> Option<&'static str> {
    let trimmed = name.trim().strip_suffix(".lsn").unwrap_or(name.trim());
    let norm = crate::diagnostics::normalize_arabic(trimmed);
    match norm.as_str() {
        "اردوينو" | "arduino" => Some(crate::arduino::STDLIB_ARDUINO),
        "esp" | "esp32" | "esp8266" | "اي_اس_بي" | "اي_اس_بي32" => Some(crate::arduino::STDLIB_ESP32),
        "stm" | "stm32" | "اس_تي_ام" | "اس_تي_ام32" => Some(crate::arduino::STDLIB_STM32),
        "متحكمات" | "embedded" | "microcontrollers" => Some(crate::arduino::STDLIB_EMBEDDED),
        "راسبيري" | "راسبيري_باي" | "راسپيري" | "raspberry" | "raspberry_pi" | "raspberrypi" | "rasspiry" | "rpi" => {
            Some(crate::raspberry::STDLIB_RASPBERRY)
        }
        "بيكو" | "راسبيري_بيكو" | "pico" | "rp2040" | "pico_w" => {
            Some(crate::pico::STDLIB_PICO)
        }
        "روبوت" | "روبوتات" | "تحكم" | "بروتوكولات" | "صناعي" | "robotics" | "robot" | "protocols" => {
            Some(crate::robotics::STDLIB_ROBOTICS)
        }
        "ذكاء" | "ذكاء_اصطناعي" | "بيانات" | "علوم_البيانات" | "مخططات" | "رسوم" | "رسوميات"
        | "ai" | "ml" | "data" | "charts" | "svg" => {
            Some(crate::ai_data::STDLIB_AI_DATA)
        }
        "اطار_ويب" | "تطبيق_ويب" | "خادم" | "خادم_ويب" | "موجه" | "ويب_كامل" | "web_framework" | "server" => {
            Some(crate::web_framework::STDLIB_WEB_FRAMEWORK)
        }
        "انظمه" | "طرفيه" | "مهام" | "سجل" | "العاب" | "لعبه" | "العاب_2d"
        | "systems" | "cli" | "games" | "game" => {
            Some(crate::sys_games::STDLIB_SYSTEMS_AND_GAMES)
        }
        "قاعده_بيانات" | "قواعد_بيانات" | "تشفير" | "انماط" | "منصه" | "امان" | "مكتبه_قياسيه"
        | "db" | "database" | "crypto" | "regex" | "platform" => {
            Some(crate::stdlib_ext::STDLIB_EXT_LISAN)
        }
        "موقع" | "مواقع" | "موقع_ويب" | "ايقونات" | "تصميم" | "واجهه" | "هاتف" | "جوال" | "ويب" | "ui" | "mobile" | "web" | "icons" => {
            Some(crate::ui::STDLIB_UI_AND_MOBILE)
        }
        "توافق" | "ترابط" | "بايثون" | "جافاسكربت" | "ذكاء_خارجي" | "interop" | "python" | "javascript" | "js" => {
            Some(crate::interop::STDLIB_INTEROP)
        }
        _ => None,
    }
}

pub fn load_file(path: &Path) -> Result<Vec<Stmt>, String> {
    let mut visited = HashSet::new();
    let mut active = Vec::new();
    load_recursive(path, &mut visited, &mut active)
}

fn load_recursive(path: &Path, visited: &mut HashSet<PathBuf>, active: &mut Vec<PathBuf>) -> Result<Vec<Stmt>, String> {
    let path = fs::canonicalize(path)
        .map_err(|e| format!("لا يمكن العثور على ملف لسان '{}': {}", path.display(), e))?;
    if active.contains(&path) {
        return Err(format!("اكتُشفت دورة استيراد للملف '{}'", path.display()));
    }
    if visited.contains(&path) {
        return Ok(Vec::new());
    }
    if active.len() >= MAX_IMPORT_DEPTH {
        return Err("تجاوز الحد الآمن لعمق استيراد الملفات".into());
    }
    let metadata = fs::metadata(&path)
        .map_err(|e| format!("تعذر قراءة بيانات الملف '{}': {}", path.display(), e))?;
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(format!("ملف لسان '{}' تجاوز الحجم المسموح", path.display()));
    }
    let source = fs::read_to_string(&path)
        .map_err(|e| format!("تعذر قراءة ملف UTF-8 '{}': {}", path.display(), e))?;
    let tokens = crate::lexer::tokenize(&source)
        .map_err(|e| format!("في الملف '{}': {}", path.display(), e))?;
    let statements = crate::parser::parse(tokens)
        .map_err(|e| format!("في الملف '{}': {}", path.display(), e))?;
    active.push(path.clone());
    let mut merged = Vec::new();
    for statement in statements {
        match statement.unwrap_spanned() {
            Stmt::Import(relative) => {
                if let Some(builtin_src) = resolve_builtin_module(relative) {
                    let b_tokens = crate::lexer::tokenize(builtin_src)?;
                    let b_stmts = crate::parser::parse(b_tokens)?;
                    merged.extend(b_stmts);
                    continue;
                }
                let imported = Path::new(relative);
                if relative.is_empty() || imported.is_absolute() {
                    return Err("استورد تقبل مسارًا نسبيًا غير فارغ فقط".into());
                }
                let imported = path.parent().expect("canonical file has parent").join(imported);
                merged.extend(load_recursive(&imported, visited, active)?);
            }
            Stmt::ImportModule { module, alias } => {
                if is_source_name(module) || Path::new(module).exists() {
                    let imported = path.parent().expect("canonical file has parent").join(module);
                    let can = fs::canonicalize(&imported).map_err(|e| format!("لا يمكن العثور على وحدة '{}': {}", imported.display(), e))?;
                    merged.push(Stmt::ImportModule { module: can.to_string_lossy().to_string(), alias: alias.clone() });
                } else {
                    merged.push(statement);
                }
            }
            Stmt::FromImport { module, items } => {
                if is_source_name(module) || Path::new(module).exists() {
                    let imported = path.parent().expect("canonical file has parent").join(module);
                    let can = fs::canonicalize(&imported).map_err(|e| format!("لا يمكن العثور على وحدة '{}': {}", imported.display(), e))?;
                    merged.push(Stmt::FromImport { module: can.to_string_lossy().to_string(), items: items.clone() });
                } else {
                    merged.push(statement);
                }
            }
            _ => {
                validate_no_nested_imports(&statement)?;
                merged.push(statement);
            }
        }
    }
    active.pop();
    visited.insert(path);
    Ok(merged)
}

fn validate_no_nested_imports(stmt: &Stmt) -> Result<(), String> {
    let nested: &[Stmt] = match stmt.unwrap_spanned() {
        Stmt::Import(_) | Stmt::ImportModule { .. } | Stmt::FromImport { .. } => {
            return Err("استورد مسموحة في المستوى الأعلى للملف فقط".into());
        }
        Stmt::Block(body) | Stmt::While(_, body) | Stmt::ForEach(_, _, body)
        | Stmt::ForEachUnpack(_, _, body) | Stmt::With { body, .. } => body,
        Stmt::Function(function) => &function.body,
        Stmt::If(_, yes, no) => {
            for entry in yes { validate_no_nested_imports(entry)?; }
            if let Some(no) = no { for entry in no { validate_no_nested_imports(entry)?; } }
            return Ok(());
        }
        _ => &[],
    };
    for entry in nested { validate_no_nested_imports(entry)?; }
    Ok(())
}

/// صيغ ملفات المصدر المعتمدة: `.lsn` (الأصلية) و`.lisan` (الصيغة الرسمية الإضافية).
/// لإضافة صيغة جديدة مستقبلًا يكفي توسيع هذه القائمة.
pub const SOURCE_EXTENSIONS: &[&str] = &["lsn", "lisan"];

/// هل الامتداد (دون نقطة) صيغة مصدر للسان؟
pub fn is_source_ext(ext: &str) -> bool {
    SOURCE_EXTENSIONS.iter().any(|e| ext.eq_ignore_ascii_case(e))
}

/// هل اسم الملف أو مساره ينتهي بصيغة مصدر للسان؟
pub fn is_source_name(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(is_source_ext)
        .unwrap_or(false)
}
