use std::path::{Path, PathBuf};
use std::process::Command;

fn collect_lsn_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).expect("read examples directory");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_lsn_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("lsn") {
            out.push(path);
        }
    }
}

#[test]
fn all_examples_pass_syntax_and_module_check() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut files = Vec::new();
    collect_lsn_files(&examples_dir, &mut files);
    files.sort();
    assert!(
        files.len() >= 30,
        "expected at least 30 .lsn examples, found {}",
        files.len()
    );

    let mut failures = Vec::new();
    for file in &files {
        if let Err(err) = lisan::check_file(file) {
            failures.push(format!("{}: {}", file.display(), err));
        }
    }
    assert!(
        failures.is_empty(),
        "examples failed syntax/module check:\n{}",
        failures.join("\n")
    );
}

#[test]
fn core_non_network_examples_execute_cleanly() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let safe_examples = [
        "hello.lsn",
        "loops.lsn",
        "functions.lsn",
        "lists.lsn",
        "dictionary.lsn",
        "text.lsn",
        "errors_and_match.lsn",
        "json_and_random.lsn",
        "oop_car.lsn",
        "python_style_no_semicolons.lsn",
        "statistics_demo.lsn",
        "text_encoding.lsn",
        "vm_demo.lsn",
        "standard_library.lsn",
        "extended_features.lsn",
        "modules/main.lsn",
    ];

    for rel in safe_examples {
        let path = examples_dir.join(rel);
        let result = lisan::run_file(&path);
        assert!(
            result.is_ok(),
            "example {} failed to execute: {:?}",
            rel,
            result.err()
        );
    }

    #[cfg(unix)]
    {
        let path = examples_dir.join("python_parity.lsn");
        let result = lisan::run_file(&path);
        assert!(
            result.is_ok(),
            "example python_parity.lsn failed to execute: {:?}",
            result.err()
        );
    }
}

#[test]
fn backend_parity_interpreter_vm_and_native_ir() {
    let source = r#"
دالة مربع(س) {
    أرجع س * س؛
}
دع أرقام = [٢, 3, ٤]؛
دع مجموع_المربعات = 0؛
لكل ر في أرقام {
    مجموع_المربعات = مجموع_المربعات + مربع(ر)؛
}
اطبع(مجموع_المربعات)؛
اطبع("لسان")؛
"#;

    let expected = vec!["29".to_string(), "لسان".to_string()];
    let interp_out = lisan::run(source).expect("interpreter should succeed");
    assert_eq!(interp_out, expected);

    let tokens = lisan::lexer::tokenize(source).expect("tokenize");
    let ast = lisan::parser::parse(tokens).expect("parse");
    let ir = lisan::native::generate_llvm(&ast).expect("native LLVM IR generation");
    assert!(ir.contains("define double @lisan_fn_0(double %arg0)"));
    assert!(ir.contains("define i32 @main()"));

    let vm_demo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("vm_demo.lsn");
    let bin = env!("CARGO_BIN_EXE_lisan");

    let cli_interp = Command::new(bin)
        .arg("--interpret")
        .arg(&vm_demo)
        .output()
        .expect("run --interpret on vm_demo");
    let cli_vm = Command::new(bin)
        .arg("--vm")
        .arg(&vm_demo)
        .output()
        .expect("run --vm on vm_demo");

    assert!(
        cli_interp.status.success(),
        "--interpret failed: {}",
        String::from_utf8_lossy(&cli_interp.stderr)
    );
    assert!(
        cli_vm.status.success(),
        "--vm failed: {}",
        String::from_utf8_lossy(&cli_vm.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&cli_interp.stdout).replace("\r\n", "\n"),
        String::from_utf8_lossy(&cli_vm.stdout).replace("\r\n", "\n"),
        "interpreter and VM outputs diverged on vm_demo.lsn"
    );
}

#[test]
fn cli_subcommands_integration_check_tokens_ast_eval_and_emit_llvm() {
    let bin = env!("CARGO_BIN_EXE_lisan");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let hello = manifest.join("examples").join("hello.lsn");
    let native_ex = manifest.join("examples").join("native.lsn");

    let check = Command::new(bin).arg("--check").arg(&hello).output().expect("--check");
    assert!(check.status.success());

    let tokens = Command::new(bin).arg("--tokens").arg(&hello).output().expect("--tokens");
    assert!(tokens.status.success());
    assert!(!tokens.stdout.is_empty());

    let ast = Command::new(bin).arg("--ast").arg(&hello).output().expect("--ast");
    assert!(ast.status.success());
    assert!(!ast.stdout.is_empty());

    let eval = Command::new(bin).arg("-e").arg("اطبع(10 + 5)").output().expect("-e");
    assert!(eval.status.success());
    assert!(String::from_utf8_lossy(&eval.stdout).contains("15"));

    let ll_out = std::env::temp_dir().join("lisan_coverage_emit.ll");
    let emit = Command::new(bin)
        .arg("--emit-llvm")
        .arg(&native_ex)
        .arg("--output")
        .arg(&ll_out)
        .output()
        .expect("--emit-llvm");
    assert!(emit.status.success());
    let ll_content = std::fs::read_to_string(&ll_out).expect("read emitted ll");
    let _ = std::fs::remove_file(&ll_out);
    assert!(ll_content.contains("define i32 @main()"));
}
