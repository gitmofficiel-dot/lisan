# AGENTS.md — Authoritative Guide for AI Coding Agents Writing Lisan (لسان)

This repository implements **Lisan (لسان, v1.1.0)**, a modern, full-stack, **100% Arabic-only** programming language compiled and interpreted in Rust. This guide applies to **all AI coding agents** (Claude, Gemini, GPT/Codex, GitHub Copilot, Cursor, Antigravity, and MCP clients).

---

## 1. The Golden Rule: 100% Arabic-Only Keywords (Zero English Keywords!)

1. **NEVER emit English keywords** (`if`, `else`, `elif`, `def`, `function`, `return`, `import`, `from`, `as`, `class`, `while`, `for`, `in`, `print`, `true`, `false`, `null`, `None`, `let`, `var`, `try`, `except`, `catch`, `finally`, `raise`, `throw`, `and`, `or`, `not`, `break`, `continue`, `with`, `match`, `case`, `default`, `new`, `self`, `this`, `extends`).
2. **Foreign library names appear ONLY inside string literals**, e.g.:
   ```lisan
   دع مصفوفات = استورد_بايثون("numpy")
   دع بيانات = استورد_بايثون("pandas")
   دع مكتبة = استورد_جافاسكريبت("lodash")
   ```
3. **Prefer Arabic identifiers** (`النتيجة`, `المجموع`, `المستخدم`) and Arabic commas `،` (both `،` and `,` are accepted by the lexer, but `،` is canonical).
4. **Always verify before returning code**: run `lisan --check` / `lisan --test` / `lisan --interpret` (or MCP tools `lisan_check` and `lisan_run`) and never fabricate program output.

---

## 2. Exact Python → Lisan Mapping Table

| Python | Pure Arabic Lisan (لسان) | Notes |
| --- | --- | --- |
| `x = 10` | `دع س = 10` | Also `ليكن`, `متغير`. Reassignment omits `دع`: `س = 20` |
| `a, b = [1, 2]` | `دع أ، ب = [1، 2]` | Tuple/list unpacking |
| `x += 1` | `س = س + 1` | **No `+=` / `-=` / `*=` operators** — write full assignment |
| `print(x)` | `اطبع(س)` | **Single argument only!** Multi-arg: `اطبع(نص(أ) + " " + نص(ب))` or `ن"{أ} {ب}"` |
| `f"Hi {name}"` | `ن"مرحبا {الاسم}"` | Always double quotes `"..."`; single quotes `'...'` are invalid |
| `if c:` | `إذا (شرط) { ... }` | Parentheses `(...)` and braces `{...}` are required |
| `elif c2:` | `} وإلا { إذا (شرط2) { ... } }` | **`وإلا` MUST be followed by `{`** (never bare `وإلا إذا`) |
| `else:` | `} وإلا { ... }` | |
| `while c:` | `طالما (شرط) { ... }` | `بينما` is an exact synonym |
| `for x in xs:` | `لكل (س في قائمة) { ... }` | Also `لكل س في قائمة { ... }` and unpacking `لكل ك، ق في أزواج { }` |
| `range(n)` | `مدى(ن)` | `مدى(بداية، نهاية، خطوة)` supports negative steps |
| `break` / `continue` | `توقف` / `تابع` | (`استمر` is also accepted as `continue`) |
| `def f(a, b=0):` | `دالة اسم(أ، ب = 0) { ... }` | Varargs: `دالة جمع(*أرقام) { ... }` |
| `def sq(x): return x*x` | `دالة مربع(س) => س * س` | Arrow shorthand for single-expression functions |
| `lambda x: x + 1` | `دالة(س) => س + 1` | First-class functions with lexical closures & `ف(1)(2)` chaining |
| `return val` | `أرجع قيمة` | |
| `class A(B):` | `صنف أ يرث ب { ... }` | Methods use `دالة` without a `self` parameter |
| `def __init__(self, x):` | `دالة بناء(س) { هذا.س = س }` | Constructor is `بناء`; instance reference is `هذا` |
| `A(1)` | `جديد أ(1)` | Instantiate class with `جديد` |
| `True` / `False` / `None` | `صحيح` / `خطأ` / `لاشيء` | |
| `and` / `or` / `not` | `و` / `أو` / `ليس` | Also `&&`, `||`, `!` |
| `x in xs` / `x not in xs` | `س في قائمة` / `س ليس في قائمة` | Works on lists, strings, dicts, sets |
| `[x*2 for x in xs if x>0]` | `[س * 2 لكل س في قائمة إذا س > 0]` | No parens needed after `إذا` inside comprehensions |
| `try / except E as e / finally` | `حاول { ... } التقط (رسالة) { ... } أخيرًا { ... }` | **NEVER name the catch variable `خطأ`** (see §3!) |
| `raise "msg"` | `ارمِ "رسالة"` | |
| `match x: case 1: ...` | `اختر (قيمة) { حالة 1: { ... } افتراضي: { ... } }` | |
| `with open(...) as f:` | `مع افتح("ملف.txt"، "ك") كـ م { ... }` | |
| `import math as m` | `استورد رياضيات كـ ر` | File import: `استورد "ملف.lsn"` |
| `from math import sqrt, pi` | `من رياضيات استورد جذر، ط` | |
| `# comment` | `// تعليق` or `# تعليق` | Both single-line comment styles work |
| `len / str / int / float / sum` | `طول / نص / عدد_صحيح / عدد / مجموع` | Also `نوع`, `مطلق`, `جذر`, `فرز`, `عكس`, `سحاب`, `مرقم`, `أي`, `كل` |

---

## 3. Reserved Words — NEVER Use as Variable or Parameter Names

Lisan normalizes Arabic hamza/tāʾ marbūṭa/alif maqṣūra on keywords, so **all of these are reserved** and will trigger a parse error if used as identifiers:

> `حالة`, `افتراضي`, `و`, `أو`, `خطأ`, `صحيح`, `لاشيء`, `فارغ`, `في`, `من`, `مع`, `باسم`, `كـ`, `كاسم`, `فئة`, `هيكل`, `صنف`, `يرث`, `يمتد`, `جديد`, `دع`, `ليكن`, `متغير`, `دالة`, `أرجع`, `إذا`, `وإلا`, `إلا`, `طالما`, `بينما`, `لكل`, `توقف`, `تابع`, `استمر`, `حاول`, `التقط`, `أمسك`, `أخيرًا`, `نهائيا`, `ارمِ`, `ألقِ`, `أطلق`, `اختر`, `استورد`, `استيراد`, `اطبع`, `ليس`

**Most common naming traps & safe replacements:**
- ❌ `التقط (خطأ)` → ✅ `التقط (رسالة)` or `التقط (عطل)` (`خطأ` is the boolean literal `false`!)
- ❌ `دع حالة = "جاهز"` → ✅ `دع الوضع = "جاهز"` (`حالة` is the `case` keyword!)
- ❌ `دع افتراضي = 0` → ✅ `دع قيمة_أساسية = 0` (`افتراضي` is the `default` keyword!)
- ❌ `دع فئة = "أ"` → ✅ `دع تصنيف = "أ"` (`فئة` is a synonym for `صنف`!)
- ❌ `دع فارغ = صحيح` → ✅ `دع خالٍ = صحيح` (`فارغ` is a synonym for `لاشيء`!)

---

## 4. Pure Arabic Standard Library & Hardware Module Aliases

### Built-in Runtime Modules (`استورد <اسم> كـ <اختصار>` or `من <اسم> استورد ...`)
- `رياضيات` — `جذر`, `جا`, `جتا`, `ظا`, `لوغاريتم`, `أسي`, `مضروب`, `توافيق`, `تباديل`, `قاسم_مشترك`, `مضاعف_مشترك`, `تقارب`, `ط`, `هـ`
- `إحصاء` — `متوسط`, `وسيط`, `منوال`, `تباين`, `انحراف_معياري`, `ارتباط`
- `نصوص` — `ترميز_قاعدة64`, `فك_قاعدة64`, `ترميز_url`, `فك_url`
- `ملفات` — `افتح`, `اقرأ_ملف`, `اكتب_ملف`, `ألحق_ملف`, `ملف_موجود`, `مجلد_موجود`, `أنشئ_مجلد`, `قائمة_ملفات`, `احذف_ملف`
- `نظام` — `نظام_تشغيل`, `معلومات_النظام`, `مسار_حالي`
- `وقت` / `تاريخ` — `طابع_زمني`, `انتظر_ملي`, `تاريخ_اليوم`, `فرق_أيام`, `أضف_أيام`, `يوم_الأسبوع`
- `عشوائي` — `عشوائي`, `عدد_عشوائي`, `عنصر_عشوائي`, `خلط`
- `جيسون` — `تحليل_جيسون`, `إلى_جيسون`
- `شبكة` — `طلب_وب`

### Embedded Ecosystem & Hardware Modules (`استورد "<اسم_عربي>"`)
Always use the **pure Arabic module name** inside `استورد "..."`:

| Domain | Pure Arabic Import | Key Classes / Capabilities |
| --- | --- | --- |
| Web Framework | `استورد "إطار_ويب"` | `تطبيق_ويب`, `قالب_صفحة`, `عميل_واجهة_برمجية` |
| Reactive Web, WebSocket & Capacitor/Tauri/Figma | `استورد "ويب_حديث"` | `مكون_تفاعلي`, `تطبيق_تفاعلي`, `قناة_مقبس_حي` (RFC 6455), `تصدير_كاباسيتور`, `تصدير_تاوري`, `تصدير_فيجما` |
| Responsive Website | `استورد "موقع"` | `موقع_ويب` (nav, hero, grids, pricing, tables, forms, HTML export) |
| Mobile PWA 2.0 | `استورد "هاتف"` | `تطبيق_هاتف` (multi-screen, tabs, FAB, bottom sheets, PWA export) |
| AI / ML / Data / SVG | `استورد "ذكاء"` | `مصفوفة`, `جدول_بيانات`, `انحدار_خطي`, `مصنف_الجيران_الأقرب`, `شبكة_عصبية`, `محلل_نص_عربي`, `مخطط_أعمدة`, `مخطط_خطي`, `مخطط_دائري`, `لوحة_رسم_متجه` |
| AI RAG, Vector Store, Agents & Unified Hardware | `استورد "وكلاء"` | `تضمين_نصي`, `مخزن_متجهات`, `محرك_استرجاع_وتوليد` (RAG), `ذاكرة_محادثة`, `وكيل_ذكي`, `موجه_بايتورش`, `لوحة_موحدة` |
| Python/JS Interop & HTTPS LLM Client | `استورد "توافق"` | `مكتبة_بايثون`, `مكتبة_جافاسكربت`, `نموذج_ذكاء` (Gemini/OpenAI/Claude/Ollama), `طلب_شبكي_آمن` (HTTPS/TLS) |
| Async Concurrency & Parallel Tasks | `استورد "تزامن"` | `مهمة`, `وعد`, `قناة`, `حلقة_أحداث`, `دالة_غير_متزامنة`, `انتظر`, `تشغيل_متوازٍ`, `طلب_شبكي_متوازي` |
| Native C ABI FFI | `استورد "ربط_خارجي"` | `مكتبة_خارجية`, `مكتبة_نظام_متوفرة`, `استدعِ_دالة_نظام` (`libm` / `libc` / shared libs) |
| DB / Crypto / Regex | `استورد "قاعدة_بيانات"` | `قاعدة_بيانات`, `تشفير_أمان` (SHA-256/HMAC/Cipher/UUID), `محلل_أنماط` (Regex) |
| Systems & 2D Games | `استورد "ألعاب"` / `استورد "أنظمة"` | `لعبة_ثنائية_الأبعاد` (HTML5 Canvas), `أداة_سطر_أوامر`, `سجل_أحداث`, `مجدول_مهام`, `بصمة_crc32`, `ضغط_rle` |
| Robotics & Industrial | `استورد "روبوتات"` | `متحكم_pid`, `مرشح_كالمان`, `روبوت_متحرك`, `ذراع_آلي`, `آلة_حالات`, `مودباس_صناعي`, `لورا_lora`, `شريحة_اتصال_gsm`, `سائق_خطوي_a4988`, `مراقب_بطارية_bms` |
| Arduino (38 classes) | `استورد "اردوينو"` | `محرك_سيرفو`, `حساس_مسافة`, `حساس_حرارة`, `شاشة_oled`, `قارئ_rfid`, `محدد_مواقع_gps`, `حساس_تسارع_جيروسكوب`, `خلية_وزن_hx711`, ... |
| ESP32 (18 classes) | `استورد "اي_اس_بي32"` | `واي_فاي_esp`, `خادم_ويب_esp`, `عميل_mqtt`, `بلوتوث_ble`, `تحديث_هوائي_ota`, `اتصال_مباشر_espnow`, `كاميرا_esp32`, `مهمة_متوازية_rtos`, ... |
| STM32 (15 classes) | `استورد "اس_تي_ام32"` | `مؤقت_عتادي_stm32`, `ناقل_can_stm32`, `ناقل_مباشر_dma`, `ساعة_الوقت_الحقيقي_rtc`, `مراقب_النظام_iwdg`, `مقاطعة_خارجية_exti`, ... |
| Raspberry Pi (12 classes) | `استورد "راسبيري"` | `لوحة_راسبيري`, `منفذ_gpio_راسبيري`, `كاميرا_راسبيري`, `قبعة_الحساسات_sensehat`, `قبعة_محركات_pca9685`, `شاشة_حبر_إلكتروني_epaper`, ... |
| Raspberry Pi Pico (12) | `استورد "بيكو"` | `لوحة_بيكو`, `منفذ_بيكو`, `نبضة_بيكو`, `وحدة_pio_بيكو`, `واي_فاي_بيكو_w`, `شريط_نيوبكسل_بيكو`, `مستشعر_حرارة_داخلي_بيكو`, ... |

---

## 5. CLI & MCP Server Commands for AI Agents

### MCP Server (`lisan --mcp`)
Start the built-in zero-dependency JSON-RPC 2.0 MCP server (supports both newline-delimited JSON-RPC and `Content-Length:` framing):
```bash
lisan --mcp
```
Exposed MCP tools:
- `lisan_run` — Safely execute Lisan code inside `Limits::sandbox()` (optional `max_steps`).
- `lisan_check` — Tokenize, parse, and flag accidental English keywords with exact Arabic fixes.
- `lisan_format` — Format Lisan code via `format_source`.
- `lisan_transpile` — Transpile to `arduino`, `esp32`, `stm32`, `raspberry`, or `pico`.
- `lisan_docs` — Search Lisan's embedded Arabic documentation.
- `lisan_from_python` — Deterministically convert common Python code to valid pure Arabic Lisan.

Exposed MCP resource:
- `lisan://llms.txt` — Compact Arabic-only language reference.

### Core CLI Workflow
```bash
lisan --check ملف.lsn          # Syntax & import validation without execution
lisan --fmt ملف.lsn            # Auto-format code (4-space indentation, spacing)
lisan --test ملف.lsn           # Run تحقق(...) and تحقق_يساوي(فعلي، متوقع) tests
lisan --sandbox ملف.lsn        # Safe bounded execution (250k steps)
lisan --interpret ملف.lsn      # Full tree-walking interpreter
lisan --vm ملف.lsn             # Fast bytecode VM
lisan --native ملف.lsn --output برنامج   # Native machine code via Clang/LLVM
lisan --lsp                    # IDE Language Server Protocol
lisan --arduino ملف.lsn        # Transpile to Arduino C++ (.ino)
lisan --esp32 ملف.lsn          # Transpile to ESP32 C++ (.ino)
lisan --stm32 ملف.lsn          # Transpile to STM32 C++ (.ino)
lisan --raspberry ملف.lsn      # Transpile to Raspberry Pi Python 3 (.py)
lisan --pico ملف.lsn           # Transpile to Raspberry Pi Pico MicroPython (.py)
```

---

## 6. Common Mistakes & Instant Fixes

| Mistake (Fails!) | Fix (Valid Lisan!) | Why |
| --- | --- | --- |
| `إذا (أ) { ... } وإلا إذا (ب) { ... }` | `إذا (أ) { ... } وإلا { إذا (ب) { ... } }` | `وإلا` requires a `{ ... }` block |
| `اطبع("العدد:"، س)` | `اطبع(ن"العدد: {س}")` or `اطبع("العدد: " + نص(س))` | `اطبع(...)` accepts a single expression |
| `دع نص = 'مرحبا'` | `دع نص = "مرحبا"` | Only double quotes `"..."` are valid strings |
| `المجموع += 5` | `المجموع = المجموع + 5` | Compound assignment `+=` does not exist |
| `التقط (خطأ) { ... }` | `التقط (رسالة) { ... }` | `خطأ` is the reserved keyword `false` |
| `دع حالة = 1` | `دع الوضع = 1` | `حالة` is the reserved `case` keyword |
| `"المجموع: " + 42` | `"المجموع: " + نص(42)` or `ن"المجموع: {42}"` | Explicitly convert numbers with `نص(...)` before `+` |
| `صنف س { دالة عمل(هذا، ق) { } }` | `صنف س { دالة عمل(ق) { هذا.ق = ق } }` | `هذا` is implicit inside class methods — never declare it as a parameter |
| `import numpy as np` | `دع مصفوفات = استورد_بايثون("numpy")` | Zero English keywords; foreign names belong inside string literals |

For the compact LLM reference, see [`docs/llms.txt`](docs/llms.txt). For the complete self-contained reference covering all 105+ hardware/robotics modules, AI/ML, web, mobile, and games, see [`docs/llms-full.txt`](docs/llms-full.txt).
