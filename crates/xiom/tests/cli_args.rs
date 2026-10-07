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

// m214 (--icon): the Windows exe icon flag must compile the .rc with
// llvm-rc (or MSVC rc) and link the .res in; a missing icon file is a hard
// error, never silently ignored. The ICO is generated in memory (no binary
// fixture in the repo, which ascii_guard requires to be pure text).
#[test]
fn m214_icon_embeds_or_reports_cleanly() {
    let base = std::env::temp_dir().join(format!("xiom_m214_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let src = base.join("main.xi");
    let ico = base.join("icon.ico");
    let out_exe = base.join("icon_out.exe");
    std::fs::write(&src, "fn main() -> Int { return 0; }\n").unwrap();

    // Minimal 16x16 32bpp ICO: 6-byte header + 16-byte directory entry +
    // 40-byte BITMAPINFOHEADER + BGRA pixels + AND mask (all zero).
    let pixels = 16usize * 16 * 4;
    let mask = 4 * 16;
    let dib = 40 + pixels + mask;
    let mut ico_bytes: Vec<u8> = vec![0, 0, 1, 0, 1, 0, 16, 16, 0, 0, 1, 0, 32, 0];
    ico_bytes.extend_from_slice(&(dib as u32).to_le_bytes());
    ico_bytes.extend_from_slice(&22u32.to_le_bytes());
    ico_bytes.extend_from_slice(&40u32.to_le_bytes()); // biSize
    ico_bytes.extend_from_slice(&16i32.to_le_bytes()); // biWidth
    ico_bytes.extend_from_slice(&32i32.to_le_bytes()); // biHeight (x2)
    ico_bytes.extend_from_slice(&1u16.to_le_bytes()); // planes
    ico_bytes.extend_from_slice(&32u16.to_le_bytes()); // bpp
    ico_bytes.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    ico_bytes.extend_from_slice(&(pixels as u32).to_le_bytes());
    for _ in 0..4 {
        ico_bytes.extend_from_slice(&0u32.to_le_bytes()); // ppm x/y, clr used/important
    }
    ico_bytes.resize(22 + dib, 0);
    std::fs::write(&ico, &ico_bytes).unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_xiom"))
        .arg(&src)
        .arg("-o")
        .arg(&out_exe)
        .arg("--icon")
        .arg(&ico)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn xiom: {e}"));
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() && stderr.contains("no resource compiler succeeded") {
        let _ = std::fs::remove_dir_all(&base);
        return; // no llvm-rc/rc on this machine -- manual evidence in docs
    }
    assert!(
        out.status.success(),
        "icon compile must succeed (m214); exit={:?}\nstderr:\n{stderr}",
        out.status.code()
    );
    assert!(out_exe.is_file(), "icon exe missing (m214)");

    // Error path: a missing icon file must be a hard, explicit error.
    let bad = std::process::Command::new(env!("CARGO_BIN_EXE_xiom"))
        .arg(&src)
        .arg("-o")
        .arg(base.join("bad.exe"))
        .arg("--icon")
        .arg(base.join("nope.ico"))
        .output()
        .unwrap();
    let bad_err = String::from_utf8_lossy(&bad.stderr);
    assert!(
        !bad.status.success() && bad_err.contains("icon file not found"),
        "missing icon must be a hard error (m214); exit={:?}\nstderr:\n{bad_err}",
        bad.status.code()
    );
    let _ = std::fs::remove_dir_all(&base);
}

// m212 (C-PULSE-02): `xiom build` in a project whose `[dependencies]` use a
// path dependency must discover the dependency's modules. Pre-fix the
// dependency was never on the catalog path: the build reported only the
// project module and failed with T001 undefined variable for the
// dependency's functions (Pulse hard-coded a `source-roots` workaround).
#[test]
fn m212_project_path_dependency_is_discovered() {
    let base = std::env::temp_dir().join(format!("xiom_m212_it_{}", std::process::id()));
    let app = base.join("app");
    let core = base.join("core");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::create_dir_all(&core).unwrap();
    std::fs::write(
        app.join("xiom.toml"),
        "[project]\nname = \"app\"\nroot = \"src\"\n\n[dependencies]\ncore = { path = \"../core\" }\n",
    )
    .unwrap();
    std::fs::write(
        app.join("src/main.xi"),
        "module app\n\nuse app.core;\n\nvar g = make(2, 3);\n\nfn main() -> Int {\n  return 0;\n}\n",
    )
    .unwrap();
    std::fs::write(
        core.join("core.xi"),
        "module app.core\n\npub fn make(a: Int, b: Int) -> Int {\n  return a + b;\n}\n",
    )
    .unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_xiom"))
        .arg("build")
        .current_dir(&app)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn xiom: {e}"));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "project build must discover the path dependency (m212); exit={:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        out.status.code()
    );
    assert!(
        !stderr.contains("T001"),
        "the dependency call must resolve (m212); stderr:\n{stderr}"
    );
    assert!(
        stdout.contains("make"),
        "the dependency function must be emitted (m212); stdout head:\n{}",
        &stdout[..stdout.len().min(400)]
    );
    let _ = std::fs::remove_dir_all(&base);
}

// m207 (found during the C-PULSE-07 repro): the `build` subcommand token must
// not be read back as a source path -- `xiom build` from a project root died
// with `cannot read 'build' (os error 2)` before the project-graph branch.
// A real file named `build` stays a source (exists() guard, mirroring the
// wasm/arm/riscv positional sugar).
#[test]
fn m207_build_subcommand_token_not_a_source() {
    let bare = resolve_source_files(&["xiom".to_string(), "build".to_string()]);
    assert!(
        bare.is_empty(),
        "the bare `build` token must not become a source path; got {bare:?}"
    );

    let with_file = resolve_source_files(&[
        "xiom".to_string(),
        "build".to_string(),
        "src.xi".to_string(),
    ]);
    assert_eq!(
        with_file,
        vec!["src.xi".to_string()],
        "`xiom build src.xi` must keep only the real source"
    );

    // m214: `--icon <path>` takes a value too -- the .ico must never be
    // treated as a source file (it read as UTF-8 and failed).
    let with_icon = resolve_source_files(&[
        "xiom".to_string(),
        "src.xi".to_string(),
        "--icon".to_string(),
        "app.ico".to_string(),
    ]);
    assert_eq!(
        with_icon,
        vec!["src.xi".to_string()],
        "`--icon app.ico` must skip the icon value"
    );
    // The exists() guard keeps a REAL file named `build` compilable; that
    // behavior is locked by target_named_source_files_are_kept_but_bare_sugar
    // _is_skipped above (torch: no second CWD mutation here -- the process
    // CWD is shared across the parallel test threads).
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
