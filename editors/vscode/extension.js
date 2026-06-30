const vscode = require('vscode');
const { spawn } = require('child_process');

let client;

function activate(context) {
  console.log('AXIOM extension activated');

  // Try to start LSP
  startLsp(context).catch(() => {
    console.log('AXIOM LSP not available — syntax highlighting only');
  });
}

async function startLsp(context) {
  const config = vscode.workspace.getConfiguration('axiom');
  let lspPath = config.get('lsp.path') || '';

  if (!lspPath) {
    // Try common locations
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
    return;
  }

  // Simple LSP via stdio
  const server = spawn(lspPath, [], { stdio: ['pipe', 'pipe', 'pipe'] });

  // Minimal LSP client — in production, use vscode-languageclient
  // For MVP, register basic features manually
}

function deactivate() {
  if (client) {
    client.kill();
  }
}

module.exports = { activate, deactivate };
