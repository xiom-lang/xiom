const vscode = require('vscode');
const { spawn } = require('child_process');

let client;

function activate(context) {
  console.log('AXIOM extension activated');
  client = new LspClient(context);
  client.start().catch(() => {
    console.log('AXIOM LSP not available — syntax highlighting only');
  });
}

function deactivate() {
  if (client) {
    client.shutdown();
  }
}

class LspClient {
  constructor(context) {
    this.context = context;
    this.server = null;
    this.buffer = '';
    this.nextId = 1;
    this.pending = new Map();
    this.diagCollection = vscode.languages.createDiagnosticCollection('axiom');
    this.initialized = false;
  }

  async start() {
    const config = vscode.workspace.getConfiguration('axiom');
    let lspPath = config.get('lsp.path') || '';

    if (!lspPath) {
      const candidates = [
        'axiom-lsp',
        './axiom-lsp',
        'target/debug/axiom-lsp.exe',
        'target/release/axiom-lsp.exe'
      ];
      for (const c of candidates) {
        try {
          await vscode.workspace.fs.stat(vscode.Uri.file(c));
          lspPath = c;
          break;
        } catch {}
      }
    }

    if (!lspPath) {
      vscode.window.showInformationMessage(
        'AXIOM LSP not found. Install with: cargo build -p axiom-lsp\nSyntax highlighting is still active.'
      );
      throw new Error('LSP binary not found');
    }

    this.server = spawn(lspPath, [], { stdio: ['pipe', 'pipe', 'pipe'] });

    this.server.stdout.on('data', (data) => this._handleData(data));
    this.server.stderr.on('data', (data) => console.log('LSP stderr: ' + data.toString()));
    this.server.on('error', (err) => console.log('LSP process error: ' + err.message));
    this.server.on('exit', (code) => {
      console.log('LSP process exited with code ' + code);
      this.server = null;
    });

    await this._initialize();
    this._registerProviders();
    this._registerDocumentListeners();
  }

  _handleData(data) {
    this.buffer += data.toString();
    while (true) {
      const headerEnd = this.buffer.indexOf('\r\n\r\n');
      if (headerEnd === -1) break;

      const header = this.buffer.slice(0, headerEnd);
      const match = header.match(/Content-Length:\s*(\d+)/i);
      if (!match) {
        this.buffer = this.buffer.slice(headerEnd + 4);
        continue;
      }

      const length = parseInt(match[1], 10);
      const bodyStart = headerEnd + 4;
      if (this.buffer.length < bodyStart + length) break;

      const body = this.buffer.slice(bodyStart, bodyStart + length);
      this.buffer = this.buffer.slice(bodyStart + length);

      try {
        this._handleMessage(JSON.parse(body));
      } catch (e) {
        console.log('Failed to parse LSP message: ' + e.message);
      }
    }
  }

  _handleMessage(msg) {
    if (msg.method === 'textDocument/publishDiagnostics') {
      const uri = vscode.Uri.parse(msg.params.uri);
      const diagnostics = (msg.params.diagnostics || []).map((d) => {
        const range = new vscode.Range(
          d.range.start.line, d.range.start.character,
          d.range.end.line, d.range.end.character
        );
        const diag = new vscode.Diagnostic(range, d.message);
        if (d.severity !== undefined) {
          diag.severity = d.severity;
        }
        if (d.source) diag.source = d.source;
        return diag;
      });
      this.diagCollection.set(uri, diagnostics);
      return;
    }

    if (msg.id !== undefined) {
      const pending = this.pending.get(msg.id);
      if (pending) {
        clearTimeout(pending.timer);
        this.pending.delete(msg.id);
        if (msg.error) {
          pending.reject(new Error(msg.error.message || 'LSP error'));
        } else {
          pending.resolve(msg.result);
        }
      }
    }
  }

  _sendNotification(method, params) {
    this._write({ jsonrpc: '2.0', method, params });
  }

  _sendRequest(method, params) {
    return new Promise((resolve, reject) => {
      const id = this.nextId++;
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error('LSP request timed out: ' + method));
      }, 5000);
      this.pending.set(id, { resolve, reject, timer });
      this._write({ jsonrpc: '2.0', id, method, params });
    });
  }

  _write(msg) {
    const body = JSON.stringify(msg);
    const header = 'Content-Length: ' + Buffer.byteLength(body) + '\r\n\r\n';
    this.server.stdin.write(header + body);
  }

  async _initialize() {
    const rootUri = vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0
      ? vscode.workspace.workspaceFolders[0].uri.toString()
      : null;

    await this._sendRequest('initialize', {
      processId: process.pid,
      rootUri,
      capabilities: {
        textDocument: {
          synchronization: { dynamicRegistration: false, willSave: false, didSave: false, willSaveWaitUntil: false },
          completion: { completionItem: { snippetSupport: false } },
          hover: { dynamicRegistration: false },
          signatureHelp: { dynamicRegistration: false },
          definition: { dynamicRegistration: false }
        },
        workspace: { didChangeConfiguration: { dynamicRegistration: false } }
      },
      trace: 'off'
    });

    this._sendNotification('initialized', {});
    this.initialized = true;
    console.log('AXIOM LSP initialized');
  }

  _registerProviders() {
    const selector = { language: 'axiom', scheme: 'file' };

    this.context.subscriptions.push(
      vscode.languages.registerCompletionItemProvider(selector, {
        provideCompletionItems: (document, position) => {
          return this._sendRequest('textDocument/completion', {
            textDocument: { uri: document.uri.toString() },
            position: { line: position.line, character: position.character }
          }).then((result) => {
            const items = result || [];
            return items.map((item) => {
              const ci = new vscode.CompletionItem(item.label);
              if (item.kind !== undefined) ci.kind = item.kind;
              if (item.detail) ci.detail = item.detail;
              if (item.documentation) ci.documentation = item.documentation;
              if (item.insertText) ci.insertText = item.insertText;
              return ci;
            });
          }, () => undefined);
        }
      }, '.')
    );

    this.context.subscriptions.push(
      vscode.languages.registerHoverProvider(selector, {
        provideHover: (document, position) => {
          return this._sendRequest('textDocument/hover', {
            textDocument: { uri: document.uri.toString() },
            position: { line: position.line, character: position.character }
          }).then((result) => {
            if (!result || !result.contents) return undefined;
            const contents = Array.isArray(result.contents) ? result.contents : [result.contents];
            const markdown = new vscode.MarkdownString(contents.map(c => typeof c === 'string' ? c : c.value).join('\n'));
            const range = result.range
              ? new vscode.Range(result.range.start.line, result.range.start.character, result.range.end.line, result.range.end.character)
              : undefined;
            return new vscode.Hover(markdown, range);
          }, () => undefined);
        }
      })
    );

    this.context.subscriptions.push(
      vscode.languages.registerDefinitionProvider(selector, {
        provideDefinition: (document, position) => {
          return this._sendRequest('textDocument/definition', {
            textDocument: { uri: document.uri.toString() },
            position: { line: position.line, character: position.character }
          }).then((result) => {
            if (!result) return undefined;
            if (Array.isArray(result)) {
              if (result.length === 0) return undefined;
              const loc = result[0];
              return new vscode.Location(
                vscode.Uri.parse(loc.uri),
                new vscode.Range(loc.range.start.line, loc.range.start.character, loc.range.end.line, loc.range.end.character)
              );
            }
            return new vscode.Location(
              vscode.Uri.parse(result.uri),
              new vscode.Range(result.range.start.line, result.range.start.character, result.range.end.line, result.range.end.character)
            );
          }, () => undefined);
        }
      })
    );

    this.context.subscriptions.push(
      vscode.languages.registerSignatureHelpProvider(selector, {
        provideSignatureHelp: (document, position) => {
          return this._sendRequest('textDocument/signatureHelp', {
            textDocument: { uri: document.uri.toString() },
            position: { line: position.line, character: position.character }
          }).then((result) => {
            if (!result || !result.signatures || result.signatures.length === 0) return undefined;
            const sh = new vscode.SignatureHelp();
            sh.signatures = result.signatures.map((sig) => {
              const si = new vscode.SignatureInformation(sig.label);
              if (sig.documentation) si.documentation = sig.documentation;
              if (sig.parameters) {
                si.parameters = sig.parameters.map((p) => new vscode.ParameterInformation(p.label, p.documentation));
              }
              return si;
            });
            sh.activeSignature = result.activeSignature || 0;
            sh.activeParameter = result.activeParameter || 0;
            return sh;
          }, () => undefined);
        }
      }, '(', ',')
    );
  }

  _registerDocumentListeners() {
    this.context.subscriptions.push(
      vscode.workspace.onDidOpenTextDocument((doc) => {
        if (doc.languageId !== 'axiom' || !this.server) return;
        this._sendNotification('textDocument/didOpen', {
          textDocument: {
            uri: doc.uri.toString(),
            languageId: 'axiom',
            version: 1,
            text: doc.getText()
          }
        });
      })
    );

    this.context.subscriptions.push(
      vscode.workspace.onDidChangeTextDocument((e) => {
        if (e.document.languageId !== 'axiom' || !this.server) return;
        this._sendNotification('textDocument/didChange', {
          textDocument: {
            uri: e.document.uri.toString(),
            version: e.document.version
          },
          contentChanges: [
            { text: e.document.getText() }
          ]
        });
      })
    );

    this.context.subscriptions.push(
      vscode.workspace.onDidCloseTextDocument((doc) => {
        if (doc.languageId !== 'axiom' || !this.server) return;
        this._sendNotification('textDocument/didClose', {
          textDocument: { uri: doc.uri.toString() }
        });
      })
    );

    // Send didOpen for already-open .ax documents
    vscode.workspace.textDocuments.forEach((doc) => {
      if (doc.languageId === 'axiom' && this.server) {
        this._sendNotification('textDocument/didOpen', {
          textDocument: {
            uri: doc.uri.toString(),
            languageId: 'axiom',
            version: 1,
            text: doc.getText()
          }
        });
      }
    });
  }

  async shutdown() {
    if (!this.server) return;
    try {
      await this._sendRequest('shutdown', {});
    } catch (e) {
      console.log('LSP shutdown error: ' + e.message);
    }
    this._sendNotification('exit', {});
    this.diagCollection.dispose();
    this.server.kill();
    this.server = null;
  }
}

module.exports = { activate, deactivate };
