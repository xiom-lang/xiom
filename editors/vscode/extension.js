// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

const vscode = require('vscode');
const { spawn, spawnSync } = require('child_process');

const TOOLCHAIN_INSTALL_URL = 'https://xiom-lang.org/install';

/**
 * Resolve a XIOM toolchain binary. Production resolution order:
 *   1. Explicit VS Code setting (highest priority)
 *   2. PATH -- the installed release toolchain (the official installer puts
 *      bin/ on PATH)
 *   3. Workspace target/release, target/debug -- compiler developers
 * The VSIX bundles NO platform binaries: one universal package works on
 * Windows, Linux and macOS, and the adapter always matches the user's
 * installed toolchain version. Returns the resolved path/command or null.
 */
async function resolveXiomBinary(name, settingValue) {
  if (settingValue) return settingValue;

  // 2. PATH resolution via where/which -- end-user installs
  const probe = process.platform === 'win32' ? 'where' : 'which';
  try {
    const res = spawnSync(probe, [name], { encoding: 'utf8', timeout: 3000 });
    if (res.status === 0 && res.stdout) {
      const first = res.stdout.split(/\r?\n/).find((l) => l.trim().length > 0);
      if (first) return first.trim();
    }
  } catch { /* where/which unavailable */ }

  // 3. Workspace target dirs -- developing the compiler itself
  const rootFolder = vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath;
  const fileNames = process.platform === 'win32' ? [`${name}.exe`, name] : [name];
  const candidates = [];
  if (rootFolder) {
    for (const fn of fileNames) {
      candidates.push(vscode.Uri.joinPath(vscode.Uri.file(rootFolder), 'target', 'release', fn));
      candidates.push(vscode.Uri.joinPath(vscode.Uri.file(rootFolder), 'target', 'debug', fn));
    }
  }
  for (const uri of candidates) {
    try {
      await vscode.workspace.fs.stat(uri);
      return uri.fsPath;
    } catch { /* not found */ }
  }
  return null;
}

/**
 * Resolve the three toolchain entry points the extension needs. The command
 * palette (`XIOM: Recheck toolchain`) and activation both use this.
 */
async function resolveToolchain() {
  const config = vscode.workspace.getConfiguration('xiom');
  const [compiler, lsp, dbg] = await Promise.all([
    resolveXiomBinary('xiom', ''),
    resolveXiomBinary('xiom-lsp', config.get('lsp.path') || ''),
    resolveXiomBinary('xiom-dbg', config.get('debugAdapterPath') || config.get('dbg.path') || ''),
  ]);
  const missing = [];
  if (!compiler) missing.push('xiom');
  if (!lsp) missing.push('xiom-lsp');
  if (!dbg) missing.push('xiom-dbg');
  return { compiler, lsp, dbg, missing };
}

/**
 * First-run UX: NEVER install the toolchain silently -- the official
 * installer is the single install path. Points at xiom-lang.org/install and
 * offers a recheck.
 */
async function showToolchainMissing(missing) {
  const pick = await vscode.window.showWarningMessage(
    `XIOM toolchain not found (${missing.join(', ')}). Install it from xiom-lang.org/install`,
    'Open install page',
    'Recheck toolchain'
  );
  if (pick === 'Open install page') {
    await vscode.env.openExternal(vscode.Uri.parse(TOOLCHAIN_INSTALL_URL));
  } else if (pick === 'Recheck toolchain') {
    await vscode.commands.executeCommand('xiom.recheckToolchain');
  }
}

let client;

async function activate(context) {
  console.log('XIOM extension activated');

  context.subscriptions.push(
    vscode.commands.registerCommand('xiom.recheckToolchain', async () => {
      const toolchain = await resolveToolchain();
      if (toolchain.missing.length === 0) {
        vscode.window.showInformationMessage(
          'XIOM toolchain OK: xiom, xiom-lsp and xiom-dbg resolved.'
        );
      } else {
        await showToolchainMissing(toolchain.missing);
      }
    })
  );

  // Verify the toolchain once per activation; never install silently.
  const toolchain = await resolveToolchain();
  if (toolchain.missing.length > 0) {
    await showToolchainMissing(toolchain.missing);
  }

  // LSP client
  client = new LspClient(context, toolchain.lsp);
  client.start().catch(() => {
    console.log('XIOM LSP not available -- syntax highlighting only');
  });

  // DAP debug adapter
  const dbgProvider = new XiomDebugAdapterDescriptorFactory();
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
// Debug Adapter Descriptor Factory -- resolves xiom-dbg AT RUNTIME
// ============================================================================

class XiomDebugAdapterDescriptorFactory {
  async createDebugAdapterDescriptor(session, executable) {
    const config = vscode.workspace.getConfiguration('xiom');
    const dbgPath = await resolveXiomBinary(
      'xiom-dbg',
      config.get('debugAdapterPath') || config.get('dbg.path') || ''
    );

    if (!dbgPath) {
      await showToolchainMissing(['xiom-dbg']);
      throw new Error('xiom-dbg not found');
    }

    console.log(`XIOM Debugger: using ${dbgPath}`);
    return new vscode.DebugAdapterExecutable(dbgPath, [], {});
  }
}

// ============================================================================
// Debug Configuration Provider -- provides default launch configs
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
  constructor(context, lspPath) {
    this.context = context;
    this.lspPath = lspPath;
    this.server = null;
    this.buffer = '';
    this.nextId = 1;
    this.pending = new Map();
    this.diagCollection = vscode.languages.createDiagnosticCollection('xiom');
    this.initialized = false;
  }

  async start() {
    if (!this.lspPath) {
      // The activation toolchain check already surfaced the install hint.
      console.log('XIOM LSP not found -- syntax highlighting only');
      this._registerProviders();
      this._registerDocumentListeners();
      return;
    }

    this.server = spawn(this.lspPath, [], { stdio: ['pipe', 'pipe', 'pipe'] });

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
