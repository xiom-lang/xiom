// C22 (playground relay): `xiom run <script>` must resolve sibling modules
// AND packages/ layouts relative to the SCRIPT'S directory exactly like
// `xiom --check`.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `xiom run` compiles a %TEMP%/xiom_run copy, so without the source-dir hint
// the checker's catalog never saw the script's directory: sibling modules
// and package modules failed with undefined-variable errors while `--check`
// passed. Lock: tests/regression/c22_run_sibling_module/ (main.xi imports
// c22_sibling_lib.xi and packages/c22-pkg/src/hello.xi -> c22pkg.hello).

use std::path::{Path, PathBuf};
use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("c22_run_sibling_module")
        .join("main.xi")
}

#[test]
fn c22_run_resolves_sibling_module() {
    let out = Command::new(xiom_bin())
        .arg("run")
        .arg(fixture())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let stderr = String::from_utf8_lossy(&out.stderr);
    // The defect was a compile-stage failure: the sibling module never
    // reached the catalog ("undefined variable 'c22_sibling_lib'"). The
    // success signal for resolution+codegen is the `compiled:` line.
    assert!(
        out.status.success() || stderr.contains("compiled:"),
        "xiom run must resolve sibling modules and reach codegen (C22); exit={:?}\nstderr:\n{}",
        out.status.code(),
        stderr
    );
    // This dev box's Defender blocks freshly built %TEMP%/xiom_run exes
    // (os error 225), so the execution step may be environment-blocked
    // locally; CI (Linux) takes the full success path.
    if !out.status.success() {
        assert!(
            stderr.contains("os error 225") || stderr.contains("contains a virus"),
            "unexpected xiom run failure (C22); exit={:?}\nstderr:\n{}",
            out.status.code(),
            stderr
        );
    }
}

#[test]
fn c22_check_still_resolves_sibling_module() {
    let out = Command::new(xiom_bin())
        .arg("--check")
        .arg(fixture())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(
        out.status.success(),
        "xiom --check must keep resolving sibling modules (C22 parity); exit={:?}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
}

// C25 (playground relay): `xiom run` executes a WARM script-cache hit via
// Command::output(), whose default stdin is NULL -- cached reruns printed
// `got: []` while the cold run read the piped line. The cached path must
// inherit stdin. `--no-cache` must bypass the cache entirely: the flags were
// filtered out of the `effective` list before they were read, so the check
// was dead and a warm cache could not be bypassed.
fn c25_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("c25_run_cached_stdin")
        .join("main.xi")
}

fn run_c25(args: &[&str], home: &Path) -> std::process::Output {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = Command::new(xiom_bin())
        .arg("run")
        .args(args)
        .arg(c25_fixture())
        .env("HOME", home)
        .env("USERPROFILE", home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    child
        .stdin
        .as_mut()
        .expect("child stdin")
        .write_all(b"Ada\n")
        .expect("write child stdin");
    child.wait_with_output().expect("wait for xiom run")
}

#[test]
fn c25_warm_cache_inherits_stdin_and_no_cache_bypasses() {
    let home = std::env::temp_dir().join(format!("xiom_c25_home_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&home);
    let cold = run_c25(&[], &home);
    let cold_out = String::from_utf8_lossy(&cold.stdout);
    let cold_err = String::from_utf8_lossy(&cold.stderr);
    // This dev box's Defender can block freshly built %TEMP%/xiom_run exes
    // (os error 225); CI (Linux) takes the full path.
    if !cold.status.success()
        && (cold_err.contains("os error 225") || cold_err.contains("contains a virus"))
    {
        eprintln!("SKIP c25: environment blocks script execution (Defender os error 225)");
        let _ = std::fs::remove_dir_all(&home);
        return;
    }
    assert!(
        cold.status.success() && cold_out.contains("got: [Ada]"),
        "C25 cold run must read piped stdin; exit={:?}\nstdout: {cold_out}\nstderr: {cold_err}",
        cold.status.code()
    );
    let warm = run_c25(&[], &home);
    let warm_out = String::from_utf8_lossy(&warm.stdout);
    assert!(
        warm.status.success() && warm_out.contains("got: [Ada]"),
        "C25 warm-cache run must inherit stdin (defect printed `got: []`); exit={:?}\nstdout: {warm_out}\nstderr: {}",
        warm.status.code(),
        String::from_utf8_lossy(&warm.stderr)
    );
    let nocache = run_c25(&["--no-cache"], &home);
    let nocache_out = String::from_utf8_lossy(&nocache.stdout);
    assert!(
        nocache.status.success() && nocache_out.contains("got: [Ada]"),
        "C25 --no-cache run must bypass the cache and still read stdin; exit={:?}\nstdout: {nocache_out}\nstderr: {}",
        nocache.status.code(),
        String::from_utf8_lossy(&nocache.stderr)
    );
    let _ = std::fs::remove_dir_all(&home);
}

// UX (user-flagged): bare `xiom file.xi` prints IR with no hint that
// `xiom run file.xi` executes it. The hint is stderr-only so the IR stdout
// stays byte-identical to `--emit-ir` (the selfhost T3 gates compare stdout
// bytes); the explicit flag stays quiet.
fn ux_bare_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("ux_bare_ir_hint")
        .join("main.xi")
}

#[test]
fn ux_bare_compile_hints_run_on_stderr() {
    let bare = Command::new(xiom_bin())
        .arg(ux_bare_fixture())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(
        bare.status.success(),
        "bare compile must succeed; exit={:?}\nstderr:\n{}",
        bare.status.code(),
        String::from_utf8_lossy(&bare.stderr)
    );
    let bare_err = String::from_utf8_lossy(&bare.stderr);
    assert!(
        bare.stdout.starts_with(b"; XIOM v"),
        "the IR must stay on stdout; stdout head:\n{}",
        String::from_utf8_lossy(&bare.stdout[..bare.stdout.len().min(120)])
    );
    assert!(
        bare_err.contains("xiom run"),
        "the implicit IR path must hint at `xiom run` on stderr; stderr:\n{bare_err}"
    );

    let explicit = Command::new(xiom_bin())
        .arg("--emit-ir")
        .arg(ux_bare_fixture())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(explicit.status.success(), "--emit-ir must succeed");
    assert_eq!(
        bare.stdout, explicit.stdout,
        "the hint must not change the IR stdout bytes (byte-identical requirement)"
    );
    let explicit_err = String::from_utf8_lossy(&explicit.stderr);
    assert!(
        !explicit_err.contains("xiom run"),
        "--emit-ir is explicit; no hint expected; stderr:\n{explicit_err}"
    );
}

// m208 (perf regression, 2026-10-07): plain `xiom run --jit <same source>`
// must serve the script cache on hit again. The 2026-10-05 gate
// (`!use_jit || jit_reuse`) made every benchmark JIT sample re-run the full
// compile+JIT pipeline: v0.62.3's scripting lane measured 35-45ms, the
// gated lane ~1900ms+. A hit prints neither `compiled:` nor `JIT exit code`.
// `--no-cache` remains the explicit cold switch (covered by the same
// fixture's first run + the cold assertions in the perf docs).
#[test]
fn m208_jit_serves_script_cache_on_hit() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m208_jit_cache_hit")
        .join("main.xi");

    // Populate the script cache through the plain run path.
    let warm = Command::new(xiom_bin())
        .arg("run")
        .arg(&fixture)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let warm_err = String::from_utf8_lossy(&warm.stderr).into_owned();
    if !warm.status.success() && (warm_err.contains("os error 225") || warm_err.contains("contains a virus")) {
        return; // Defender blocks freshly built %TEMP% exes on this dev box
    }
    assert!(
        warm.status.success() || warm_err.contains("compiled:"),
        "warm-up run must reach codegen; exit={:?}\nstderr:\n{warm_err}",
        warm.status.code()
    );

    // Second run with --jit: same source content, must hit the cache.
    let jit = Command::new(xiom_bin())
        .arg("run")
        .arg("--jit")
        .arg(&fixture)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let jit_err = String::from_utf8_lossy(&jit.stderr).into_owned();
    if jit_err.contains("os error 225") || jit_err.contains("contains a virus") {
        return; // cached-exe launch blocked locally; CI (Linux) covers it
    }
    assert!(
        !jit_err.contains("JIT exit code") && !jit_err.contains("compiled:"),
        "`xiom run --jit` on a warm cache must replay the cached build, not recompile (m208); stderr:\n{jit_err}"
    );
    let jit_out = String::from_utf8_lossy(&jit.stdout);
    assert!(
        jit_out.contains("m208-jit-cache"),
        "the cached replay must produce the program output (m208); stdout:\n{jit_out}"
    );
}

// m204 (C-PULSE-07, Pulse relay): a module-level `var` initialized by a
// cross-module constructor call must emit the callee body. Pre-fix the
// checker's reachability filter ignored TopDecl::Const initializers, so the
// catalog fn referenced only from the global init was pruned and codegen
// emitted `call i64 @rate_keyed_new` with no definition (clang "use of
// undefined value '@rate_keyed_new'"; Pulse's xiom.rate limiter).
fn m204_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m204_module_ginit_cross_module")
        .join("main.xi")
}

#[test]
fn m204_module_ginit_emits_cross_module_callee() {
    let out = Command::new(xiom_bin())
        .arg("run")
        .arg("--no-cache")
        .arg(m204_fixture())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let stderr = String::from_utf8_lossy(&out.stderr);
    // The defect was a compile-stage clang failure: the callee body was
    // pruned by the reachability filter. It must never resurface.
    assert!(
        !stderr.contains("undefined value '@rate_keyed_new'"),
        "module-scope init must emit the cross-module callee (m204); exit={:?}\nstderr:\n{stderr}",
        out.status.code()
    );
    assert!(
        out.status.success() || stderr.contains("compiled:"),
        "m204 module-scope init must reach codegen; exit={:?}\nstderr:\n{stderr}",
        out.status.code()
    );
    // Defender can block freshly built %TEMP%/xiom_run exes on this dev box
    // (os error 225); CI (Linux) takes the full run path.
    if out.status.success() {
        assert_eq!(out.status.code(), Some(0), "m204 fixture must exit 0");
    } else {
        assert!(
            stderr.contains("os error 225") || stderr.contains("contains a virus"),
            "unexpected xiom run failure (m204); exit={:?}\nstderr:\n{stderr}",
            out.status.code()
        );
    }
}

// m228 (B-08, bindings-lane): `xiom --run` must mirror the program's exit
// code. The defect printed `exit code: 5` on stderr but the xiom process
// exited 0, so suites that trusted $LASTEXITCODE saw success.
#[test]
fn m228_run_propagates_program_exit_code() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m228_run_exit_code")
        .join("main.xi");
    let out_exe = std::env::temp_dir().join(format!("m228_{}.exe", std::process::id()));
    let out = Command::new(xiom_bin())
        .arg("--run")
        .arg("-o")
        .arg(&out_exe)
        .arg(&fixture)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    let stderr = String::from_utf8_lossy(&out.stderr);
    let _ = std::fs::remove_file(&out_exe);
    assert_eq!(
        out.status.code(),
        Some(5),
        "xiom --run must exit with the program's code (B-08); stderr:\n{stderr}"
    );
}
