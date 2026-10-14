// m313 (REL-20261013-1445 compiler half): alloc-guard-spin runtime lock.
// `ffi.alloc` + write + read + `ffi.free` in ONE confined block must complete
// quickly -- pre-fix the `free` call site lowered to bare libc `@free` while
// the alloc sat in the guard arena, and the Windows heap manager corrupted /
// spun (VEH retry loop at 100% CPU, no output; 1548-bindings-5). The probe is
// WATCHDOG-bounded (15 s): a regression kills the process and fails the test
// instead of hanging the suite. Runs only when the selected stdlib root
// carries the wave-124 `xiom_guard_free` runtime (skips loudly otherwise, so
// CI at the older pin stays honest); the compile-only IR twin
// (e2e_m313_ffi_free_guard_ir) locks the call-site shape on every pin.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::Command;

/// The selected stdlib root must carry the arena-aware free runtime (wave
/// 124): without it the wrapper body cannot pair with the guard arena.
fn runtime_has_guard_free(root: &std::path::Path) -> bool {
    std::fs::read_to_string(root.join("runtime").join("xiom_runtime.c"))
        .map(|src| src.contains("xiom_guard_free"))
        .unwrap_or(false)
}

/// First stdlib root (XIOM_STDLIB -> exe ancestors -> ./stdlib, mirroring
/// xiom-graph::paths::stdlib_candidates) that has a module tree AND the
/// xiom_guard_free runtime symbol.
fn guard_stdlib_root() -> Option<std::path::PathBuf> {
    let mut cands: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(s) = std::env::var("XIOM_STDLIB") {
        if !s.trim().is_empty() {
            cands.push(std::path::PathBuf::from(s.trim()));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..=8 {
            let Some(d) = cur else { break };
            cands.push(d.join("stdlib"));
            cands.push(d.join("lib"));
            cands.push(d.join("share").join("xiom"));
            cur = d.parent().map(|p| p.to_path_buf());
        }
    }
    cands.push(std::path::PathBuf::from("stdlib"));
    cands
        .into_iter()
        .find(|c| c.join("xiom").is_dir() && runtime_has_guard_free(c))
}

#[test]
fn m313_ffi_free_guard_watchdog() {
    let Some(root) = guard_stdlib_root() else {
        eprintln!(
            "SKIP: no stdlib root with the xiom_guard_free runtime found \
             (looked at XIOM_STDLIB, exe ancestors, ./stdlib) -- runtime half \
             not yet pinned"
        );
        return;
    };
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m313_alloc_guard_free");
    let dir = std::env::temp_dir().join(format!("xiom_m313_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let main_xi = dir.join("main.xi");
    std::fs::copy(fixture.join("main.xi"), &main_xi).expect("fixture main.xi must exist");

    let exe_name = if cfg!(windows) { "m313_probe.exe" } else { "m313_probe" };
    let out_bin = dir.join(exe_name);
    let compile = Command::new(env!("CARGO_BIN_EXE_xiom"))
        .arg(&main_xi)
        .arg("--force")
        .arg("--no-cache")
        .arg("-o")
        .arg(&out_bin)
        .env("XIOM_STDLIB", &root)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", env!("CARGO_BIN_EXE_xiom")));
    assert!(
        compile.status.success(),
        "m313 fixture must compile; exit={:?}\nstdout:\n{}\nstderr:\n{}",
        compile.status.code(),
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    // Watchdog run: file redirection (no pipe) so a killed child cannot
    // deadlock the reader; bounded at 15 s.
    let out_log = dir.join("run.out");
    let err_log = dir.join("run.err");
    let stdout_file = std::fs::File::create(&out_log).unwrap();
    let stderr_file = std::fs::File::create(&err_log).unwrap();
    let mut child = Command::new(&out_bin)
        .stdout(stdout_file)
        .stderr(stderr_file)
        .spawn()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", out_bin.display()));
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(15);
    let mut killed = false;
    let status = loop {
        match child.try_wait().unwrap() {
            Some(status) => break status,
            None => {
                if start.elapsed() > timeout {
                    killed = true;
                    let _ = child.kill();
                    // Reap; a killed child's status is its terminating signal.
                    break child.wait().unwrap();
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    };
    let stdout = std::fs::read_to_string(&out_log).unwrap_or_default();
    let stderr = std::fs::read_to_string(&err_log).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        !killed,
        "m313: the confined alloc/free probe still ran after 15 s -- \
         alloc-guard-spin regressed (bare @free on a guard-arena pointer); \
         stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        status.code(),
        Some(0),
        "m313 probe must exit 0\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("m=9"),
        "m313: the main-heap alloc/free roundtrip must read back 9; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("v=7"),
        "m313: the confined alloc/free roundtrip must read back 7; stdout:\n{stdout}"
    );
}
