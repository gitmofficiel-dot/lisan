//! استوديو لسان لسطح المكتب (Lisan Studio Desktop Application)
//!
//! بيئة تطوير عربية متكاملة وعابرة للمنصات (Linux, Windows, macOS, Raspberry Pi ARM64)
//! تعمل بالاعتماد الحصري على مكتبة Rust القياسية (`std`) دون أي حزم خارجية.

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// إصدار استوديو لسان لسطح المكتب
pub const LISAN_STUDIO_VERSION: &str = "1.0.0";

/// نسخة مدمجة من واجهة الملعب التفاعلي للويب
pub const EMBEDDED_WEB_HTML: &str = include_str!("../web/index.html");

/// نسخة مدمجة من جسر تشغيل WebAssembly
pub const EMBEDDED_WEB_JS: &str = include_str!("../web/lisan.js");

/// شعار لغة لسان الرسمي المدمج بصيغة SVG عالية الدقة
pub const EMBEDDED_LOGO_SVG: &str = include_str!("../assets/logo.svg");

/// واجهة استوديو لسان لسطح المكتب المتصلة بالمحرك الأصلي عبر واجهة REST المحلية
pub const STUDIO_DESKTOP_HTML: &str = r##"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>استوديو لسان لسطح المكتب | Lisan Studio Desktop</title>
    <link rel="icon" type="image/svg+xml" href="/logo.svg">
    <style>
        :root {
            --primary: #0284c7;
            --primary-hover: #0369a1;
            --accent: #10b981;
            --accent-hover: #059669;
            --bg: #0b1120;
            --panel: #1e293b;
            --panel-alt: #0f172a;
            --text: #f8fafc;
            --muted: #94a3b8;
            --border: #334155;
            --danger: #ef4444;
            --warning: #f59e0b;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: 'Segoe UI', Tahoma, 'Noto Sans Arabic', Arial, sans-serif;
            background: var(--bg);
            color: var(--text);
            display: flex;
            flex-direction: column;
            height: 100vh;
            overflow: hidden;
        }
        header {
            background: var(--panel);
            border-bottom: 1px solid var(--border);
            padding: 0.65rem 1.25rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            gap: 0.75rem;
            flex-wrap: wrap;
        }
        .brand {
            display: flex;
            align-items: center;
            gap: 0.7rem;
        }
        .brand img {
            width: 36px;
            height: 36px;
            border-radius: 8px;
        }
        .brand-title {
            font-size: 1.15rem;
            font-weight: 700;
        }
        .brand-sub {
            font-size: 0.78rem;
            color: var(--muted);
        }
        .sys-badge {
            background: rgba(16, 185, 129, 0.15);
            color: #34d399;
            border: 1px solid rgba(16, 185, 129, 0.35);
            padding: 0.28rem 0.7rem;
            border-radius: 999px;
            font-size: 0.78rem;
            direction: ltr;
        }
        .toolbar {
            background: var(--panel-alt);
            border-bottom: 1px solid var(--border);
            padding: 0.55rem 1.25rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            flex-wrap: wrap;
        }
        select, button {
            font-family: inherit;
            font-size: 0.86rem;
            border-radius: 6px;
            padding: 0.45rem 0.85rem;
            cursor: pointer;
            transition: all 0.15s ease;
        }
        select {
            background: var(--panel);
            color: var(--text);
            border: 1px solid var(--border);
            min-width: 210px;
        }
        button {
            border: 1px solid var(--border);
            background: var(--panel);
            color: var(--text);
            font-weight: 600;
        }
        button:hover {
            border-color: var(--primary);
            background: #26354d;
        }
        button.btn-run {
            background: var(--accent);
            border-color: var(--accent);
            color: #fff;
        }
        button.btn-run:hover {
            background: var(--accent-hover);
        }
        button.btn-target {
            background: rgba(2, 132, 199, 0.18);
            border-color: rgba(2, 132, 199, 0.55);
            color: #7dd3fc;
        }
        button.btn-target:hover {
            background: var(--primary);
            color: #fff;
        }
        .workspace {
            flex: 1;
            display: grid;
            grid-template-columns: 1.15fr 0.85fr;
            min-height: 0;
        }
        @media (max-width: 900px) {
            .workspace { grid-template-columns: 1fr; grid-template-rows: 1fr 1fr; }
        }
        .pane {
            display: flex;
            flex-direction: column;
            min-height: 0;
            border-left: 1px solid var(--border);
        }
        .pane:last-child { border-left: none; }
        .pane-header {
            background: rgba(30, 41, 59, 0.75);
            border-bottom: 1px solid var(--border);
            padding: 0.45rem 1rem;
            font-size: 0.82rem;
            color: var(--muted);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        textarea#editor {
            flex: 1;
            width: 100%;
            background: var(--bg);
            color: #e2e8f0;
            border: none;
            outline: none;
            resize: none;
            padding: 1rem 1.2rem;
            font-family: 'Cascadia Code', 'Fira Code', Consolas, 'Courier New', monospace;
            font-size: 1rem;
            line-height: 1.75;
            direction: rtl;
            tab-size: 4;
        }
        pre#output {
            flex: 1;
            background: #070d19;
            color: #38bdf8;
            padding: 1rem 1.2rem;
            overflow: auto;
            font-family: 'Cascadia Code', 'Fira Code', Consolas, monospace;
            font-size: 0.93rem;
            line-height: 1.65;
            white-space: pre-wrap;
            word-break: break-word;
        }
        iframe#preview {
            flex: 1;
            width: 100%;
            border: none;
            background: #fff;
            display: none;
        }
        footer {
            background: var(--panel);
            border-top: 1px solid var(--border);
            padding: 0.35rem 1.25rem;
            font-size: 0.78rem;
            color: var(--muted);
            display: flex;
            justify-content: space-between;
            align-items: center;
            gap: 1rem;
        }
        footer a { color: #38bdf8; text-decoration: none; }
    </style>
</head>
<body>
    <header>
        <div class="brand">
            <img src="/logo.svg" alt="شعار لسان">
            <div>
                <div class="brand-title">استوديو لسان لسطح المكتب <span style="color:#38bdf8">1.0</span></div>
                <div class="brand-sub">تشغيل أصلي كامل • ملفات وقواعد بيانات وشبكات • أردوينو • ESP32 • STM32 • Raspberry Pi</div>
            </div>
        </div>
        <div id="sysBadge" class="sys-badge">جاري الاتصال بمحرك لسان الأصلي...</div>
    </header>

    <div class="toolbar">
        <button class="btn-run" id="runBtn" title="Ctrl+Enter">▶ تشغيل أصلي</button>
        <button id="fmtBtn">✨ تنسيق الشيفرة</button>
        <button class="btn-target" data-target="arduino">Arduino C++</button>
        <button class="btn-target" data-target="esp32">ESP32 IoT</button>
        <button class="btn-target" data-target="stm32">STM32</button>
        <button class="btn-target" data-target="raspberry">Raspberry Pi 🐍</button>
        <select id="exampleSelect">
            <option value="">📂 اختر مثالاً جاهزاً...</option>
            <optgroup label="قوالب استوديو لسان المدمجة">
                <option value="tpl:desktop">🖥️ تطبيق سطح مكتب ومعلومات النظام</option>
                <option value="tpl:rpi">🍓 راسبيري باي (GPIO وحساسات وكاميرا)</option>
                <option value="tpl:esp32">📡 محطة إنترنت الأشياء ESP32 WiFi</option>
                <option value="tpl:arduino">🤖 روبوت أردوينو ذكي</option>
                <option value="tpl:ui">📱 واجهة تطبيق تفاعلية مرئية</option>
            </optgroup>
            <optgroup id="diskExamplesGroup" label="أمثلة المشروع (examples/*.lsn)"></optgroup>
        </select>
        <button id="downloadBtn">💾 حفظ الملف</button>
    </div>

    <div class="workspace">
        <div class="pane">
            <div class="pane-header">
                <span>📝 محرر شيفرة لسان (.lsn)</span>
                <span id="cursorStats">الأسطر: 1</span>
            </div>
            <textarea id="editor" spellcheck="false"></textarea>
        </div>
        <div class="pane">
            <div class="pane-header">
                <span id="outputTitle">⚡ مخرجات المحرك الأصلي</span>
                <div style="display:flex;gap:0.4rem;">
                    <button id="togglePreviewBtn" style="display:none;padding:0.2rem 0.55rem;font-size:0.76rem;">معاينة / نص</button>
                    <button id="copyBtn" style="padding:0.2rem 0.55rem;font-size:0.76rem;">نسخ</button>
                    <button id="clearBtn" style="padding:0.2rem 0.55rem;font-size:0.76rem;">مسح</button>
                </div>
            </div>
            <pre id="output">جاهز للتشغيل عبر نواة لسان الأصلية.</pre>
            <iframe id="preview" sandbox="allow-scripts"></iframe>
        </div>
    </div>

    <footer>
        <span id="workspaceLabel">مسار العمل: ...</span>
        <span id="execTimer">اضغط Ctrl+Enter للتشغيل السريع | <a href="/web" target="_blank">ملعب WebAssembly</a></span>
    </footer>

    <script>
        const starterTemplates = {
            "tpl:desktop": `# تطبيق سطح مكتب أصلي بلغة لسان 1.0
استورد رياضيات كـ ر

اطبع("🚀 مرحباً بك في استوديو لسان لسطح المكتب!")
اطبع(ن"قيمة الثابت ط = {ر.ط} | الجذر التربيعي لـ 256 = {ر.جذر(256)}")

صنف مهمة {
    دالة بناء(العنوان، الأولوية) {
        هذا.العنوان = العنوان
        هذا.الأولوية = الأولوية
    }
    دالة ملخص() {
        أرجع "✅ [" + هذا.الأولوية + "] " + هذا.العنوان
    }
}

دع المهام = [
    جديد مهمة("بناء تطبيق سطح مكتب عابر للمنصات (Linux / Windows / macOS)"، "عالية")،
    جديد مهمة("برمجة Raspberry Pi و ESP32 و STM32 بالعربية"، "عالية")،
    جديد مهمة("تصدير حزم .deb و .app و .zip الجاهزة للتثبيت"، "متوسطة")
]

لكل (م في المهام) {
    اطبع(م.ملخص())
}
`,
            "tpl:rpi": `# التحكم في حاسوب Raspberry Pi 4 / 5 بلغة لسان
استورد "راسبيري"

دع لوحة = جديد لوحة_راسبيري("Raspberry Pi 5")
اطبع("الطراز: " + لوحة.اسم_الطراز() + " | حرارة المعالج: " + نص(لوحة.حرارة_المعالج()) + " °C")

دع مصباح = جديد منفذ_gpio_راسبيري(17، "مخرج")
مصباح.شغل()
اطبع("تم تشغيل المنفذ 17 بنجاح | IP: " + لوحة.عنوان_ip())
`,
            "tpl:esp32": `# محطة إنترنت الأشياء عبر متحكم ESP32
استورد "esp32"

دع شبكة = جديد واي_فاي_esp()
شبكة.اتصل("Lisan_Smart_Lab"، "12345678")
اطبع("عنوان IP: " + شبكة.عنوان_ip())

دع خادم = جديد خادم_ويب_esp(80)
خادم.عند_طلب("/"، "مرحباً من متحكم ESP32 المبرمج بلغة لسان!")
خادم.ابدأ()
`,
            "tpl:arduino": `# روبوت ذكي على لوحة Arduino
استورد "اردوينو"

دع التراسوني = جديد حساس_مسافة(9، 10)
دع ذراع = جديد محرك_سيرفو(6)

إذا (التراسوني.اقرأ_سم() < 30) {
    ذراع.حرك(90)
} وإلا {
    ذراع.حرك(180)
}
اطبع("زاوية السيرفو الحالية: " + نص(ذراع.اقرأ_زاوية()))
`,
            "tpl:ui": `# تصميم واجهة تطبيق تفاعلية مرئية بلغة لسان
استورد "هاتف"

دع تطبيقي = جديد تطبيق_هاتف("استوديو لسان الذكي"، "زمردي")
تطبيقي.أضف_إحصائية("حالة المحرك الأصلي"، "متصل 100%"، "شريحة")
تطبيقي.أضف_بطاقة("منصة لسان 1.0"، "تطبيقات سطح مكتب وهواتف ومتحكمات دقيقة وراسبيري باي بلغة عربية فصيحة."، "أردوينو")
تطبيقي.أضف_مفتاح_تحكم("الوضع الليلي الذكي"، "قمر"، صحيح)
تطبيقي.أضف_زر("ابدأ الابتكار الآن"، "تشغيل"، "تم إطلاق مشروعك بلغة لسان بنجاح!")
اطبع(تطبيقي.بناء_واجهة())
`
        };

        const editor = document.getElementById('editor');
        const outputEl = document.getElementById('output');
        const outputTitle = document.getElementById('outputTitle');
        const previewEl = document.getElementById('preview');
        const togglePreviewBtn = document.getElementById('togglePreviewBtn');
        const sysBadge = document.getElementById('sysBadge');
        const workspaceLabel = document.getElementById('workspaceLabel');
        const execTimer = document.getElementById('execTimer');
        const exampleSelect = document.getElementById('exampleSelect');
        const diskGroup = document.getElementById('diskExamplesGroup');
        const cursorStats = document.getElementById('cursorStats');

        let lastExtension = 'lsn';
        let lastOutputCode = '';

        editor.value = starterTemplates['tpl:desktop'];
        updateStats();

        function updateStats() {
            const lines = editor.value.split('\n').length;
            cursorStats.textContent = `الأسطر: ${lines} | الحروف: ${editor.value.length}`;
        }

        editor.addEventListener('input', updateStats);
        editor.addEventListener('keydown', (e) => {
            if (e.key === 'Tab') {
                e.preventDefault();
                const start = editor.selectionStart;
                const end = editor.selectionEnd;
                editor.value = editor.value.substring(0, start) + '    ' + editor.value.substring(end);
                editor.selectionStart = editor.selectionEnd = start + 4;
                updateStats();
            } else if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                e.preventDefault();
                runNative();
            }
        });

        function renderOutput(text, isError, forceLtr, title) {
            outputTitle.textContent = title || '⚡ مخرجات المحرك الأصلي';
            lastOutputCode = text || '';
            if (!isError && (text.includes('<!DOCTYPE html>') || text.includes('<svg '))) {
                outputEl.textContent = text;
                previewEl.srcdoc = text;
                previewEl.style.display = 'block';
                outputEl.style.display = 'none';
                togglePreviewBtn.style.display = 'inline-block';
            } else {
                previewEl.style.display = 'none';
                outputEl.style.display = 'block';
                togglePreviewBtn.style.display = 'none';
                outputEl.style.direction = forceLtr ? 'ltr' : 'rtl';
                outputEl.style.color = isError ? '#f87171' : '#38bdf8';
                outputEl.textContent = text || '(اكتمل التنفيذ بنجاح دون مخرجات نصية)';
            }
        }

        async function loadInfoAndExamples() {
            try {
                const res = await fetch('/api/info');
                const info = await res.json();
                sysBadge.textContent = `${info.os.toUpperCase()} • ${info.arch} • Lisan v${info.version}`;
                workspaceLabel.textContent = `مسار العمل: ${info.workspace}`;
            } catch (e) {
                sysBadge.textContent = 'وضع غير متصل';
            }

            try {
                const exRes = await fetch('/api/examples');
                const exData = await exRes.json();
                if (exData.ok && Array.isArray(exData.examples)) {
                    exData.examples.forEach(name => {
                        const opt = document.createElement('option');
                        opt.value = 'file:' + name;
                        opt.textContent = '📄 ' + name;
                        diskGroup.appendChild(opt);
                    });
                }
            } catch (_) {}
        }

        async function runNative() {
            const t0 = performance.now();
            lastExtension = 'lsn';
            try {
                const res = await fetch('/api/run', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ code: editor.value })
                });
                const data = await res.json();
                const ms = (performance.now() - t0).toFixed(1);
                execTimer.textContent = `زمن التنفيذ الأصلي: ${ms} مللي ثانية`;
                if (data.ok) {
                    renderOutput(data.output, false, false, '⚡ مخرجات المحرك الأصلي');
                } else {
                    renderOutput(data.error || data.output || 'خطأ غير معروف', true, false, '❌ خطأ في التنفيذ');
                }
            } catch (err) {
                renderOutput('تعذر الاتصال بخادم الاستوديو المحلي: ' + err.message, true, false);
            }
        }

        async function transpileTo(target) {
            const t0 = performance.now();
            try {
                const res = await fetch('/api/transpile', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ code: editor.value, target })
                });
                const data = await res.json();
                const ms = (performance.now() - t0).toFixed(1);
                execTimer.textContent = `زمن التحويل (${target}): ${ms} مللي ثانية`;
                if (!data.ok) {
                    renderOutput(data.error || 'خطأ في الترجمة', true, false, '❌ خطأ في التحويل');
                    return;
                }
                const resultCode = data.code || data.output || '';
                if (target === 'fmt') {
                    editor.value = resultCode;
                    updateStats();
                    renderOutput('✓ تم تنسيق الشيفرة العربية وضبط المحاذاة بنجاح.', false, false, '✨ منسق لسان');
                } else {
                    lastExtension = target === 'raspberry' ? 'py' : 'ino';
                    renderOutput(resultCode, false, true, `🔧 الشيفرة المولّدة للهدف (${target.toUpperCase()})`);
                }
            } catch (err) {
                renderOutput('خطأ في الاتصال: ' + err.message, true, false);
            }
        }

        document.getElementById('runBtn').addEventListener('click', runNative);
        document.getElementById('fmtBtn').addEventListener('click', () => transpileTo('fmt'));
        document.querySelectorAll('.btn-target').forEach(btn => {
            btn.addEventListener('click', () => transpileTo(btn.dataset.target));
        });

        exampleSelect.addEventListener('change', async () => {
            const val = exampleSelect.value;
            if (!val) return;
            if (starterTemplates[val]) {
                editor.value = starterTemplates[val];
                updateStats();
                runNative();
            } else if (val.startsWith('file:')) {
                const name = val.slice(5);
                const res = await fetch('/api/example?name=' + encodeURIComponent(name));
                const data = await res.json();
                if (data.ok) {
                    editor.value = data.code;
                    updateStats();
                    runNative();
                } else {
                    renderOutput(data.error || 'تعذر قراءة المثال', true, false);
                }
            }
        });

        togglePreviewBtn.addEventListener('click', () => {
            if (previewEl.style.display === 'none') {
                previewEl.style.display = 'block';
                outputEl.style.display = 'none';
            } else {
                previewEl.style.display = 'none';
                outputEl.style.display = 'block';
            }
        });

        document.getElementById('copyBtn').addEventListener('click', () => {
            navigator.clipboard.writeText(lastOutputCode || outputEl.textContent);
        });

        document.getElementById('clearBtn').addEventListener('click', () => {
            renderOutput('', false, false);
        });

        document.getElementById('downloadBtn').addEventListener('click', () => {
            const content = lastExtension === 'lsn' ? editor.value : (lastOutputCode || editor.value);
            const filename = lastExtension === 'lsn' ? 'برنامج_لسان.lsn' : `lisan_generated.${lastExtension}`;
            const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
            const a = document.createElement('a');
            a.href = URL.createObjectURL(blob);
            a.download = filename;
            a.click();
        });

        loadInfoAndExamples().then(runNative);
    </script>
</body>
</html>
"##;

/// بنية مبسطة لطلب HTTP وارد
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub body: String,
}

/// طباعة جميع المنصات والمعماريات والأنظمة المدمجة التي تدعمها لغة لسان
pub fn print_platform_targets() {
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║         منصات وأنظمة تشغيل لغة البرمجة العربية لسان (Lisan 1.0)         ║");
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!("║ المنصة الحالية : {} ({})", env::consts::OS, env::consts::ARCH);
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!("║ 🖥️ أنظمة سطح المكتب والخوادم (Desktop & Server OS):                     ║");
    println!("║   • Linux x86_64    (x86_64-unknown-linux-gnu)   [.tar.gz / .deb]        ║");
    println!("║   • Linux ARM64     (aarch64-unknown-linux-gnu)  [.tar.gz / .deb]        ║");
    println!("║   • Windows 10/11   (x86_64-pc-windows-msvc)     [.exe / .zip / winget]  ║");
    println!("║   • Windows ARM64   (aarch64-pc-windows-msvc)    [.exe]                  ║");
    println!("║   • macOS Silicon   (aarch64-apple-darwin M1-M4) [.app / .tar.gz]        ║");
    println!("║   • macOS Intel     (x86_64-apple-darwin)        [Universal Binary]      ║");
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!("║ 🍓 الحواسيب المصغرة والمتحكمات الدقيقة (SBC & Embedded Hardware):       ║");
    println!("║   • Raspberry Pi    (Pi 4 / Pi 5 / Zero 2 W — ARM64 + Python GPIO/I2C)   ║");
    println!("║   • Arduino         (Uno / Nano / Mega — AVR C++ .ino)                   ║");
    println!("║   • ESP32 / ESP8266 (WiFi / BLE / MQTT / WebServer / FreeRTOS C++)       ║");
    println!("║   • STM32           (Cortex-M F1/F4/Nucleo — Timers / CAN / DAC / RTC)   ║");
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!("║ 🌐 الويب والهواتف الذكية (Web & Mobile):                                ║");
    println!("║   • WebAssembly     (wasm32-unknown-unknown — متصفحات الويب بلا خادم)    ║");
    println!("║   • Android & iOS   (تصدير تطبيقات هواتف ذكية PWA مستقلة --mobile)       ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
}

/// تشغيل تطبيق استوديو لسان لسطح المكتب على منفذ محلي وفتح نافذة التطبيق
pub fn launch_desktop_app(port_hint: u16, open_window: bool) -> Result<(), String> {
    let listener = if port_hint > 0 {
        TcpListener::bind(("127.0.0.1", port_hint))
            .or_else(|_| TcpListener::bind(("127.0.0.1", 0)))
            .map_err(|e| format!("تعذّر فتح منفذ محلي لاستوديو لسان: {}", e))?
    } else {
        TcpListener::bind(("127.0.0.1", 0))
            .map_err(|e| format!("تعذّر تخصيص منفذ حر لاستوديو لسان: {}", e))?
    };

    let local_addr = listener
        .local_addr()
        .map_err(|e| format!("تعذّر قراءة عنوان الخادم المحلي: {}", e))?;
    let url = format!("http://127.0.0.1:{}", local_addr.port());

    println!("🚀 استوديو لسان لسطح المكتب يعمل الآن على: {}", url);
    println!(
        "💻 البيئة الأصلية: {} ({}) | الإصدار {}",
        env::consts::OS,
        env::consts::ARCH,
        LISAN_STUDIO_VERSION
    );
    println!("اضغط Ctrl+C لإغلاق خادم الاستوديو.");

    if open_window {
        open_native_app_window(&url);
    }

    for incoming in listener.incoming() {
        let mut stream = match incoming {
            Ok(s) => s,
            Err(_) => continue,
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));

        if let Some(req) = read_http_request(&mut stream) {
            let (status, content_type, body) =
                handle_http_request(&req.method, &req.path, &req.body);
            let _ = write_http_response(&mut stream, status, content_type, &body);
        }
    }

    Ok(())
}

/// فتح نافذة تطبيق سطح مكتب مستقلة (--app=URL) بحسب نظام التشغيل المتاح
pub fn open_native_app_window(url: &str) {
    let app_arg = format!("--app={}", url);
    let size_arg = "--window-size=1280,820";

    #[cfg(target_os = "linux")]
    {
        let browsers = [
            "google-chrome",
            "google-chrome-stable",
            "chromium",
            "chromium-browser",
            "microsoft-edge",
            "microsoft-edge-stable",
            "brave-browser",
        ];
        for bin in browsers {
            if spawn_detached(bin, &[&app_arg, size_arg]) {
                return;
            }
        }
        let _ = spawn_detached("xdg-open", &[url]);
    }

    #[cfg(target_os = "windows")]
    {
        let candidates = [
            "msedge",
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
            "chrome",
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ];
        for bin in candidates {
            if spawn_detached(bin, &[&app_arg, size_arg]) {
                return;
            }
        }
        let _ = spawn_detached("cmd", &["/C", "start", "", url]);
    }

    #[cfg(target_os = "macos")]
    {
        let mac_browsers = [
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
        ];
        for bin in mac_browsers {
            if std::path::Path::new(bin).exists() && spawn_detached(bin, &[&app_arg, size_arg]) {
                return;
            }
        }
        let _ = spawn_detached("open", &[url]);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = (&app_arg, size_arg, url);
    }
}

fn spawn_detached(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

/// قراءة طلب HTTP كامل بما يشمل الترويسات وجسم الطلب وفق ترويسة Content-Length
pub fn read_http_request(stream: &mut TcpStream) -> Option<HttpRequest> {
    const MAX_HEADER_BYTES: usize = 64 * 1024;
    const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

    let mut buffer = Vec::with_capacity(4096);
    let mut temp = [0u8; 4096];

    loop {
        let n = match stream.read(&mut temp) {
            Ok(0) => {
                if buffer.is_empty() {
                    return None;
                }
                break;
            }
            Ok(n) => n,
            Err(_) => {
                if buffer.is_empty() {
                    return None;
                }
                break;
            }
        };
        buffer.extend_from_slice(&temp[..n]);
        if find_header_end(&buffer).is_some() {
            break;
        }
        if buffer.len() > MAX_HEADER_BYTES {
            return None;
        }
    }

    let sep_pos = find_header_end(&buffer)?;
    let header_text = String::from_utf8_lossy(&buffer[..sep_pos]);
    let mut lines = header_text.lines();
    let request_line = lines.next()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_uppercase();
    let path = parts.next()?.to_string();

    let mut content_length: usize = 0;
    for line in lines {
        if let Some((name, val)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                content_length = val.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    if content_length > MAX_BODY_BYTES {
        content_length = MAX_BODY_BYTES;
    }

    let body_start = sep_pos + 4;
    while buffer.len().saturating_sub(body_start) < content_length {
        match stream.read(&mut temp) {
            Ok(0) => break,
            Ok(n) => buffer.extend_from_slice(&temp[..n]),
            Err(_) => break,
        }
    }

    let available = buffer.len().saturating_sub(body_start);
    let take_len = if content_length > 0 {
        available.min(content_length)
    } else {
        available
    };
    let body = if body_start <= buffer.len() {
        String::from_utf8_lossy(&buffer[body_start..body_start + take_len]).into_owned()
    } else {
        String::new()
    };

    Some(HttpRequest { method, path, body })
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    };
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        status,
        reason,
        content_type,
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    if !body.is_empty() {
        stream.write_all(body)?;
    }
    stream.flush()
}

/// معالجة طلبات الواجهة الرسومية ونقاط نهاية JSON API
pub fn handle_http_request(
    method: &str,
    raw_path: &str,
    body: &str,
) -> (u16, &'static str, Vec<u8>) {
    if method == "OPTIONS" {
        return (204, "text/plain; charset=utf-8", Vec::new());
    }

    let (path, query) = match raw_path.split_once('?') {
        Some((p, q)) => (p, q),
        None => (raw_path, ""),
    };

    match (method, path) {
        ("GET", "/") | ("GET", "/studio") => (
            200,
            "text/html; charset=utf-8",
            STUDIO_DESKTOP_HTML.as_bytes().to_vec(),
        ),
        ("GET", "/web") | ("GET", "/playground") | ("GET", "/index.html") => {
            let html = fs::read("web/index.html")
                .unwrap_or_else(|_| EMBEDDED_WEB_HTML.as_bytes().to_vec());
            (200, "text/html; charset=utf-8", html)
        }
        ("GET", "/lisan.js") => {
            let js =
                fs::read("web/lisan.js").unwrap_or_else(|_| EMBEDDED_WEB_JS.as_bytes().to_vec());
            (200, "application/javascript; charset=utf-8", js)
        }
        ("GET", "/lisan.wasm") => match fs::read("web/lisan.wasm") {
            Ok(wasm) => (200, "application/wasm", wasm),
            Err(_) => (
                404,
                "application/json; charset=utf-8",
                r#"{"ok":false,"error":"ملف lisan.wasm غير موجود"}"#.as_bytes().to_vec(),
            ),
        },
        ("GET", "/logo.svg") | ("GET", "/assets/logo.svg") | ("GET", "/favicon.ico") => {
            let svg = fs::read("assets/logo.svg")
                .unwrap_or_else(|_| EMBEDDED_LOGO_SVG.as_bytes().to_vec());
            (200, "image/svg+xml; charset=utf-8", svg)
        }
        ("GET", "/api/info") => (
            200,
            "application/json; charset=utf-8",
            platform_info_json().into_bytes(),
        ),
        ("POST", "/api/run") => (
            200,
            "application/json; charset=utf-8",
            api_run_native(body).into_bytes(),
        ),
        ("POST", "/api/transpile") => (
            200,
            "application/json; charset=utf-8",
            api_transpile(body).into_bytes(),
        ),
        ("GET", "/api/examples") => (
            200,
            "application/json; charset=utf-8",
            api_list_examples().into_bytes(),
        ),
        ("GET", "/api/example") => {
            let (code_status, payload) = api_read_example(query);
            (code_status, "application/json; charset=utf-8", payload.into_bytes())
        }
        _ => (
            404,
            "application/json; charset=utf-8",
            format!(
                "{{\"ok\":false,\"error\":\"المسار غير موجود: {}\"}}",
                escape_json_str(path)
            )
            .into_bytes(),
        ),
    }
}

/// إرجاع معلومات النظام والمعمارية ومسار العمل بصيغة JSON
pub fn platform_info_json() -> String {
    let workspace = env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| ".".to_string());
    format!(
        "{{\"ok\":true,\"name\":\"استوديو لسان لسطح المكتب\",\"version\":\"{}\",\"os\":\"{}\",\"arch\":\"{}\",\"workspace\":\"{}\",\"targets\":[\"native\",\"arduino\",\"esp32\",\"stm32\",\"raspberry\",\"fmt\",\"wasm\",\"mobile\"]}}",
        escape_json_str(LISAN_STUDIO_VERSION),
        escape_json_str(env::consts::OS),
        escape_json_str(env::consts::ARCH),
        escape_json_str(&workspace)
    )
}

fn api_run_native(body: &str) -> String {
    let code = extract_json_field(body, "code").unwrap_or_else(|| body.to_string());
    let started = Instant::now();
    let mut interpreter = crate::Interpreter::with_limits(crate::Limits::full());
    match interpreter.run_source(&code) {
        Ok(lines) => {
            let joined = lines.join("\n");
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            format!(
                "{{\"ok\":true,\"output\":\"{}\",\"elapsed_ms\":{:.2}}}",
                escape_json_str(&joined),
                elapsed_ms
            )
        }
        Err(err) => format!(
            "{{\"ok\":false,\"output\":\"{}\",\"error\":\"{}\"}}",
            escape_json_str(&err),
            escape_json_str(&err)
        ),
    }
}

fn api_transpile(body: &str) -> String {
    let code = match extract_json_field(body, "code") {
        Some(c) => c,
        None => {
            return r#"{"ok":false,"error":"الحقل 'code' مطلوب في طلب التحويل"}"#.to_string();
        }
    };
    let target = extract_json_field(body, "target")
        .unwrap_or_else(|| "arduino".to_string())
        .trim()
        .to_lowercase();

    let result = match target.as_str() {
        "arduino" | "ino" | "اردوينو" => crate::arduino::transpile_arduino(&code),
        "esp32" | "esp" | "esp8266" => crate::arduino::transpile_esp32(&code),
        "stm32" | "stm" => crate::arduino::transpile_stm32(&code),
        "raspberry" | "rpi" | "raspberrypi" | "راسبيري" => {
            crate::raspberry::transpile_raspberry_python(&code)
        }
        "fmt" | "format" | "نسق" | "نسّق" => Ok(crate::fmt::format_source(&code)),
        other => Err(format!(
            "الهدف '{}' غير مدعوم. الأهداف المتاحة: arduino, esp32, stm32, raspberry, fmt",
            other
        )),
    };

    match result {
        Ok(generated) => format!(
            "{{\"ok\":true,\"target\":\"{}\",\"code\":\"{}\",\"output\":\"{}\"}}",
            escape_json_str(&target),
            escape_json_str(&generated),
            escape_json_str(&generated)
        ),
        Err(err) => format!(
            "{{\"ok\":false,\"target\":\"{}\",\"error\":\"{}\"}}",
            escape_json_str(&target),
            escape_json_str(&err)
        ),
    }
}

fn discover_examples_dirs() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    candidates.push(PathBuf::from("examples"));

    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("examples"));
            candidates.push(dir.join("../Resources/examples"));
            candidates.push(dir.join("../share/lisan/examples"));
        }
    }

    if let Ok(home) = env::var("HOME") {
        candidates.push(PathBuf::from(home).join(".local/share/lisan/examples"));
    }
    candidates.push(PathBuf::from("/usr/share/lisan/examples"));
    candidates
}

fn api_list_examples() -> String {
    let mut names = Vec::new();
    for dir in discover_examples_dirs() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()).map(crate::modules::is_source_ext).unwrap_or(false) {
                    if let Some(fname) = p.file_name().and_then(|s| s.to_str()) {
                        let s = fname.to_string();
                        if !names.contains(&s) {
                            names.push(s);
                        }
                    }
                }
            }
            if !names.is_empty() {
                break;
            }
        }
    }
    names.sort();
    let json_items: Vec<String> = names
        .iter()
        .map(|n| format!("\"{}\"", escape_json_str(n)))
        .collect();
    format!("{{\"ok\":true,\"examples\":[{}]}}", json_items.join(","))
}

fn api_read_example(query: &str) -> (u16, String) {
    let raw_name = extract_query_param(query, "name").unwrap_or_default();
    let name = percent_decode(&raw_name);
    if name.is_empty()
        || name.contains("..")
        || name.contains('/')
        || name.contains('\\')
        || !crate::modules::is_source_name(&name)
    {
        return (
            400,
            r#"{"ok":false,"error":"اسم ملف المثال غير صالح؛ يجب أن ينتهي بـ .lsn أو .lisan دون مسارات نسبية"}"#
                .to_string(),
        );
    }

    for dir in discover_examples_dirs() {
        let candidate = dir.join(&name);
        if candidate.is_file() {
            if let Ok(code) = fs::read_to_string(&candidate) {
                return (
                    200,
                    format!(
                        "{{\"ok\":true,\"name\":\"{}\",\"code\":\"{}\"}}",
                        escape_json_str(&name),
                        escape_json_str(&code)
                    ),
                );
            }
        }
    }

    (
        404,
        format!(
            "{{\"ok\":false,\"error\":\"لم يتم العثور على المثال '{}'\"}}",
            escape_json_str(&name)
        ),
    )
}

fn extract_json_field(body: &str, key: &str) -> Option<String> {
    let parsed = crate::json::parse_json(body).ok()?;
    if let crate::Value::Map(pairs) = parsed {
        let borrowed = pairs.borrow();
        for (k, v) in borrowed.iter() {
            if k == key {
                return match v {
                    crate::Value::Text(s) => Some(s.clone()),
                    other => Some(other.to_string()),
                };
            }
        }
    }
    None
}

fn extract_query_param<'a>(query: &'a str, target_key: &str) -> Option<&'a str> {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == target_key {
                return Some(v);
            }
        }
    }
    None
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push(((h << 4) | l) as u8);
                i += 3;
                continue;
            }
            out.push(bytes[i]);
            i += 1;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn escape_json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            ctrl if (ctrl as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", ctrl as u32));
            }
            normal => out.push(normal),
        }
    }
    out
}

fn probe_tool(candidates: &[&str], version_arg: &str) -> Option<String> {
    for bin in candidates {
        if let Ok(out) = std::process::Command::new(bin).arg(version_arg).output() {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let first_line = stdout.lines().next()
                    .or_else(|| stderr.lines().next())
                    .unwrap_or("ok")
                    .trim();
                return Some(format!("{} ({})", bin, first_line));
            }
        }
    }
    None
}

pub fn collect_environment_report() -> String {
    let mut report = String::new();
    report.push_str("=== Lisan Developer Environment (فحص بيئة التطوير) ===\n");
    report.push_str(&format!("OS: {}\n", std::env::consts::OS));
    report.push_str(&format!("Arch: {}\n", std::env::consts::ARCH));
    if let Ok(exe) = std::env::current_exe() {
        report.push_str(&format!("Lisan Binary: {}\n", exe.display()));
    }
    let cc_env = std::env::var("CC").ok().filter(|s| !s.trim().is_empty());
    let clang_status = if let Some(ref cc) = cc_env {
        probe_tool(&[cc.as_str(), "clang"], "--version")
    } else {
        probe_tool(&["clang"], "--version")
    };
    match clang_status {
        Some(v) => report.push_str(&format!("Native Compiler (LLVM/Clang): OK - {}\n", v)),
        None => report.push_str("Native Compiler (LLVM/Clang): MISSING (install clang or set CC for --native)\n"),
    }
    match probe_tool(&["rustc"], "--version") {
        Some(v) => report.push_str(&format!("Rust Compiler: OK - {}\n", v)),
        None => report.push_str("Rust Compiler: MISSING (optional unless building from source)\n"),
    }
    match probe_tool(&["python3", "python"], "--version") {
        Some(v) => report.push_str(&format!("Python (Raspberry Pi target): OK - {}\n", v)),
        None => report.push_str("Python (Raspberry Pi target): MISSING (optional)\n"),
    }
    report
}

pub fn run_doctor() {
    print!("{}", collect_environment_report());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_info_endpoint() {
        let (status, content_type, body) = handle_http_request("GET", "/api/info", "");
        assert_eq!(status, 200);
        assert!(content_type.contains("application/json"));
        let text = String::from_utf8(body).unwrap();
        assert!(text.contains("\"version\":\"1.0.0\""));
        assert!(text.contains(env::consts::OS));
        assert!(text.contains(env::consts::ARCH));
    }

    #[test]
    fn test_desktop_native_run_endpoint() {
        let payload = r#"{"code": "دع س = 21 * 2\nاطبع(\"الناتج = \" + نص(س))"}"#;
        let (status, _, body) = handle_http_request("POST", "/api/run", payload);
        assert_eq!(status, 200);
        let text = String::from_utf8(body).unwrap();
        assert!(text.contains("\"ok\":true"));
        assert!(text.contains("الناتج = 42"));
    }

    #[test]
    fn test_desktop_transpile_endpoints() {
        let arduino_req = r#"{"target":"esp32","code":"استورد \"esp32\"\nدع ش = جديد واي_فاي_esp()"}"#;
        let (status, _, body) = handle_http_request("POST", "/api/transpile", arduino_req);
        assert_eq!(status, 200);
        let text = String::from_utf8(body).unwrap();
        assert!(text.contains("\"ok\":true"));
        assert!(text.contains("LISAN_TARGET_ESP32"));

        let fmt_req = r#"{"target":"fmt","code":"إذا(صحيح){اطبع(1)}"}"#;
        let (status2, _, body2) = handle_http_request("POST", "/api/transpile", fmt_req);
        assert_eq!(status2, 200);
        let text2 = String::from_utf8(body2).unwrap();
        assert!(text2.contains("\"ok\":true"));
    }

    #[test]
    fn test_desktop_examples_and_path_traversal_guard() {
        let (status, _, body) = handle_http_request("GET", "/api/examples", "");
        assert_eq!(status, 200);
        let text = String::from_utf8(body).unwrap();
        assert!(text.contains("hello.lsn"));

        let (ok_status, _, ex_body) =
            handle_http_request("GET", "/api/example?name=hello.lsn", "");
        assert_eq!(ok_status, 200);
        assert!(String::from_utf8(ex_body).unwrap().contains("\"ok\":true"));

        let (bad_status, _, _) =
            handle_http_request("GET", "/api/example?name=../Cargo.toml", "");
        assert_eq!(bad_status, 400);
    }
}
