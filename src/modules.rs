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
        "اردوينو" | "متحكمات" | "arduino" => Some(crate::arduino::STDLIB_ARDUINO),
        "ايقونات" | "تصميم" | "واجهه" | "هاتف" | "جوال" | "ويب" | "ui" | "mobile" | "web" | "icons" => {
            Some(crate::ui::STDLIB_UI_AND_MOBILE)
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
                validate_no_nested_imports(&statement)?;
                if module.ends_with(".lsn") || Path::new(module).exists() {
                    let imported = path.parent().expect("canonical file has parent").join(module);
                    let can = fs::canonicalize(&imported).map_err(|e| format!("لا يمكن العثور على وحدة '{}': {}", imported.display(), e))?;
                    merged.push(Stmt::ImportModule { module: can.to_string_lossy().to_string(), alias: alias.clone() });
                } else {
                    merged.push(statement);
                }
            }
            Stmt::FromImport { module, items } => {
                validate_no_nested_imports(&statement)?;
                if module.ends_with(".lsn") || Path::new(module).exists() {
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
