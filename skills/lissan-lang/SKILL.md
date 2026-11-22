---
name: lissan-lang
description: Write, run, check, test and debug programs in Lissan (لسان, also spelled Lisan), an Arabic-keyword programming language. Use for .lsn files, code with Arabic keywords (دع، دالة، إذا، لكل، صنف), برمجة بالعربية, Lissan hardware targets (Arduino, ESP32, STM32, Raspberry Pi) and generating web pages with the lisan-ui library. Do not use for the human tongue (اللسان as the organ) or for general programming in other languages.
license: MIT
compatibility: Needs the lisan binary (v1.0.0 or newer) to run code. Writing code needs no tools.
metadata:
  author: gitmofficiel-dot
  version: "1.0"
  repo: https://github.com/gitmofficiel-dot/lisan
---

# Lissan (لسان) programming skill

Lissan programs are plain text files with the `.lsn` extension, written with Arabic keywords. Statements end at a newline. A `؛` or `;` is only needed to put several statements on one line.

## Run and check

- Run: `lisan --interpret file.lsn` (direct interpreter, no step limit). Other paths: `--vm` (bytecode VM), `--native` (via Clang).
- Check syntax and imports without running: `lisan --check file.lsn`.
- Tests: `lisan --test file.lsn` (uses `تحقق(شرط)` and `تحقق_يساوي(فعلي، متوقع)`).
- Format: `lisan --fmt file.lsn`. REPL: `lisan`. One-liner: `lisan -e 'اطبع("مرحبا")'`.
- If the `lisan` binary is available, run new code before presenting it and show the real output. If it is not available, say clearly that the code was NOT run. Never invent output.
- Safety: generated programs can call `اكتب_ملف`, `احذف_ملف` and `اطلب_ويب`. Run untrusted code with `lisan --sandbox file.lsn` or in a temporary folder.

## Core syntax (all verified, see examples/)

| Need | Lissan |
| --- | --- |
| variable | `دع س = 5` (also `ليكن`, `متغير`) |
| print | `اطبع("نص")` |
| if / else | `إذا (شرط) { ... } وإلا { ... }` |
| loops | `بينما (شرط) { }` (or `طالما`), `لكل (ع في قائمة) { }`, `توقف`, `تابع` |
| range | `مدى(5)`, `مدى(1، 10)`, `مدى(5، 0، -2)` gives `[5، 3، 1]` |
| function | `دالة اسم(أ، ب) { أرجع أ + ب }` |
| class | `صنف اسم { دالة بناء(...) { هذا.حقل = ... } }`, create with `جديد اسم(...)` |
| list / dict | `[1، 2، 3]`, `{"مفتاح": قيمة}`; index with `[ ]`, from 0 |
| errors | `حاول { ارمِ "رسالة" } التقط (م) { } أخيرًا { }` |
| match | `اختر (قيمة) { حالة 1: { } افتراضي: { } }` |
| import | `استورد "ملف.lsn"` or `استورد رياضيات كـ ر` |
| logic | `و`, `أو`, `ليس`, `صحيح`, `خطأ`, `لاشيء` |
| comment | `# تعليق` or `// تعليق` |

Argument and list separator: the Arabic comma `،`. A Latin `,` is a syntax error.

## Rules and traps (learned by running real code)

1. Both Latin (`123`) and Arabic-Indic (`١٢٣`) digits are accepted. Output prints Latin digits. See docs/NUMBERS-AND-STYLE.md.
2. `خطأ` is the keyword **false**. Never use it as a variable name, for example `التقط (خطأ)` fails. Use `رسالة` or `عطل`.
3. Convert numbers to text before joining: `"عدد: " + نص(ن)`.
4. Functions are values: you can pass a named function or an anonymous `دالة(س) { ... }` as an argument, store it in a variable, and `نوع(f)` gives `دالة`. Limits found by testing v1.0.0: inner functions do NOT capture outer variables (using one fails with "متغير غير معرّف"), and a call needs a name, so `قائمة[0](5)` is rejected (assign it to a variable first).
5. `استورد "x.lsn"` resolves **relative to the importing file**, not the working directory.
6. Do not name a library file `ui.lsn`. It collides with a built-in module name and the functions silently stay undefined. Use a distinctive name (the bundled UI library is `lisan_ui.lsn`).
7. `أضف(قائمة، عنصر)` mutates the list in place. Dict assignment `d["k"] = v` also mutates.
8. Keep each program in one `.lsn` file per concern. Large generators build a string, then call `اكتب_ملف(مسار، نص)`.

## Standard library (most used)

Numbers: `مطلق جذر قرّب أدنى أعلى أرضية سقف قوة`. Text: `نظّف اقتطع قسّم ادمج استبدل يبدأ_ب ينتهي_ب كرر أحرف_كبيرة أحرف_صغيرة`. Lists: `مدى أضف احذف عكس فرز مجموع متوسط ابحث`. Dicts: `مفاتيح قيم يحتوي`. Files: `اقرأ_ملف اكتب_ملف ألحق_ملف ملف_موجود`. General: `طول نوع نص عدد عدد_صحيح الآن`. Details: [references/syntax.md](references/syntax.md).

Hardware: `استورد "اردوينو"`, `"esp32"`, `"stm32"`, `"راسبيري"` (83 hardware modules, translated to C++ or Python with `--arduino`, `--esp32`, `--stm32`, `--raspberry`).

## Workflow for an agent

1. Write the program with Arabic identifiers (English names are allowed but break the Arabic-first style).
2. Save as `.lsn`, run `lisan --check`, then `lisan --interpret`.
3. If the error message suggests a name ("هل تقصد ..."), apply it.
4. Show the user the code and the **real** output.

## Examples

See `examples/01_basics.lsn` to `examples/04_errors_match.lsn`, run through the real interpreter. The UI library for generating styled web pages is in `libs/ui/` of the repo (`lisan_ui.lsn`, escapes text and attributes and rejects `javascript:` links; test: `libs/ui/test_security.lsn`). The paths `docs/`, `libs/`, `editors/` refer to the repository, not to this skill folder. Syntax highlighting: `editors/highlightjs/lisan.js` (highlight.js) and `editors/vscode/` (TextMate).
