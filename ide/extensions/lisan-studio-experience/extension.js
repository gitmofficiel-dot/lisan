'use strict';

const vscode = require('vscode');

const MCP_SNIPPET = JSON.stringify(
  {
    mcpServers: {
      lisan: {
        command: 'lisan',
        args: ['--mcp'],
      },
    },
  },
  null,
  2
);

function getWelcomeHtml() {
  return `<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>استوديو لسان — Lisan Studio</title>
<style>
  :root {
    --bg: #0b1120;
    --card: #111c33;
    --border: #1e3a5f;
    --primary: #38bdf8;
    --accent: #10b981;
    --gold: #f59e0b;
    --text: #f8fafc;
    --muted: #94a3b8;
  }
  * { box-sizing: border-box; }
  body {
    margin: 0;
    padding: 32px;
    background: radial-gradient(circle at top right, #0f294a 0%, var(--bg) 60%);
    color: var(--text);
    font-family: "Tajawal", "Cairo", "Noto Sans Arabic", system-ui, sans-serif;
    line-height: 1.6;
  }
  .wrap { max-width: 980px; margin: 0 auto; }
  .hero {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding: 24px 28px;
    background: linear-gradient(135deg, #0f172a, #172554);
    border: 1px solid var(--border);
    border-radius: 18px;
    margin-bottom: 26px;
  }
  .hero h1 { margin: 0 0 6px; font-size: 28px; color: var(--primary); }
  .hero p { margin: 0; color: var(--muted); font-size: 15px; }
  .badge {
    display: inline-block;
    padding: 4px 12px;
    border-radius: 999px;
    background: rgba(16, 185, 129, 0.16);
    color: var(--accent);
    border: 1px solid rgba(16, 185, 129, 0.35);
    font-size: 13px;
    font-weight: 700;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 16px;
    margin-bottom: 26px;
  }
  .card {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 18px;
    cursor: pointer;
    text-align: right;
    color: var(--text);
    transition: transform 0.15s ease, border-color 0.15s ease, background 0.15s ease;
  }
  .card:hover {
    transform: translateY(-2px);
    border-color: var(--primary);
    background: #162442;
  }
  .card .icon { font-size: 24px; margin-bottom: 8px; }
  .card h3 { margin: 0 0 6px; font-size: 16px; color: var(--text); }
  .card p { margin: 0; font-size: 13px; color: var(--muted); }
  .section-title { font-size: 17px; color: var(--gold); margin: 0 0 12px; }
  .actions { display: flex; flex-wrap: wrap; gap: 10px; }
  .btn {
    background: #0284c7;
    color: #fff;
    border: none;
    border-radius: 10px;
    padding: 10px 16px;
    font-family: inherit;
    font-size: 14px;
    font-weight: 700;
    cursor: pointer;
  }
  .btn.secondary { background: #1e293b; border: 1px solid var(--border); color: var(--text); }
  .btn:hover { opacity: 0.92; }
  .footer {
    margin-top: 28px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
    font-size: 13px;
    color: var(--muted);
    display: flex;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 8px;
  }
</style>
</head>
<body>
<div class="wrap">
  <div class="hero">
    <div>
      <span class="badge">عربية خالصة 100% · v1.2.0 · مفتوح المصدر</span>
      <h1>استوديو لسان — Lisan Studio</h1>
      <p>بيئة التطوير العربية الشاملة للذكاء الاصطناعي، الويب، تطبيقات الهاتف، الأنظمة، والمتحكمات الدقيقة.</p>
    </div>
    <svg width="72" height="72" viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg">
      <rect x="16" y="16" width="224" height="224" rx="52" fill="#0f172a" stroke="#38bdf8" stroke-width="10"/>
      <circle cx="192" cy="64" r="12" fill="#10b981"/>
      <path d="M168 76 L168 158 C168 182 150 196 120 196 C92 196 76 181 76 160 C76 144 88 134 104 134" fill="none" stroke="#38bdf8" stroke-width="20" stroke-linecap="round"/>
      <text x="128" y="118" text-anchor="middle" font-family="Tahoma, Arial, sans-serif" font-size="38" font-weight="bold" fill="#f8fafc">ض</text>
    </svg>
  </div>

  <h2 class="section-title">ابدأ مشروعاً جديداً بضغطة واحدة</h2>
  <div class="grid">
    <button class="card" onclick="send('newProject')">
      <div class="icon">🚀</div>
      <h3>معالج مشروع لسان جديد</h3>
      <p>ويب تفاعلي، تطبيق هاتف PWA/Capacitor، ذكاء اصطناعي RAG، متحكمات، أو أداة نظام.</p>
    </button>
    <button class="card" onclick="send('openFolder')">
      <div class="icon">📂</div>
      <h3>فتح مجلد مشروع</h3>
      <p>افتح مشروعك الحالي مع تفعيل خادم LSP ومصحح الأخطاء DAP تلقائياً.</p>
    </button>
    <button class="card" onclick="send('newFile')">
      <div class="icon">📝</div>
      <h3>ملف لسان جديد (.lsn)</h3>
      <p>اكتب شيفرة عربية خالصة مع إكمال تلقائي وفحص فوري للأنواع والقواعد.</p>
    </button>
    <button class="card" onclick="send('copyMcp')">
      <div class="icon">🤖</div>
      <h3>ربط وكلاء الذكاء (MCP)</h3>
      <p>انسخ إعداد خادم lisan --mcp لتعليم Claude وGemini وCursor لغة لسان.</p>
    </button>
  </div>

  <h2 class="section-title">أدوات سريعة داخل استوديو لسان</h2>
  <div class="actions">
    <button class="btn" onclick="send('openTerminal')">💻 فتح طرفية لسان</button>
    <button class="btn secondary" onclick="send('openPreview')">🌐 معاينة ويب / هاتف حية</button>
    <button class="btn secondary" onclick="send('transpileMcu')">🔌 ترجمة ورفع للمتحكمات</button>
    <button class="btn secondary" onclick="send('openSettings')">⚙️ الإعدادات والسمات</button>
  </div>

  <div class="footer">
    <span>مبني على Code-OSS مفتوح المصدر — بلا تتبّع (Zero Telemetry)</span>
    <span>اختصار التشغيل: F5 للتصحيح · Ctrl+Shift+P لكل أوامر لسان</span>
  </div>
</div>
<script>
  const vscode = acquireVsCodeApi();
  function send(action) { vscode.postMessage({ action }); }
</script>
</body>
</html>`;
}

function openWelcomePanel(context) {
  const panel = vscode.window.createWebviewPanel(
    'lisanStudioWelcome',
    'استوديو لسان — الرئيسية',
    vscode.ViewColumn.One,
    { enableScripts: true }
  );
  panel.webview.html = getWelcomeHtml();
  panel.webview.onDidReceiveMessage(
    async (msg) => {
      switch (msg.action) {
        case 'newProject':
          await vscode.commands.executeCommand('lisan.newProject');
          break;
        case 'openFolder':
          await vscode.commands.executeCommand('workbench.action.files.openFolder');
          break;
        case 'newFile': {
          const doc = await vscode.workspace.openTextDocument({
            language: 'lisan',
            content: '// مرحباً بك في استوديو لسان (Lisan Studio v1.2.0)\nدالة تحية(الاسم: نص) -> نص {\n    أرجع "أهلاً بك في لغة لسان يا " + الاسم\n}\n\nاطبع(تحية("مبرمج"))\n',
          });
          await vscode.window.showTextDocument(doc);
          break;
        }
        case 'copyMcp':
          await vscode.commands.executeCommand('lisanStudio.copyMcpConfig');
          break;
        case 'openTerminal': {
          const term = vscode.window.createTerminal({ name: 'طرفية لسان' });
          term.show();
          term.sendText('lisan --version', true);
          break;
        }
        case 'openPreview':
          await vscode.commands.executeCommand('lisan.previewWeb');
          break;
        case 'transpileMcu':
          await vscode.commands.executeCommand('lisan.transpileMcu');
          break;
        case 'openSettings':
          await vscode.commands.executeCommand('workbench.action.openSettings', 'lisan');
          break;
        default:
          break;
      }
    },
    undefined,
    context.subscriptions
  );
}

function activate(context) {
  const badge = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 1000);
  badge.text = '$(sparkle) استوديو لسان v1.2';
  badge.tooltip = 'فتح لوحة الترحيب الرئيسية لاستوديو لسان';
  badge.command = 'lisanStudio.openWelcome';
  badge.show();
  context.subscriptions.push(badge);

  context.subscriptions.push(
    vscode.commands.registerCommand('lisanStudio.openWelcome', () => openWelcomePanel(context)),
    vscode.commands.registerCommand('lisanStudio.copyMcpConfig', async () => {
      await vscode.env.clipboard.writeText(MCP_SNIPPET);
      vscode.window.showInformationMessage('تم نسخ إعداد خادم MCP لـ lisan --mcp إلى الحافظة!');
    })
  );

  // إظهار لوحة الترحيب العربية عند الفتح بدون مجلد مفتوح أو عند أول تشغيل
  const hasShown = context.globalState.get('lisanStudio.welcomeShown', false);
  if (!hasShown || (!vscode.workspace.workspaceFolders && vscode.window.visibleTextEditors.length === 0)) {
    context.globalState.update('lisanStudio.welcomeShown', true);
    openWelcomePanel(context);
  }
}

function deactivate() {}

module.exports = { activate, deactivate };
