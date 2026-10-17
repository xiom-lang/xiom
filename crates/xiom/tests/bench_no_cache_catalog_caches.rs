// REL-20261011-2100-benchmark (m252 follow-up, owner option (a)):
// `--no-cache` must disable the PERSISTENT catalog index cache and the
// catalog-body check cache -- READ AND WRITE -- not just the script/binary
// caches. The benchmark's scored cold lanes (`xiom run --jit --no-cache`,
// `xiom run --no-cache`) otherwise replay clean catalog bodies from a warm
// cache, so sample 2 on is no longer cold.
//
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::{Path, PathBuf};
use std::process::Command;

fn xiom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xiom")
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("regression")
        .join("m252_body_check_cache")
}

fn run(home: &Path, pre: &[&str], main: &Path) -> std::process::Output {
    Command::new(xiom_bin())
        .args(pre)
        .arg("--check")
        .arg(main)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()))
}

fn cache_file(home: &Path, name: &str) -> PathBuf {
    home.join(".xiom").join(name)
}

#[test]
fn no_cache_bypasses_persistent_catalog_caches() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let root = std::env::temp_dir().join(format!(
        "xiom_nocache_{}_{nanos:x}",
        std::process::id()
    ));
    let proj = root.join("proj");
    let home_control = root.join("home_control");
    let home_nocache = root.join("home_nocache");
    std::fs::create_dir_all(&proj).expect("create no-cache project dir");
    std::fs::create_dir_all(&home_control).expect("create control home");
    std::fs::create_dir_all(&home_nocache).expect("create no-cache home");
    for name in ["main.xi", "m252_helper.xi", "m252_math.xi"] {
        std::fs::copy(fixture_dir().join(name), proj.join(name))
            .unwrap_or_else(|e| panic!("copy {name}: {e}"));
    }
    let main = proj.join("main.xi");

    // Control: the default path writes the positive catalog-body cache.
    let c1 = run(&home_control, &[], &main);
    assert_eq!(
        c1.status.code(),
        Some(0),
        "control --check must pass:\n{}",
        String::from_utf8_lossy(&c1.stderr)
    );
    assert!(
        cache_file(&home_control, "checkbodies.txt").exists(),
        "control --check must persist the catalog-body check cache (fixture proof)"
    );

    // Cold switch: repeated runs must neither write nor replay either cache.
    for pass in 0..2 {
        let n = run(&home_nocache, &["--no-cache"], &main);
        assert_eq!(
            n.status.code(),
            Some(0),
            "--no-cache check pass {pass} must pass:\n{}",
            String::from_utf8_lossy(&n.stderr)
        );
    }
    assert!(
        !cache_file(&home_nocache, "checkbodies.txt").exists(),
        "--no-cache must not WRITE the catalog-body check cache"
    );
    assert!(
        !cache_file(&home_nocache, "catidx.txt").exists(),
        "--no-cache must not WRITE the catalog index cache"
    );

    // The benchmark's AOT lane (`--emit-ir`) takes the same switch.
    let ir = Command::new(xiom_bin())
        .arg("--emit-ir")
        .arg("--no-cache")
        .arg(&main)
        .env("HOME", &home_nocache)
        .env("USERPROFILE", &home_nocache)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn '{}': {e}", xiom_bin()));
    assert!(
        ir.status.success(),
        "--no-cache --emit-ir must pass:\n{}",
        String::from_utf8_lossy(&ir.stderr)
    );
    assert!(!cache_file(&home_nocache, "checkbodies.txt").exists());
    assert!(!cache_file(&home_nocache, "catidx.txt").exists());

    let _ = std::fs::remove_dir_all(&root);
}
