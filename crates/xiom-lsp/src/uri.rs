// XIOM Language Server -- URI helpers
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::PathBuf;

/// Convert an LSP file URI to its parent directory path.
/// `file:///e%3A/Projects/AXIOM/stdlib/xiom/alloc.xi` -> `e:\Projects\AXIOM\stdlib\xiom`
pub fn uri_to_parent_dir(uri: &str) -> Option<String> {
    uri_to_file_path(uri).and_then(|p| p.parent().map(|p| p.to_string_lossy().to_string()))
}

/// Convert a filesystem path to a `file:///` URI. The colon of a Windows
/// drive letter is percent-encoded so the result round-trips through
/// `uri_to_file_path`.
pub fn path_to_uri(path: &std::path::Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    let payload = if s.len() > 1 && s.as_bytes()[1] == b':' {
        format!("{}%3A{}", &s[..1], &s[2..])
    } else {
        s
    };
    format!("file:///{}", payload)
}

/// Convert a percent-encoded file:// URI to a filesystem PathBuf.
pub fn uri_to_file_path(uri: &str) -> Option<PathBuf> {
    let path = uri.strip_prefix("file:///")?;
    let mut decoded = String::with_capacity(path.len());
    let bytes = path.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&path[i + 1..i + 3], 16) {
                decoded.push(byte as char);
                i += 3;
                continue;
            }
        }
        decoded.push(bytes[i] as char);
        i += 1;
    }
    Some(PathBuf::from(decoded))
}
