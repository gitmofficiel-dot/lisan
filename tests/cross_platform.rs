use std::path::Path;
use std::process::Command;

#[test]
fn doctor_report_contains_os_arch_and_tool_checks() {
    let report = lisan::desktop::collect_environment_report();
    assert!(report.contains(&format!("OS: {}", std::env::consts::OS)));
    assert!(report.contains(&format!("Arch: {}", std::env::consts::ARCH)));
    assert!(report.contains("Native Compiler (LLVM/Clang):"));
    assert!(report.contains("Rust Compiler:"));
    assert!(report.contains("Python (Raspberry Pi target):"));
}

#[test]
fn cli_doctor_and_platforms_succeed_across_os() {
    let bin = env!("CARGO_BIN_EXE_lisan");

    let doctor = Command::new(bin)
        .arg("--doctor")
        .output()
        .expect("run lisan --doctor");
    assert!(doctor.status.success());
    let doctor_out = String::from_utf8_lossy(&doctor.stdout);
    assert!(doctor_out.contains(&format!("OS: {}", std::env::consts::OS)));
    assert!(doctor_out.contains(&format!("Arch: {}", std::env::consts::ARCH)));

    let platforms = Command::new(bin)
        .arg("--platforms")
        .output()
        .expect("run lisan --platforms");
    assert!(platforms.status.success());
}

#[test]
fn normalize_executable_path_matches_target_os() {
    let p = lisan::native::normalize_executable_path(Path::new("my_app"));
    #[cfg(target_os = "windows")]
    assert_eq!(p, Path::new("my_app.exe"));
    #[cfg(not(target_os = "windows"))]
    assert_eq!(p, Path::new("my_app"));

    let with_ext = lisan::native::normalize_executable_path(Path::new("my_app.exe"));
    assert_eq!(with_ext, Path::new("my_app.exe"));
}