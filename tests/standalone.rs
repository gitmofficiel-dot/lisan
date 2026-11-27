use lisan::bundle::{build_executable, generate_rust_main};
use lisan::modules;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct TempDirGuard(PathBuf);
impl Drop for TempDirGuard {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

fn make_temp_dir(prefix: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}_{}", std::process::id(), seq, nanos));
    fs::create_dir(&dir).unwrap();
    dir
}

fn toml_escaped_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\").replace('"', "\\\"")
}

fn assert_standalone_binary_output(binary: &Path) {
    let output = Command::new(binary).output().unwrap();
    assert!(output.status.success(), "standalone binary failed: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "مربع 7 = 49");
}

#[test]
fn emits_valid_program_code_for_imported_app() {
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/modules/main.lsn");
    let program = modules::load_file(std::path::Path::new(source)).unwrap();
    let emitted = generate_rust_main(&program).unwrap();
    assert!(emitted.contains("مربع 7 = "));
    assert!(!emitted.contains("Stmt::Import"));
}

#[test]
fn compiles_and_executes_standalone_binary() {
    // Only execute a nested compilation when Cargo is available (as in CI runners).
    if Command::new("cargo").arg("--version").output().is_err() { return; }
    let name = format!("lisan_standalone_test_{}_{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let dir = std::env::temp_dir().join(name);
    fs::create_dir(&dir).unwrap();
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/modules/main.lsn");
    let result = dir.join(format!("standalone{}", std::env::consts::EXE_SUFFIX));
    build_executable(&input, Some(&result)).unwrap();
    let output = Command::new(&result).output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "مربع 7 = 49");
    assert!(build_executable(&input, Some(&result)).unwrap_err().contains("موجود"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn compiles_standalone_when_cargo_target_dir_env_is_set() {
    if Command::new("cargo").arg("--version").output().is_err() { return; }
    let root = make_temp_dir("lisan_standalone_env_target_dir");
    let _guard = TempDirGuard(root.clone());
    let trap_target = root.join("trap_target");
    fs::create_dir(&trap_target).unwrap();
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/modules/main.lsn");
    let out = root.join(format!("standalone{}", std::env::consts::EXE_SUFFIX));

    let build = Command::new(env!("CARGO_BIN_EXE_lisan"))
        .arg("--build").arg(&input).arg("--output").arg(&out)
        .env("CARGO_TARGET_DIR", &trap_target)
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET")
        .output().unwrap();
    assert!(build.status.success(), "lisan --build failed: {}", String::from_utf8_lossy(&build.stderr));

    assert_standalone_binary_output(&out);
    assert!(fs::read_dir(&trap_target).unwrap().next().is_none(), "CARGO_TARGET_DIR must not receive standalone artifacts");
}

#[test]
fn compiles_standalone_when_cargo_build_target_dir_env_is_set() {
    if Command::new("cargo").arg("--version").output().is_err() { return; }
    let root = make_temp_dir("lisan_standalone_build_target_dir");
    let _guard = TempDirGuard(root.clone());
    let trap_target = root.join("trap_target");
    fs::create_dir(&trap_target).unwrap();
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/modules/main.lsn");
    let out = root.join(format!("standalone{}", std::env::consts::EXE_SUFFIX));

    let build = Command::new(env!("CARGO_BIN_EXE_lisan"))
        .arg("--build").arg(&input).arg("--output").arg(&out)
        .env_remove("CARGO_TARGET_DIR")
        .env("CARGO_BUILD_TARGET_DIR", &trap_target)
        .env_remove("CARGO_BUILD_TARGET")
        .output().unwrap();
    assert!(build.status.success(), "lisan --build failed: {}", String::from_utf8_lossy(&build.stderr));

    assert_standalone_binary_output(&out);
    assert!(fs::read_dir(&trap_target).unwrap().next().is_none(), "CARGO_BUILD_TARGET_DIR must not receive standalone artifacts");
}

#[test]
fn compiles_standalone_when_cargo_home_config_sets_target_dir() {
    if Command::new("cargo").arg("--version").output().is_err() { return; }
    let root = make_temp_dir("lisan_standalone_home_config_target_dir");
    let _guard = TempDirGuard(root.clone());
    let trap_target = root.join("trap_target");
    let cargo_home = root.join("cargo_home");
    fs::create_dir(&trap_target).unwrap();
    fs::create_dir(&cargo_home).unwrap();
    fs::write(
        cargo_home.join("config.toml"),
        format!("[build]\ntarget-dir = \"{}\"\n", toml_escaped_path(&trap_target)),
    ).unwrap();
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/modules/main.lsn");
    let out = root.join(format!("standalone{}", std::env::consts::EXE_SUFFIX));

    let build = Command::new(env!("CARGO_BIN_EXE_lisan"))
        .arg("--build").arg(&input).arg("--output").arg(&out)
        .env("CARGO_HOME", &cargo_home)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET")
        .output().unwrap();
    assert!(build.status.success(), "lisan --build failed: {}", String::from_utf8_lossy(&build.stderr));

    assert_standalone_binary_output(&out);
    assert!(fs::read_dir(&trap_target).unwrap().next().is_none(), "$CARGO_HOME/config.toml target-dir must not receive standalone artifacts");
}

#[test]
fn compiles_standalone_when_ancestor_cargo_config_sets_target_dir() {
    if Command::new("cargo").arg("--version").output().is_err() { return; }
    let root = make_temp_dir("lisan_standalone_ancestor_config_target_dir");
    let _guard = TempDirGuard(root.clone());
    let trap_target = root.join("trap_target");
    let tmp_ancestor = root.join("tmp_ancestor");
    fs::create_dir(&trap_target).unwrap();
    fs::create_dir_all(tmp_ancestor.join(".cargo")).unwrap();
    fs::write(
        tmp_ancestor.join(".cargo").join("config.toml"),
        format!("[build]\ntarget-dir = \"{}\"\n", toml_escaped_path(&trap_target)),
    ).unwrap();
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/modules/main.lsn");
    let out = root.join(format!("standalone{}", std::env::consts::EXE_SUFFIX));

    let build = Command::new(env!("CARGO_BIN_EXE_lisan"))
        .arg("--build").arg(&input).arg("--output").arg(&out)
        .env("TMPDIR", &tmp_ancestor)
        .env("TMP", &tmp_ancestor)
        .env("TEMP", &tmp_ancestor)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET")
        .output().unwrap();
    assert!(build.status.success(), "lisan --build failed: {}", String::from_utf8_lossy(&build.stderr));

    assert_standalone_binary_output(&out);
    assert!(fs::read_dir(&trap_target).unwrap().next().is_none(), "ancestor .cargo/config.toml target-dir must not receive standalone artifacts");
}
