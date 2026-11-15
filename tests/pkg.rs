use lisan::pkg::{add_dependency, create_lockfile, init_project, PackageManifest};
use std::fs;

#[test]
fn test_package_manifest_serde() {
    let mut manifest = PackageManifest::new("حزمتي");
    manifest.description = "حزمة تجريبية للسان".into();
    manifest.dependencies.push(("رياضيات_متقدمة".into(), "./مكتبات/math.lsn".into()));

    let json = manifest.to_json();
    assert!(json.contains("حزمتي"));
    assert!(json.contains("رياضيات_متقدمة"));

    let parsed = PackageManifest::from_json(&json).expect("Should parse manifest JSON");
    assert_eq!(parsed.name, "حزمتي");
    assert_eq!(parsed.description, "حزمة تجريبية للسان");
    assert_eq!(parsed.dependencies.len(), 1);
    assert_eq!(parsed.dependencies[0].0, "رياضيات_متقدمة");
}

#[test]
fn test_package_lifecycle_init_add_lock() {
    let temp_dir = std::env::temp_dir().join(format!("lisan_pkg_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);

    // 1. Initialize
    let manifest_path = init_project("مشروع_تجربة", &temp_dir).expect("Package init should succeed");
    assert!(manifest_path.exists());
    assert!(temp_dir.join("رئيسي.lsn").exists());
    assert!(temp_dir.join("مكتبات").exists());
    assert!(temp_dir.join("lisan.lock").exists());

    // 2. Add dependency
    add_dependency("مكتبة_الشبكة", "مكتبات/network.lsn", &temp_dir)
        .expect("Adding dependency should succeed");

    let updated_manifest = fs::read_to_string(&manifest_path).unwrap();
    assert!(updated_manifest.contains("مكتبة_الشبكة"));

    let lock_content = fs::read_to_string(temp_dir.join("lisan.lock")).unwrap();
    assert!(lock_content.contains("مكتبة_الشبكة"));
    assert!(lock_content.contains("مكتبات/network.lsn"));

    // 3. Re-lock
    let lock_path = create_lockfile(&temp_dir).expect("Re-locking should succeed");
    assert!(lock_path.exists());

    let _ = fs::remove_dir_all(&temp_dir);
}
