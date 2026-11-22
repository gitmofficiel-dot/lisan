use lisan::pico::transpile_pico_micropython;
use lisan::run;

#[test]
fn test_web_fullstack_app_example() {
    let source = std::fs::read_to_string("examples/web_fullstack_app.lsn")
        .expect("يجب قراءة examples/web_fullstack_app.lsn");
    let out = run(&source).expect("يجب تشغيل مثال إطار الويب الكامل بنجاح");
    assert!(!out.is_empty());
}

#[test]
fn test_mobile_super_app_example_and_pwa_export() {
    let source = std::fs::read_to_string("examples/mobile_super_app.lsn")
        .expect("يجب قراءة examples/mobile_super_app.lsn");
    let out = run(&source).expect("يجب تشغيل مثال تطبيق الهاتف الفائق بنجاح");
    assert!(!out.is_empty());

    let tmp_dir = std::env::temp_dir().join(format!("lisan_pwa_v2_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    let idx = lisan::ui::export_mobile_pwa("تطبيق لسان الفائق", "<!DOCTYPE html><html></html>", &tmp_dir)
        .expect("يجب تصدير حزمة PWA بنجاح");
    assert!(idx.exists());
    assert!(tmp_dir.join("manifest.json").exists());
    assert!(tmp_dir.join("sw.js").exists(), "يجب توليد Service Worker للعمل دون إنترنت");
    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_lisan_ui_library_security_and_demo() {
    let sec_out = lisan::run_file("libs/ui/test_security.lsn")
        .expect("يجب اجتياز اختبارات أمان مكتبة lisan-ui");
    assert!(!sec_out.is_empty());
}

#[test]
fn test_pico_w_example_and_micropython_transpiler() {
    let source = std::fs::read_to_string("examples/pico_w_smart_controller.lsn")
        .expect("يجب قراءة examples/pico_w_smart_controller.lsn");
    let out = run(&source).expect("يجب تشغيل مثال راسبيري بيكو بنجاح");
    assert!(!out.is_empty());

    let py = transpile_pico_micropython(&source)
        .expect("يجب ترجمة مثال راسبيري بيكو إلى MicroPython بنجاح");
    assert!(py.contains("import machine"));

    let tmp_py = std::env::temp_dir().join(format!("lisan_pico_check_{}.py", std::process::id()));
    std::fs::write(&tmp_py, &py).unwrap();
    if let Ok(status) = std::process::Command::new("python3")
        .arg("-m")
        .arg("py_compile")
        .arg(&tmp_py)
        .status()
    {
        assert!(status.success(), "كود MicroPython المولد لبيكو يجب أن يجتاز py_compile");
    }
    let _ = std::fs::remove_file(&tmp_py);
}

#[test]
fn test_robotics_pid_arm_example() {
    let source = std::fs::read_to_string("examples/robotics_pid_arm.lsn")
        .expect("يجب قراءة examples/robotics_pid_arm.lsn");
    let out = run(&source).expect("يجب تشغيل مثال الروبوتات والبروتوكولات بنجاح");
    assert!(!out.is_empty());
}

#[test]
fn test_ai_ml_and_svg_charts_examples() {
    let ai_src = std::fs::read_to_string("examples/ai_ml_data_science.lsn")
        .expect("يجب قراءة examples/ai_ml_data_science.lsn");
    let ai_out = run(&ai_src).expect("يجب تشغيل مثال الذكاء الاصطناعي وعلوم البيانات بنجاح");
    assert!(!ai_out.is_empty());

    let svg_src = std::fs::read_to_string("examples/svg_charts_report.lsn")
        .expect("يجب قراءة examples/svg_charts_report.lsn");
    let svg_out = run(&svg_src).expect("يجب تشغيل مثال مخططات SVG بنجاح");
    assert!(!svg_out.is_empty());
    for tmp in [
        "تقرير_الأعمدة.svg",
        "تقرير_الحصص_الدائري.svg",
        "تقرير_النمو_الخطي.svg",
        "تقرير_بيانات_لسان.html",
        "لوحة_رسم_لسان.svg",
    ] {
        let _ = std::fs::remove_file(tmp);
    }
}

#[test]
fn test_2d_game_and_systems_cli_examples() {
    let game_src = std::fs::read_to_string("examples/game_2d_space_arabic.lsn")
        .expect("يجب قراءة examples/game_2d_space_arabic.lsn");
    let game_out = run(&game_src).expect("يجب تشغيل مثال اللعبة ثنائية الأبعاد بنجاح");
    assert!(!game_out.is_empty());
    let _ = std::fs::remove_file("لعبة_الفضاء_العربية.html");

    let sys_src = std::fs::read_to_string("examples/systems_cli_toolkit.lsn")
        .expect("يجب قراءة examples/systems_cli_toolkit.lsn");
    let sys_out = run(&sys_src).expect("يجب تشغيل مثال أدوات الأنظمة والطرفية بنجاح");
    assert!(!sys_out.is_empty());
}
