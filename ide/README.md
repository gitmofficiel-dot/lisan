# استوديو لسان — Lisan Studio IDE

بيئة التطوير الرسمية المعدّلة من **Code-OSS / VSCodium** (مفتوحة المصدر برخصة MIT، بلا تتبّع) ومخصّصة بالكامل للغة البرمجة العربية **لسان (Lisan v1.2.0)**.

## الميزات المدمجة افتراضياً
- **هوية عربية كاملة (`product.patch.json`):** الاسم `استوديو لسان — Lisan Studio`، أمر الطرفية `lisan-studio` / `lisan-code`، الشعار الرسمي، متجر Open VSX المفتوح، وبدون أي تتبّع (`enableTelemetry: false`).
- **إضافة لسان الكاملة v1.2.0 مدمجة داخل المحرر:** تلوين نحوي عربي، خادم اللغة `LSP`، مصحح الأخطاء `DAP (F5)`، فحص العربية الخالصة `ل0001` وإصلاحات سريعة، 11 أمراً عربياً، 23 مقتطفاً، معاينة ويب/هاتف حية، ورفع للمتحكمات.
- **سمات وواجهة عربية (`extensions/lisan-studio-experience/`):**
  - سمة **ليلي لسان (Lisan Midnight)** وسمة **نهاري لسان (Lisan Daylight)**.
  - **لوحة الترحيب العربية** لإنشاء مشاريع الويب والهاتف والذكاء الاصطناعي والمتحكمات بضغطة واحدة، ونسخ إعداد خادم وكلاء الذكاء `lisan --mcp`.
  - دعم حزمة اللغة العربية (`ms-ceintl.vscode-language-pack-ar`) وإلغاء تحذيرات Unicode المزعجة على المعرّفات العربية.

## البناء والتثبيت المحلي (Linux)
```bash
./ide/build-ide.sh \
  --source-tar /path/to/VSCodium-linux-x64.tar.gz \
  --target-dir /media/mohammed/External/Lisan-Studio \
  --install-desktop
```

## البناء السحابي لجميع الأنظمة (Linux / Windows / macOS)
يتوفر سير عمل GitHub Actions الجاهز في `.github/workflows/build-lisan-studio-ide.yml` (يعمل يدوياً عبر `workflow_dispatch` أو عند دفع وسم `v*-ide`).
