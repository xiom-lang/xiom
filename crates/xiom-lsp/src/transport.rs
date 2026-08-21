// XIOM Language Server -- Transport layer (stdin/stdout JSON-RPC)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use std::io::{self, BufRead, Read, Write};

/// Reads LSP JSON-RPC messages from stdin with Content-Length framing.
pub struct LspReader {
    stdin: io::Stdin,
}

impl LspReader {
    pub fn new() -> Self {
        Self {
            stdin: io::stdin(),
        }
    }

    pub fn read_message(&self) -> Option<String> {
        let mut content_length: Option<usize> = None;

        loop {
            let mut line = String::new();
            self.stdin.lock().read_line(&mut line).ok()?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
                content_length = len_str.trim().parse::<usize>().ok();
            }
        }

        let len = content_length?;
        let mut buf = vec![0u8; len];
        let mut handle = self.stdin.lock();
        let mut read_total = 0;
        while read_total < len {
            let n = handle.read(&mut buf[read_total..]).ok()?;
            if n == 0 {
                return None;
            }
            read_total += n;
        }
        String::from_utf8(buf).ok()
    }
}

/// Writes an LSP JSON-RPC message to stdout with Content-Length framing.
pub fn write_lsp_message(body: &serde_json::Value) {
    let body_str = serde_json::to_string(body).expect("JSON serialization failed");
    let header = format!("Content-Length: {}\r\n\r\n", body_str.len());
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(header.as_bytes());
    let _ = stdout.write_all(body_str.as_bytes());
    let _ = stdout.flush();
}
