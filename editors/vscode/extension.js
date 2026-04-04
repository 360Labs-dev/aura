// Aura Language Extension for VS Code
// Provides syntax highlighting + diagnostics via the Aura Agent API

const vscode = require('vscode');
const { spawn } = require('child_process');

let agentProcess = null;
let requestId = 0;
let pendingRequests = new Map();
let diagnosticCollection = null;

function activate(context) {
    console.log('Aura extension activated');

    diagnosticCollection = vscode.languages.createDiagnosticCollection('aura');
    context.subscriptions.push(diagnosticCollection);

    // Try to start the agent process
    startAgent();

    // Run diagnostics on active editor
    if (vscode.window.activeTextEditor) {
        runDiagnostics(vscode.window.activeTextEditor.document);
    }

    // Run diagnostics on document change
    context.subscriptions.push(
        vscode.workspace.onDidChangeTextDocument(event => {
            if (event.document.languageId === 'aura') {
                runDiagnostics(event.document);
            }
        })
    );

    // Run diagnostics on editor switch
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor(editor => {
            if (editor && editor.document.languageId === 'aura') {
                runDiagnostics(editor.document);
            }
        })
    );

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('aura.build', () => {
            const terminal = vscode.window.createTerminal('Aura Build');
            terminal.show();
            const file = vscode.window.activeTextEditor?.document.fileName || 'src/main.aura';
            terminal.sendText(`aura build "${file}" --target web`);
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('aura.run', () => {
            const terminal = vscode.window.createTerminal('Aura Dev');
            terminal.show();
            terminal.sendText('aura run');
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('aura.explain', () => {
            const file = vscode.window.activeTextEditor?.document.fileName;
            if (file) {
                const terminal = vscode.window.createTerminal('Aura Explain');
                terminal.show();
                terminal.sendText(`aura explain "${file}"`);
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('aura.sketch', async () => {
            const description = await vscode.window.showInputBox({
                prompt: 'Describe your app',
                placeHolder: 'todo app with dark mode and swipe to delete'
            });
            if (description) {
                const terminal = vscode.window.createTerminal('Aura Sketch');
                terminal.show();
                terminal.sendText(`aura sketch "${description}"`);
            }
        })
    );
}

function startAgent() {
    try {
        agentProcess = spawn('aura', ['agent', 'serve'], {
            stdio: ['pipe', 'pipe', 'pipe']
        });

        let buffer = '';
        agentProcess.stdout.on('data', (data) => {
            buffer += data.toString();
            const lines = buffer.split('\n');
            buffer = lines.pop() || '';
            for (const line of lines) {
                if (line.trim()) {
                    try {
                        const response = JSON.parse(line);
                        const callback = pendingRequests.get(response.id);
                        if (callback) {
                            pendingRequests.delete(response.id);
                            callback(response);
                        }
                    } catch (e) {
                        // Ignore non-JSON output
                    }
                }
            }
        });

        agentProcess.on('error', () => {
            console.log('Aura agent not found — using fallback diagnostics');
            agentProcess = null;
        });

        agentProcess.on('exit', () => {
            agentProcess = null;
        });
    } catch (e) {
        agentProcess = null;
    }
}

function sendRequest(method, params) {
    return new Promise((resolve) => {
        if (!agentProcess) {
            resolve(null);
            return;
        }
        const id = ++requestId;
        const request = JSON.stringify({
            jsonrpc: '2.0',
            id,
            method,
            params
        }) + '\n';

        pendingRequests.set(id, resolve);
        agentProcess.stdin.write(request);

        // Timeout after 5s
        setTimeout(() => {
            if (pendingRequests.has(id)) {
                pendingRequests.delete(id);
                resolve(null);
            }
        }, 5000);
    });
}

async function runDiagnostics(document) {
    if (document.languageId !== 'aura') return;

    const source = document.getText();
    const response = await sendRequest('diagnostics.get', { source });

    const diagnostics = [];

    if (response && response.result && response.result.diagnostics) {
        for (const diag of response.result.diagnostics) {
            const severity = diag.severity === 'error'
                ? vscode.DiagnosticSeverity.Error
                : diag.severity === 'warning'
                    ? vscode.DiagnosticSeverity.Warning
                    : vscode.DiagnosticSeverity.Information;

            const line = Math.max(0, (diag.location?.line || 1) - 1);
            const col = Math.max(0, (diag.location?.column || 1) - 1);
            const range = new vscode.Range(line, col, line, col + 10);

            const d = new vscode.Diagnostic(range, diag.message, severity);
            d.code = diag.code;
            d.source = 'aura';

            diagnostics.push(d);
        }
    }

    diagnosticCollection.set(document.uri, diagnostics);
}

function deactivate() {
    if (agentProcess) {
        agentProcess.kill();
        agentProcess = null;
    }
}

module.exports = { activate, deactivate };
