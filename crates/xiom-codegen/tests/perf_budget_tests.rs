// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stage 6: compiler performance budgets (audit Phase 4: "benchmark suite
//! with CI regression budgets").
//!
//! Metrics are DETERMINISTIC: emitted LLVM IR bytes for a fixed corpus,
//! measured through the real driver pipeline (checker included). Wall time is
//! asserted only against a generous ceiling so a debug-profile CI runner
//! passes; the byte budgets carry the regression signal. Budgets sit ~10%
//! above the round-75 measurements:
//!
//! | source                        | bytes   | budget   | debug time |
//! |-------------------------------|---------|----------|------------|
//! | examples/benchmark/main.xi    | 5,808,645 | 6,300,000 | ~10 s    |
//! | selfhost/xiomc_v092.xi        |   155,936 |   175,000 | ~2 s    |
//! | tests/ecosystem/test_json.xi  |   164,787 |   185,000 | ~1 s    |
//!
//! The bench baseline moved from 5,687,052 to 5,764,620 with R39 (same-leaf
//! TYPE collision qualification: `benchmark.borrow.Metrics` /
//! `benchmark.derive.Metrics` and the `Record` pair now emit distinct
//! module-qualified `%struct.` names instead of one collapsed bare key).
//!
//! The determinism test (same source compiled twice -> byte-identical IR) is
//! the canary for HashMap-order-dependent emission, the defect class fixed
//! repeatedly in rounds 55-75.

use std::path::{Path, PathBuf};
use std::process::Command;

fn project_root() -> &'static Path {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    })
    .as_path()
}

fn xiom_path() -> String {
    let exe_name = if cfg!(target_os = "windows") { "xiom.exe" } else { "xiom" };
    let mut path = project_root().join("target").join("debug").join(exe_name);
    if !path.exists() {
        path = project_root().join("target").join("release").join(exe_name);
    }
    path.to_str().unwrap().to_string()
}

fn emit_ir(source: &str) -> (Vec<u8>, u128) {
    let started = std::time::Instant::now();
    let output = Command::new(xiom_path())
        .args(["--emit-ir", source])
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {}: {e}", xiom_path()));
    let ms = started.elapsed().as_millis();
    assert!(
        output.status.success(),
        "{source}: driver failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    (output.stdout, ms)
}

#[test]
fn perf_budget_emitted_ir_bytes() {
    // (source, byte budget) -- see the module docs for the baseline table.
    let cases: &[(&str, u64)] = &[
        ("examples\\benchmark\\main.xi", 6_300_000),
        ("selfhost\\xiomc_v092.xi", 175_000),
        ("tests\\ecosystem\\test_json.xi", 185_000),
    ];
    for (source, budget) in cases {
        let (ir, ms) = emit_ir(source);
        let bytes = ir.len() as u64;
        println!("[perf] {source}: {bytes} bytes IR in {ms} ms (budget {budget})");
        assert!(
            bytes <= *budget,
            "{source}: emitted IR regressed: {bytes} bytes > budget {budget}"
        );
        assert!(
            ms < 180_000,
            "{source}: compile took {ms} ms, over the 180 s ceiling"
        );
    }
}

#[test]
fn perf_determinism_ir_is_byte_identical() {
    // R25: the bench graph (30 modules, same-leaf fns in several modules) was
    // the live reproducer: `ptrtoint @benchmark.math.is_even` vs
    // `@benchmark.comptime.is_even`, swapped mono emission order, and
    // unsorted concrete-Option builtins all drifted the IR. Both graphs must
    // now emit byte-identically.
    for source in ["selfhost\\xiomc_v092.xi", "examples\\benchmark\\main.xi"] {
        let (first, _) = emit_ir(source);
        let (second, _) = emit_ir(source);
        assert_eq!(
            first.len(),
            second.len(),
            "IR length differs across compiles of {source} (nondeterministic emission)"
        );
        assert!(
            first == second,
            "IR is NOT byte-identical across compiles of {source} -- \
             HashMap-order-dependent emission regression"
        );
    }
}
