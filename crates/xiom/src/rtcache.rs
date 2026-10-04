// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stage 6 (STAGE6_PERF_PLAN item 1): persistent object cache for the C
//! runtime sources.
//!
//! Every `xiom` compile used to recompile all `stdlib/runtime/*.c` in the
//! SAME clang invocation as the program IR -- measured 5.5-6.3s of the ~8s
//! clang+link stage on the dev box, even for a hello-world. The runtime is
//! immutable between edits, so compile it to objects once per
//! (clang identity, compile flags, source contents) tuple and link the
//! cached objects on subsequent compiles.
//!
//! Cache layout: `<cache_root>/<key>/<NN>_<stem>.o`. The key is a 64-bit
//! FNV-1a of the clang path, `clang --version`, every compile flag, and the
//! content hash of every runtime source. Any failure falls back to the
//! historical single-invocation C compile (correctness first).

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::jit;

/// 64-bit FNV-1a. Deterministic across Rust releases (unlike DefaultHasher),
/// which keeps cache entries valid across toolchain upgrades.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Default cache root: `$HOME/.xiom/rtobj` (same family as the jit cache,
/// with the same temp fallback), created on demand.
pub fn default_cache_root() -> PathBuf {
    jit::jit_cache_dir().join("rtobj")
}

fn clang_version(clang: &Path) -> String {
    match Command::new(clang).arg("--version").output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string(),
        Err(_) => String::new(),
    }
}

fn object_name(index: usize, source: &str) -> String {
    let stem = Path::new(source)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("rt");
    format!("{index:02}_{stem}.o")
}

/// Return (building when needed) the cached runtime objects for `rt_files`
/// compiled with `compile_args`.
///
/// The returned paths are stable and can be appended to a link command in
/// the same order as `rt_files`. Any failure (unwritable cache, clang error)
/// returns `Err` so the caller can fall back to compiling the C sources in
/// the main invocation.
pub fn runtime_objects(
    cache_root: &Path,
    clang: &Path,
    rt_files: &[String],
    compile_args: &[String],
) -> Result<Vec<PathBuf>, String> {
    // ---- cache key -----------------------------------------------------
    let mut key_input = String::new();
    key_input.push_str(&clang.to_string_lossy());
    key_input.push('\n');
    key_input.push_str(&clang_version(clang));
    key_input.push('\n');
    for a in compile_args {
        key_input.push_str(a);
        key_input.push('\n');
    }
    let mut contents: Vec<(String, u64)> = Vec::with_capacity(rt_files.len());
    for f in rt_files {
        let bytes = std::fs::read(f).map_err(|e| format!("read {f}: {e}"))?;
        contents.push((f.clone(), fnv1a(&bytes)));
    }
    contents.sort();
    for (path, hash) in &contents {
        key_input.push_str(path);
        key_input.push(' ');
        key_input.push_str(&format!("{hash:016x}"));
        key_input.push('\n');
    }
    let key = format!("{:016x}", fnv1a(key_input.as_bytes()));

    // ---- hit? -----------------------------------------------------------
    let dir = cache_root.join(&key);
    let mut objects: Vec<PathBuf> = Vec::with_capacity(rt_files.len());
    let mut complete = true;
    for (i, f) in rt_files.iter().enumerate() {
        let obj = dir.join(object_name(i, f));
        if obj.is_file() {
            objects.push(obj);
        } else {
            complete = false;
            break;
        }
    }
    if complete {
        if std::env::var_os("XIOM_TIMINGS").is_some() {
            eprintln!("[timings] rt-cache HIT {key}");
        }
        return Ok(objects);
    }

    // ---- build ----------------------------------------------------------
    std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let staging = dir.join(format!(
        "stage-{}-{:x}",
        std::process::id(),
        jit::rand_suffix()
    ));
    std::fs::create_dir_all(&staging).map_err(|e| format!("create {}: {e}", staging.display()))?;

    let started = std::time::Instant::now();
    // One clang invocation per source with an explicit `-o` (input basenames
    // can collide across directories; explicit outputs keep names stable).
    for (i, f) in rt_files.iter().enumerate() {
        let mut cmd = Command::new(clang);
        cmd.args(compile_args);
        cmd.arg("-c");
        cmd.arg(f);
        cmd.arg("-o");
        cmd.arg(staging.join(object_name(i, f)));
        cmd.current_dir(&staging);
        let out = cmd
            .output()
            .map_err(|e| format!("spawn {}: {e}", clang.display()))?;
        if !out.status.success() {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(format!(
                "runtime compile failed for {f}: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
    }

    // Move the produced objects to their stable names. A concurrent build of
    // the same key writes identical bytes; renames are atomic per file.
    for (i, f) in rt_files.iter().enumerate() {
        let produced = staging.join(object_name(i, f));
        let target = dir.join(object_name(i, f));
        std::fs::rename(&produced, &target)
            .map_err(|e| format!("move {}: {e}", produced.display()))?;
    }
    let _ = std::fs::remove_dir_all(&staging);

    if std::env::var_os("XIOM_TIMINGS").is_some() {
        eprintln!(
            "[timings] rt-cache MISS {key} (built in {:.2}s)",
            started.elapsed().as_secs_f64()
        );
    }
    objects.clear();
    for (i, f) in rt_files.iter().enumerate() {
        objects.push(dir.join(object_name(i, f)));
    }
    Ok(objects)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_file(path: &Path, content: &str) {
        std::fs::write(path, content).expect("write test source");
    }

    #[test]
    fn runtime_objects_builds_then_hits() {
        // A compiler is required to actually build; skip when absent.
        let clang = match crate::toolchain::probe_clang() {
            Some(info) => PathBuf::from(info.path),
            None => return,
        };
        let root = std::env::temp_dir().join(format!("xiom-rtcache-test-{}-{:x}", std::process::id(), jit::rand_suffix()));
        let src = root.join("src");
        std::fs::create_dir_all(&src).expect("create test dir");
        let c1 = src.join("t_one.c");
        write_file(&c1, "int xiom_test_one(void) { return 1; }\n");
        let c2 = src.join("t_two.c");
        write_file(&c2, "int xiom_test_two(void) { return 2; }\n");
        let files = vec![
            c1.to_string_lossy().to_string(),
            c2.to_string_lossy().to_string(),
        ];
        let args: Vec<String> = Vec::new();

        let first = runtime_objects(&root, &clang, &files, &args).expect("first build");
        assert_eq!(first.len(), 2);
        assert!(first.iter().all(|p| p.is_file()), "objects must exist after build");

        let second = runtime_objects(&root, &clang, &files, &args).expect("cache hit");
        assert_eq!(first, second, "second call must reuse the same objects");

        // Content change must invalidate.
        write_file(&c1, "int xiom_test_one(void) { return 42; }\n");
        let third = runtime_objects(&root, &clang, &files, &args).expect("rebuild");
        assert_ne!(first, third, "changed source must produce a new key");

        let _ = std::fs::remove_dir_all(&root);
    }
}
