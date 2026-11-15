use std::path::{Path, PathBuf};
use std::time::Instant;
use crate::runtime::Interpreter;
use crate::lexer::tokenize;
use crate::parser::parse;
use crate::ast::Stmt;

#[derive(Debug, Clone)]
pub struct TestCaseResult {
    pub name: String,
    pub passed: bool,
    pub duration_secs: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestFileResult {
    pub path: PathBuf,
    pub cases: Vec<TestCaseResult>,
}

#[derive(Debug, Clone, Default)]
pub struct TestSummary {
    pub total_files: usize,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_duration_secs: f64,
    pub file_results: Vec<TestFileResult>,
}

pub fn discover_test_files<P: AsRef<Path>>(root: P) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_dir(root.as_ref(), &mut files);
    files.sort();
    files
}

fn walk_dir(dir: &Path, acc: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                walk_dir(&path, acc);
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "lsn" {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if stem.starts_with("test_")
                                || stem.ends_with("_test")
                                || stem.starts_with("اختبار_")
                                || stem.ends_with("_اختبار")
                                || stem == "اختبار"
                                || stem == "test"
                                || path.components().any(|c| c.as_os_str() == "tests")
                            {
                                acc.push(path);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn run_test_file(path: &Path) -> TestFileResult {
    let mut result = TestFileResult {
        path: path.to_path_buf(),
        cases: Vec::new(),
    };

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            result.cases.push(TestCaseResult {
                name: "قراءة_الملف".to_string(),
                passed: false,
                duration_secs: 0.0,
                error: Some(format!("فشل قراءة الملف: {}", e)),
            });
            return result;
        }
    };

    let tokens = match tokenize(&content) {
        Ok(t) => t,
        Err(e) => {
            result.cases.push(TestCaseResult {
                name: "تحليل_الرموز".to_string(),
                passed: false,
                duration_secs: 0.0,
                error: Some(e.to_string()),
            });
            return result;
        }
    };

    let ast = match parse(tokens) {
        Ok(a) => a,
        Err(e) => {
            result.cases.push(TestCaseResult {
                name: "تحليل_البناء".to_string(),
                passed: false,
                duration_secs: 0.0,
                error: Some(e.to_string()),
            });
            return result;
        }
    };

    let mut test_fn_names = Vec::new();
    for stmt in &ast {
        if let Stmt::Function(f) = stmt.unwrap_spanned() {
            if f.name.starts_with("اختبار") || f.name.starts_with("test") {
                test_fn_names.push(f.name.clone());
            }
        }
    }

    if test_fn_names.is_empty() {
        let mut interp = Interpreter::for_cli();
        let start = Instant::now();
        match interp.run_file(path) {
            Ok(_) => {
                result.cases.push(TestCaseResult {
                    name: path.file_name().and_then(|f| f.to_str()).unwrap_or("اختبار").to_string(),
                    passed: true,
                    duration_secs: start.elapsed().as_secs_f64(),
                    error: None,
                });
            }
            Err(e) => {
                result.cases.push(TestCaseResult {
                    name: path.file_name().and_then(|f| f.to_str()).unwrap_or("اختبار").to_string(),
                    passed: false,
                    duration_secs: start.elapsed().as_secs_f64(),
                    error: Some(e),
                });
            }
        }
    } else {
        for fn_name in test_fn_names {
            let mut interp = Interpreter::for_cli();
            if let Err(e) = interp.run_file(path) {
                result.cases.push(TestCaseResult {
                    name: fn_name,
                    passed: false,
                    duration_secs: 0.0,
                    error: Some(format!("فشل تهيئة الملف: {}", e)),
                });
                continue;
            }
            let call_src = format!("{}()", fn_name);
            let start = Instant::now();
            match interp.run_source(&call_src) {
                Ok(_) => {
                    result.cases.push(TestCaseResult {
                        name: fn_name,
                        passed: true,
                        duration_secs: start.elapsed().as_secs_f64(),
                        error: None,
                    });
                }
                Err(e) => {
                    result.cases.push(TestCaseResult {
                        name: fn_name,
                        passed: false,
                        duration_secs: start.elapsed().as_secs_f64(),
                        error: Some(e),
                    });
                }
            }
        }
    }

    result
}

pub fn run_tests(targets: &[PathBuf]) -> TestSummary {
    let mut files = Vec::new();
    if targets.is_empty() {
        files = discover_test_files(".");
    } else {
        for t in targets {
            if t.is_dir() {
                files.extend(discover_test_files(t));
            } else if t.is_file() {
                files.push(t.clone());
            }
        }
    }

    let overall_start = Instant::now();
    let mut summary = TestSummary {
        total_files: files.len(),
        ..Default::default()
    };

    for file in &files {
        let file_res = run_test_file(file);
        for c in &file_res.cases {
            summary.total_tests += 1;
            if c.passed {
                summary.passed += 1;
            } else {
                summary.failed += 1;
            }
        }
        summary.file_results.push(file_res);
    }

    summary.total_duration_secs = overall_start.elapsed().as_secs_f64();
    summary
}

pub fn print_test_summary(summary: &TestSummary) {
    println!("🧪 بدء تشغيل اختبارات لسان...\n");
    for file_res in &summary.file_results {
        println!("📁 اختبار: {}", file_res.path.display());
        for c in &file_res.cases {
            if c.passed {
                println!("  ✓ {} ({:.3} ثانية)", c.name, c.duration_secs);
            } else {
                println!("  ✗ {} ({:.3} ثانية)", c.name, c.duration_secs);
                if let Some(err) = &c.error {
                    println!("    ◄ {}", err);
                }
            }
        }
        println!();
    }
    println!("--------------------------------------------------");
    if summary.failed == 0 {
        println!(
            "النتيجة: نجح {} من أصل {} اختباراً في {:.3} ثانية (كل الاختبارات ناجحة ✓)",
            summary.passed, summary.total_tests, summary.total_duration_secs
        );
    } else {
        println!(
            "النتيجة: فشل {} | نجح {} | إجمالي {} اختبارات في {:.3} ثانية",
            summary.failed, summary.passed, summary.total_tests, summary.total_duration_secs
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_executes_isolated_test_functions() {
        let temp_dir = std::env::temp_dir().join(format!("lisan_test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("حساب_اختبار.lsn");
        let code = r#"
دالة مساعدة(س) {
    أرجع س * 2
}

دالة اختبار_النجاح() {
    تحقق_يساوي(مساعدة(5)، 10)
}

دالة اختبار_الفشل() {
    تحقق_يساوي(مساعدة(3)، 99)
}
"#;
        std::fs::write(&file_path, code).unwrap();

        let res = run_test_file(&file_path);
        assert_eq!(res.cases.len(), 2);
        assert_eq!(res.cases[0].name, "اختبار_النجاح");
        assert!(res.cases[0].passed);

        assert_eq!(res.cases[1].name, "اختبار_الفشل");
        assert!(!res.cases[1].passed);
        assert!(res.cases[1].error.as_ref().unwrap().contains("فشل التحقق"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
