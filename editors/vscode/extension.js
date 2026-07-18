const vscode = require('vscode');
const { spawn } = require('child_process');

let client;

function activate(context) {
  console.log('XIOM extension activated');

  // LSP client
  client = new LspClient(context);
  client.start().catch(() => {
    console.log('XIOM LSP not available — syntax highlighting only');
  });

  // DAP debug adapter
  const dbgProvider = new XiomDebugAdapterDescriptorFactory(context);
  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory('xiom', dbgProvider)
  );
  context.subscriptions.push(
    vscode.debug.registerDebugConfigurationProvider('xiom', new XiomDebugConfigProvider())
  );

  console.log('XIOM debug adapter registered (xiom-dbg)');
}

function deactivate() {
  if (client) {
    client.shutdown();
  }
}

// ============================================================================
// Debug Adapter Descriptor Factory — resolves xiom-dbg binary path
// ============================================================================

class XiomDebugAdapterDescriptorFactory {
  constructor(context) {
    this.context = context;
  }

  async createDebugAdapterDescriptor(session, executable) {
    const config = vscode.workspace.getConfiguration('xiom');
    let dbgPath = config.get('dbg.path') || '';

    if (!dbgPath) {
      const rootFolder = vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath;
      const names = ['xiom-dbg', 'xiom-dbg.exe'];
      const candidates = [];

      // Workspace target directories
      if (rootFolder) {
        for (const name of names) {
          candidates.push(vscode.Uri.joinPath(vscode.Uri.file(rootFolder), 'target', 'debug', name));
          candidates.push(vscode.Uri.joinPath(vscode.Uri.file(rootFolder), 'target', 'release', name));
        }
      }
      // Extension directory
      if (this.context.extensionUri) {
        for (const name of names) {
          candidates.push(vscode.Uri.joinPath(this.context.extensionUri, name));
        }
      }
      // PATH
      candidates.push(vscode.Uri.file('xiom-dbg'));
      candidates.push(vscode.Uri.file('xiom-dbg.exe'));

      for (const candidate of candidates) {
        try {
          await vscode.workspace.fs.stat(candidate);
          dbgPath = candidate.fsPath;
          break;
        } catch { /* not found */ }
      }
    }

    if (!dbgPath) {
      vscode.window.showErrorMessage('XIOM Debugger: xiom-dbg binary not found. Set xiom.dbg.path in settings or build xiom-dbg.');
      throw new Error('xiom-dbg not found');
    }

    console.log(`XIOM Debugger: using ${dbgPath}`);
    return new vscode.DebugAdapterExecutable(dbgPath, [], {});
  }
}

// ============================================================================
// Debug Configuration Provider — provides default launch configs
// ============================================================================

class XiomDebugConfigProvider {
  resolveDebugConfiguration(folder, config) {
    if (!config.type && !config.request && !config.name) {
      config.type = 'xiom';
      config.request = 'launch';
      config.name = 'Debug XIOM Program';
    }
    if (!config.program) {
      config.program = '${workspaceFolder}/a.exe';
    }
    if (config.stopOnEntry === undefined) {
      config.stopOnEntry = true;
    }
    return config;
  }
}

class LspClient {
  constructor(context) {
    this.context = context;
    this.server = null;
    this.buffer = '';
    this.nextId = 1;
    this.pending = new Map();
    this.diagCollection = vscode.languages.createDiagnosticCollection('xiom');
    this.initialized = false;
  }

  async start() {
    const config = vscode.workspace.getConfiguration('xiom');
    let lspPath = config.get('lsp.path') || '';

    // Resolve workspace root for relative path lookups
    const rootFolder = vscode.workspace.workspaceFolders
      && vscode.workspace.workspaceFolders.length > 0
      ? vscode.workspace.workspaceFolders[0].uri.fsPath
      : null;

    if (!lspPath) {
      const candidates = [];
      const names = ['xiom-lsp', 'xiom-lsp.exe'];
      // Try workspace-root-relative paths first
      if (rootFolder) {
        for (const name of names) {
          candidates.push(vscode.Uri.joinPath(vscode.Uri.file(rootFolder), 'target', 'debug', name));
          candidates.push(vscode.Uri.joinPath(vscode.Uri.file(rootFolder), 'target', 'release', name));
        }
      }
      // Try extension directory
      if (this.context.extensionUri) {
        for (const name of names) {
          candidates.push(vscode.Uri.joinPath(this.context.extensionUri, name));
        }
      }
      // Try PATH
      candidates.push(vscode.Uri.file('xiom-lsp'));
      candidates.push(vscode.Uri.file('xiom-lsp.exe'));

      for (const uri of candidates) {
        try {
          await vscode.workspace.fs.stat(uri);
          lspPath = uri.fsPath;
          break;
        } catch {}
      }
    }

    if (!lspPath) {
      vscode.window.showInformationMessage(
        'XIOM LSP not found. Build it with: cargo build -p xiom-lsp\nSyntax highlighting is still active. Auto-completion and go-to-definition will be unavailable.'
      );
      // Register providers anyway so they show "LSP not found" hints
      this._registerProviders();
      this._registerDocumentListeners();
      return;
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
    if (this.server) {
      this._write({ jsonrpc: '2.0', method, params });
    }
  }

  _sendRequest(method, params) {
    if (!this.server) {
      return Promise.reject(new Error('LSP server not available'));
    }
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
    if (!this.server || !this.server.stdin) return;
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
    console.log('XIOM LSP initialized');
  }

  _registerProviders() {
    const selector = { language: 'xiom', scheme: 'file' };

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
        if (doc.languageId !== 'xiom' || !this.server) return;
        this._sendNotification('textDocument/didOpen', {
          textDocument: {
            uri: doc.uri.toString(),
            languageId: 'xiom',
            version: 1,
            text: doc.getText()
          }
        });
      })
    );

    this.context.subscriptions.push(
      vscode.workspace.onDidChangeTextDocument((e) => {
        if (e.document.languageId !== 'xiom' || !this.server) return;
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
        if (doc.languageId !== 'xiom' || !this.server) return;
        this._sendNotification('textDocument/didClose', {
          textDocument: { uri: doc.uri.toString() }
        });
      })
    );

    // Send didOpen for already-open .xi documents
    vscode.workspace.textDocuments.forEach((doc) => {
      if (doc.languageId === 'xiom' && this.server) {
        this._sendNotification('textDocument/didOpen', {
          textDocument: {
            uri: doc.uri.toString(),
            languageId: 'xiom',
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
