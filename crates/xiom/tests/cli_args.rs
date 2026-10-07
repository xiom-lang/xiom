// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// Driver argument-resolution regression (Stage 5 driver item): the manual
// parser used to drop files literally named wasm/arm/riscv as if they were
// positional target sugar.

use xiom::resolve_source_files;

#[test]
fn target_named_source_files_are_kept_but_bare_sugar_is_skipped() {
    let dir = std::env::temp_dir().join("xiom_cli_args_test");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("wasm"), "fn main() -> Int { return 0; }").unwrap();
    std::fs::write(dir.join("arm"), "fn main() -> Int { return 0; }").unwrap();

    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();

    // Real files named like targets are SOURCES, not sugar.
    let kept = resolve_source_files(&["xiom".to_string(), "wasm".to_string(), "arm".to_string()]);
    assert_eq!(kept, vec!["wasm".to_string(), "arm".to_string()]);

    // A bare `wasm` with no such file stays skipped (positional target).
    std::fs::remove_file(dir.join("wasm")).unwrap();
    let skipped = resolve_source_files(&["xiom".to_string(), "wasm".to_string(), "src.xi".to_string()]);
    assert_eq!(skipped, vec!["src.xi".to_string()]);

    std::env::set_current_dir(old).unwrap();
}

// m205 (handoff tooling item): `xiom doc` (and the -doc/--doc spellings) must
// dispatch to the xiom-doc tool -- sibling binary first, XIOM_HOME/bin
// fallback -- and `--help` must reach the tool instead of being swallowed by
// the main --help short-circuit. Pre-fix the main compiler USAGE text was
// printed for every spelling. When no xiom-doc binary is installed next to
// the test binary or in XIOM_HOME/bin, the dispatch attempt's not-found
// diagnostic (naming xiom-doc) is the recognition signal.
#[test]
fn m205_doc_subcommand_dispatches_to_tool() {
    for token in ["doc", "-doc", "--doc"] {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_xiom"))
            .arg(token)
            .arg("--help")
            .output()
            .unwrap_or_else(|e| panic!("failed to spawn xiom: {e}"));
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        let reached_doc = stdout.contains("xiom-doc")
            || stderr.contains("xiom-doc")
            || stdout.contains("XIOM Doc")
            || stderr.contains("XIOM Doc");
        assert!(
            reached_doc,
            "`xiom {token} --help` must reach the xiom-doc dispatch (m205); exit={:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            out.status.code()
        );
    }
}
