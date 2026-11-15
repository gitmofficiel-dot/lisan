const vscode = require('vscode');
const cp = require('child_process');

let lspProcess = null;

/**
 * تفعيل إضافة لسان في Visual Studio Code
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
    const config = vscode.workspace.getConfiguration('lisan');
    const lisanPath = config.get('executablePath') || 'lisan';
    const lspEnabled = config.get('lsp.enabled', true);

    if (!lspEnabled) {
        return;
    }

    // محاولة تشغيل خادم لسان عبر مصفوفة تشغيل LSP
    try {
        const { LanguageClient, TransportKind } = require('vscode-languageclient/node');
        const serverOptions = {
            command: lisanPath,
            args: ['--lsp'],
            transport: TransportKind.stdio
        };
        const clientOptions = {
            documentSelector: [{ scheme: 'file', language: 'lisan' }],
            synchronize: {
                fileEvents: vscode.workspace.createFileSystemWatcher('**/*.lsn')
            }
        };
        const client = new LanguageClient(
            'lisanLanguageServer',
            'Lisan Language Server',
            serverOptions,
            clientOptions
        );
        client.start();
        context.subscriptions.push({
            dispose: () => client.stop()
        });
        return;
    } catch (_) {
        // إذا لم تكن حزمة vscode-languageclient مثبتة محلياً، نستخدم مشغل stdio المباشر
    }

    // تسجيل مزود التنسيق عبر أمر lisan --fmt
    const formatProvider = vscode.languages.registerDocumentFormattingEditProvider('lisan', {
        provideDocumentFormattingEdits(document) {
            const text = document.getText();
            return new Promise((resolve) => {
                const proc = cp.spawn(lisanPath, ['--fmt'], { stdio: ['pipe', 'pipe', 'pipe'] });
                let stdout = '';
                proc.stdout.on('data', (d) => { stdout += d.toString(); });
                proc.on('close', (code) => {
                    if (code === 0 && stdout.trim().length > 0) {
                        const fullRange = new vscode.Range(
                            document.positionAt(0),
                            document.positionAt(text.length)
                        );
                        resolve([vscode.TextEdit.replace(fullRange, stdout)]);
                    } else {
                        resolve([]);
                    }
                });
                proc.on('error', () => resolve([]));
                proc.stdin.write(text);
                proc.stdin.end();
            });
        }
    });

    context.subscriptions.push(formatProvider);
}

function deactivate() {
    if (lspProcess) {
        lspProcess.kill();
        lspProcess = null;
    }
}

module.exports = {
    activate,
    deactivate
};
