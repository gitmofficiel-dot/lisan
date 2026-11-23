'use strict';

const vscode = require('vscode');
const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const os = require('os');
const cp = require('child_process');

const AUTH_PROVIDER_ID = 'lisan-account';
const AUTH_PROVIDER_LABEL = 'حساب لسان (Lisan Account)';
const SECRET_SESSION_KEY = 'lisanStudio.activeSession.v1';

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

function escapeHtml(str) {
  return String(str || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function resolveLisanBinary() {
  const configured = vscode.workspace.getConfiguration('lisan').get('executablePath', 'lisan') || 'lisan';
  if (configured !== 'lisan') {
    return configured;
  }
  const exe = process.platform === 'win32' ? 'lisan.exe' : 'lisan';
  const candidates = [];
  if (vscode.env && vscode.env.appRoot) {
    candidates.push(path.join(vscode.env.appRoot, '..', '..', 'bin', exe));
    candidates.push(path.join(vscode.env.appRoot, '..', '..', exe));
    candidates.push(path.join(vscode.env.appRoot, 'bin', exe));
  }
  const home = os.homedir();
  candidates.push(
    path.join(home, '.local', 'bin', exe),
    path.join(home, '.cargo', 'bin', exe),
    path.join(home, '.lisan', exe),
    '/usr/local/bin/lisan'
  );
  for (const c of candidates) {
    try {
      if (fs.existsSync(c)) {
        return c;
      }
    } catch (_) {}
  }
  return configured;
}

function convertPythonViaMcp(pythonCode) {
  return new Promise((resolve) => {
    const lisanBin = resolveLisanBinary();
    const req = JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/call',
      params: {
        name: 'lisan_from_python',
        arguments: { python_code: pythonCode },
      },
    }) + '\n';

    const child = cp.spawn(lisanBin, ['--mcp'], { stdio: ['pipe', 'pipe', 'pipe'] });
    let stdout = '';
    let stderr = '';
    const timer = setTimeout(() => {
      try { child.kill(); } catch (_) {}
      resolve({ ok: false, error: 'انتهت مهلة التحويل' });
    }, 8000);

    child.stdout.on('data', (chunk) => { stdout += chunk.toString('utf8'); });
    child.stderr.on('data', (chunk) => { stderr += chunk.toString('utf8'); });
    child.on('error', (err) => {
      clearTimeout(timer);
      resolve({ ok: false, error: `تعذّر تشغيل lisan --mcp: ${err.message}` });
    });
    child.on('close', () => {
      clearTimeout(timer);
      try {
        const line = stdout.trim().split('\n').filter(Boolean).pop();
        const rpc = JSON.parse(line || '{}');
        const text = rpc && rpc.result && rpc.result.content && rpc.result.content[0] && rpc.result.content[0].text;
        const inner = JSON.parse(text || '{}');
        if (inner.lisan_code) {
          resolve({ ok: true, code: inner.lisan_code, notes: inner.notes || [] });
          return;
        }
        resolve({ ok: false, error: inner.error || stderr || 'لم يتم إرجاع شيفرة محوّلة' });
      } catch (e) {
        resolve({ ok: false, error: `خطأ في قراءة نتيجة التحويل: ${e.message}` });
      }
    });
    child.stdin.write(req);
    child.stdin.end();
  });
}

// ─── مدير حسابات استوديو لسان (تجزئة Scrypt مشفرة + SecretStorage + مزامنة) ───

class LisanAccountManager {
  constructor(context) {
    this.context = context;
    this.storageDir = context.globalStorageUri.fsPath;
    this.accountsFile = path.join(this.storageDir, 'lisan-accounts.json');
    this._onDidChange = new vscode.EventEmitter();
    this.onDidChange = this._onDidChange.event;
    this._activeSession = null;
    fs.mkdirSync(this.storageDir, { recursive: true });
  }

  _readDb() {
    try {
      if (fs.existsSync(this.accountsFile)) {
        const raw = fs.readFileSync(this.accountsFile, 'utf8');
        const data = JSON.parse(raw);
        if (data && Array.isArray(data.accounts)) {
          return data;
        }
      }
    } catch (_) {}
    return { version: 1, accounts: [] };
  }

  _writeDb(db) {
    fs.mkdirSync(this.storageDir, { recursive: true });
    fs.writeFileSync(this.accountsFile, JSON.stringify(db, null, 2), { encoding: 'utf8', mode: 0o600 });
  }

  _hashPassword(password, saltHex) {
    const salt = saltHex || crypto.randomBytes(16).toString('hex');
    const derived = crypto.scryptSync(String(password), salt, 64).toString('hex');
    return { salt, hash: derived };
  }

  _verifyPassword(password, salt, expectedHash) {
    const actual = crypto.scryptSync(String(password), salt, 64);
    const expected = Buffer.from(expectedHash, 'hex');
    if (actual.length !== expected.length) {
      return false;
    }
    return crypto.timingSafeEqual(actual, expected);
  }

  async init() {
    try {
      const raw = await this.context.secrets.get(SECRET_SESSION_KEY);
      if (raw) {
        const saved = JSON.parse(raw);
        const db = this._readDb();
        const user = db.accounts.find((a) => a.id === saved.userId);
        if (user) {
          this._activeSession = this._buildAuthSession(user, saved.accessToken);
        }
      }
    } catch (_) {
      this._activeSession = null;
    }
  }

  _buildAuthSession(user, token) {
    return {
      id: `lisan-session-${user.id}`,
      accessToken: token || crypto.randomBytes(24).toString('hex'),
      account: {
        id: user.id,
        label: `${user.fullName} (@${user.username})`,
      },
      scopes: ['profile', 'sync', 'lisan:mcp'],
      profile: {
        id: user.id,
        fullName: user.fullName,
        username: user.username,
        email: user.email,
        specialty: user.specialty || 'تطوير شامل',
        createdAt: user.createdAt,
      },
    };
  }

  getActiveSession() {
    return this._activeSession;
  }

  listAccountCount() {
    return this._readDb().accounts.length;
  }

  async signUp({ fullName, username, email, password, specialty }) {
    const cleanName = String(fullName || '').trim();
    const cleanUser = String(username || '').trim().toLowerCase().replace(/\s+/g, '_');
    const cleanEmail = String(email || '').trim().toLowerCase();
    const cleanPass = String(password || '');

    if (!cleanName || cleanName.length < 2) {
      throw new Error('يرجى إدخال الاسم الكامل (حرفان على الأقل).');
    }
    if (!cleanUser || cleanUser.length < 2) {
      throw new Error('يرجى إدخال اسم مستخدم صالح.');
    }
    if (!cleanEmail || !cleanEmail.includes('@')) {
      throw new Error('يرجى إدخال بريد إلكتروني صحيح.');
    }
    if (cleanPass.length < 6) {
      throw new Error('يجب أن تتكون كلمة المرور من 6 أحرف على الأقل.');
    }

    const db = this._readDb();
    if (db.accounts.some((a) => a.email === cleanEmail || a.username === cleanUser)) {
      throw new Error('يوجد حساب مسجّل مسبقاً بهذا البريد الإلكتروني أو اسم المستخدم.');
    }

    const { salt, hash } = this._hashPassword(cleanPass);
    const user = {
      id: crypto.randomUUID ? crypto.randomUUID() : crypto.randomBytes(12).toString('hex'),
      fullName: cleanName,
      username: cleanUser,
      email: cleanEmail,
      specialty: String(specialty || 'تطوير شامل بلغة لسان').trim(),
      salt,
      passwordHash: hash,
      createdAt: new Date().toISOString(),
    };
    db.accounts.push(user);
    this._writeDb(db);

    const session = this._buildAuthSession(user);
    this._activeSession = session;
    await this.context.secrets.store(
      SECRET_SESSION_KEY,
      JSON.stringify({ userId: user.id, accessToken: session.accessToken })
    );
    this._onDidChange.fire({ added: [session], removed: [], changed: [] });
    return session;
  }

  async signIn({ identifier, password }) {
    const idKey = String(identifier || '').trim().toLowerCase();
    const pass = String(password || '');
    if (!idKey || !pass) {
      throw new Error('يرجى إدخال البريد/اسم المستخدم وكلمة المرور.');
    }

    const db = this._readDb();
    const user = db.accounts.find((a) => a.email === idKey || a.username === idKey);
    if (!user || !this._verifyPassword(pass, user.salt, user.passwordHash)) {
      throw new Error('بيانات الدخول غير صحيحة — تحقّق من البريد/اسم المستخدم وكلمة المرور.');
    }

    const previous = this._activeSession;
    const session = this._buildAuthSession(user);
    this._activeSession = session;
    await this.context.secrets.store(
      SECRET_SESSION_KEY,
      JSON.stringify({ userId: user.id, accessToken: session.accessToken })
    );
    this._onDidChange.fire({
      added: previous ? [] : [session],
      removed: previous ? [previous] : [],
      changed: previous ? [session] : [],
    });
    return session;
  }

  async updateProfile({ fullName, specialty }) {
    if (!this._activeSession) {
      throw new Error('يجب تسجيل الدخول أولاً.');
    }
    const db = this._readDb();
    const user = db.accounts.find((a) => a.id === this._activeSession.account.id);
    if (!user) {
      throw new Error('الحساب غير موجود.');
    }
    if (fullName && fullName.trim().length >= 2) {
      user.fullName = fullName.trim();
    }
    if (specialty && specialty.trim()) {
      user.specialty = specialty.trim();
    }
    this._writeDb(db);
    const updated = this._buildAuthSession(user, this._activeSession.accessToken);
    this._activeSession = updated;
    this._onDidChange.fire({ added: [], removed: [], changed: [updated] });
    return updated;
  }

  async signOut() {
    const removed = this._activeSession ? [this._activeSession] : [];
    this._activeSession = null;
    await this.context.secrets.delete(SECRET_SESSION_KEY);
    if (removed.length > 0) {
      this._onDidChange.fire({ added: [], removed, changed: [] });
    }
  }

  async exportSyncBackup() {
    if (!this._activeSession) {
      throw new Error('سجّل الدخول أولاً لتصدير نسخة مزامنة الحساب.');
    }
    const uri = await vscode.window.showSaveDialog({
      defaultUri: vscode.Uri.file(path.join(os.homedir(), `lisan-profile-${this._activeSession.profile.username}.json`)),
      filters: { 'Lisan Account Backup': ['json'] },
      saveLabel: 'حفظ نسخة الحساب والإعدادات',
    });
    if (!uri) {
      return null;
    }
    const payload = {
      exportedAt: new Date().toISOString(),
      studioVersion: '1.2.0',
      profile: this._activeSession.profile,
      lisanSettings: {
        theme: vscode.workspace.getConfiguration('workbench').get('colorTheme'),
        fontSize: vscode.workspace.getConfiguration('editor').get('fontSize'),
        lisan: vscode.workspace.getConfiguration('lisan'),
      },
    };
    fs.writeFileSync(uri.fsPath, JSON.stringify(payload, null, 2), 'utf8');
    return uri.fsPath;
  }
}

// ─── مزوّد المصادقة الأصلي لـ VS Code (يظهر في قائمة Accounts أسفل اليسار) ───

class LisanAuthProvider {
  constructor(accountManager) {
    this.accountManager = accountManager;
    this._onDidChangeSessions = new vscode.EventEmitter();
    this.onDidChangeSessions = this._onDidChangeSessions.event;
    accountManager.onDidChange((e) => this._onDidChangeSessions.fire(e));
  }

  async getSessions() {
    const active = this.accountManager.getActiveSession();
    return active ? [active] : [];
  }

  async createSession() {
    const choice = await vscode.window.showQuickPick(
      [
        { label: '$(person-add) إنشاء حساب لسان جديد', action: 'signup' },
        { label: '$(sign-in) تسجيل الدخول إلى حساب موجود', action: 'signin' },
      ],
      { placeHolder: 'اختر طريقة المصادقة في استوديو لسان' }
    );
    if (!choice) {
      throw new Error('تم إلغاء عملية تسجيل الدخول.');
    }

    if (choice.action === 'signup') {
      const fullName = await vscode.window.showInputBox({ prompt: 'الاسم الكامل (مثال: محمد العربي)' });
      if (!fullName) throw new Error('تم الإلغاء.');
      const username = await vscode.window.showInputBox({ prompt: 'اسم المستخدم (مثال: mohammed)' });
      if (!username) throw new Error('تم الإلغاء.');
      const email = await vscode.window.showInputBox({ prompt: 'البريد الإلكتروني' });
      if (!email) throw new Error('تم الإلغاء.');
      const specialtyPick = await vscode.window.showQuickPick(
        ['تطوير شامل بلغة لسان', 'الذكاء الاصطناعي ووكلاء RAG', 'الويب وتطبيقات الهاتف', 'المتحكمات الدقيقة والروبوتات'],
        { placeHolder: 'اختر مجالك الرئيسي' }
      );
      const password = await vscode.window.showInputBox({ prompt: 'كلمة المرور (6 أحرف على الأقل)', password: true });
      if (!password) throw new Error('تم الإلغاء.');
      return await this.accountManager.signUp({
        fullName,
        username,
        email,
        password,
        specialty: specialtyPick || 'تطوير شامل بلغة لسان',
      });
    }

    const identifier = await vscode.window.showInputBox({ prompt: 'البريد الإلكتروني أو اسم المستخدم' });
    if (!identifier) throw new Error('تم الإلغاء.');
    const password = await vscode.window.showInputBox({ prompt: 'كلمة المرور', password: true });
    if (!password) throw new Error('تم الإلغاء.');
    return await this.accountManager.signIn({ identifier, password });
  }

  async removeSession() {
    await this.accountManager.signOut();
  }
}

// ─── لوحة الحساب الجانبية (Sidebar Webview 1: حسابي في لسان) ───

class LisanAccountSidebarProvider {
  constructor(accountManager) {
    this.accountManager = accountManager;
    this._view = null;
    accountManager.onDidChange(() => this.refresh());
  }

  resolveWebviewView(webviewView) {
    this._view = webviewView;
    webviewView.webview.options = { enableScripts: true };
    this.refresh();

    webviewView.webview.onDidReceiveMessage(async (msg) => {
      try {
        if (msg.type === 'signUp') {
          await this.accountManager.signUp(msg.payload);
          vscode.window.showInformationMessage('مرحباً بك! تم إنشاء حسابك في استوديو لسان بنجاح 🎉');
        } else if (msg.type === 'signIn') {
          await this.accountManager.signIn(msg.payload);
          vscode.window.showInformationMessage('تم تسجيل الدخول إلى حساب لسان بنجاح ✅');
        } else if (msg.type === 'signOut') {
          await this.accountManager.signOut();
          vscode.window.showInformationMessage('تم تسجيل الخروج من حساب لسان.');
        } else if (msg.type === 'updateProfile') {
          await this.accountManager.updateProfile(msg.payload);
          vscode.window.showInformationMessage('تم تحديث بيانات ملفك الشخصي ✅');
        } else if (msg.type === 'exportBackup') {
          const saved = await this.accountManager.exportSyncBackup();
          if (saved) {
            vscode.window.showInformationMessage(`تم حفظ نسخة مزامنة الحساب في: ${saved}`);
          }
        }
      } catch (err) {
        if (this._view) {
          this._view.webview.postMessage({ type: 'error', message: err.message });
        }
      }
    });
  }

  refresh() {
    if (!this._view) return;
    const session = this.accountManager.getActiveSession();
    const count = this.accountManager.listAccountCount();
    this._view.webview.html = this._getHtml(session, count);
  }

  _getHtml(session, accountCount) {
    if (session && session.profile) {
      const p = session.profile;
      const initial = escapeHtml((p.fullName || 'ل').trim().charAt(0));
      return `<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><style>
        body{font-family:system-ui,"Cairo",sans-serif;padding:12px;color:#e2e8f0;background:#0f172a;margin:0}
        .card{background:#111c33;border:1px solid #1e3a5f;border-radius:12px;padding:14px;margin-bottom:12px}
        .avatar{width:46px;height:46px;border-radius:50%;background:linear-gradient(135deg,#0284c7,#10b981);display:flex;align-items:center;justify-content:center;font-size:20px;font-weight:800;color:#fff;margin-bottom:8px}
        .name{font-size:16px;font-weight:700;color:#38bdf8}
        .meta{font-size:12px;color:#94a3b8;margin-top:2px}
        .tag{display:inline-block;margin-top:8px;padding:3px 9px;border-radius:999px;background:rgba(16,185,129,.18);color:#34d399;font-size:11px;font-weight:700}
        label{display:block;font-size:12px;color:#94a3b8;margin:8px 0 4px}
        input,select{width:100%;box-sizing:border-box;padding:7px 9px;border-radius:8px;border:1px solid #334155;background:#0b1120;color:#f8fafc;font-family:inherit;font-size:13px}
        button{width:100%;margin-top:8px;padding:8px;border:none;border-radius:8px;font-family:inherit;font-size:13px;font-weight:700;cursor:pointer;color:#fff;background:#0284c7}
        button.sec{background:#1e293b;border:1px solid #334155}
        button.danger{background:#991b1b}
        .err{color:#fca5a5;font-size:12px;margin-top:6px}
      </style></head><body>
        <div class="card">
          <div class="avatar">${initial}</div>
          <div class="name">${escapeHtml(p.fullName)}</div>
          <div class="meta">@${escapeHtml(p.username)} · ${escapeHtml(p.email)}</div>
          <span class="tag">✓ متصل · ${escapeHtml(p.specialty)}</span>
        </div>
        <div class="card">
          <label>تحديث الاسم الكامل</label>
          <input id="fn" value="${escapeHtml(p.fullName)}" />
          <label>التخصص</label>
          <input id="sp" value="${escapeHtml(p.specialty)}" />
          <button onclick="save()">💾 حفظ التعديلات</button>
          <button class="sec" onclick="post('exportBackup')">☁️ تصدير نسخة الحساب والإعدادات</button>
          <button class="danger" onclick="post('signOut')">تسجيل الخروج</button>
          <div id="err" class="err"></div>
        </div>
        <script>
          const vscode = acquireVsCodeApi();
          function post(type, payload){ vscode.postMessage({type, payload}); }
          function save(){ post('updateProfile', { fullName: document.getElementById('fn').value, specialty: document.getElementById('sp').value }); }
          window.addEventListener('message', e => { if(e.data.type==='error') document.getElementById('err').textContent = e.data.message; });
        </script>
      </body></html>`;
    }

    return `<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><style>
      body{font-family:system-ui,"Cairo",sans-serif;padding:12px;color:#e2e8f0;background:#0f172a;margin:0}
      .tabs{display:flex;gap:6px;margin-bottom:10px}
      .tab{flex:1;padding:7px;border-radius:8px;border:1px solid #334155;background:#1e293b;color:#cbd5e1;cursor:pointer;font-family:inherit;font-weight:700;font-size:12px}
      .tab.active{background:#0284c7;border-color:#38bdf8;color:#fff}
      .box{background:#111c33;border:1px solid #1e3a5f;border-radius:12px;padding:12px}
      label{display:block;font-size:12px;color:#94a3b8;margin:7px 0 3px}
      input,select{width:100%;box-sizing:border-box;padding:7px 9px;border-radius:8px;border:1px solid #334155;background:#0b1120;color:#f8fafc;font-family:inherit;font-size:13px}
      button.submit{width:100%;margin-top:12px;padding:9px;border:none;border-radius:8px;background:#10b981;color:#042f2e;font-family:inherit;font-weight:800;font-size:13px;cursor:pointer}
      .note{font-size:11px;color:#94a3b8;margin-top:8px;line-height:1.5}
      .err{color:#fca5a5;font-size:12px;margin-top:8px}
      .hidden{display:none}
    </style></head><body>
      <div class="tabs">
        <button id="tUp" class="tab ${accountCount === 0 ? 'active' : ''}" onclick="mode('up')">إنشاء حساب جديد</button>
        <button id="tIn" class="tab ${accountCount > 0 ? 'active' : ''}" onclick="mode('in')">تسجيل الدخول</button>
      </div>
      <div id="fUp" class="box ${accountCount > 0 ? 'hidden' : ''}">
        <label>الاسم الكامل</label><input id="uName" placeholder="مثال: محمد العربي" />
        <label>اسم المستخدم</label><input id="uUser" placeholder="مثال: mohammed" />
        <label>البريد الإلكتروني</label><input id="uEmail" type="email" placeholder="name@example.com" />
        <label>التخصص الرئيسي</label>
        <select id="uSpec">
          <option>تطوير شامل بلغة لسان</option>
          <option>الذكاء الاصطناعي ووكلاء RAG</option>
          <option>الويب وتطبيقات الهاتف</option>
          <option>المتحكمات الدقيقة والروبوتات</option>
        </select>
        <label>كلمة المرور (6+ أحرف)</label><input id="uPass" type="password" placeholder="••••••" />
        <button class="submit" onclick="doSignUp()">✨ إنشاء حساب لسان</button>
        <div class="note">تُحفظ كلمة المرور بتجزئة Scrypt مشفّرة محلياً مع دعم مزامنة الإعدادات.</div>
      </div>
      <div id="fIn" class="box ${accountCount === 0 ? 'hidden' : ''}">
        <label>البريد الإلكتروني أو اسم المستخدم</label><input id="lId" placeholder="mohammed أو name@example.com" />
        <label>كلمة المرور</label><input id="lPass" type="password" placeholder="••••••" />
        <button class="submit" onclick="doSignIn()">🔑 تسجيل الدخول</button>
      </div>
      <div id="err" class="err"></div>
      <script>
        const vscode = acquireVsCodeApi();
        function mode(m){
          document.getElementById('fUp').classList.toggle('hidden', m!=='up');
          document.getElementById('fIn').classList.toggle('hidden', m!=='in');
          document.getElementById('tUp').classList.toggle('active', m==='up');
          document.getElementById('tIn').classList.toggle('active', m==='in');
          document.getElementById('err').textContent = '';
        }
        function doSignUp(){
          vscode.postMessage({ type:'signUp', payload:{
            fullName: document.getElementById('uName').value,
            username: document.getElementById('uUser').value,
            email: document.getElementById('uEmail').value,
            specialty: document.getElementById('uSpec').value,
            password: document.getElementById('uPass').value
          }});
        }
        function doSignIn(){
          vscode.postMessage({ type:'signIn', payload:{
            identifier: document.getElementById('lId').value,
            password: document.getElementById('lPass').value
          }});
        }
        window.addEventListener('message', e => { if(e.data.type==='error') document.getElementById('err').textContent = e.data.message; });
      </script>
    </body></html>`;
  }
}

// ─── لوحة الأدوات ومحوّل بايثون ← لسان (Sidebar Webview 2) ───

class LisanToolsSidebarProvider {
  resolveWebviewView(webviewView) {
    webviewView.webview.options = { enableScripts: true };
    webviewView.webview.html = `<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><style>
      body{font-family:system-ui,"Cairo",sans-serif;padding:12px;color:#e2e8f0;background:#0f172a;margin:0}
      h4{margin:0 0 8px;color:#38bdf8;font-size:13px}
      .grid{display:grid;grid-template-columns:1fr 1fr;gap:6px;margin-bottom:14px}
      button{padding:8px 6px;border-radius:8px;border:1px solid #1e3a5f;background:#111c33;color:#f8fafc;font-family:inherit;font-size:12px;font-weight:700;cursor:pointer}
      button:hover{border-color:#38bdf8;background:#162442}
      button.primary{background:#0284c7;border-color:#38bdf8;grid-column:span 2}
      textarea{width:100%;box-sizing:border-box;min-height:92px;border-radius:8px;border:1px solid #334155;background:#0b1120;color:#34d399;padding:8px;font-family:monospace;font-size:12px;direction:ltr}
      pre{background:#0b1120;border:1px solid #1e3a5f;border-radius:8px;padding:8px;font-size:12px;white-space:pre-wrap;color:#f8fafc;max-height:160px;overflow:auto}
      .row{display:flex;gap:6px;margin-top:6px}
      .row button{flex:1}
    </style></head><body>
      <h4>⚡ تشغيل وتطوير لسان السريع</h4>
      <div class="grid">
        <button class="primary" onclick="cmd('lisan.runFile')">▶️ تشغيل ملف لسان الحالي</button>
        <button onclick="cmd('lisan.checkFile')">✅ فحص العربية</button>
        <button onclick="cmd('lisan.formatFile')">✨ تنسيق الكود</button>
        <button onclick="cmd('lisan.runTests')">🧪 الاختبارات</button>
        <button onclick="cmd('lisan.previewWeb')">🌐 معاينة ويب</button>
        <button onclick="cmd('lisan.transpilePython')">🐍 إلى بايثون</button>
        <button onclick="cmd('lisan.transpileJs')">🟨 إلى JS</button>
        <button onclick="cmd('lisan.transpileMcu')">🔌 متحكمات</button>
        <button onclick="cmd('lisan.newProject')">🚀 مشروع جديد</button>
      </div>
      <h4>🔄 محوّل بايثون ← لسان عربي خالص (بدون إنترنت)</h4>
      <textarea id="pyIn" placeholder="def add(a, b):&#10;    return a + b&#10;print(add(3, 4))"></textarea>
      <div class="row">
        <button onclick="convertPy()">تحويل إلى لسان</button>
        <button onclick="insertOut()">إدراج في المحرر</button>
      </div>
      <pre id="lsnOut">// سيظهر كود لسان العربي الخالص هنا...</pre>
      <script>
        const vscode = acquireVsCodeApi();
        let lastCode = '';
        function cmd(name){ vscode.postMessage({ type:'command', command: name }); }
        function convertPy(){
          document.getElementById('lsnOut').textContent = 'جارٍ التحويل عبر lisan --mcp...';
          vscode.postMessage({ type:'convertPy', code: document.getElementById('pyIn').value });
        }
        function insertOut(){
          if(lastCode) vscode.postMessage({ type:'insertCode', code: lastCode });
        }
        window.addEventListener('message', e => {
          if(e.data.type==='converted'){
            lastCode = e.data.code || '';
            document.getElementById('lsnOut').textContent = e.data.ok ? lastCode : ('خطأ: ' + e.data.error);
          }
        });
      </script>
    </body></html>`;

    webviewView.webview.onDidReceiveMessage(async (msg) => {
      if (msg.type === 'command') {
        await vscode.commands.executeCommand(msg.command);
      } else if (msg.type === 'convertPy') {
        const res = await convertPythonViaMcp(msg.code || '');
        webviewView.webview.postMessage({ type: 'converted', ...res });
      } else if (msg.type === 'insertCode') {
        const editor = vscode.window.activeTextEditor;
        if (editor) {
          await editor.edit((eb) => eb.insert(editor.selection.active, msg.code));
        } else {
          const doc = await vscode.workspace.openTextDocument({ language: 'lisan', content: msg.code });
          await vscode.window.showTextDocument(doc);
        }
      }
    });
  }
}

// ─── لوحة الترحيب الرئيسية ───

function getWelcomeHtml(session) {
  const greeting = session && session.profile
    ? `مرحباً بك يا ${escapeHtml(session.profile.fullName)} (@${escapeHtml(session.profile.username)})`
    : 'بيئة التطوير العربية الشاملة — أنشئ حسابك من شريط «حسابي في لسان» لمزامنة إعداداتك';

  return `<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>استوديو لسان — Lisan Studio</title>
<style>
  :root { --bg:#0b1120; --card:#111c33; --border:#1e3a5f; --primary:#38bdf8; --accent:#10b981; --gold:#f59e0b; --text:#f8fafc; --muted:#94a3b8; }
  * { box-sizing:border-box; }
  body { margin:0; padding:32px; background:radial-gradient(circle at top right,#0f294a 0%,var(--bg) 60%); color:var(--text); font-family:"Tajawal","Cairo",system-ui,sans-serif; line-height:1.6; }
  .wrap { max-width:980px; margin:0 auto; }
  .hero { display:flex; align-items:center; justify-content:space-between; gap:20px; padding:24px 28px; background:linear-gradient(135deg,#0f172a,#172554); border:1px solid var(--border); border-radius:18px; margin-bottom:24px; }
  .hero h1 { margin:0 0 6px; font-size:28px; color:var(--primary); }
  .hero p { margin:0; color:var(--muted); font-size:15px; }
  .badge { display:inline-block; padding:4px 12px; border-radius:999px; background:rgba(16,185,129,.16); color:var(--accent); border:1px solid rgba(16,185,129,.35); font-size:13px; font-weight:700; }
  .grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(215px,1fr)); gap:16px; margin-bottom:24px; }
  .card { background:var(--card); border:1px solid var(--border); border-radius:14px; padding:18px; cursor:pointer; text-align:right; color:var(--text); transition:transform .15s,border-color .15s; }
  .card:hover { transform:translateY(-2px); border-color:var(--primary); background:#162442; }
  .card .icon { font-size:24px; margin-bottom:8px; }
  .card h3 { margin:0 0 6px; font-size:16px; }
  .card p { margin:0; font-size:13px; color:var(--muted); }
  .section-title { font-size:17px; color:var(--gold); margin:0 0 12px; }
  .actions { display:flex; flex-wrap:wrap; gap:10px; }
  .btn { background:#0284c7; color:#fff; border:none; border-radius:10px; padding:10px 16px; font-family:inherit; font-size:14px; font-weight:700; cursor:pointer; }
  .btn.secondary { background:#1e293b; border:1px solid var(--border); }
</style>
</head>
<body>
<div class="wrap">
  <div class="hero">
    <div>
      <span class="badge">عربية خالصة 100% · Lisan v1.2.0 مدمج · مفتوح المصدر</span>
      <h1>استوديو لسان — Lisan Studio</h1>
      <p>${greeting}</p>
    </div>
    <svg width="72" height="72" viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg">
      <rect x="16" y="16" width="224" height="224" rx="52" fill="#0f172a" stroke="#38bdf8" stroke-width="10"/>
      <circle cx="192" cy="64" r="12" fill="#10b981"/>
      <path d="M168 76 L168 158 C168 182 150 196 120 196 C92 196 76 181 76 160 C76 144 88 134 104 134" fill="none" stroke="#38bdf8" stroke-width="20" stroke-linecap="round"/>
      <text x="128" y="118" text-anchor="middle" font-family="Tahoma, Arial, sans-serif" font-size="38" font-weight="bold" fill="#f8fafc">ض</text>
    </svg>
  </div>
  <h2 class="section-title">ابدأ فوراً بلغة لسان</h2>
  <div class="grid">
    <button class="card" onclick="send('account')"><div class="icon">👤</div><h3>حسابي في لسان</h3><p>إنشاء حساب، تسجيل الدخول، وحفظ إعداداتك ومزامنتها.</p></button>
    <button class="card" onclick="send('newProject')"><div class="icon">🚀</div><h3>مشروع لسان جديد</h3><p>ويب تفاعلي، تطبيق هاتف Capacitor، ذكاء RAG، متحكمات، أو نظام.</p></button>
    <button class="card" onclick="send('newFile')"><div class="icon">📝</div><h3>ملف لسان جديد (.lsn)</h3><p>تلوين، إكمال، فحص العربية الخالصة، وإصلاحات سريعة بضغطة.</p></button>
    <button class="card" onclick="send('copyMcp')"><div class="icon">🤖</div><h3>ربط وكلاء الذكاء (MCP)</h3><p>انسخ إعداد خادم lisan --mcp لتعليم كل الوكلاء لغة لسان.</p></button>
  </div>
  <div class="actions">
    <button class="btn" onclick="send('openFolder')">📂 فتح مجلد مشروع</button>
    <button class="btn secondary" onclick="send('openTerminal')">💻 فتح طرفية لسان</button>
    <button class="btn secondary" onclick="send('openPreview')">🌐 معاينة ويب / هاتف حية</button>
    <button class="btn secondary" onclick="send('transpileMcu')">🔌 ترجمة للمتحكمات</button>
  </div>
</div>
<script>
  const vscode = acquireVsCodeApi();
  function send(action){ vscode.postMessage({ action }); }
</script>
</body>
</html>`;
}

function openWelcomePanel(context, accountManager) {
  const panel = vscode.window.createWebviewPanel(
    'lisanStudioWelcome',
    'استوديو لسان — الرئيسية',
    vscode.ViewColumn.One,
    { enableScripts: true }
  );
  panel.webview.html = getWelcomeHtml(accountManager.getActiveSession());
  panel.webview.onDidReceiveMessage(async (msg) => {
    switch (msg.action) {
      case 'account':
        await vscode.commands.executeCommand('lisanStudio.accountView.focus');
        break;
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
      default:
        break;
    }
  }, undefined, context.subscriptions);
}

async function activate(context) {
  const accountManager = new LisanAccountManager(context);
  await accountManager.init();

  const authProvider = new LisanAuthProvider(accountManager);
  const accountSidebar = new LisanAccountSidebarProvider(accountManager);
  const toolsSidebar = new LisanToolsSidebarProvider();

  context.subscriptions.push(
    vscode.authentication.registerAuthenticationProvider(
      AUTH_PROVIDER_ID,
      AUTH_PROVIDER_LABEL,
      authProvider,
      { supportsMultipleAccounts: false }
    ),
    vscode.window.registerWebviewViewProvider('lisanStudio.accountView', accountSidebar),
    vscode.window.registerWebviewViewProvider('lisanStudio.toolsView', toolsSidebar)
  );

  // شارة الحساب في شريط الحالة
  const accountBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 1001);
  const updateAccountBar = () => {
    const s = accountManager.getActiveSession();
    if (s && s.profile) {
      accountBar.text = `$(account) ${s.profile.fullName}`;
      accountBar.tooltip = `مسجّل الدخول بحساب لسان (@${s.profile.username}) — انقر لفتح ملفك الشخصي`;
    } else {
      accountBar.text = '$(account) حساب لسان';
      accountBar.tooltip = 'إنشاء حساب أو تسجيل الدخول إلى استوديو لسان';
    }
    accountBar.command = 'lisanStudio.accountView.focus';
    accountBar.show();
  };
  updateAccountBar();
  accountManager.onDidChange(updateAccountBar);
  context.subscriptions.push(accountBar);

  const badge = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 1000);
  badge.text = '$(sparkle) استوديو لسان v1.2';
  badge.tooltip = 'فتح لوحة الترحيب الرئيسية لاستوديو لسان';
  badge.command = 'lisanStudio.openWelcome';
  badge.show();
  context.subscriptions.push(badge);

  context.subscriptions.push(
    vscode.commands.registerCommand('lisanStudio.openWelcome', () => openWelcomePanel(context, accountManager)),
    vscode.commands.registerCommand('lisanStudio.signUp', async () => {
      await vscode.commands.executeCommand('lisanStudio.accountView.focus');
    }),
    vscode.commands.registerCommand('lisanStudio.signIn', async () => {
      await vscode.authentication.getSession(AUTH_PROVIDER_ID, ['profile'], { createIfNone: true });
    }),
    vscode.commands.registerCommand('lisanStudio.signOut', async () => {
      await accountManager.signOut();
      vscode.window.showInformationMessage('تم تسجيل الخروج من حساب لسان.');
    }),
    vscode.commands.registerCommand('lisanStudio.copyMcpConfig', async () => {
      await vscode.env.clipboard.writeText(MCP_SNIPPET);
      vscode.window.showInformationMessage('تم نسخ إعداد خادم MCP لـ lisan --mcp إلى الحافظة!');
    })
  );

  const hasShown = context.globalState.get('lisanStudio.welcomeShown', false);
  if (!hasShown || (!vscode.workspace.workspaceFolders && vscode.window.visibleTextEditors.length === 0)) {
    context.globalState.update('lisanStudio.welcomeShown', true);
    openWelcomePanel(context, accountManager);
  }
}

function deactivate() {}

module.exports = { activate, deactivate };
