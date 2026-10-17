// m351 (REL-20261016-2031-bindings-3, B-33): in-repo ros2-abi probe ladder +
// direct extern aggregate lock. Pre-fix (v0.64.6) the 16B by-value call and
// the 16B return either hard-AV'd or were swallowed into a silent exit 0.
// Post-fix all ladder values are EXACT and no fault diagnostic is printed.
//
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::{Path, PathBuf};
use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn bundle_dir() -> PathBuf {
    project_root()
        .join("docs")
        .join("repro")
        .join("bindings-pilot")
        .join("ros2-abi")
}

/// Build the probe shared library from the in-repo C source (binaries are
/// never committed). Returns the DLL/SO path.
fn build_probe_lib(dir: &Path) -> PathBuf {
    let src = bundle_dir().join("probe.c");
    assert!(src.is_file(), "probe.c must exist at {}", src.display());
    let (out, extra): (PathBuf, Vec<&str>) = if cfg!(windows) {
        (dir.join("probe.dll"), vec!["-shared", "-O1"])
    } else {
        (dir.join("probe.so"), vec!["-shared", "-fPIC", "-O1"])
    };
    let out_run = Command::new("clang")
        .args(&extra)
        .arg("-o")
        .arg(&out)
        .arg(&src)
        .output()
        .unwrap_or_else(|e| {
            panic!("failed to spawn clang (needed to build the probe library): {e}")
        });
    assert!(
        out_run.status.success(),
        "probe library build failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out_run.stdout),
        String::from_utf8_lossy(&out_run.stderr)
    );
    out
}

struct RunResult {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Compile one bundle fixture to a unique executable and run it.
fn compile_and_run(
    dir: &Path,
    fixture: &Path,
    extra: &[&str],
    envs: &[(&str, String)],
) -> RunResult {
    let stem = fixture
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("probe");
    let exe = dir.join(format!(
        "m351_{}{}",
        stem,
        if cfg!(windows) { ".exe" } else { "" }
    ));
    let _ = std::fs::remove_file(&exe);
    let out = Command::new(xiom_bin())
        .arg(fixture)
        .arg("-o")
        .arg(&exe)
        .args(extra)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(
        out.status.success(),
        "fixture {} must compile; exit={:?}\nstdout:\n{}\nstderr:\n{}",
        fixture.display(),
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let mut cmd = Command::new(&exe);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let run = cmd
        .output()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", exe.display()));
    RunResult {
        code: run.status.code(),
        stdout: String::from_utf8_lossy(&run.stdout).to_string(),
        stderr: String::from_utf8_lossy(&run.stderr).to_string(),
    }
}

fn assert_exact(result: &RunResult, markers: &[&str], label: &str) {
    assert_eq!(
        result.code,
        Some(0),
        "{label} must exit 0\nstdout:\n{}\nstderr:\n{}",
        result.stdout,
        result.stderr
    );
    for marker in markers {
        assert!(
            result.stdout.contains(marker),
            "{label}: missing exact marker '{marker}'\nstdout:\n{}\nstderr:\n{}",
            result.stdout,
            result.stderr
        );
    }
    assert!(
        !result.stderr.contains("[fault]") && !result.stderr.contains("[trampoline] fault"),
        "{label}: no fault may surface on the fixed ladder\nstderr:\n{}",
        result.stderr
    );
}

#[test]
fn e2e_m351_ros2_abi_ladder() {
    let dir = std::env::temp_dir().join(format!("xiom_m351_ladder_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let probe = build_probe_lib(&dir);
    let envs = vec![("XIOM_M351_PROBE_DLL", probe.to_string_lossy().to_string())];
    let bundle = bundle_dir();

    let ladder = compile_and_run(&dir, &bundle.join("probe_ladder.xi"), &[], &envs);
    assert_exact(
        &ladder,
        &[
            "c1 returned 165",
            "c2 returned 17036",
            "c3 fields 507,509",
            "c4 fields 1001,1005",
            "c5 returned 165",
            "done",
        ],
        "probe_ladder (c1..c5)",
    );

    let ladder2 = compile_and_run(&dir, &bundle.join("probe_ladder2.xi"), &[], &envs);
    assert_exact(
        &ladder2,
        &[
            "c3 fields 507,509",
            "c4 fields 1001,1005",
            "c5 returned 165",
            "done",
        ],
        "probe_ladder2 (c3..c5)",
    );

    let plain = compile_and_run(&dir, &bundle.join("probe_c2_plain.xi"), &[], &envs);
    assert_exact(
        &plain,
        &["c2 returned 17036", "done"],
        "probe_c2_plain (no unsafe control)",
    );

    // Pointer-pretend workaround + exact rcl multi-arg shapes: ABI-correct on
    // Windows x64 by construction. System V passes these structs differently
    // (the channel is refused there by xiom.ros2), so this control is
    // Windows-only.
    #[cfg(windows)]
    {
        // Small-aggregate coercion: 8B records travel as ONE i64 in C;
        // LLVM's naked %struct lowering splits the fields into separate
        // registers (and Microsoft only defines this class for its x64
        // ABI). Windows-only until the SysV/AArch64 classification is
        // validated separately.
        let small = compile_and_run(&dir, &bundle.join("probe_small.xi"), &[], &envs);
        assert_exact(
            &small,
            &["c6 returned 17036", "c7 fields 507,509", "done"],
            "probe_small (8B by value/return)",
        );

        let pretend = compile_and_run(&dir, &bundle.join("probe_pretend.xi"), &[], &envs);
        assert_exact(
            &pretend,
            &["p1 returned 165", "p2 returned 17036", "done"],
            "probe_pretend",
        );
        let multi = compile_and_run(&dir, &bundle.join("probe_multi.xi"), &[], &envs);
        assert_exact(
            &multi,
            &[
                "m1 returned 8186",
                "m2 returned 2166",
                "m3 returned 969",
                "done",
            ],
            "probe_multi",
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn e2e_m351_extern_aggregate() {
    let dir = std::env::temp_dir().join(format!("xiom_m351_extern_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let bundle = bundle_dir();
    let c_source = bundle.join("probe.c").to_string_lossy().to_string();

    let direct = compile_and_run(
        &dir,
        &bundle.join("probe_extern_direct.xi"),
        &["--c-source", &c_source],
        &[],
    );
    assert_exact(
        &direct,
        &["direct extern returned 17036"],
        "direct extern 16B by value",
    );

    let unsafe_direct = compile_and_run(
        &dir,
        &bundle.join("probe_extern_direct_unsafe.xi"),
        &["--c-source", &c_source],
        &[],
    );
    assert_exact(
        &unsafe_direct,
        &["direct extern returned 17036"],
        "direct extern 16B by value (confined)",
    );

    let make2 = compile_and_run(
        &dir,
        &bundle.join("probe_extern_make2.xi"),
        &["--c-source", &c_source],
        &[],
    );
    assert_exact(
        &make2,
        &["direct extern fields 507,509"],
        "direct extern 16B return",
    );

    let _ = std::fs::remove_dir_all(&dir);
}
