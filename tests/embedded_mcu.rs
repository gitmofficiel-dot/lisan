use lisan::run;
use lisan::arduino::{transpile_arduino, transpile_esp32, transpile_stm32};

#[test]
fn test_arduino_simulation_and_classes() {
    let code = r#"
استورد "اردوينو"

دع سيرفو = جديد محرك_سيرفو(9)
اطبع(سيرفو.حرك(135))

دع خطوي = جديد محرك_خطوي(2048، 4، 5، 6، 7)
اطبع(خطوي.حرك(500))

دع محرك = جديد متحكم_محركات_l298n(10، 2، 3، 11، 4، 5)
اطبع(محرك.تحريك_أمام(220))
اطبع(محرك.إيقاف())

دع زر = جديد زر_ضاغط(8)
اطبع(زر.نقرة())
"#;
    let output = run(code).expect("يجب تنفيذ محاكاة الأردوينو بنجاح");
    assert_eq!(output[0], "135");
    assert_eq!(output[1], "500");
    assert_eq!(output[2], "صحيح");
    assert_eq!(output[3], "صحيح");
}

#[test]
fn test_esp32_simulation_and_classes() {
    let code = r#"
استورد "esp32"

دع واي_فاي = جديد واي_فاي_esp()
واي_فاي.اتصل("منزل_ذكي"، "12345678")
اطبع(واي_فاي.عنوان_ip())
اطبع(واي_فاي.هل_متصل())

دع خادم = جديد خادم_ويب_esp(80)
خادم.عند_طلب("/", "مرحبا من ESP32")
اطبع(خادم.ابدأ())

دع لمس = جديد حساس_لمس_esp(4)
اطبع(لمس.هل_ملموس())

دع dac = جديد محول_رقمي_esp(25)
اطبع(dac.اكتب_جهد(2.5))
"#;
    let output = run(code).expect("يجب تنفيذ محاكاة ESP32 بنجاح");
    assert_eq!(output[0], "192.168.1.150");
    assert_eq!(output[1], "صحيح");
    assert_eq!(output[2], "صحيح");
    assert_eq!(output[3], "صحيح");
}

#[test]
fn test_stm32_simulation_and_classes() {
    let code = r#"
استورد "stm32"

دع مؤقت = جديد مؤقت_عتادي_stm32(1)
اطبع(مؤقت.اضبط_تردد(20000))
اطبع(مؤقت.اضبط_pwm(5، 75))

دع dac = جديد محول_رقمي_stm32(1)
اطبع(dac.اكتب_جهد(3.0))

دع كان = جديد ناقل_can_stm32(500)
اطبع(كان.أرسل_إطار(256، [1, 2, 3]))

دع rtc = جديد ساعة_الوقت_الحقيقي_rtc()
اطبع(rtc.اقرأ_وقت_نصي())
"#;
    let output = run(code).expect("يجب تنفيذ محاكاة STM32 بنجاح");
    assert_eq!(output[0], "20000");
    assert_eq!(output[1], "75");
    assert!(output[3].contains("صحيح"));
    assert_eq!(output[4], "12:0:0");
}

#[test]
fn test_transpiler_outputs_for_all_targets() {
    // 1. Arduino
    let code_ard = r#"
استورد "اردوينو"
دع سيرفو = جديد محرك_سيرفو(9)
سيرفو.حرك(90)
"#;
    let ino_ard = transpile_arduino(code_ard).unwrap();
    assert!(ino_ard.contains("#include <Servo.h>"));

    // 2. ESP32
    let code_esp = r#"
استورد "esp32"
دع واي_فاي = جديد واي_فاي_esp()
واي_فاي.اتصل("شبكة"، "سر")
"#;
    let ino_esp = transpile_esp32(code_esp).unwrap();
    assert!(ino_esp.contains("LISAN_TARGET_ESP32"));
    assert!(ino_esp.contains("#include <WiFi.h>"));

    // 3. STM32
    let code_stm = r#"
استورد "stm32"
دع مؤقت = جديد مؤقت_عتادي_stm32(2)
مؤقت.اضبط_تردد(1000)
"#;
    let ino_stm = transpile_stm32(code_stm).unwrap();
    assert!(ino_stm.contains("LISAN_TARGET_STM32"));
    assert!(ino_stm.contains("#include <HardwareTimer.h>"));
}

#[test]
fn test_esp32_iot_station_full_program() {
    let source = std::fs::read_to_string("examples/esp32_iot_station.lsn")
        .expect("يجب قراءة ملف examples/esp32_iot_station.lsn بنجاح");
    let output = run(&source).expect("يجب تشغيل محطة ESP32 بنجاح");
    assert!(output.iter().any(|line| line.contains("حالة الاتصال بالواي فاي: صحيح")));
    assert!(output.iter().any(|line| line.contains("broker.hivemq.com") || line.contains("MQTT")));

    let transpiled = transpile_esp32(&source).expect("يجب ترجمة كود ESP32 إلى C++ بنجاح");
    assert!(transpiled.contains("LISAN_TARGET_ESP32"));
    assert!(transpiled.contains("#include <WiFi.h>"));
    assert!(transpiled.contains("#include <PubSubClient.h>"));
}

#[test]
fn test_stm32_industrial_automation_full_program() {
    let source = std::fs::read_to_string("examples/stm32_industrial_automation.lsn")
        .expect("يجب قراءة ملف examples/stm32_industrial_automation.lsn بنجاح");
    let output = run(&source).expect("يجب تشغيل نظام أتمتة STM32 بنجاح");
    assert!(output.iter().any(|line| line.contains("20kHz")));
    assert!(output.iter().any(|line| line.contains("500kbps")));

    let transpiled = transpile_stm32(&source).expect("يجب ترجمة كود STM32 إلى C++ بنجاح");
    assert!(transpiled.contains("LISAN_TARGET_STM32"));
    assert!(transpiled.contains("#include <HardwareTimer.h>"));
    assert!(transpiled.contains("#include <STM32RTC.h>"));
    assert!(transpiled.contains("#include <HalCan.h>"));
}

