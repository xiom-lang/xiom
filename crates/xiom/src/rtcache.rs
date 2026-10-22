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
//!
//! m340 (REL-20261016-1128-engine class 3): dependency `port.args.json`
//! C sources use the SAME cache root and key scheme through
//! [`dependency_objects`]; their object names additionally embed a hash of
//! the source path because a vendored dependency tree (assimp ~125 TUs)
//! ships many same-stem files that would collide under the runtime's
//! `NN_stem.o` naming.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};

use crate::jit;

/// m365: a per-key build lock guards the MISS path. A stale lock (builder
/// killed mid-build) is broken after this long; the engine's 139-TU assimp
/// build is ~438 s, so the threshold leaves ample headroom.
const KEY_LOCK_STALE_AFTER: Duration = Duration::from_secs(30 * 60);
/// Maximum time a waiter blocks for a builder before giving up (caller falls
/// back to the historical single-invocation compile path on error).
const KEY_LOCK_WAIT_LIMIT: Duration = Duration::from_secs(30 * 60);

/// RAII lock file for one cache key. `create_new` is atomic on both Windows
/// and Unix; the holder removes it on drop.
struct KeyLock {
    path: PathBuf,
}

impl Drop for KeyLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// m1531 (XVC-C-21): best-effort liveness probe for a lock holder. A builder
/// killed by a Job Object / OOM leaves `.lock` behind; the 30-minute waiter
/// cap otherwise looks like a build hang (the xvector watchdog is 120 s).
fn lock_holder_pid(path: &Path) -> Option<u32> {
    let text = std::fs::read_to_string(path).ok()?;
    let first = text.lines().next()?;
    first.strip_prefix("pid=")?.trim().parse().ok()
}

#[cfg(windows)]
fn pid_is_alive(pid: u32) -> bool {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut core::ffi::c_void;
        fn GetExitCodeProcess(handle: *mut core::ffi::c_void, code: *mut u32) -> i32;
        fn CloseHandle(handle: *mut core::ffi::c_void) -> i32;
    }
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    const STILL_ACTIVE: u32 = 259;
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }
        let mut code = 0u32;
        let ok = GetExitCodeProcess(handle, &mut code);
        CloseHandle(handle);
        ok != 0 && code == STILL_ACTIVE
    }
}

#[cfg(not(windows))]
fn pid_is_alive(pid: u32) -> bool {
    // Linux exposes /proc; other Unix has no dependency-free probe, so assume
    // alive and keep the stale-age threshold.
    if cfg!(target_os = "linux") {
        Path::new("/proc").join(pid.to_string()).exists()
    } else {
        true
    }
}

fn acquire_key_lock(dir: &Path) -> Result<KeyLock, String> {
    let path = dir.join(".lock");
    let deadline = Instant::now() + KEY_LOCK_WAIT_LIMIT;
    loop {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut f) => {
                let _ = writeln!(f, "pid={}", std::process::id());
                return Ok(KeyLock { path });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // m1531: break immediately when the recorded holder is dead
                // (killed mid-build by a memory job); otherwise a waiter
                // would sleep for KEY_LOCK_WAIT_LIMIT and look hung.
                if let Some(pid) = lock_holder_pid(&path) {
                    if !pid_is_alive(pid) {
                        let _ = std::fs::remove_file(&path);
                        continue;
                    }
                }
                // Break a lock left by a killed builder. A live long build
                // refreshes nothing, so choose the threshold well above the
                // largest expected build (see KEY_LOCK_STALE_AFTER).
                if let Ok(meta) = std::fs::metadata(&path) {
                    if let Ok(modified) = meta.modified() {
                        if SystemTime::now()
                            .duration_since(modified)
                            .unwrap_or_default()
                            > KEY_LOCK_STALE_AFTER
                        {
                            let _ = std::fs::remove_file(&path);
                            continue;
                        }
                    }
                }
                if Instant::now() >= deadline {
                    return Err(format!(
                        "timed out waiting for cache lock {}",
                        path.display()
                    ));
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(e) => return Err(format!("create lock {}: {e}", path.display())),
        }
    }
}

/// m365: a HIT requires the ready marker AND every object. Returning the
/// object list only when both hold keeps the check identical before and
/// after acquiring the per-key lock.
fn complete_objects(
    dir: &Path,
    files: &[String],
    object_name: fn(usize, &str) -> String,
    ready: &Path,
) -> Option<Vec<PathBuf>> {
    if !ready.is_file() {
        return None;
    }
    let mut objects = Vec::with_capacity(files.len());
    for (i, f) in files.iter().enumerate() {
        let obj = dir.join(object_name(i, f));
        if obj.is_file() {
            objects.push(obj);
        } else {
            return None;
        }
    }
    Some(objects)
}

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
///
/// `XIOM_OBJ_CACHE_DIR` overrides the root (hermetic e2e cache locks point
/// it at a temp dir; the m340 dependency-object cache shares this root).
pub fn default_cache_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("XIOM_OBJ_CACHE_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
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

fn runtime_object_name(index: usize, source: &str) -> String {
    let stem = Path::new(source)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("rt");
    format!("{index:02}_{stem}.o")
}

/// m340: dependency object names embed a hash of the full source PATH.
/// Vendored dependency trees ship many same-stem files (`codec.c` across
/// several directories); `runtime_object_name`'s index+stem scheme would
/// overwrite one with the other inside a single cache key dir.
fn dependency_object_name(index: usize, source: &str) -> String {
    let stem = Path::new(source)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("dep");
    let path_hash = fnv1a(source.as_bytes());
    format!("{index:04}_{stem}_{path_hash:08x}.o")
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
    ensure_objects(cache_root, clang, rt_files, compile_args, "rt-cache", runtime_object_name)
}

/// m340 (REL-20261016-1128-engine class 3): cached objects for dependency
/// `port.args.json` C sources.
///
/// Same cache root, key scheme and fallback contract as
/// [`runtime_objects`]; only the object file names differ (path-hashed, see
/// [`dependency_object_name`]) because dependency trees routinely carry
/// same-stem sources. `XIOM_TIMINGS=1` prints `dep-cache HIT/MISS` lines.
pub fn dependency_objects(
    cache_root: &Path,
    clang: &Path,
    dep_files: &[String],
    compile_args: &[String],
) -> Result<Vec<PathBuf>, String> {
    ensure_objects(cache_root, clang, dep_files, compile_args, "dep-cache", dependency_object_name)
}

/// Shared body of the two caches: key on (clang identity, flags, source
/// paths + content hashes), build missing objects one clang `-c` per source
/// into a staging dir, then atomically rename them to their stable names.
fn ensure_objects(
    cache_root: &Path,
    clang: &Path,
    files: &[String],
    compile_args: &[String],
    label: &str,
    object_name: fn(usize, &str) -> String,
) -> Result<Vec<PathBuf>, String> {
    // m365: canonicalize the dependency inputs for BOTH the key and the
    // compile. The same source reached through different relative spellings
    // (per-target working directories) must produce ONE key/object set. This
    // is part of the fix for the consumer-specific keys the engine reported;
    // the caller-side flag split is still tracked in m365.
    let files: Vec<String> = files
        .iter()
        .map(|f| {
            std::fs::canonicalize(f)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| f.clone())
        })
        .collect();
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
    let mut contents: Vec<(String, u64)> = Vec::with_capacity(files.len());
    for f in &files {
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
    // m365: a HIT requires the ready marker AND every object. The marker is
    // written last, so a concurrent builder's half-populated directory is a
    // MISS (rebuild is safe: identical bytes) rather than a link list that
    // can race a missing object (the engine's `no such file ... .o`).
    let ready = dir.join(".ready");
    if let Some(objects) = complete_objects(&dir, &files, object_name, &ready) {
        if std::env::var_os("XIOM_TIMINGS").is_some() {
            eprintln!("[timings] {label} HIT {key}");
        }
        return Ok(objects);
    }

    // ---- build (serialized per key) ------------------------------------
    std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    // m365 addendum: a per-key lock stops concurrent compilers from building
    // the same dependency set twice and, together with the marker gate,
    // removes the missing-object window the engine hit under parallel builds.
    let _lock = acquire_key_lock(&dir)?;
    // Another compiler may have completed this key while we waited.
    if let Some(objects) = complete_objects(&dir, &files, object_name, &ready) {
        if std::env::var_os("XIOM_TIMINGS").is_some() {
            eprintln!("[timings] {label} HIT {key} (waited for builder)");
        }
        return Ok(objects);
    }
    // A rebuild must not expose an old `.ready` while objects are replaced.
    let _ = std::fs::remove_file(&ready);
    let staging = dir.join(format!(
        "stage-{}-{:x}",
        std::process::id(),
        jit::rand_suffix()
    ));
    std::fs::create_dir_all(&staging).map_err(|e| format!("create {}: {e}", staging.display()))?;

    let started = std::time::Instant::now();
    // One clang invocation per source with an explicit `-o` (input basenames
    // can collide across directories; explicit outputs keep names stable).
    for (i, f) in files.iter().enumerate() {
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
                "compile failed for {f}: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
    }

    // Move the produced objects to their stable names. A concurrent build of
    // the same key writes identical bytes; renames are atomic per file.
    for (i, f) in files.iter().enumerate() {
        let produced = staging.join(object_name(i, f));
        let target = dir.join(object_name(i, f));
        std::fs::rename(&produced, &target)
            .map_err(|e| format!("move {}: {e}", produced.display()))?;
    }
    let _ = std::fs::remove_dir_all(&staging);

    // m365: publish readiness only AFTER every object is at its stable path.
    // The marker content is the key (diagnostics); its presence is the gate.
    if let Err(e) = std::fs::write(&ready, format!("{key}\n")) {
        if std::env::var_os("XIOM_TIMINGS").is_some() {
            eprintln!("[timings] {label} ready-write-failed: {e}");
        }
    }

    if std::env::var_os("XIOM_TIMINGS").is_some() {
        eprintln!(
            "[timings] {label} MISS {key} (built in {:.2}s)",
            started.elapsed().as_secs_f64()
        );
    }
    let mut objects: Vec<PathBuf> = Vec::with_capacity(files.len());
    for (i, f) in files.iter().enumerate() {
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

    #[test]
    fn dependency_objects_hit_and_keep_same_stem_sources_apart() {
        // m340: a vendored dependency tree ships same-stem files in several
        // directories (`one/util.c` + `two/util.c`); the dependency cache
        // must give every source its OWN object (the runtime `NN_stem.o`
        // scheme would overwrite one with the other in the same key dir).
        let clang = match crate::toolchain::probe_clang() {
            Some(info) => PathBuf::from(info.path),
            None => return,
        };
        let root = std::env::temp_dir().join(format!(
            "xiom-depcache-test-{}-{:x}",
            std::process::id(),
            jit::rand_suffix()
        ));
        let a_dir = root.join("a");
        let b_dir = root.join("b");
        std::fs::create_dir_all(&a_dir).expect("create a");
        std::fs::create_dir_all(&b_dir).expect("create b");
        let a = a_dir.join("util.c");
        let b = b_dir.join("util.c");
        let c = a_dir.join("extra.c");
        write_file(&a, "int dep_a(void) { return 1; }\n");
        write_file(&b, "int dep_b(void) { return 2; }\n");
        write_file(&c, "int dep_c(void) { return 3; }\n");
        let files = vec![
            a.to_string_lossy().to_string(),
            b.to_string_lossy().to_string(),
            c.to_string_lossy().to_string(),
        ];
        let args: Vec<String> = Vec::new();

        let first = dependency_objects(&root, &clang, &files, &args).expect("first build");
        assert_eq!(first.len(), 3);
        assert!(first.iter().all(|p| p.is_file()), "objects must exist after build");
        let unique: std::collections::HashSet<_> = first.iter().collect();
        assert_eq!(unique.len(), 3, "same-stem sources must map to distinct objects");

        let second = dependency_objects(&root, &clang, &files, &args).expect("cache hit");
        assert_eq!(first, second, "second call must reuse the same objects");

        // Content change must invalidate (same contract as the runtime cache).
        write_file(&a, "int dep_a(void) { return 40; }\n");
        let third = dependency_objects(&root, &clang, &files, &args).expect("rebuild");
        assert_ne!(first, third, "changed dependency source must produce a new key");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn dep_cache_ready_marker_gates_hits() {
        // m365: a HIT requires the `.ready` marker written after the object
        // renames. A half-populated key dir (no marker) must rebuild instead
        // of handing a linker an incomplete list.
        let clang = match crate::toolchain::probe_clang() {
            Some(info) => PathBuf::from(info.path),
            None => return,
        };
        let root = std::env::temp_dir().join(format!(
            "xiom-depcache-ready-{}-{:x}",
            std::process::id(),
            jit::rand_suffix()
        ));
        let src = root.join("src");
        std::fs::create_dir_all(&src).expect("create src");
        let a = src.join("ready.c");
        write_file(&a, "int ready_dep(void) { return 7; }\n");
        let files = vec![a.to_string_lossy().to_string()];
        let args: Vec<String> = Vec::new();

        let first = dependency_objects(&root, &clang, &files, &args).expect("first build");
        let dir = first[0].parent().expect("object parent").to_path_buf();
        let ready = dir.join(".ready");
        assert!(ready.is_file(), "a completed build must publish .ready");

        std::fs::remove_file(&ready).expect("remove ready marker");
        let second = dependency_objects(&root, &clang, &files, &args).expect("rebuild");
        assert_eq!(first, second, "the same key/objects must be reused");
        assert!(
            ready.is_file(),
            "a marker-less directory is a MISS and must re-publish .ready"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn dependency_key_is_stable_across_path_spellings() {
        // m365: per-target working directories can spell the same dependency
        // source differently (`a/../a/util.c`). The key/object names must be
        // canonical so the whole project shares one dependency build.
        let clang = match crate::toolchain::probe_clang() {
            Some(info) => PathBuf::from(info.path),
            None => return,
        };
        let root = std::env::temp_dir().join(format!(
            "xiom-depcache-spelling-{}-{:x}",
            std::process::id(),
            jit::rand_suffix()
        ));
        let a_dir = root.join("a");
        std::fs::create_dir_all(&a_dir).expect("create a");
        let a = a_dir.join("spell.c");
        write_file(&a, "int spell_dep(void) { return 9; }\n");
        let canonical = vec![a.to_string_lossy().to_string()];
        let dotted = vec![a_dir
            .join("..")
            .join("a")
            .join("spell.c")
            .to_string_lossy()
            .to_string()];
        let args: Vec<String> = Vec::new();

        let first = dependency_objects(&root, &clang, &canonical, &args).expect("first build");
        let second = dependency_objects(&root, &clang, &dotted, &args).expect("spelling hit");
        assert_eq!(
            first, second,
            "the same source through a different spelling must share one key"
        );

        let _ = std::fs::remove_dir_all(&root);
    }
    #[test]
    fn key_lock_is_exclusive() {
        // m365: the per-key lock admits only one builder. Eight threads race
        // for the same key; the observed concurrency must never exceed 1.
        let root = std::env::temp_dir().join(format!(
            "xiom-keylock-{}-{:x}",
            std::process::id(),
            jit::rand_suffix()
        ));
        std::fs::create_dir_all(&root).expect("create lock dir");
        let active = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let max_active = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut handles = Vec::new();
        for _ in 0..8 {
            let root = root.clone();
            let active = std::sync::Arc::clone(&active);
            let max_active = std::sync::Arc::clone(&max_active);
            handles.push(std::thread::spawn(move || {
                let _lock = acquire_key_lock(&root).expect("acquire key lock");
                let now = active.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                max_active.fetch_max(now, std::sync::atomic::Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(5));
                active.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            }));
        }
        for h in handles {
            h.join().expect("join lock thread");
        }
        assert_eq!(
            max_active.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "only one holder at a time"
        );
        assert!(!root.join(".lock").exists(), "lock file removed on drop");
        let _ = std::fs::remove_dir_all(&root);
    }

    // m1531: a lock left by a builder killed in a memory-capped Job Object
    // must be broken immediately, not waited on for KEY_LOCK_WAIT_LIMIT.
    #[cfg(any(windows, target_os = "linux"))]
    #[test]
    fn dead_holder_lock_is_broken() {
        let root = std::env::temp_dir().join(format!(
            "xiom-keylock-dead-{}-{:x}",
            std::process::id(),
            jit::rand_suffix()
        ));
        std::fs::create_dir_all(&root).expect("create lock dir");
        std::fs::write(root.join(".lock"), format!("pid={}\n", u32::MAX))
            .expect("write dead-holder lock");
        let started = Instant::now();
        let lock = acquire_key_lock(&root).expect("dead holder lock must break");
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "dead-holder lock took {:?} to break",
            started.elapsed()
        );
        drop(lock);
        assert!(!root.join(".lock").exists(), "lock removed on drop");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn concurrent_dependency_objects_share_one_key() {
        // m365 addendum: four compilers request the same dependency set at
        // once. The lock serializes the build; every caller gets the SAME
        // complete object list (no missing .o window).
        let clang = match crate::toolchain::probe_clang() {
            Some(info) => PathBuf::from(info.path),
            None => return,
        };
        let root = std::env::temp_dir().join(format!(
            "xiom-depcache-concurrent-{}-{:x}",
            std::process::id(),
            jit::rand_suffix()
        ));
        let src = root.join("src");
        std::fs::create_dir_all(&src).expect("create src");
        let a = src.join("conc_a.c");
        let b = src.join("conc_b.c");
        write_file(&a, "int conc_a(void) { return 11; }\n");
        write_file(&b, "int conc_b(void) { return 12; }\n");
        let files = vec![
            a.to_string_lossy().to_string(),
            b.to_string_lossy().to_string(),
        ];
        let args: Vec<String> = Vec::new();

        let mut handles = Vec::new();
        for _ in 0..4 {
            let root = root.clone();
            let clang = clang.clone();
            let files = files.clone();
            let args = args.clone();
            handles.push(std::thread::spawn(move || {
                dependency_objects(&root, &clang, &files, &args)
                    .expect("concurrent dependency objects")
            }));
        }
        let results: Vec<Vec<PathBuf>> = handles
            .into_iter()
            .map(|h| h.join().expect("join dep thread"))
            .collect();
        assert!(
            results[0].iter().all(|p| p.is_file()),
            "every object must exist for the first caller"
        );
        for r in &results[1..] {
            assert_eq!(r, &results[0], "all callers must share one key/object set");
            assert!(r.iter().all(|p| p.is_file()), "no missing objects");
        }

        let _ = std::fs::remove_dir_all(&root);
    }
}
