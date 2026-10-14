// XIOM -- xiom-pkg local-fallback install lock
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// REL-20261010-1549-packages-7: a registry-unavailable `xiom pkg install`
// falls back to the workspace `packages/` directory. The old version reader
// kept the closing quote of `version: "0.1.1";` as part of the version, so
// the destination directory was invalid and the copy reported "(N files)"
// while landing nothing (silent no-op, exit 0).
//
// This drives the REAL xiom-pkg binary end to end against the fixture
// workspace with a scratch XIOM_HOME (registry pointed at a dead localhost
// endpoint to force the local fallback). It lives outside the codegen e2e
// suite because `xiom pkg` dispatches to a sibling xiom-pkg binary which
// only this crate's test build guarantees.

use std::path::Path;
use std::process::Command;

fn copy_dir(src: &Path, dest: &Path) {
    std::fs::create_dir_all(dest).unwrap();
    for entry in std::fs::read_dir(src).unwrap().flatten() {
        let path = entry.path();
        let target = dest.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}

#[test]
fn m310_local_fallback_installs_the_package() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let fixture = root.join("tests/regression/m310_pkg_local_fallback/ws");
    assert!(
        fixture.join("packages/xiom-demo/package.xi").exists(),
        "fixture missing: {}",
        fixture.display()
    );

    let scratch = std::env::temp_dir().join(format!("xiom_m310_pkg_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let ws = scratch.join("ws");
    let home = scratch.join("home");
    copy_dir(&fixture, &ws);

    let out = Command::new(env!("CARGO_BIN_EXE_xiom-pkg"))
        .arg("install")
        .arg("xiom.demo")
        .env("XIOM_HOME", &home)
        // Force the local fallback: a dead localhost endpoint keeps the
        // registry unavailable; http://localhost needs the dev opt-in.
        .env("XIOM_REGISTRY", "http://localhost:9")
        .env("XIOM_PKG_ALLOW_HTTP", "1")
        .current_dir(&ws)
        .output()
        .expect("spawn xiom-pkg");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "local fallback install must succeed; stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let installed: Vec<String> = std::fs::read_dir(home.join("packages"))
        .expect("the packages store must exist after the install")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert!(
        installed.iter().any(|n| n == "xiom.demo-0.1.1"),
        "the destination dir name must carry exactly 0.1.1 (the old reader \
         produced `0.1.1\"`, an invalid dir); got {installed:?}"
    );
    let dest = home.join("packages").join("xiom.demo-0.1.1");
    assert!(dest.join("package.xi").exists(), "package.xi must land; stdout:\n{stdout}");
    assert!(dest.join("demo.xi").exists(), "demo.xi must land; stdout:\n{stdout}");

    let _ = std::fs::remove_dir_all(&scratch);
}
