# لسان — Lisan Programming Language

**لسان 1.2.0 — لغة البرمجة العربية الشاملة والمنظومة العتادية والسحابية**

[![CI](https://github.com/gitmofficiel-dot/lisan/actions/workflows/rust.yml/badge.svg)](https://github.com/gitmofficiel-dot/lisan/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/gitmofficiel-dot/lisan)](https://github.com/gitmofficiel-dot/lisan/releases/latest)
[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/gitmofficiel-dot/lisan)

## English summary

**Lisan (لسان)** is an Arabic programming language written in Rust. You write code with Arabic keywords, Arabic identifiers and right-to-left text. It accepts Western (`0-9`), Arabic-Indic (`٠-٩`) and Persian (`۰-۹`) digits. Programs run on a built-in interpreter or a VM, and can be compiled natively through LLVM/Clang. It targets Linux, macOS and Windows.

**Quick start**

```sh
# Linux / macOS / Raspberry Pi
curl -sSf https://raw.githubusercontent.com/gitmofficiel-dot/lisan/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/gitmofficiel-dot/lisan/main/install.ps1 | iex
```

Or build from source (needs a Rust toolchain):

```sh
git clone https://github.com/gitmofficiel-dot/lisan.git
cd lisan
cargo run -- examples/hello.lsn
```

Prebuilt packages are on the [Releases page](https://github.com/gitmofficiel-dot/lisan/releases/latest). Docs are in [`docs/`](docs/), starting with [SYNTAX](docs/SYNTAX.md). To contribute, see [CONTRIBUTING.md](CONTRIBUTING.md).

---

**لسان (Lisan)** هي لغة برمجة عربية فصيحة وعصرية عالية الأداء تجمع بين سهولة وخفة بايثون، وقوة وأمان لغة **Rust**، وسرعة البرامج المترجمة عبر **LLVM/Clang**، مع دعم أصيل لأنظمة **Linux** و**Windows** و**macOS**، واللوحات المدمجة **Arduino** و**ESP32** و**STM32** و**Raspberry Pi**.

---

## التثبيت والتشغيل السريع

### التثبيت الفوري بأمر واحد
* **Linux وmacOS وRaspberry Pi (`x86_64` و`ARM64`):**
  ```sh
  curl -sSf https://raw.githubusercontent.com/gitmofficiel-dot/lisan/main/install.sh | sh
  ```
* **Windows (PowerShell):**
  ```powershell
  irm https://raw.githubusercontent.com/gitmofficiel-dot/lisan/main/install.ps1 | iex
  ```
* **حزم الإصدار الجاهزة:** راجع صفحة [Releases (`v1.2.0`)](https://github.com/gitmofficiel-dot/lisan/releases/tag/v1.2.0) لتحميل حزم `.deb` و`.tar.gz` و`.zip` وتطبيق سطح المكتب `Lisan Studio`.

### التشغيل والبناء من المصدر
```sh
cargo test --all-targets
cargo build --release
cargo run -- examples/dictionary.lsn
cargo run -- examples/standard_library.lsn
cargo run -- --check examples/modules/main.lsn
cargo run -- --test examples/testing.lsn
cargo run -- --tokens examples/hello.lsn
cargo run -- --ast examples/hello.lsn
```

---

## تجربة الكتابة العربية والأرقام

* **دعم جميع الأرقام:** تدعم الشيفرة الأرقام الغربية (`0-9`) والأرقام العربية (`٠-٩`) والفارسية (`۰-۹`)، مع دعم النقطة العشرية (`.`) والفاصلة العشرية العربية (`٫`).
* **تطبيع الكلمات العربية:** يطبّع المحلل المعجمي الفروق الشائعة في الكلمات المفتاحية (مثل الهمزات `أ/إ/آ`، التاء المربوطة `ة/ه`، الألف المقصورة `ى/ي`، والتشكيل والتطويل).
* **كتابة حرة بدون فواصل منقوطة إلزامية:** الفاصلة المنقوطة `؛` اختيارية، ويمكن كتابة كل أمر في سطر منفصل أو الجمع بين أوامر متعددة في سطر واحد باستخدام `؛`.
* **تتبع الأخطاء والاقتراحات الذكية (Diagnostics):** تقارير أخطاء فصيحة تحدد السطر والعمود وسلسلة الاستدعاءات، مع اقتراحات تصحيح إملائي لأسماء المتغيرات والدوال ومفاتيح القواميس.

---

## الميزات الكبرى في اللغة (تكافؤ بايثون)

### التقطيع العصري (Slicing)
تقطيع القوائم والنصوص والصفوف مع دعم الخطوات والمؤشرات السالبة:
```lisan
دع أرقام = [10, 20, 30, 40, 50]
اطبع(أرقام[1:4])          # [20, 30, 40]
اطبع(أرقام[:3])           # [10, 20, 30]
اطبع(أرقام[2:])           # [30, 40, 50]
اطبع(أرقام[::-1])         # [50, 40, 30, 20, 10]
اطبع("مرحبا بالعالم"[0:5]) # "مرحبا"
```

### التوليد المختصر (List Comprehensions)
```lisan
دع مضاعفات = [س * 2 لكل س في مدى(5)]
دع أعداد_زوجية = [س لكل س في أرقام إذا س % 2 == 0]
دع حروف = [ح لكل ح في "لسان"]
```

### المعامل `في` و`ليس في` (`in` / `not in`)
```lisan
إذا ("تفاح" في سلة) { اطبع("موجود") }
إذا ("عمر" ليس في قاموس_المستخدمين) { اطبع("مستخدم جديد") }
إذا ("عربي" في "محتوى عربي أصيل") { اطبع("صحيح") }
```

### المعاملات الافتراضية ووسائط الدوال المتغيرة (`*args`)
```lisan
دالة تحية(اسم, لقب = "سيد") {
    أرجع "مرحبا " + لقب + " " + اسم
}
اطبع(تحية("أحمد"))             # مرحبا سيد أحمد
اطبع(تحية("فاطمة", "أستاذة"))   # مرحبا أستاذة فاطمة

دالة مجموع_شامل(*أرقام) {
    دع إجمالي = 0
    لكل ر في أرقام { إجمالي = إجمالي + ر }
    أرجع إجمالي
}
اطبع(مجموع_شامل(1, 2, 3, 4, 5)) # 15
```

### هياكل البيانات المرجعية والثابتة
* **القوائم والقواميس المرجعية:** تعديل مباشر في الذاكرة ومشاركة المراجع (`ب = أ؛ ب.أضف(10)` يعدّل `أ` في مكانها)، مع الطرق `.أضف()` و`.احذف()` و`.أدرج()` و`.تمديد()` و`.مسح()` و`.نسخ()`.
* **الصفوف (`صف` / Tuples):** بنية ثابتة غير قابلة للتعديل (`ص = صف(1, 2, 3)`).
* **المجموعات (`مجموعة` / Sets):** عناصر فريدة غير مكررة (`م = مجموعة([1, 2, 2, 3])`).

### دوال بايثون القياسية وفك التعبئة
* الدوال: `أي(قائمة)`، `كل(قائمة)`، `مرقم(قائمة)`، `سحاب(قائمة1, قائمة2)`.
* فك التعبئة المباشر وفي الحلقات:
  ```lisan
  دع أ، ب = [10, 20]
  أ، ب = [ب، أ]
  لكل مفتاح، قيمة في قاموس.عناصر() { ... }
  لكل ترقيم، عنصر في مرقم(قائمة) { ... }
  لكل س، ص في سحاب(ق1, ق2) { ... }
  ```

### النصوص المنسقة (`f-strings`) ومدير السياق (`مع`)
```lisan
دع اسم = "زيد"
دع عمر = 25
اطبع(ن"مرحبا {اسم}، عمرك بعد 5 سنوات: {عمر + 5}")
اطبع(f"المجموع: {10 * 5} ريال")

مع افتح("بيانات.txt", "ك") كـ ملف {
    ملف.اكتب("مرحبا يا لسان")
}
```

### التحكم بالتدفق والاستثناءات والوحدات
* **الاستثناءات والمطابقة:** `حاول` / `التقط` / `أخيرًا` / `ارمِ`، وبنية `اختر` / `حالة` / `افتراضي`.
* **الوحدات والأسماء المستعارة:**
  ```lisan
  استورد رياضيات كـ ر
  اطبع(ر.جذر(25)) # 5
  من رياضيات استورد جا، جتا، ط
  اطبع(جا(0))     # 0
  ```
* **أداء المكدس:** رفع حد المكدس إلى 512 ميغابايت لدعم الخوارزميات المتداخلة ومعالجة البيانات الكبيرة.

---

## المكتبات القياسية المدمجة

* **الرياضيات والإحصاء (`رياضيات`، `إحصاء`):** `جا`، `جتا`، `ظا`، `لوغاريتم`، `أسي`، `ط`، `هـ`، `مضروب`، `توافيق`، `قاسم_مشترك`، `متوسط`، `وسيط`، `منوال`، `انحراف_معياري`، بالإضافة إلى `أرضية` و`سقف` و`قوة` و`مجموع`.
* **العشوائية (PRNG SplitMix64):** `عشوائي()`، `عدد_عشوائي(أدنى، أقصى)`، `عنصر_عشوائي(قائمة)`، `خلط(قائمة)`.
* **التاريخ والتقويم (`تاريخ`):** `تاريخ_اليوم()`، `الآن()`، `يوم_الأسبوع()`، `فرق_أيام()`، `أضف_أيام()`.
* **تنسيقات البيانات:** `تحليل_json()` و`ترميز_json()`، ومكتبة `csv` المتوافقة مع RFC 4180 (`تحليل_csv` و`ترميز_csv`).
* **الشبكات والاتصالات (`شبكة`):** `طلب_وب()` (`HTTP GET`)، مقابس `TCP`، وخادم ويب مصغر.
* **النصوص والتشفير والأمان (`نصوص`، `تشفير_أمان`، `محلل_أنماط`):** `ترميز_قاعدة64`، `فك_قاعدة64`، `ترميز_url`، `تجزئة_sha256`، `توقيع_hmac`، `معرف_فريد()` (UUID v4)، `شفر_نص` / `فك_تشفير_نص`، والتعابير النمطية (`يطابق_نمط`، `استخرج_نمط`، `استبدل_نمط`).
* **قاعدة البيانات المحلية (`قاعدة_بيانات`):** جداول وسجلات مستديمة بصيغة JSON (`أنشئ_جدول`، `أدرج`، `الكل`) مع `معلومات_النظام()`.
* **إطار الويب الكامل وتطبيقات الهاتف 2.0 (`إطار_ويب`، `موقع`، `هاتف`، `libs/ui/lisan_ui.lsn`):** خادم HTTP متعدد المسارات (`تطبيق_ويب`)، محرك قوالب آمن (`قالب_صفحة` / `عرض_قالب`)، عميل HTTP متقدم (`طلب_http_متقدم` لـ GET/POST/PUT/DELETE)، منشئ مواقع متجاوبة (جداول أسعار، أسئلة شائعة، شهادات، معارض)، ومنشئ تطبيقات هاتف PWA 2.0 متعدد الشاشات مع شريط سفلي وزر عائم ونافذة سفلية و`sw.js` للعمل دون إنترنت.
* **الذكاء الاصطناعي وعلوم البيانات ومعالجة العربية ومخططات SVG (`ذكاء` / `بيانات` / `مخططات`):** جبر المصفوفات (`مصفوفة`)، جداول البيانات (`جدول_بيانات` شبيه Pandas)، الانحدار الخطي (`انحدار_خطي`)، مصنّف KNN (`مصنف_جيران_knn`)، شبكة عصبية اصطناعية حقيقية بالانتشار العكسي (`شبكة_عصبية`)، معالجة النصوص العربية وإزالة التشكيل وتحليل المشاعر (`محلل_نص_عربي`)، ورسم مخططات SVG عربية (`مخطط_أعمدة`، `مخطط_خطي`، `مخطط_دائري`، `لوحة_رسم_svg`).
* **أدوات الأنظمة والطرفية ومحرك الألعاب ثنائي الأبعاد (`أنظمة` / `ألعاب`):** عمليات المسارات والملفات (`مسار_ادمج`، `انسخ_ملف`، `انقل_ملف`، `حجم_ملف`)، تنفيذ أوامر النظام (`نفّذ_أمر`)، بصمة IEEE `بصمة_crc32`، ضغط `ضغط_rle`، أدوات CLI (`أداة_سطر_أوامر`، `سجل_أحداث`، `مجدول_مهام`)، ومحرك ألعاب أركيد HTML5 Canvas ثنائي الأبعاد عربي (`لعبة_ثنائية_الأبعاد`).
* **الإغلاقات المعجمية والاستدعاءات المتسلسلة (Closures & Chained Calls):** دعم كامل للدوال كقيم من الدرجة الأولى، التقاط المتغيرات الخارجية داخل الدوال الداخلية، واستدعاء أي تعبير مباشرة مثل `ف(1)(2)` و`ق[0](5)`.

---

## منظومة العتاد والأنظمة المدمجة والروبوتات (105 وحدة عتادية وبروتوكول)

* **Arduino (`استورد "اردوينو"` — 38 صنفاً):** حساسات (MFRC522، MPU6050، BME280، HX711، TCS3200، MAX30102، PIR، MQ، GPS، رطوبة، مطر، لهب، مسافة، حرارة، صوت، تيار، ألوان، لوحة مفاتيح، عصا تحكم)، وشاشات ومشغلات (SSD1306، LCD، TM1637، MAX7219، NeoPixel، Servo، Stepper، L298N، Relay، SD، EEPROM، I2C، SPI).
  ```sh
  lisan --arduino ملف.lsn --output sketch.ino
  ```
* **ESP32 / ESP8266 (`استورد "esp32"` — 18 صنفاً):** ESP32-CAM، ESP-NOW، HTTP، WebSockets، NVS، NTP، FreeRTOS، Bluetooth/BLE، OTA، Wi-Fi، خادم ويب، MQTT، SPIFFS، نوم عميق، ADC/DAC.
  ```sh
  lisan --esp32 ملف.lsn --output esp_sketch.ino
  ```
* **STM32 (`استورد "stm32"` — 15 صنفاً):** DMA، IWDG، EXTI، مشفر عتادي، Flash، USB CDC، SPI، I2C، مؤقت، ADC، 12-bit DAC، CAN Bus، RTC، توفير الطاقة، UART.
  ```sh
  lisan --stm32 ملف.lsn --output stm32_sketch.ino
  ```
* **Raspberry Pi (`استورد "راسبيري"` — 12 صنفاً):** لوحة راسبيري، GPIO، PWM، I2C، SPI، UART، Picamera2، SenseHAT، PCA9685، شاشة لمس DSI، صوت مع نطق عربي TTS، وشاشة حبر إلكتروني e-Paper.
  ```sh
  lisan --raspberry ملف.lsn --output app_rpi.py
  ```
* **Raspberry Pi Pico / Pico W — RP2040 (`استورد "بيكو"` — 12 صنفاً):** لوحة بيكو، GPIO، PWM 16-bit، ADC، I2C، SPI، UART، Wi-Fi Pico W، NeoPixel، مؤقتات، وحدة PIO القابلة للبرمجة، ومستشعر الحرارة الداخلي (مترجم إلى MicroPython).
  ```sh
  lisan --pico ملف.lsn --output main_pico.py
  ```
* **الروبوتات ونظرية التحكم والبروتوكولات الصناعية (`استورد "روبوتات"` — 10 أصناف):** `متحكم_pid`، `مرشح_كالمان`، `روبوت_متحرك` (حركية وأودومتري)، `ذراع_آلي` (2-DOF)، `آلة_حالات`، `مودباس_صناعي` (Modbus RTU/TCP)، `لورا_lora` (SX1278)، `شريحة_اتصال_gsm` (SIM800L/SIM7600)، `سائق_خطوي_a4988`، و`مراقب_بطارية_bms`.

---

## أدوات التطوير، الاستوديو، والترجمة

* **خادم اللغة الذكي (LSP):** تشخيص فوري، توثيق عند التحويم، وإكمال تلقائي لـ VS Code وNeovim عبر `lisan --lsp` أو `lisan lsp`.
* **مدير الحزم (`lisan حزمة` / `lisan --pkg`):** تهيئة المشاريع (`تهيئة`)، إضافة المكتبات (`أضف`)، وتثبيت الاعتماديات (`قفل` عبر `lisan.json` و`lisan.lock`).
* **منسّق الشيفرة (`lisan نسّق` / `lisan --fmt`):** تنسيق آلي وفحص `--فحص` في مسارات CI.
* **مشغّل الاختبارات (`lisan اختبار` / `lisan --test`):** اكتشاف وتشغيل اختبارات `اختبار_*()` والتحققات `تحقق_يساوي()`.
* **استوديو لسان لسطح المكتب:** يعمل عبر `lisan استوديو` أو `lisan --studio` (و`lisan منصات` لعرض المنصات المدعومة).
* **الترجمة الأصلية (AOT via LLVM/Clang) وWebAssembly:**
  ```sh
  cargo run -- --native examples/native.lsn --output native-program
  cargo run -- --emit-llvm examples/native.lsn --output native.ll
  cargo run -- --build examples/modules/main.lsn --output app
  ```
* **التطوير السحابي عبر GitHub Codespaces:** حاوية مهيأة مسبقاً مع Rust و`lisan` وإضافة VS Code و`arduino-cli` و`gcc-arm-none-eabi` و`esptool` وهدف `wasm32-unknown-unknown`.

---

## الوثائق وخارطة الطريق

* سجل التغييرات الكامل: [CHANGELOG.md](CHANGELOG.md)
* خارطة الطريق المرحلية: [docs/ROADMAP.md](https://github.com/gitmofficiel-dot/lisan/blob/main/docs/ROADMAP.md)
* خط الأساس ومعيار القبول: [docs/COMPLETION-BASELINE.md](https://github.com/gitmofficiel-dot/lisan/blob/main/docs/COMPLETION-BASELINE.md)
* مسارات التنفيذ والترجمة: [docs/EXECUTION-PATHS.md](https://github.com/gitmofficiel-dot/lisan/blob/main/docs/EXECUTION-PATHS.md)
* مرجع النحو: `docs/SYNTAX.md` | سياسة الأرقام والأسلوب: `docs/NUMBERS-AND-STYLE.md`

### قيد التطوير للإصدارات القادمة
* توسيع المترجم الأصلي (`--native`) ليشمل القواميس والقوائم مختلطة الأنواع وتمرير النصوص والقوائم للدوال ونظام الأنواع الكامل.
* توسيع مكتبات الشبكات المتقدمة.
* إضافة واجهات الربط المباشر (`FFI`) مع مكتبات **C** و**Rust**.

**تنبيه أمني:** دوال الملفات (`اقرأ_ملف` و`اكتب_ملف` و`افتح`) تعمل بصلاحيات العملية الحالية؛ شغّل البرامج الموثوقة فقط.

**الترخيص:** MIT
