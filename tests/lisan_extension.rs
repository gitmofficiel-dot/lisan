use lisan::modules::{is_source_ext, is_source_name};
use lisan::{check_file, run_file};
use std::fs;

#[test]
fn both_extensions_are_recognised() {
    assert!(is_source_ext("lsn"));
    assert!(is_source_ext("lisan"));
    assert!(is_source_ext("LISAN"));
    assert!(!is_source_ext("txt"));
    assert!(is_source_name("a/b/hello.lisan"));
    assert!(is_source_name("hello.lsn"));
    assert!(!is_source_name("hello.html"));
    assert!(!is_source_name("lisan"));
}

#[test]
fn lisan_file_runs_and_imports_both_extensions() {
    let dir = std::env::temp_dir().join(format!("lisan_ext_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("part.lisan"), r#"اطبع("من lisan")؛"#).unwrap();
    fs::write(dir.join("old.lsn"), r#"اطبع("من lsn")؛"#).unwrap();
    fs::write(
        dir.join("app.lisan"),
        r#"استورد "part.lisan"؛ استورد "old.lsn"؛"#,
    )
    .unwrap();
    assert!(check_file(dir.join("app.lisan")).is_ok());
    assert_eq!(run_file(dir.join("app.lisan")).unwrap(), ["من lisan", "من lsn"]);
}
