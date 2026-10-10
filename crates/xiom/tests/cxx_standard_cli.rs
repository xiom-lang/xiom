// m258 (REL-20261010-1815-bindings / packages-6): `--cxx-standard` must
// reach C++ `--c-source` compilations. The probe uses std::string_view -- a
// real C++17 header feature, NOT a clang extension -- so a clang whose
// default std is pre-17 (this dev box: clang 22, default gnu++14) only
// builds it when the flag actually selects C++17 for the C++ TU.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

#[test]
fn cxx_standard_applies_to_c_source_cpp() {
    let dir = std::env::temp_dir().join(format!("xiom_cxx_std_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let main_xi = dir.join("main.xi");
    std::fs::write(&main_xi, "fn main() -> Int { return 0; }\n").unwrap();
    let probe_cpp = dir.join("probe.cpp");
    std::fs::write(
        &probe_cpp,
        "#include <string_view>\nextern \"C\" int probe_sv_len(const char* p) {\n  std::string_view s(p);\n  return (int)s.size();\n}\n",
    )
    .unwrap();
    let out_bin = dir.join("probe_out.exe");
    let out = Command::new(xiom_bin())
        .arg(&main_xi)
        .arg("--c-source")
        .arg(&probe_cpp)
        .arg("--cxx-standard")
        .arg("17")
        .arg("-o")
        .arg(&out_bin)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(
        out.status.success(),
        "xiom --cxx-standard 17 must compile a C++17 --c-source; exit={:?}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out_bin.exists(), "output binary must exist");
    let _ = std::fs::remove_dir_all(&dir);
}

// m257 + m258: the help surface must list both flags.
#[test]
fn help_lists_icon_and_cxx_standard() {
    let out = Command::new(xiom_bin())
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("--icon"),
        "--help must list --icon (m257):\n{text}"
    );
    assert!(
        text.contains("--cxx-standard"),
        "--help must list --cxx-standard (m258):\n{text}"
    );
}
