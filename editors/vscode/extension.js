'use strict';

const vscode = require('vscode');
const cp = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

/** @type {LisanLspBridge | null} */
let lspBridge = null;
/** @type {vscode.OutputChannel | null} */
let outputChannel = null;
/** @type {vscode.Terminal | null} */
let lisanTerminal = null;
/** @type {vscode.DiagnosticCollection | null} */
let diagnosticCollection = null;
/** @type {vscode.WebviewPanel | null} */
let previewPanel = null;
/** @type {string | null} */
let previewTargetFile = null;

// ─── توثيق عربي فوري عند التحويم (يعمل كمساند لخادم LSP) ───────────────────

const ARABIC_HOVER_DOCS = {
    'دع': '**تعريف متغير (`دع`):** الإعلان عن متغير جديد في النطاق الحالي.\n```lisan\nدع الاسم = "زيد"\nدع العمر = 25\n```',
    'دالة': '**تعريف دالة (`دالة`):** إنشاء دالة عربية جديدة مع معاملات اختيارية أو أنواع تدريجية.\n```lisan\nدالة جمع(س: عدد، ص: عدد) -> عدد {\n    أرجع س + ص\n}\n```',
    'أرجع': '**إرجاع قيمة (`أرجع`):** إعادة نتيجة تنفيذ الدالة إلى المستدعي.',
    'إذا': '**جملة شرطية (`إذا`):** تنفيذ كتلة برمجية عند تحقق الشرط المنطقي.',
    'وإلا': '**البديل الشرطي (`وإلا`):** يُنفّذ عندما لا يتحقق شرط `إذا`.',
    'لكل': '**حلقة تكرار (`لكل`):** المرور على عناصر قائمة أو نص أو قاموس أو `مدى(...)`.',
    'طالما': '**حلقة تكرار شرطية (`طالما`):** تستمر بالتكرار ما دام الشرط صحيحاً.',
    'في': '**معامل الانتماء والتكرار (`في`):** يُستخدم في حلقات `لكل` أو لفحص وجود عنصر.',
    'صنف': '**تعريف صنف كائني (`صنف`):** تعريف فئة كائنية بدالة `بناء` والوراثة عبر `يرث`.',
    'هذا': '**المشير للكائن الحالي (`هذا`):** يشير إلى نسخة الكائن الحالية داخل طرق الصنف.',
    'يرث': '**وراثة الأصناف (`يرث`):** وراثة الخصائص والطرق من صنف أب.',
    'جديد': '**إنشاء كائن (`جديد`):** إنشاء نسخة جديدة من صنف معرّف.',
    'استورد': '**استيراد وحدة (`استورد`):** استيراد مكتبة قياسية أو ملف `.lsn` آخر.',
    'من': '**استيراد مخصص (`من ... استورد`):** استيراد رموز محددة من وحدة.',
    'مع': '**مدير السياق (`مع`):** فتح وإغلاق الملفات والموارد تلقائياً بأمان.',
    'اختر': '**مطابقة الأنماط (`اختر / حالة / افتراضي`):** مطابقة قيمة مع حالات متعددة.',
    'حاول': '**معالجة الأخطاء (`حاول / التقط / أخيرًا`):** التقاط الاستثناءات ومنع توقف البرنامج.',
    'اطبع': '**الطباعة (`اطبع`):** طباعة القيم والنصوص المنسّقة إلى المخرجات القياسية.',
    'مدى': '**توليد نطاق أرقام (`مدى`):** إنشاء تسلسل عددي مثل `مدى(10)` أو `مدى(1، 10، 2)`.',
    'طول': '**حساب الطول (`طول`):** إرجاع عدد العناصر في القائمة أو النص أو القاموس.',
    'نوع': '**معرفة النوع (`نوع`):** إرجاع نوع القيمة (عدد، نص، منطقي، قائمة، قاموس...).',
    'تحقق': '**التحقق الاختباري (`تحقق`):** فحص صحة شرط منطقي داخل اختبارات لسان.',
    'تحقق_يساوي': '**تحقق المساواة (`تحقق_يساوي`):** التأكد من تساوي القيمة الفعلية مع المتوقعة.',
    'عدد': '**النوع التدريجي (`عدد`):** يمثل الأعداد الصحيحة والعشرية.',
    'نص': '**النوع التدريجي (`نص`):** يمثل السلاسل النصية العربية والعالمية.',
    'منطقي': '**النوع التدريجي (`منطقي`):** يقبل القيمتين `صحيح` أو `خطأ`.',
    'قائمة': '**النوع التدريجي (`قائمة`):** مصفوفة مرتبة قابلة للتعديل.',
    'قاموس': '**النوع التدريجي (`قاموس`):** أزواج مفاتيح وقيم.',
    'صف': '**النوع التدريجي (`صف`):** مجموعة مرتبة غير قابلة للتعديل.',
    'مجموعة': '**النوع التدريجي (`مجموعة`):** عناصر فريدة غير مكررة.',
    'فارغ': '**النوع التدريجي (`فارغ`):** يدل على عدم إرجاع قيمة (`لاشيء`).',
    'أي': '**النوع التدريجي (`أي`):** يقبل أي نوع قيمة في التحقق التدريجي.',
    'موقع_ويب': '**بناء مواقع الويب (`موقع_ويب`):** تصميم وتصدير مواقع ويب عربية متجاوبة.',
    'تطبيق_هاتف': '**تطبيقات الهواتف (`تطبيق_هاتف`):** بناء وتصدير تطبيقات ويب تقدمية (PWA).',
    'تطبيق_ويب': '**إطار الويب (`تطبيق_ويب`):** خادم ويب متعدد المسارات وواجهات بيانات.',
    'شبكة_عصبية': '**الشبكة العصبية (`شبكة_عصبية`):** شبكة عصبية اصطناعية بالانتشار العكسي في وحدة `ذكاء`.',
    'انحدار_خطي': '**الانحدار الخطي (`انحدار_خطي`):** نموذج تعلم آلي للتنبؤ العددي في وحدة `ذكاء`.',
    'لعبة_ثنائية_الأبعاد': '**محرك الألعاب (`لعبة_ثنائية_الأبعاد`):** بناء ألعاب أركيد عربية متكاملة.',
    'لوحة_راسبيري': '**راسبيري باي (`لوحة_راسبيري`):** إدارة لوحات Raspberry Pi ومنافذ GPIO.',
    'لوحة_بيكو': '**راسبيري بيكو (`لوحة_بيكو`):** برمجة لوحات RP2040 / Pico W.'
};

const FALLBACK_COMPLETIONS = [
    ['دع', 'تعريف متغير جديد', vscode.CompletionItemKind.Keyword],
    ['دالة', 'تعريف دالة جديدة', vscode.CompletionItemKind.Keyword],
    ['أرجع', 'إرجاع قيمة من الدالة', vscode.CompletionItemKind.Keyword],
    ['إذا', 'جملة شرطية', vscode.CompletionItemKind.Keyword],
    ['وإلا', 'البديل الشرطي', vscode.CompletionItemKind.Keyword],
    ['لكل', 'حلقة تكرار للعناصر', vscode.CompletionItemKind.Keyword],
    ['طالما', 'حلقة تكرار شرطية', vscode.CompletionItemKind.Keyword],
    ['في', 'معامل الانتماء أو التكرار', vscode.CompletionItemKind.Keyword],
    ['صنف', 'تعريف صنف كائني', vscode.CompletionItemKind.Keyword],
    ['هذا', 'المشير للكائن الحالي', vscode.CompletionItemKind.Keyword],
    ['يرث', 'وراثة صنف أب', vscode.CompletionItemKind.Keyword],
    ['جديد', 'إنشاء نسخة من صنف', vscode.CompletionItemKind.Keyword],
    ['استورد', 'استيراد وحدة أو ملف', vscode.CompletionItemKind.Keyword],
    ['من', 'استيراد رموز محددة', vscode.CompletionItemKind.Keyword],
    ['مع', 'مدير السياق للملفات والموارد', vscode.CompletionItemKind.Keyword],
    ['حاول', 'بدء كتلة معالجة الأخطاء', vscode.CompletionItemKind.Keyword],
    ['التقط', 'التقاط الاستثناء', vscode.CompletionItemKind.Keyword],
    ['أخيرًا', 'كتلة تنفذ دائماً بعد المحاولة', vscode.CompletionItemKind.Keyword],
    ['اختر', 'مطابقة الأنماط', vscode.CompletionItemKind.Keyword],
    ['حالة', 'فرع مطابقة نمط', vscode.CompletionItemKind.Keyword],
    ['افتراضي', 'الحالة الافتراضية للمطابقة', vscode.CompletionItemKind.Keyword],
    ['صحيح', 'قيمة منطقية موجبة', vscode.CompletionItemKind.Constant],
    ['خطأ', 'قيمة منطقية سالبة', vscode.CompletionItemKind.Constant],
    ['لاشيء', 'قيمة فارغة', vscode.CompletionItemKind.Constant],
    ['اطبع', 'طباعة إلى المخرجات', vscode.CompletionItemKind.Function],
    ['طول', 'حساب الطول', vscode.CompletionItemKind.Function],
    ['مدى', 'إنشاء تسلسل أرقام', vscode.CompletionItemKind.Function],
    ['نوع', 'معرفة نوع القيمة', vscode.CompletionItemKind.Function],
    ['تحقق', 'تحقق اختباري لشرط', vscode.CompletionItemKind.Function],
    ['تحقق_يساوي', 'تحقق من تساوي قيمتين', vscode.CompletionItemKind.Function],
    ['عدد', 'نوع عددي تدريجي', vscode.CompletionItemKind.TypeParameter],
    ['نص', 'نوع نصي تدريجي', vscode.CompletionItemKind.TypeParameter],
    ['منطقي', 'نوع منطقي تدريجي', vscode.CompletionItemKind.TypeParameter],
    ['قائمة', 'نوع قائمة تدريجي', vscode.CompletionItemKind.TypeParameter],
    ['قاموس', 'نوع قاموس تدريجي', vscode.CompletionItemKind.TypeParameter],
    ['صف', 'نوع صف ثابت', vscode.CompletionItemKind.TypeParameter],
    ['مجموعة', 'نوع مجموعة فريدة', vscode.CompletionItemKind.TypeParameter],
    ['فارغ', 'نوع إرجاع فارغ', vscode.CompletionItemKind.TypeParameter],
    ['أي', 'نوع عام يقبل أي قيمة', vscode.CompletionItemKind.TypeParameter]
];

const FORBIDDEN_ENGLISH_KEYWORDS = new Set([
    'function', 'def', 'class', 'return', 'import', 'from', 'if', 'else', 'elif',
    'while', 'for', 'let', 'var', 'const', 'print', 'true', 'false', 'null', 'none',
    'try', 'catch', 'except', 'finally', 'throw', 'raise', 'switch', 'case', 'default',
    'break', 'continue', 'new', 'this', 'self', 'extends'
]);

// ─── أدوات مساعدة عامة ──────────────────────────────────────────────────────

function getConfig() {
    return vscode.workspace.getConfiguration('lisan');
}

function getLisanPath() {
    return getConfig().get('executablePath', 'lisan') || 'lisan';
}

function getOutputChannel() {
    if (!outputChannel) {
        outputChannel = vscode.window.createOutputChannel('لسان (Lisan)');
    }
    return outputChannel;
}

function getOrCreateTerminal() {
    if (lisanTerminal && lisanTerminal.exitStatus === undefined) {
        return lisanTerminal;
    }
    lisanTerminal = vscode.window.createTerminal({ name: 'لسان (Lisan)' });
    return lisanTerminal;
}

function quoteShellArg(arg) {
    if (process.platform === 'win32') {
        return `"${arg.replace(/"/g, '\\"')}"`;
    }
    return `'${arg.replace(/'/g, "'\\''")}'`;
}

function execLisan(args, cwd) {
    const lisanPath = getLisanPath();
    return new Promise((resolve) => {
        cp.execFile(
            lisanPath,
            args,
            { cwd, maxBuffer: 10 * 1024 * 1024, encoding: 'utf8' },
            (err, stdout, stderr) => {
                resolve({
                    code: err ? (typeof err.code === 'number' ? err.code : 1) : 0,
                    stdout: stdout || '',
                    stderr: stderr || '',
                    error: err || null
                });
            }
        );
    });
}

async function getActiveLisanDocument() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('يرجى فتح ملف لسان (.lsn) أولاً.');
        return null;
    }
    const doc = editor.document;
    const isLisan =
        doc.languageId === 'lisan' ||
        doc.fileName.endsWith('.lsn') ||
        doc.fileName.endsWith('.lisan');
    if (!isLisan) {
        vscode.window.showWarningMessage('الملف المفتوح حالياً ليس ملف لسان (.lsn).');
        return null;
    }
    if (doc.isDirty) {
        await doc.save();
    }
    return doc;
}

// ─── جسر خادم اللغة المدمج (Zero-Dependency LSP Stdio Bridge) ──────────────

class LisanLspBridge {
    /**
     * @param {string} command
     * @param {vscode.DiagnosticCollection} diagnostics
     * @param {vscode.OutputChannel} logger
     */
    constructor(command, diagnostics, logger) {
        this.command = command;
        this.diagnostics = diagnostics;
        this.logger = logger;
        /** @type {cp.ChildProcess | null} */
        this.proc = null;
        this.nextId = 1;
        /** @type {Map<number, { resolve: Function, reject: Function, timer: NodeJS.Timeout }>} */
        this.pending = new Map();
        this.buffer = Buffer.alloc(0);
        this.ready = false;
        this.disposed = false;
        /** @type {Map<string, number>} */
        this.docVersions = new Map();
    }

    start() {
        if (this.disposed || this.proc) {
            return;
        }
        try {
            this.proc = cp.spawn(this.command, ['--lsp'], {
                stdio: ['pipe', 'pipe', 'pipe']
            });
        } catch (err) {
            this.logger.appendLine(`[LSP] تعذّر تشغيل خادم لغة لسان: ${err.message}`);
            return;
        }

        this.proc.stdout.on('data', (chunk) => this._onData(chunk));
        this.proc.stderr.on('data', (chunk) => {
            const text = chunk.toString('utf8').trim();
            if (text) {
                this.logger.appendLine(`[LSP stderr] ${text}`);
            }
        });
        this.proc.on('error', (err) => {
            this.logger.appendLine(`[LSP] تعذّر الاتصال بـ '${this.command} --lsp': ${err.message}`);
            this.ready = false;
            this.proc = null;
            this._rejectAll(err);
        });
        this.proc.on('exit', (code) => {
            this.ready = false;
            this.proc = null;
            this._rejectAll(new Error(`توقف خادم LSP برمز الخروج ${code}`));
        });

        const rootUri =
            vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0
                ? vscode.workspace.workspaceFolders[0].uri.toString()
                : null;

        this.sendRequest('initialize', {
            processId: process.pid,
            rootUri,
            capabilities: {
                textDocument: {
                    hover: { contentFormat: ['markdown', 'plaintext'] },
                    completion: { completionItem: { snippetSupport: true } },
                    publishDiagnostics: {}
                }
            }
        })
            .then(() => {
                this.ready = true;
                this.sendNotification('initialized', {});
                // إرسال المستندات المفتوحة حالياً
                for (const doc of vscode.workspace.textDocuments) {
                    if (doc.languageId === 'lisan') {
                        this.didOpen(doc);
                    }
                }
            })
            .catch(() => {
                this.ready = false;
            });
    }

    _rejectAll(err) {
        for (const [, entry] of this.pending) {
            clearTimeout(entry.timer);
            entry.reject(err);
        }
        this.pending.clear();
    }

    _writeMessage(payload) {
        if (!this.proc || !this.proc.stdin || this.proc.stdin.destroyed) {
            return false;
        }
        const json = JSON.stringify(payload);
        const byteLen = Buffer.byteLength(json, 'utf8');
        const frame = `Content-Length: ${byteLen}\r\n\r\n${json}`;
        try {
            this.proc.stdin.write(frame, 'utf8');
            return true;
        } catch (_) {
            return false;
        }
    }

    sendRequest(method, params, timeoutMs = 4000) {
        return new Promise((resolve, reject) => {
            if (!this.proc) {
                reject(new Error('خادم LSP غير نشط'));
                return;
            }
            const id = this.nextId++;
            const timer = setTimeout(() => {
                this.pending.delete(id);
                reject(new Error(`انتهت مهلة طلب LSP: ${method}`));
            }, timeoutMs);
            this.pending.set(id, { resolve, reject, timer });
            const ok = this._writeMessage({ jsonrpc: '2.0', id, method, params });
            if (!ok) {
                clearTimeout(timer);
                this.pending.delete(id);
                reject(new Error('تعذّر إرسال رسالة إلى خادم LSP'));
            }
        });
    }

    sendNotification(method, params) {
        this._writeMessage({ jsonrpc: '2.0', method, params });
    }

    _onData(chunk) {
        this.buffer = Buffer.concat([this.buffer, chunk]);
        while (true) {
            const headerEnd = this.buffer.indexOf('\r\n\r\n');
            if (headerEnd === -1) {
                break;
            }
            const headerText = this.buffer.slice(0, headerEnd).toString('ascii');
            const match = /content-length:\s*(\d+)/i.exec(headerText);
            if (!match) {
                this.buffer = this.buffer.slice(headerEnd + 4);
                continue;
            }
            const contentLength = parseInt(match[1], 10);
            const totalLength = headerEnd + 4 + contentLength;
            if (this.buffer.length < totalLength) {
                break;
            }
            const bodyBuf = this.buffer.slice(headerEnd + 4, totalLength);
            this.buffer = this.buffer.slice(totalLength);
            try {
                const msg = JSON.parse(bodyBuf.toString('utf8'));
                this._handleIncoming(msg);
            } catch (err) {
                this.logger.appendLine(`[LSP] خطأ تحليل JSON: ${err.message}`);
            }
        }
    }

    _handleIncoming(msg) {
        if (typeof msg.id !== 'undefined' && this.pending.has(msg.id)) {
            const entry = this.pending.get(msg.id);
            this.pending.delete(msg.id);
            clearTimeout(entry.timer);
            if (msg.error) {
                entry.reject(new Error(msg.error.message || 'خطأ LSP'));
            } else {
                entry.resolve(msg.result);
            }
            return;
        }

        if (msg.method === 'textDocument/publishDiagnostics' && msg.params) {
            const uriStr = msg.params.uri;
            const rawDiags = Array.isArray(msg.params.diagnostics) ? msg.params.diagnostics : [];
            if (!uriStr) {
                return;
            }
            const uri = vscode.Uri.parse(uriStr);
            const vsDiags = rawDiags.map((d) => {
                const startLine = Math.max(0, (d.range && d.range.start && d.range.start.line) || 0);
                const startChar = Math.max(0, (d.range && d.range.start && d.range.start.character) || 0);
                const endLine = Math.max(startLine, (d.range && d.range.end && d.range.end.line) || startLine);
                const endChar = Math.max(startChar + 1, (d.range && d.range.end && d.range.end.character) || startChar + 1);
                const severity =
                    d.severity === 2
                        ? vscode.DiagnosticSeverity.Warning
                        : d.severity === 3
                            ? vscode.DiagnosticSeverity.Information
                            : vscode.DiagnosticSeverity.Error;
                const diag = new vscode.Diagnostic(
                    new vscode.Range(startLine, startChar, endLine, endChar),
                    d.message || 'خطأ في شيفرة لسان',
                    severity
                );
                diag.source = d.source || 'لسان';
                return diag;
            });

            // دمج فحص العربية الخالصة إذا كان مفعّلاً
            const openDoc = vscode.workspace.textDocuments.find((doc) => doc.uri.toString() === uriStr);
            if (openDoc && getConfig().get('arabicOnlyStrict', true)) {
                vsDiags.push(...collectArabicOnlyDiagnostics(openDoc));
            }
            this.diagnostics.set(uri, vsDiags);
        }
    }

    didOpen(doc) {
        const uri = doc.uri.toString();
        const ver = 1;
        this.docVersions.set(uri, ver);
        if (this.ready) {
            this.sendNotification('textDocument/didOpen', {
                textDocument: {
                    uri,
                    languageId: 'lisan',
                    version: ver,
                    text: doc.getText()
                }
            });
        } else if (getConfig().get('arabicOnlyStrict', true)) {
            this.diagnostics.set(doc.uri, collectArabicOnlyDiagnostics(doc));
        }
    }

    didChange(doc) {
        const uri = doc.uri.toString();
        const ver = (this.docVersions.get(uri) || 0) + 1;
        this.docVersions.set(uri, ver);
        if (this.ready) {
            this.sendNotification('textDocument/didChange', {
                textDocument: { uri, version: ver },
                contentChanges: [{ text: doc.getText() }]
            });
        } else if (getConfig().get('arabicOnlyStrict', true)) {
            this.diagnostics.set(doc.uri, collectArabicOnlyDiagnostics(doc));
        }
    }

    didSave(doc) {
        this.didChange(doc);
    }

    didClose(doc) {
        const uri = doc.uri.toString();
        this.docVersions.delete(uri);
        this.diagnostics.delete(doc.uri);
        if (this.ready) {
            this.sendNotification('textDocument/didClose', {
                textDocument: { uri }
            });
        }
    }

    stop() {
        this.disposed = true;
        this.ready = false;
        this._rejectAll(new Error('تم إغلاق خادم LSP'));
        if (this.proc) {
            try {
                this.sendRequest('shutdown', null, 500).catch(() => {});
                this.sendNotification('exit', null);
                this.proc.kill();
            } catch (_) {
                // تجاهل أخطاء الإغلاق
            }
            this.proc = null;
        }
    }
}

/**
 * فحص العربية الخالصة: اكتشاف الكلمات المفتاحية الإنجليزية الدخيلة خارج النصوص والتعليقات
 * @param {vscode.TextDocument} document
 * @returns {vscode.Diagnostic[]}
 */
function collectArabicOnlyDiagnostics(document) {
    const diagnostics = [];
    const lineCount = document.lineCount;

    for (let lineIdx = 0; lineIdx < lineCount; lineIdx++) {
        const line = document.lineAt(lineIdx).text;
        let inString = false;
        let i = 0;
        while (i < line.length) {
            const ch = line[i];
            if (inString) {
                if (ch === '\\') {
                    i += 2;
                    continue;
                }
                if (ch === '"') {
                    inString = false;
                }
                i++;
                continue;
            }
            if (ch === '#' || (ch === '/' && line[i + 1] === '/')) {
                break;
            }
            if (ch === '"') {
                inString = true;
                i++;
                continue;
            }
            if (/[A-Za-z]/.test(ch)) {
                const start = i;
                while (i < line.length && /[A-Za-z0-9_]/.test(line[i])) {
                    i++;
                }
                const word = line.slice(start, i);
                if (FORBIDDEN_ENGLISH_KEYWORDS.has(word.toLowerCase())) {
                    const diag = new vscode.Diagnostic(
                        new vscode.Range(lineIdx, start, lineIdx, i),
                        `تنبيه العربية الخالصة: الكلمة الإنجليزية «${word}» غير مستخدمة ككلمة مفتاحية في لسان، استخدم المقابل العربي الفصيح.`,
                        vscode.DiagnosticSeverity.Warning
                    );
                    diag.source = 'لسان (العربية الخالصة)';
                    diagnostics.push(diag);
                }
                continue;
            }
            i++;
        }
    }
    return diagnostics;
}

function mapLspCompletionKind(lspKind) {
    switch (lspKind) {
        case 3: return vscode.CompletionItemKind.Function;
        case 7: return vscode.CompletionItemKind.Class;
        case 9: return vscode.CompletionItemKind.Module;
        case 14: return vscode.CompletionItemKind.Keyword;
        case 21: return vscode.CompletionItemKind.Constant;
        default: return vscode.CompletionItemKind.Text;
    }
}

// ─── تنسيق ملفات لسان عبر lisan --fmt ───────────────────────────────────────

async function formatLisanSource(text) {
    const tmpFile = path.join(os.tmpdir(), `lisan_fmt_${process.pid}_${Date.now()}.lsn`);
    try {
        fs.writeFileSync(tmpFile, text, 'utf8');
        const res = await execLisan(['--fmt', tmpFile], os.tmpdir());
        if (res.code === 0 && fs.existsSync(tmpFile)) {
            return fs.readFileSync(tmpFile, 'utf8');
        }
        return null;
    } catch (_) {
        return null;
    } finally {
        try {
            if (fs.existsSync(tmpFile)) {
                fs.unlinkSync(tmpFile);
            }
        } catch (_) {
            // تجاهل
        }
    }
}

// ─── معاينة الويب وتطبيقات الهاتف داخل VS Code ──────────────────────────────

function escapeHtml(str) {
    return str
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

function extractRenderablePreview(stdout, cwd, startedAtMs) {
    const trimmed = stdout.trim();
    const lower = trimmed.toLowerCase();

    // 1) إذا طُبع HTML كامل مباشرة في المخرجات
    const docIdx = lower.indexOf('<!doctype html');
    if (docIdx !== -1) {
        return { kind: 'html', content: trimmed.slice(docIdx) };
    }
    const htmlIdx = lower.indexOf('<html');
    if (htmlIdx !== -1) {
        return { kind: 'html', content: trimmed.slice(htmlIdx) };
    }

    // 2) إذا طُبع رسم SVG مباشرة
    const svgIdx = lower.indexOf('<svg');
    if (svgIdx !== -1) {
        const svgEnd = lower.lastIndexOf('</svg>');
        const svgContent = svgEnd !== -1 ? trimmed.slice(svgIdx, svgEnd + 6) : trimmed.slice(svgIdx);
        return { kind: 'svg', content: svgContent };
    }

    // 3) البحث عن أحدث ملف HTML أو SVG مُولَّد في نفس المجلد (مثل index.html أو dist_app/index.html)
    const candidates = [
        path.join(cwd, 'index.html'),
        path.join(cwd, 'dist_app', 'index.html'),
        path.join(cwd, 'موقع', 'index.html'),
        path.join(cwd, 'تطبيق', 'index.html')
    ];
    try {
        for (const entry of fs.readdirSync(cwd)) {
            if (entry.endsWith('.html') || entry.endsWith('.svg')) {
                const full = path.join(cwd, entry);
                if (!candidates.includes(full)) {
                    candidates.push(full);
                }
            }
        }
    } catch (_) {
        // تجاهل
    }

    let newest = null;
    for (const file of candidates) {
        try {
            if (fs.existsSync(file)) {
                const stat = fs.statSync(file);
                if (stat.mtimeMs >= startedAtMs - 2000) {
                    if (!newest || stat.mtimeMs > newest.mtimeMs) {
                        newest = { file, mtimeMs: stat.mtimeMs };
                    }
                }
            }
        } catch (_) {
            // تجاهل
        }
    }

    if (newest) {
        const content = fs.readFileSync(newest.file, 'utf8');
        if (newest.file.endsWith('.svg')) {
            return { kind: 'svg', content, sourceFile: newest.file };
        }
        return { kind: 'html', content, sourceFile: newest.file };
    }

    return { kind: 'text', content: stdout };
}

function buildPreviewWrapperHtml(title, bodyHtml, isError = false) {
    return `<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>${escapeHtml(title)}</title>
<style>
  body {
    font-family: "Tajawal", "Cairo", system-ui, -apple-system, sans-serif;
    background: #0f172a;
    color: #e2e8f0;
    margin: 0;
    padding: 20px;
    direction: rtl;
  }
  .card {
    background: #1e293b;
    border: 1px solid ${isError ? '#ef4444' : '#334155'};
    border-radius: 14px;
    padding: 18px 22px;
    max-width: 960px;
    margin: 0 auto;
    box-shadow: 0 12px 30px rgba(0,0,0,0.35);
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 14px;
    border-bottom: 1px solid #334155;
    padding-bottom: 10px;
  }
  .badge {
    background: ${isError ? '#991b1b' : '#0f766e'};
    color: #fff;
    padding: 4px 12px;
    border-radius: 999px;
    font-size: 12px;
  }
  pre {
    background: #0b1120;
    color: #f8fafc;
    padding: 14px;
    border-radius: 10px;
    overflow-x: auto;
    line-height: 1.7;
    white-space: pre-wrap;
  }
  .svg-stage {
    background: #ffffff;
    border-radius: 10px;
    padding: 16px;
    display: flex;
    justify-content: center;
    align-items: center;
    overflow: auto;
  }
</style>
</head>
<body>
  <div class="card">
    <div class="header">
      <strong>${escapeHtml(title)}</strong>
      <span class="badge">${isError ? 'خطأ تشغيل' : 'معاينة لسان الحية'}</span>
    </div>
    ${bodyHtml}
  </div>
</body>
</html>`;
}

async function updateLivePreview(filePath) {
    if (!previewPanel) {
        return;
    }
    const cwd = path.dirname(filePath);
    const fileName = path.basename(filePath);
    const startedAt = Date.now();
    const res = await execLisan(['--interpret', filePath], cwd);

    if (res.code !== 0) {
        const errText = res.stderr || res.stdout || (res.error ? res.error.message : 'خطأ غير معروف');
        previewPanel.webview.html = buildPreviewWrapperHtml(
            `معاينة لسان: ${fileName}`,
            `<pre>${escapeHtml(errText)}</pre>`,
            true
        );
        return;
    }

    const preview = extractRenderablePreview(res.stdout, cwd, startedAt);
    if (preview.kind === 'html') {
        previewPanel.webview.html = preview.content;
    } else if (preview.kind === 'svg') {
        previewPanel.webview.html = buildPreviewWrapperHtml(
            `معاينة رسم SVG — ${fileName}`,
            `<div class="svg-stage">${preview.content}</div>`
        );
    } else {
        const out = preview.content.trim() || '(لم تُطبع مخرجات — استخدم موقع.بناء_html() أو تطبيق.بناء_واجهة() أو حفظ("index.html"))';
        previewPanel.webview.html = buildPreviewWrapperHtml(
            `مخرجات لسان — ${fileName}`,
            `<pre>${escapeHtml(out)}</pre>`
        );
    }
}

// ─── قوالب معالج إنشاء مشروع لسان جديد ──────────────────────────────────────

const PROJECT_TEMPLATES = {
    'ويب': {
        label: 'موقع ويب متجاوب (ويب)',
        description: 'قالب موقع ويب عربي متجاوب باستخدام وحدة موقع',
        code: `# مشروع موقع ويب عربي بلغة لسان
استورد "موقع"

دع موقع = جديد موقع_ويب("بوابتي العربية"، "ملكي")
موقع.شريط_تنقل("بوابتي العربية")
موقع.قسم_رئيسي(
    "مرحباً بك في لسان"،
    "موقع ويب عربي سريع ومتجاوب مبني بالكامل بلغة البرمجة العربية لسان."،
    "ابدأ الآن"،
    "شمس"
)
موقع.أضف_بطاقة("عربية خالصة"، "شيفرة واضحة ومعبّرة بدون كلمات مفتاحية دخيلة."، "نجمة")
موقع.أضف_بطاقة("معاينة فورية"، "اضغط Ctrl+Alt+P لمعاينة الموقع حياً داخل VS Code."، "برق")
موقع.أضف_تذييل("جميع الحقوق محفوظة © مشروع لسان")
موقع.حفظ("index.html")
اطبع(موقع.بناء_html())
`
    },
    'تطبيق_هاتف': {
        label: 'تطبيق هاتف ذكي (تطبيق_هاتف)',
        description: 'تطبيق هاتف ويب تقدمي (PWA) بواجهة عربية تفاعلية',
        code: `# مشروع تطبيق هاتف ذكي بلغة لسان
استورد "هاتف"

دع تطبيق = جديد تطبيق_هاتف("مهامي اليومية"، "زمردي")
تطبيق.أضف_إحصائية("المهام المنجزة"، "8"، "نجمة")
تطبيق.أضف_إحصائية("قيد التنفيذ"، "3"، "برق")
تطبيق.أضف_بطاقة("أهلاً بك"، "تطبيق هاتف ذكي مكتوب بالكامل بلغة لسان."، "هاتف")
تطبيق.أضف_مفتاح_تحكم("تفعيل التنبيهات الذكية"، "جرس"، صحيح)
تطبيق.أضف_زر("إضافة مهمة جديدة"، "زائد"، "تمت إضافة المهمة بنجاح")
اطبع(تطبيق.بناء_واجهة())
`
    },
    'ذكاء_اصطناعي': {
        label: 'ذكاء اصطناعي وعلوم بيانات (ذكاء_اصطناعي)',
        description: 'نموذج انحدار خطي ومخطط بياني SVG باستخدام وحدة ذكاء',
        code: `# مشروع ذكاء اصطناعي وتعلم آلي بلغة لسان
استورد "ذكاء"

دع نموذج = جديد انحدار_خطي()
دع س = [1، 2، 3، 4، 5]
دع ص = [2.5، 5.0، 7.5، 10.0، 12.5]
نموذج.درّب(س، ص)

دع توقع = نموذج.توقّع(6)
اطبع(ن"التوقّع للنقطة 6 = {توقع}")

دع مخطط = جديد مخطط_أعمدة("نمو القيم المتوقعة")
مخطط.أضف_بيان("س=2"، 5.0)
مخطط.أضف_بيان("س=4"، 10.0)
مخطط.أضف_بيان("س=6"، توقع)
اطبع(مخطط.بناء_svg())
`
    },
    'متحكم': {
        label: 'متحكم دقيق وإنترنت الأشياء (متحكم)',
        description: 'مشروع ESP32 / Arduino / Pico لقراءة الحساسات والتحكم بالعتاد',
        code: `# مشروع متحكم دقيق بلغة لسان (قابل للترجمة إلى ESP32 / Arduino / STM32 / Pico)
استورد "اردوينو"

دع حساس = جديد حساس_حرارة(2)
دع مرحل = جديد مرحل_كهربائي(8)

دالة فحص_المناخ() {
    دع الحرارة = حساس.اقرأ_حرارة()
    اطبع(ن"درجة الحرارة الحالية: {الحرارة} م°")
    إذا (الحرارة > 28) {
        مرحل.تشغيل()
        اطبع("تم تشغيل نظام التبريد")
    } وإلا {
        مرحل.إيقاف()
        اطبع("درجة الحرارة ضمن النطاق الطبيعي")
    }
}

فحص_المناخ()
`
    },
    'أداة_نظام': {
        label: 'أداة نظام وسطر أوامر (أداة_نظام)',
        description: 'أداة سطر أوامر عربية مع سجل أحداث واختبارات مدمجة',
        code: `# مشروع أداة نظام بلغة لسان
استورد "ألعاب"

دع سجل = جديد سجل_أحداث("أداة_لسان")
سجل.معلومات("بدء تشغيل الأداة")

دالة احسب_المجموع(عناصر: قائمة) -> عدد {
    دع المجموع = 0
    لكل قيمة في عناصر {
        المجموع = المجموع + قيمة
    }
    أرجع المجموع
}

دع النتيجة = احسب_المجموع([10، 20، 30، 40])
تحقق_يساوي(النتيجة، 100)
سجل.معلومات(ن"اكتمل الحساب بنجاح: {النتيجة}")
اطبع(ن"النتيجة النهائية = {النتيجة}")
`
    }
};

// ─── أهداف المتحكمات الدقيقة ────────────────────────────────────────────────

const MCU_TARGETS = [
    {
        id: 'arduino',
        label: 'Arduino (UNO / Mega / Nano — C++)',
        flag: '--arduino',
        ext: '.ino',
        fqbn: 'arduino:avr:uno',
        uploadTool: 'arduino-cli'
    },
    {
        id: 'esp32',
        label: 'ESP32 / ESP8266 (WiFi / BLE / CAM — C++)',
        flag: '--esp32',
        ext: '.ino',
        fqbn: 'esp32:esp32:esp32',
        uploadTool: 'arduino-cli'
    },
    {
        id: 'stm32',
        label: 'STM32 (ARM Cortex-M — C++)',
        flag: '--stm32',
        ext: '.ino',
        fqbn: 'STMicroelectronics:stm32:GenF1',
        uploadTool: 'arduino-cli'
    },
    {
        id: 'raspberry',
        label: 'Raspberry Pi (GPIO / Camera / SenseHAT — Python 3)',
        flag: '--raspberry',
        ext: '_rpi.py',
        fqbn: null,
        uploadTool: 'python3'
    },
    {
        id: 'pico',
        label: 'Raspberry Pi Pico / Pico W (RP2040 — MicroPython)',
        flag: '--pico',
        ext: '_pico.py',
        fqbn: null,
        uploadTool: 'mpremote'
    }
];

function getSortedMcuTargets() {
    const def = getConfig().get('mcuDefaultTarget', 'esp32');
    return [...MCU_TARGETS].sort((a, b) => (a.id === def ? -1 : b.id === def ? 1 : 0));
}

// ─── نقطة التفعيل الرئيسية للإضافة ──────────────────────────────────────────

/**
 * تفعيل إضافة لسان الكاملة في Visual Studio Code
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
    const logger = getOutputChannel();
    diagnosticCollection = vscode.languages.createDiagnosticCollection('lisan');
    context.subscriptions.push(logger, diagnosticCollection);

    // 1) تشغيل جسر خادم لغة لسان المدمج (LSP) بدون اعتماديات خارجية
    const lspEnabled = getConfig().get('lsp.enabled', true);
    if (lspEnabled) {
        lspBridge = new LisanLspBridge(getLisanPath(), diagnosticCollection, logger);
        lspBridge.start();
    }

    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument((doc) => {
            if (doc.languageId === 'lisan' && lspBridge) {
                lspBridge.didOpen(doc);
            }
        }),
        vscode.workspace.onDidChangeTextDocument((event) => {
            if (event.document.languageId === 'lisan' && lspBridge) {
                lspBridge.didChange(event.document);
            }
        }),
        vscode.workspace.onDidSaveTextDocument((doc) => {
            if (doc.languageId === 'lisan') {
                if (lspBridge) {
                    lspBridge.didSave(doc);
                }
                if (previewPanel && previewTargetFile && doc.fileName === previewTargetFile) {
                    updateLivePreview(doc.fileName);
                }
            }
        }),
        vscode.workspace.onDidCloseTextDocument((doc) => {
            if (doc.languageId === 'lisan' && lspBridge) {
                lspBridge.didClose(doc);
            }
        })
    );

    // 2) مزوّد التحويم (Hover) عبر LSP مع مساند عربي مدمج
    context.subscriptions.push(
        vscode.languages.registerHoverProvider('lisan', {
            async provideHover(document, position) {
                const wordRange = document.getWordRangeAtPosition(
                    position,
                    /[\u0600-\u06FF_a-zA-Z0-9\u0640]+/
                );
                const word = wordRange ? document.getText(wordRange) : '';

                if (lspBridge && lspBridge.ready) {
                    try {
                        const res = await lspBridge.sendRequest('textDocument/hover', {
                            textDocument: { uri: document.uri.toString() },
                            position: { line: position.line, character: position.character }
                        }, 1500);
                        if (res && res.contents) {
                            const md = typeof res.contents === 'string'
                                ? res.contents
                                : (res.contents.value || '');
                            if (md) {
                                return new vscode.Hover(new vscode.MarkdownString(md), wordRange);
                            }
                        }
                    } catch (_) {
                        // الانتقال للمساند المحلي
                    }
                }

                if (word && ARABIC_HOVER_DOCS[word]) {
                    return new vscode.Hover(new vscode.MarkdownString(ARABIC_HOVER_DOCS[word]), wordRange);
                }
                return null;
            }
        })
    );

    // 3) مزوّد الإكمال التلقائي (Completion) عبر LSP مع مساند مدمج
    context.subscriptions.push(
        vscode.languages.registerCompletionItemProvider(
            'lisan',
            {
                async provideCompletionItems(document, position) {
                    if (lspBridge && lspBridge.ready) {
                        try {
                            const res = await lspBridge.sendRequest('textDocument/completion', {
                                textDocument: { uri: document.uri.toString() },
                                position: { line: position.line, character: position.character }
                            }, 2000);
                            const rawItems = Array.isArray(res) ? res : (res && Array.isArray(res.items) ? res.items : []);
                            if (rawItems.length > 0) {
                                return rawItems.map((it) => {
                                    const ci = new vscode.CompletionItem(it.label, mapLspCompletionKind(it.kind));
                                    ci.detail = it.detail || '';
                                    if (it.documentation) {
                                        ci.documentation = new vscode.MarkdownString(
                                            typeof it.documentation === 'string'
                                                ? it.documentation
                                                : (it.documentation.value || '')
                                        );
                                    }
                                    return ci;
                                });
                            }
                        } catch (_) {
                            // الانتقال للمساند المحلي
                        }
                    }

                    return FALLBACK_COMPLETIONS.map(([label, detail, kind]) => {
                        const ci = new vscode.CompletionItem(label, kind);
                        ci.detail = detail;
                        return ci;
                    });
                }
            },
            '.',
            ' '
        )
    );

    // 4) مزوّد تنسيق الشيفرة عبر lisan --fmt
    context.subscriptions.push(
        vscode.languages.registerDocumentFormattingEditProvider('lisan', {
            async provideDocumentFormattingEdits(document) {
                const original = document.getText();
                const formatted = await formatLisanSource(original);
                if (!formatted || formatted === original) {
                    return [];
                }
                const fullRange = new vscode.Range(
                    document.positionAt(0),
                    document.positionAt(original.length)
                );
                return [vscode.TextEdit.replace(fullRange, formatted)];
            }
        })
    );

    // 5) مصحح الأخطاء (DAP): تسجيل المصنع ومزوّد الإعدادات
    context.subscriptions.push(
        vscode.debug.registerDebugAdapterDescriptorFactory('lisan', {
            createDebugAdapterDescriptor() {
                return new vscode.DebugAdapterExecutable(getLisanPath(), ['--dap']);
            }
        }),
        vscode.debug.registerDebugConfigurationProvider('lisan', {
            resolveDebugConfiguration(_folder, config) {
                if (!config.type && !config.request && !config.name) {
                    const editor = vscode.window.activeTextEditor;
                    if (editor && editor.document.languageId === 'lisan') {
                        config.type = 'lisan';
                        config.name = 'تصحيح ملف لسان الحالي';
                        config.request = 'launch';
                        config.program = '${file}';
                        config.stopOnEntry = false;
                    }
                }
                if (!config.program) {
                    vscode.window.showErrorMessage('لم يُحدَّد ملف لسان (.lsn) لتصحيحه.');
                    return undefined;
                }
                return config;
            }
        })
    );

    // 6) تسجيل الأوامر الإحدى عشرة (11 Commands)

    // 6.1 تشغيل ملف لسان الحالي
    context.subscriptions.push(
        vscode.commands.registerCommand('lisan.runFile', async () => {
            const doc = await getActiveLisanDocument();
            if (!doc) {
                return;
            }
            const term = getOrCreateTerminal();
            term.show(true);
            term.sendText(`${quoteShellArg(getLisanPath())} --interpret ${quoteShellArg(doc.fileName)}`);
        })
    );

    // 6.2 فحص ملف لسان (قواعد + عربية خالصة)
    context.subscriptions.push(
        vscode.commands.registerCommand('lisan.checkFile', async () => {
            const doc = await getActiveLisanDocument();
            if (!doc) {
                return;
            }
            const res = await execLisan(['--check', doc.fileName], path.dirname(doc.fileName));
            const strictDiags = getConfig().get('arabicOnlyStrict', true)
                ? collectArabicOnlyDiagnostics(doc)
                : [];

            logger.show(true);
            logger.appendLine(`── فحص ملف لسان: ${doc.fileName} ──`);

            if (res.code === 0) {
                diagnosticCollection.set(doc.uri, strictDiags);
                logger.appendLine(res.stdout.trim() || 'نجح فحص ملف لسان.');
                if (strictDiags.length > 0) {
                    vscode.window.showWarningMessage(
                        `نجح فحص القواعد، لكن وُجد ${strictDiags.length} تنبيه للعربية الخالصة.`
                    );
                } else {
                    vscode.window.showInformationMessage('✓ نجح فحص ملف لسان (قواعد سليمة وعربية خالصة).');
                }
            } else {
                const errOut = (res.stderr || res.stdout || 'خطأ في فحص الملف').trim();
                logger.appendLine(errOut);
                const lineMatch = /السطر\s+(\d+)/.exec(errOut);
                const colMatch = /العمود\s+(\d+)/.exec(errOut);
                const line = lineMatch ? Math.max(0, parseInt(lineMatch[1], 10) - 1) : 0;
                const col = colMatch ? Math.max(0, parseInt(colMatch[1], 10) - 1) : 0;
                const parseDiag = new vscode.Diagnostic(
                    new vscode.Range(line, col, line, col + 1),
                    errOut,
                    vscode.DiagnosticSeverity.Error
                );
                parseDiag.source = 'لسان';
                diagnosticCollection.set(doc.uri, [parseDiag, ...strictDiags]);
                vscode.window.showErrorMessage(`فشل فحص ملف لسان: ${errOut.split('\n')[0]}`);
            }
        })
    );

    // 6.3 تنسيق ملف لسان
    context.subscriptions.push(
        vscode.commands.registerCommand('lisan.formatFile', async () => {
            const doc = await getActiveLisanDocument();
            if (!doc) {
                return;
            }
            await vscode.commands.executeCommand('editor.action.formatDocument');
        })
    );

    // 6.4 تشغيل اختبارات لسان
    context.subscriptions.push(
        vscode.commands.registerCommand('lisan.runTests', async () => {
            const editor = vscode.window.activeTextEditor;
            const term = getOrCreateTerminal();
            term.show(true);
            if (editor && (editor.document.languageId === 'lisan' || editor.document.fileName.endsWith('.lsn'))) {
                if (editor.document.isDirty) {
                    await editor.document.save();
                }
                term.sendText(`${quoteShellArg(getLisanPath())} --test ${quoteShellArg(editor.document.fileName)}`);
            } else {
                term.sendText(`${quoteShellArg(getLisanPath())} --test`);
            }
        })
    );

    // 6.5 ترجمة إلى بايثون (Python 3)
    context.subscriptions.push(
        vscode.commands.registerCommand('lisan.transpilePython', async () => {
            const doc = await getActiveLisanDocument();
            if (!doc) {
                return;
            }
            const outFile = doc.fileName.replace(/\.(lsn|lisan)$/i, '') + '.py';
            const res = await execLisan(['--python', doc.fileName, '--output', outFile], path.dirname(doc.fileName));
            logger.show(true);
            logger.appendLine(`── ترجمة إلى بايثون: ${doc.fileName} ──`);
            if (res.code === 0) {
                logger.appendLine(res.stdout.trim() || `تم توليد ملف بايثون: ${outFile}`);
                vscode.window.showInformationMessage(`تمت الترجمة إلى بايثون بنجاح: ${path.basename(outFile)}`);
                if (fs.existsSync(outFile)) {
                    const pyDoc = await vscode.workspace.openTextDocument(outFile);
                    await vscode.window.showTextDocument(pyDoc, vscode.ViewColumn.Beside);
                }
            } else {
                const msg = (res.stderr || res.stdout || 'تعذّرت الترجمة إلى بايثون').trim();
                logger.appendLine(msg);
                vscode.window.showErrorMessage(`تعذّرت الترجمة إلى بايثون: ${msg.split('\n')[0]}`);
            }
        })
    );

    // 6.6 ترجمة إلى جافاسكربت (ES2022)
    context.subscriptions.push(
        vscode.commands.registerCommand('lisan.transpileJs', async () => {
            const doc = await getActiveLisanDocument();
            if (!doc) {
                return;
            }
            const outFile = doc.fileName.replace(/\.(lsn|lisan)$/i, '') + '.js';
            const res = await execLisan(['--js', doc.fileName, '--output', outFile], path.dirname(doc.fileName));
            logger.show(true);
            logger.appendLine(`── ترجمة إلى جافاسكربت: ${doc.fileName} ──`);
            if (res.code === 0) {
                logger.appendLine(res.stdout.trim() || `تم توليد ملف جافاسكربت: ${outFile}`);
                vscode.window.showInformationMessage(`تمت الترجمة إلى جافاسكربت بنجاح: ${path.basename(outFile)}`);
                if (fs.existsSync(outFile)) {
                    const jsDoc = await vscode.workspace.openTextDocument(outFile);
                    await vscode.window.showTextDocument(jsDoc, vscode.ViewColumn.Beside);
                }
            } else {
                const msg = (res.stderr || res.stdout || 'تعذّرت الترجمة إلى جافاسكربت').trim();
                logger.appendLine(msg);
                vscode.window.showErrorMessage(`تعذّرت الترجمة إلى جافاسكربت: ${msg.split('\n')[0]}`);
            }
        })
    );

    // 6.7 ترجمة للمتحكمات الدقيقة
    context.subscriptions.push(
        vscode.commands.registerCommand('lisan.transpileMcu', async () => {
            const doc = await getActiveLisanDocument();
            if (!doc) {
                return;
            }
            const targets = getSortedMcuTargets();
            const picked = await vscode.window.showQuickPick(
                targets.map((t) => ({
                    label: t.label,
                    description: t.flag,
                    target: t
                })),
                { placeHolder: 'اختر لوحة المتحكم المستهدفة للترجمة' }
            );
            if (!picked) {
                return;
            }
            const target = picked.target;
            const stem = doc.fileName.replace(/\.(lsn|lisan)$/i, '');
            const outFile = `${stem}${target.ext}`;
            const res = await execLisan([target.flag, doc.fileName, '--output', outFile], path.dirname(doc.fileName));
            logger.show(true);
            logger.appendLine(`── ترجمة للمتحكم (${target.id}): ${doc.fileName} ──`);
            if (res.code === 0) {
                logger.appendLine(res.stdout.trim() || `تم التوليد في: ${outFile}`);
                vscode.window.showInformationMessage(`تم توليد شيفرة ${target.id} بنجاح: ${path.basename(outFile)}`);
                if (fs.existsSync(outFile)) {
                    const genDoc = await vscode.workspace.openTextDocument(outFile);
                    await vscode.window.showTextDocument(genDoc, vscode.ViewColumn.Beside);
                }
            } else {
                const msg = (res.stderr || res.stdout || 'فشل توليد شيفرة المتحكم').trim();
                logger.appendLine(msg);
                vscode.window.showErrorMessage(`فشل ترجمة المتحكم: ${msg.split('\n')[0]}`);
            }
        })
    );

    // 6.8 رفع الشيفرة إلى المتحكم (Arduino CLI / mpremote)
    context.subscriptions.push(
        vscode.commands.registerCommand('lisan.uploadMcu', async () => {
            const doc = await getActiveLisanDocument();
            if (!doc) {
                return;
            }
            const targets = getSortedMcuTargets();
            const picked = await vscode.window.showQuickPick(
                targets.map((t) => ({
                    label: t.label,
                    description: `أداة الرفع: ${t.uploadTool}`,
                    target: t
                })),
                { placeHolder: 'اختر لوحة المتحكم المراد الرفع إليها' }
            );
            if (!picked) {
                return;
            }
            const target = picked.target;
            const defaultPort =
                getConfig().get('serialPort', '') ||
                (process.platform === 'win32'
                    ? 'COM3'
                    : target.id === 'pico'
                        ? '/dev/ttyACM0'
                        : '/dev/ttyUSB0');

            let port = defaultPort;
            if (target.uploadTool !== 'python3') {
                const enteredPort = await vscode.window.showInputBox({
                    prompt: 'أدخل المنفذ التسلسلي المتصل بالمتحكم (مثل /dev/ttyUSB0 أو /dev/ttyACM0 أو COM3)',
                    value: defaultPort
                });
                if (!enteredPort) {
                    return;
                }
                port = enteredPort.trim();
            }

            const baseDir = path.dirname(doc.fileName);
            const baseName = path.basename(doc.fileName).replace(/\.(lsn|lisan)$/i, '');
            const term = getOrCreateTerminal();
            term.show(true);

            if (target.uploadTool === 'arduino-cli') {
                // Arduino CLI يتطلب مجلداً باسم ملف .ino نفسه
                const sketchDir = path.join(baseDir, `${baseName}_${target.id}`);
                fs.mkdirSync(sketchDir, { recursive: true });
                const sketchFile = path.join(sketchDir, `${baseName}_${target.id}.ino`);
                const res = await execLisan([target.flag, doc.fileName, '--output', sketchFile], baseDir);
                if (res.code !== 0) {
                    vscode.window.showErrorMessage(`فشل ترجمة المتحكم قبل الرفع: ${(res.stderr || res.stdout).trim()}`);
                    return;
                }
                term.sendText(
                    `arduino-cli compile --fqbn ${target.fqbn} ${quoteShellArg(sketchDir)} && arduino-cli upload -p ${quoteShellArg(port)} --fqbn ${target.fqbn} ${quoteShellArg(sketchDir)}`
                );
            } else if (target.uploadTool === 'mpremote') {
                const outFile = path.join(baseDir, `${baseName}_pico.py`);
                const res = await execLisan([target.flag, doc.fileName, '--output', outFile], baseDir);
                if (res.code !== 0) {
                    vscode.window.showErrorMessage(`فشل ترجمة بيكو قبل الرفع: ${(res.stderr || res.stdout).trim()}`);
                    return;
                }
                term.sendText(
                    `mpremote connect ${quoteShellArg(port)} fs cp ${quoteShellArg(outFile)} :main.py + reset`
                );
            } else {
                const outFile = path.join(baseDir, `${baseName}_rpi.py`);
                const res = await execLisan([target.flag, doc.fileName, '--output', outFile], baseDir);
                if (res.code !== 0) {
                    vscode.window.showErrorMessage(`فشل ترجمة راسبيري باي: ${(res.stderr || res.stdout).trim()}`);
                    return;
                }
                term.sendText(`python3 ${quoteShellArg(outFile)}`);
            }
        })
    );

    // 6.9 معاينة موقع أو تطبيق هاتف لسان داخل VS Code
    context.subscriptions.push(
        vscode.commands.registerCommand('lisan.previewWeb', async () => {
            const doc = await getActiveLisanDocument();
            if (!doc) {
                return;
            }
            previewTargetFile = doc.fileName;
            if (!previewPanel) {
                previewPanel = vscode.window.createWebviewPanel(
                    'lisanLivePreview',
                    `معاينة لسان: ${path.basename(doc.fileName)}`,
                    vscode.ViewColumn.Beside,
                    { enableScripts: true }
                );
                previewPanel.onDidDispose(() => {
                    previewPanel = null;
                    previewTargetFile = null;
                });
            } else {
                previewPanel.title = `معاينة لسان: ${path.basename(doc.fileName)}`;
                previewPanel.reveal(vscode.ViewColumn.Beside, true);
            }
            await updateLivePreview(doc.fileName);
        })
    );

    // 6.10 معالج إنشاء مشروع لسان جديد
    context.subscriptions.push(
        vscode.commands.registerCommand('lisan.newProject', async () => {
            const templatePick = await vscode.window.showQuickPick(
                Object.entries(PROJECT_TEMPLATES).map(([key, tpl]) => ({
                    label: tpl.label,
                    description: tpl.description,
                    key
                })),
                { placeHolder: 'اختر نوع مشروع لسان الجديد' }
            );
            if (!templatePick) {
                return;
            }

            const projectName = await vscode.window.showInputBox({
                prompt: 'أدخل اسم مشروع لسان الجديد (يفضّل بالعربية)',
                value: `مشروع_${templatePick.key}`,
                validateInput: (v) => (v && v.trim().length > 0 ? null : 'يرجى إدخال اسم صالح للمشروع')
            });
            if (!projectName) {
                return;
            }

            let parentDir = null;
            if (vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0) {
                parentDir = vscode.workspace.workspaceFolders[0].uri.fsPath;
            } else {
                const chosen = await vscode.window.showOpenDialog({
                    canSelectFiles: false,
                    canSelectFolders: true,
                    canSelectMany: false,
                    openLabel: 'اختر المجلد الحاضن للمشروع'
                });
                if (!chosen || chosen.length === 0) {
                    return;
                }
                parentDir = chosen[0].fsPath;
            }

            const projectDir = path.join(parentDir, projectName.trim());
            const srcDir = path.join(projectDir, 'src');
            const vscodeDir = path.join(projectDir, '.vscode');
            fs.mkdirSync(srcDir, { recursive: true });
            fs.mkdirSync(vscodeDir, { recursive: true });

            const manifest = {
                'الاسم': projectName.trim(),
                'الإصدار': '1.2.0',
                'القالب': templatePick.key,
                'المدخل': 'src/الرئيسي.lsn'
            };
            fs.writeFileSync(
                path.join(projectDir, 'lisan.json'),
                JSON.stringify(manifest, null, 2) + '\n',
                'utf8'
            );

            const launchConfig = {
                version: '0.2.0',
                configurations: [
                    {
                        type: 'lisan',
                        request: 'launch',
                        name: 'تصحيح مشروع لسان',
                        program: '${workspaceFolder}/src/الرئيسي.lsn',
                        stopOnEntry: false
                    }
                ]
            };
            fs.writeFileSync(
                path.join(vscodeDir, 'launch.json'),
                JSON.stringify(launchConfig, null, 2) + '\n',
                'utf8'
            );

            const mainFile = path.join(srcDir, 'الرئيسي.lsn');
            fs.writeFileSync(mainFile, PROJECT_TEMPLATES[templatePick.key].code, 'utf8');

            const openedDoc = await vscode.workspace.openTextDocument(mainFile);
            await vscode.window.showTextDocument(openedDoc);
            vscode.window.showInformationMessage(`تم إنشاء مشروع لسان «${projectName.trim()}» بنجاح!`);
        })
    );

    // 6.11 فتح استوديو لسان
    context.subscriptions.push(
        vscode.commands.registerCommand('lisan.openStudio', () => {
            const term = getOrCreateTerminal();
            term.show(true);
            term.sendText(`${quoteShellArg(getLisanPath())} استوديو`);
            vscode.window.showInformationMessage('جارٍ تشغيل استوديو لسان لسطح المكتب...');
        })
    );

    // 7) شريط الحالة (Status Bar): «لسان جاهز» + زر التشغيل السريع
    const readyBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    readyBar.text = '$(symbol-misc) لسان جاهز';
    readyBar.tooltip = 'بيئة لسان نشطة — انقر لفحص الملف الحالي';
    readyBar.command = 'lisan.checkFile';
    readyBar.show();

    const runBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 99);
    runBar.text = '$(play) تشغيل لسان';
    runBar.tooltip = 'تشغيل ملف لسان الحالي (Ctrl+Alt+R)';
    runBar.command = 'lisan.runFile';

    const updateStatusVisibility = (editor) => {
        if (editor && (editor.document.languageId === 'lisan' || editor.document.fileName.endsWith('.lsn'))) {
            runBar.show();
        } else {
            runBar.hide();
        }
    };
    updateStatusVisibility(vscode.window.activeTextEditor);
    context.subscriptions.push(
        readyBar,
        runBar,
        vscode.window.onDidChangeActiveTextEditor(updateStatusVisibility)
    );
}

function deactivate() {
    if (lspBridge) {
        lspBridge.stop();
        lspBridge = null;
    }
    if (previewPanel) {
        previewPanel.dispose();
        previewPanel = null;
    }
}

module.exports = {
    activate,
    deactivate
};
