// m359 (REL-20261016-1128-engine class 4): `xiom <file> --timeout 900
// --timeout 900` panicked inside clap (rc 101). With `ignore_errors(true)` a
// duplicated single-value flag after the first positional returns PARTIAL
// matches that carry no SetTrue defaults; `Cli::flag` then panicked with
// `arg 'emit-ir''s 'ArgAction' should be one of SetTrue/SetFalse`.
//
// The driver now reports a clean usage error (rc 2) for that shape and never
// reads flags off default-less matches. Duplicates BEFORE the positional keep
// their legacy semantics (accepted; the repeated value goes unread).
//
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::{Path, PathBuf};
use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

fn temp_source(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "xiom_m359_{tag}_{}_{}",
        std::process::id(),
        std::line!()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create m359 temp dir");
    let src = dir.join("main.xi");
    std::fs::write(&src, "fn main() -> Int { return 0; }\n").expect("write m359 source");
    src
}

fn cleanup(src: &Path) {
    if let Some(dir) = src.parent() {
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn m359_duplicate_value_flags_after_positional_are_clean_usage_errors() {
    let src = temp_source("dup_after");
    for flag in ["--timeout", "--jobs"] {
        let out = Command::new(xiom_bin())
            .arg(&src)
            .arg(flag)
            .arg("4")
            .arg(flag)
            .arg("4")
            .output()
            .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(2),
            "duplicate {flag} after the positional must be a usage error (m359); stderr:\n{stderr}"
        );
        assert!(
            stderr.contains("cannot be used multiple times"),
            "the diagnostic must name the duplicate rule (m359); stderr:\n{stderr}"
        );
        assert!(
            stderr.contains(flag),
            "the diagnostic must name {flag} (m359); stderr:\n{stderr}"
        );
        assert!(
            !stderr.contains("panicked") && !stderr.contains("ArgAction"),
            "the clap panic must be gone (m359); stderr:\n{stderr}"
        );
    }
    cleanup(&src);
}

#[test]
fn m359_duplicate_value_flags_before_positional_keep_legacy_shape() {
    // No positional at all: the pre-existing usage path (rc 1) still wins.
    // The duplicate must NOT become the new rc-2 usage error.
    let out = Command::new(xiom_bin())
        .args(["--timeout", "900", "--timeout", "900"])
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "pre-positional duplicates keep the legacy shape (m359); stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("cannot be used multiple times"),
        "pre-positional duplicates must not become usage errors (m359); stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("panicked") && !stderr.contains("ArgAction"),
        "pre-positional duplicates must not panic (m359); stderr:\n{stderr}"
    );
}
