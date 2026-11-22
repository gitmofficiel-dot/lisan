use lisan::arduino::{transpile_arduino, transpile_esp32, transpile_stm32};
use lisan::raspberry::transpile_raspberry_python;
use lisan::run;

#[test]
fn test_all_hardware_showcase_runs_and_transpiles() {
    let source = std::fs::read_to_string("examples/all_hardware_showcase.lsn")
        .expect("يجب قراءة ملف examples/all_hardware_showcase.lsn");
    let output = run(&source).expect("يجب تنفيذ عرض جميع قطع العتاد بنجاح");
    assert!(!output.is_empty(), "يجب أن ينتج عرض العتاد مخرجات غير فارغة");

    let ino_ard = transpile_arduino(&source).expect("يجب ترجمة الكود إلى C++ للأردوينو");
    assert!(ino_ard.contains("#include <MFRC522.h>"));
    assert!(ino_ard.contains("#include <Keypad.h>"));
    assert!(ino_ard.contains("#include <HX711.h>"));
    assert!(ino_ard.contains("#include <MPU6050.h>"));

    let ino_esp = transpile_esp32(&source).expect("يجب ترجمة الكود إلى C++ لـ ESP32");
    assert!(ino_esp.contains("LISAN_TARGET_ESP32"));
    assert!(ino_esp.contains("#include <esp_now.h>"));
    assert!(ino_esp.contains("#include <esp_camera.h>"));

    let ino_stm = transpile_stm32(&source).expect("يجب ترجمة الكود إلى C++ لـ STM32");
    assert!(ino_stm.contains("LISAN_TARGET_STM32"));
    assert!(ino_stm.contains("#include <IWatchdog.h>"));
}

#[test]
fn test_raspberry_pi_smart_hub_runs_and_transpiles_to_valid_python() {
    let source = std::fs::read_to_string("examples/raspberry_smart_hub.lsn")
        .expect("يجب قراءة ملف examples/raspberry_smart_hub.lsn");
    let output = run(&source).expect("يجب تنفيذ مثال راسبيري باي بنجاح");
    assert!(!output.is_empty());

    let py_code = transpile_raspberry_python(&source)
        .expect("يجب ترجمة كود راسبيري باي إلى Python 3 بنجاح");
    assert!(py_code.starts_with("#!/usr/bin/env python3"));
    assert!(py_code.contains("coding: utf-8"));

    // التحقق من صحة قواعد Python 3 المولدة إن كان مفسر python3 متوفراً
    let tmp_py = std::env::temp_dir().join(format!("lisan_rpi_check_{}.py", std::process::id()));
    std::fs::write(&tmp_py, &py_code).unwrap();
    if let Ok(status) = std::process::Command::new("python3")
        .arg("-m")
        .arg("py_compile")
        .arg(&tmp_py)
        .status()
    {
        assert!(status.success(), "كود Python 3 المولد لراسبيري باي يجب أن يكون سليم القواعد 100%");
    }
    let _ = std::fs::remove_file(&tmp_py);
}

#[test]
fn test_rasspiry_alias_and_all_rpi_peripherals() {
    let code = r#"
استورد "rasspiry"

دع لوحة = جديد لوحة_راسبيري("Raspberry Pi 5")
اطبع(لوحة.اسم_الطراز())
اطبع(لوحة.عدد_الأنوية())

دع منفذ = جديد منفذ_gpio_راسبيري(17، "مخرج")
اطبع(منفذ.شغّل())

دع كاميرا = جديد كاميرا_راسبيري(1920، 1080)
كاميرا.ابدأ()
اطبع(كاميرا.التقط_صورة("frame.jpg"))

دع قبعة = جديد قبعة_الحساسات_sensehat()
اطبع(قبعة.اقرأ_حرارة())

دع محركات = جديد قبعة_محركات_pca9685(64)
اطبع(محركات.اضبط_زاوية_سيرفو(0، 90))
"#;
    let out = run(code).expect("يجب استيراد وتشغيل rasspiry بنجاح");
    assert_eq!(out[0], "Raspberry Pi 5");
    assert_eq!(out[1], "4");
}

#[test]
fn test_language_complete_showcase_crypto_regex_db_platform() {
    let source = std::fs::read_to_string("examples/language_complete_showcase.lsn")
        .expect("يجب قراءة ملف examples/language_complete_showcase.lsn");
    let out = run(&source).expect("يجب تنفيذ مثال اللغة المتكامل (تشفير، أنماط، قاعدة بيانات) بنجاح");
    assert!(!out.is_empty());
}

#[test]
fn test_desktop_studio_api_endpoints() {
    let (status, ctype, body) = lisan::desktop::handle_http_request("GET", "/", "");
    assert_eq!(status, 200);
    assert!(ctype.contains("text/html"));
    assert!(String::from_utf8_lossy(&body).contains("لسان"));

    let (status, _, info_body) = lisan::desktop::handle_http_request("GET", "/api/info", "");
    assert_eq!(status, 200);
    let info_str = String::from_utf8_lossy(&info_body);
    assert!(info_str.contains("1.1.0"));

    let run_req = r#"{"code": "اطبع(10 + 32)"}"#;
    let (status, _, run_body) = lisan::desktop::handle_http_request("POST", "/api/run", run_req);
    assert_eq!(status, 200);
    let run_str = String::from_utf8_lossy(&run_body);
    assert!(run_str.contains("42"));

    let rpi_req = r#"{"target": "raspberry", "code": "استورد \"راسبيري\"\nدع م = جديد منفذ_gpio_راسبيري(18)\nم.شغّل()"}"#;
    let (status, _, rpi_body) = lisan::desktop::handle_http_request("POST", "/api/transpile", rpi_req);
    assert_eq!(status, 200);
    let rpi_str = String::from_utf8_lossy(&rpi_body);
    assert!(rpi_str.contains("python3"));
}
