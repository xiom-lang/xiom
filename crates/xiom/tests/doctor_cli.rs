// Doctor CLI integration lock (front-end audit Sprint A: FE-1..FE-5, FE-7).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Runs the freshly built `xiom` binary with a temp XIOM_HOME/XIOM_STDLIB so
// the identity + version-parity behavior is deterministic on any machine:
// the fake stdlib root reports a version that cannot match the compiler, so
// the report MUST warn and `--json` MUST stay parseable.

use std::path::{Path, PathBuf};
use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

/// Temp XIOM_HOME + a fake, content-valid stdlib root (`package.xi` +
/// `xiom/`) so `paths::stdlib_root()` selects exactly this directory.
struct FakeInstall {
    root: PathBuf,
}

impl FakeInstall {
    fn new(tag: &str, stdlib_version: &str) -> FakeInstall {
        let root = std::env::temp_dir().join(format!(
            "xiom_doctor_cli_{}_{}_{}",
            tag,
            std::process::id(),
            std::line!()
        ));
        let stdlib = root.join("stdlib");
        std::fs::create_dir_all(stdlib.join("xiom")).expect("create fake stdlib tree");
        std::fs::write(
            stdlib.join("package.xi"),
            format!(
                "package xiom_std {{\n  name: \"xiom-std\";\n  version: \"{stdlib_version}\";\n}}\n"
            ),
        )
        .expect("write fake package.xi");
        FakeInstall { root }
    }

    fn home(&self) -> PathBuf {
        self.root.join("home")
    }

    fn stdlib_root(&self) -> PathBuf {
        self.root.join("stdlib")
    }

    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(xiom_bin())
            .args(args)
            .env("XIOM_HOME", self.home())
            .env("XIOM_STDLIB", self.stdlib_root())
            .output()
            .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()))
    }
}

impl Drop for FakeInstall {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn doctor_json_reports_identity_and_exit_codes_match_the_report() {
    let install = FakeInstall::new("json", "0.0.1");
    let out = install.run(&["doctor", "--json"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("doctor --json must emit valid JSON: {e}\nstdout: {stdout}"));

    // FE-7 shape.
    for key in ["schema", "ok", "compiler", "identity", "checks", "warnings", "errors"] {
        assert!(parsed.get(key).is_some(), "missing key {key}: {parsed}");
    }

    // FE-5: doctor reports the SAME stdlib root the compiler resolves for
    // this environment (XIOM_STDLIB is the first candidate).
    let reported = parsed["identity"]["stdlib_root"]
        .as_str()
        .expect("identity.stdlib_root");
    let expected = install.stdlib_root();
    assert!(
        Path::new(reported) == expected,
        "doctor reported '{reported}', expected '{}'",
        expected.display()
    );
    assert_eq!(
        parsed["identity"]["stdlib_version"],
        serde_json::Value::String("0.0.1".to_string())
    );

    // FE-4/FE-16: a fake (mismatched) stdlib version must produce a warning.
    let warnings = parsed["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|w| w
            .as_str()
            .unwrap_or_default()
            .contains("does not match compiler")),
        "expected a version-parity warning, got {warnings:?}"
    );

    // FE-7 exit-code contract: 0 all-OK, 1 warnings, 2 errors.
    let errors = parsed["errors"].as_array().expect("errors array");
    let want = if !errors.is_empty() {
        2
    } else if !warnings.is_empty() {
        1
    } else {
        0
    };
    assert_eq!(out.status.code(), Some(want), "stdout: {stdout}");
    assert_eq!(
        parsed["ok"],
        serde_json::Value::Bool(errors.is_empty() && warnings.is_empty())
    );
}

#[test]
fn doctor_text_has_identity_block_and_no_retired_command() {
    let install = FakeInstall::new("text", "0.61.3");
    let out = install.run(&["doctor"]);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(text.contains("XIOM Doctor"), "text: {text}");
    assert!(text.contains("IDENTITY"), "text: {text}");
    assert!(text.contains("install root"), "text: {text}");
    assert!(
        text.contains(&install.stdlib_root().display().to_string()),
        "doctor must print the resolved stdlib root: {text}"
    );
    // FE-2 acceptance: the dead `xiom install llvm` guidance is gone.
    assert!(
        !text.contains("xiom install"),
        "retired command reference leaked: {text}"
    );
}
