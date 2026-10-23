// XIOM -- dependency port.args.json build-arg application
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// REL-20261010-1549-packages-5: registry consumers of vendored-C packages
// (xiom.sqlite) resolved the dependency's module but the consumer link still
// failed with `undefined symbol: sqlite3_*` unless the user passed
// `--c-source <installed>/vendor/sqlite3.c` manually. The dependency ships a
// `port.args.json` (a JSON array of build flags, `${PACKAGE_DIR}` standing
// for the package dir); consumer builds now apply it.
//
// Only a bounded, known flag set is honored -- build args travel with a
// package, so the consumer build must not blindly accept every switch the
// author wrote (`--opt-level` etc. belong to the consumer's own build).
//
// m340 (REL-20261016-1128-engine class 3): two follow-ups on that merge.
// (a) Dependency `--c-source`s land in `CompileConfig::dependency_c_sources`
// (NOT `c_sources`) so the link stage compiles them through the rtcache
// object cache -- m310 shipped them into the single clang invocation, so a
// project with vendored C deps (assimp ~125 TUs) recompiled every TU on
// every compile. (b) `--timeout <secs>` is now recognized: historically
// every non-whitelisted flag, timeouts included, was dropped with a "not
// applied at consumer build" note, so a dependency's declared budget
// (assimp 1800, exr 600, harfbuzz 900) never reached the consumer and the
// global 300 s watchdog default applied. The driver now uses the highest
// declared budget as a FLOOR when the consumer set no timeout of its own
// (see `dependency_timeout_floor` + `resolve_compile_timeout`).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use xiom_graph::manifest::ProjectManifest;

use crate::CompileConfig;

/// Escape hatch: set to `1`/`true` to ignore dependency port.args.json for a
/// build (diagnosing a package whose port args misbehave). Also disables the
/// m340 dependency timeout floor.
pub const NO_DEP_PORT_ARGS_ENV: &str = "XIOM_NO_DEP_PORT_ARGS";

/// The compile watchdog's historical global default (main.rs): used as the
/// floor that a dependency's declared `--timeout` may raise, never lower.
pub const DEFAULT_COMPILE_TIMEOUT_SECS: u64 = 300;

fn dep_port_args_disabled() -> bool {
    std::env::var(NO_DEP_PORT_ARGS_ENV)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Apply every dependency's shipped `port.args.json` to the consumer build.
///
/// Returns the number of flags applied. Never fails the build itself: a
/// missing/malformed/misplaced file is reported and skipped (the consumer
/// can still pass flags manually, which keeps the historical recipe alive).
pub fn apply_dependency_port_args(
    config: &mut CompileConfig,
    manifest: &ProjectManifest,
) -> usize {
    if dep_port_args_disabled() {
        return 0;
    }
    let mut applied = 0usize;
    for dir in dependency_closure(manifest) {
        applied += apply_one(config, &dir);
    }
    applied
}

/// m1511: the TRANSITIVE dependency closure -- direct dependencies plus each
/// dependency's own dependencies, parsed from its shipped `package.xi`.
///
/// A consumer that depends on `xiom.fontraster` never lists
/// `xiom.harfbuzz` directly, but fontraster's `port.args.json` (the sibling
/// workaround) and harfbuzz's vendored-source flags must still reach the
/// link. Iteration is depth-first with a CANONICAL-path visited set, so a
/// package that (incorrectly) depends on its consumer -- or two packages
/// that depend on each other -- terminate; the consumer root is seeded as
/// visited so a cycle cannot apply the consumer's own port args as a
/// dependency's.
fn dependency_closure(manifest: &ProjectManifest) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let consumer_root = manifest
        .manifest_dir
        .canonicalize()
        .unwrap_or_else(|_| manifest.manifest_dir.clone());
    seen.insert(consumer_root);
    let mut pending: Vec<PathBuf> = xiom_graph::manifest::dependency_package_dirs(manifest);
    while let Some(dir) = pending.pop() {
        let canonical = dir.canonicalize().unwrap_or_else(|_| dir.clone());
        if !seen.insert(canonical) {
            continue;
        }
        // Parse the dependency's own manifest for the next level. A missing
        // or malformed package.xi is not fatal: the direct level was already
        // applied and the consumer can still pass flags manually.
        if let Ok(dep_manifest) = xiom_graph::manifest::find_and_parse_manifest(&dir) {
            for child in xiom_graph::manifest::dependency_package_dirs(&dep_manifest) {
                let child_canon = child.canonicalize().unwrap_or_else(|_| child.clone());
                if !seen.contains(&child_canon) {
                    pending.push(child);
                }
            }
        }
        roots.push(dir);
    }
    roots
}

/// m340: the highest `--timeout` any dependency declares in its shipped
/// `port.args.json`, in seconds (the compile-time budget the package author
/// knows its vendored C needs).
///
/// m1511: walks the TRANSITIVE closure, so a timeout declared only by a
/// second-level package (fontraster -> harfbuzz 900 s) still raises the
/// consumer's watchdog floor.
pub fn dependency_timeout_floor(manifest: &ProjectManifest) -> Option<u64> {
    if dep_port_args_disabled() {
        return None;
    }
    let mut floor: Option<u64> = None;
    for dir in dependency_closure(manifest) {
        let path = dir.join("port.args.json");
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(body) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };
        if let Some(secs) = timeout_from_args(&body) {
            floor = Some(floor.map_or(secs, |f: u64| f.max(secs)));
        }
    }
    floor
}

/// m340: resolve the compile-watchdog budget together with the label naming
/// the winning source (observability: printed under `XIOM_VERBOSE_CONFIG=1`,
/// locked by unit tests + `e2e_m340_dep_timeout_floor`).
///
/// Precedence:
/// 1. explicit `--timeout` (consumer CLI choice, `0` disables) -- wins;
/// 2. `[compiler] timeout-secs` (consumer project choice) -- wins;
/// 3. otherwise the global default 300 s, FLOORED UP by the highest
///    dependency `--timeout` (a package that declares 1800 s must not be
///    silently capped at 300).
pub fn resolve_compile_timeout(
    explicit: Option<u64>,
    manifest_timeout: Option<u64>,
    dependency_floor: Option<u64>,
) -> (u64, &'static str) {
    if let Some(t) = explicit {
        return (t, "explicit --timeout");
    }
    if let Some(t) = manifest_timeout {
        return (t, "[compiler] timeout-secs");
    }
    match dependency_floor {
        Some(f) if f > DEFAULT_COMPILE_TIMEOUT_SECS => {
            (f, "dependency port.args.json --timeout floor")
        }
        _ => (DEFAULT_COMPILE_TIMEOUT_SECS, "default"),
    }
}

/// Highest `--timeout` in one parsed port.args.json body, if any.
fn timeout_from_args(body: &serde_json::Value) -> Option<u64> {
    let mut found = None;
    for (flag, value) in arg_pairs(body) {
        if flag == "--timeout" {
            if let Some(v) = value {
                if let Ok(secs) = v.trim().parse::<u64>() {
                    found = Some(secs);
                }
            }
        }
    }
    found
}

/// Walk a port.args.json array as `(flag, value)` pairs, accepting
/// `--flag value` and `--flag=value` with the CLI consumption rules (a lone
/// flag consumes itself only). Non-string entries are skipped.
fn arg_pairs(body: &serde_json::Value) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let Some(args) = body.as_array() else {
        return out;
    };
    let mut i = 0;
    while i < args.len() {
        let Some(raw_flag) = args[i].as_str() else {
            i += 1;
            continue;
        };
        // Accept `--flag value` and `--flag=value`.
        let (flag, inline) = match raw_flag.split_once('=') {
            Some((f, v)) => (f, Some(v.to_string())),
            None => (raw_flag, None),
        };
        let value = match inline {
            Some(v) => Some(v),
            None => args.get(i + 1).and_then(|v| v.as_str()).map(|s| s.to_string()),
        };
        // One element for `--flag=value`, two for `--flag value` (a lone
        // flag with no following value consumes itself only).
        let consumed = if value.is_none() || raw_flag.contains('=') { 1 } else { 2 };
        out.push((flag.to_string(), value));
        i += consumed;
    }
    out
}

/// One dependency package dir: parse `<dir>/port.args.json` and merge its
/// supported flags into the config.
fn apply_one(config: &mut CompileConfig, dir: &Path) -> usize {
    let path = dir.join("port.args.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return 0;
    };
    let body: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("warning: {}: not valid JSON ({e}); ignored", path.display());
            return 0;
        }
    };
    if body.as_array().is_none() {
        eprintln!("warning: {}: expected a JSON array of strings; ignored", path.display());
        return 0;
    }
    // Preserve the per-entry diagnosis the raw walk produced.
    if let Some(args) = body.as_array() {
        for (i, entry) in args.iter().enumerate() {
            if entry.as_str().is_none() {
                eprintln!("warning: {}: entry {} is not a string; ignored", path.display(), i);
            }
        }
    }

    let mut applied = 0usize;
    for (flag, value) in arg_pairs(&body) {
        match flag.as_str() {
            "--c-source" => {
                if let Some(v) = value {
                    let p = resolve_path(&v, dir);
                    // m340: dependency C sources ride their own list so the
                    // link stage can cache-compile them (see the module doc).
                    if !config.dependency_c_sources.contains(&p) {
                        config.dependency_c_sources.push(p);
                    }
                    applied += 1;
                } else {
                    eprintln!("note: dependency port.args.json: '--c-source' needs a value; skipped");
                }
            }
            "--link" => {
                if let Some(v) = value {
                    let v = expand(&v, dir);
                    if !config.link_libs.contains(&v) {
                        config.link_libs.push(v);
                    }
                    applied += 1;
                } else {
                    eprintln!("note: dependency port.args.json: '--link' needs a value; skipped");
                }
            }
            "--link-path" => {
                if let Some(v) = value {
                    let p = resolve_path(&v, dir);
                    if !config.link_paths.contains(&p) {
                        config.link_paths.push(p);
                    }
                    applied += 1;
                } else {
                    eprintln!("note: dependency port.args.json: '--link-path' needs a value; skipped");
                }
            }
            "--cxx-standard" => {
                if let Some(v) = value {
                    // The consumer's own --cxx-standard wins when set.
                    if config.cxx_standard.is_none() {
                        config.cxx_standard = Some(v);
                    }
                    applied += 1;
                } else {
                    eprintln!("note: dependency port.args.json: '--cxx-standard' needs a value; skipped");
                }
            }
            "--timeout" => {
                // m340: recognized (not "ignored" anymore). The value feeds
                // `dependency_timeout_floor` -> `resolve_compile_timeout`;
                // parse-validate here so a malformed budget still reports.
                if let Some(v) = value {
                    if v.trim().parse::<u64>().is_ok() {
                        applied += 1;
                    } else {
                        eprintln!(
                            "note: dependency port.args.json: '--timeout' expects seconds; '{}' ignored",
                            v
                        );
                    }
                } else {
                    eprintln!("note: dependency port.args.json: '--timeout' needs a value; skipped");
                }
            }
            other => {
                eprintln!(
                    "note: dependency {}: port.args.json: ignoring '{}' (not applied at consumer build)",
                    dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                    other
                );
            }
        }
    }
    applied
}

/// `--flag` without a value reports once and is skipped.
fn expand(value: &str, dir: &Path) -> String {
    value.replace("${PACKAGE_DIR}", &dir.to_string_lossy())
}

/// Expand `${PACKAGE_DIR}` and make relative paths relative to the package.
///
/// The result is normalized through `Path` components so a JSON value like
/// `${PACKAGE_DIR}/vendor/x.c` never reaches clang with mixed separators
/// (clang's `\\?\`-prefixed long-path form rejects forward slashes).
fn resolve_path(value: &str, dir: &Path) -> String {
    let expanded = expand(value, dir);
    let pb = PathBuf::from(&expanded);
    let joined = if pb.is_absolute() { pb } else { dir.join(pb) };
    joined
        .components()
        .collect::<PathBuf>()
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_config() -> CompileConfig {
        CompileConfig::default()
    }

    #[test]
    fn expands_package_dir_and_resolves_relative() {
        // m330 (pin CI hygiene): a `C:/...` path is NOT absolute on POSIX,
        // so `resolve_path`'s relative re-join double-prefixed the dir there.
        // Use a POSIX-absolute dir on unix; the windows-only separator
        // assertions below stay under cfg(windows).
        #[cfg(windows)]
        let dir = Path::new("C:/home/packages/xiom-sqlite-0.4.1/xiom-sqlite");
        #[cfg(not(windows))]
        let dir = Path::new("/home/packages/xiom-sqlite-0.4.1/xiom-sqlite");
        let norm = |p: &Path| {
            p.components()
                .collect::<PathBuf>()
                .to_string_lossy()
                .to_string()
        };
        assert_eq!(
            resolve_path("${PACKAGE_DIR}/vendor/sqlite3.c", dir),
            norm(&dir.join("vendor").join("sqlite3.c"))
        );
        assert_eq!(
            resolve_path("vendor/sqlite3.c", dir),
            norm(&dir.join("vendor").join("sqlite3.c"))
        );
        // No mixed separators survive (clang's `\\?\` form rejects them).
        #[cfg(windows)]
        {
            assert_eq!(
                resolve_path("C:/abs/x.c", dir),
                norm(&PathBuf::from("C:/abs/x.c"))
            );
            assert!(!resolve_path("${PACKAGE_DIR}/vendor/x.c", dir).contains('/'));
        }
    }

    #[test]
    fn apply_one_reads_the_array_forms() {
        let dir = std::env::temp_dir().join(format!("xiom_depargs_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("vendor")).unwrap();
        std::fs::write(
            dir.join("port.args.json"),
            r#"["--c-source", "${PACKAGE_DIR}/vendor/x.c", "--link=curl", "--timeout", "600", "--opt-level", "default"]"#,
        )
        .unwrap();
        let mut config = empty_config();
        let applied = apply_one(&mut config, &dir);
        let src = dir.join("vendor").join("x.c").to_string_lossy().to_string();
        // m340: dependency C sources land in their own list (cache-routed at
        // link time), never in the consumer's `c_sources`.
        assert!(
            config.dependency_c_sources.contains(&src),
            "c-source applied: {:?}",
            config.dependency_c_sources
        );
        assert!(config.c_sources.is_empty(), "consumer list untouched");
        assert!(config.link_libs.contains(&"curl".to_string()), "link applied");
        assert_eq!(applied, 3, "c-source + link + timeout counted; unknown flags are not");
        assert_eq!(timeout_from_args(&serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(dir.join("port.args.json")).unwrap()
        ).unwrap()), Some(600), "m340: --timeout is recognized now");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_one_skips_malformed_arrays() {
        let dir = std::env::temp_dir().join(format!("xiom_depargs_bad_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("port.args.json"), r#"{"c-source": "x.c"}"#).unwrap();
        let mut config = empty_config();
        assert_eq!(apply_one(&mut config, &dir), 0);
        assert!(config.c_sources.is_empty());
        assert!(config.dependency_c_sources.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn timeout_floor_parses_both_spellings() {
        // m340: `--timeout 600` and `--timeout=600` both feed the floor; a
        // malformed value, a missing value and other flags do not.
        let v = |s: &str| serde_json::from_str::<serde_json::Value>(s).unwrap();
        assert_eq!(timeout_from_args(&v(r#"["--timeout", "600"]"#)), Some(600));
        assert_eq!(timeout_from_args(&v(r#"["--timeout=1800"]"#)), Some(1800));
        assert_eq!(timeout_from_args(&v(r#"["--c-source", "x.c", "--timeout"]"#)), None);
        assert_eq!(timeout_from_args(&v(r#"["--timeout", "soon"]"#)), None);
        assert_eq!(timeout_from_args(&v(r#"["--link", "curl"]"#)), None);
        // A later flag can never be MISREAD as the timeout's value.
        assert_eq!(timeout_from_args(&v(r#"["--timeout", "--link", "curl"]"#)), None);
        // Last declaration wins (mirrors CLI "last flag wins").
        assert_eq!(
            timeout_from_args(&v(r#"["--timeout", "300", "--timeout", "900"]"#)),
            Some(900)
        );
    }

    /// m1511: two-level INSTALLED layout -- consumer -> fontraster ->
    /// harfbuzz. Only fontraster is a direct dependency; harfbuzz's vendored
    /// C++ source, `--cxx-standard 17` and 900 s timeout must still reach the
    /// consumer build. Also proves the cycle guard terminates when a
    /// second-level package depends back on the first.
    #[test]
    fn transitive_installed_port_args_and_cycle_guard() {
        use xiom_graph::manifest::{CompilerConfig, DependencySpec, ProjectMeta};

        static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let root = std::env::temp_dir().join(format!(
            "xiom_depargs_transitive_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let home = root.join("home");
        let fr = home
            .join("packages")
            .join("xiom-fontraster-0.1.0")
            .join("xiom-fontraster");
        let hb = home
            .join("packages")
            .join("xiom-harfbuzz-0.1.0")
            .join("xiom-harfbuzz");
        std::fs::create_dir_all(fr.join("src")).unwrap();
        std::fs::create_dir_all(hb.join("vendor").join("harfbuzz").join("src")).unwrap();
        std::fs::write(fr.join("src").join("fontraster.c"), "/* fr */").unwrap();
        std::fs::write(
            hb.join("vendor").join("harfbuzz").join("src").join("harfbuzz-subset.cc"),
            "/* hb */",
        )
        .unwrap();
        std::fs::write(
            fr.join("port.args.json"),
            r#"["--c-source", "${PACKAGE_DIR}/src/fontraster.c"]"#,
        )
        .unwrap();
        std::fs::write(
            hb.join("port.args.json"),
            r#"["--c-source", "${PACKAGE_DIR}/vendor/harfbuzz/src/harfbuzz-subset.cc", "--cxx-standard", "17", "--timeout", "900"]"#,
        )
        .unwrap();
        std::fs::write(
            fr.join("package.xi"),
            "package xiom_fontraster {\n  name: \"xiom.fontraster\";\n  version: \"0.1.0\";\n  deps: { \"xiom.harfbuzz\": \">=0.1.0 <1.0.0\" };\n}\n",
        )
        .unwrap();
        std::fs::write(
            hb.join("package.xi"),
            "package xiom_harfbuzz {\n  name: \"xiom.harfbuzz\";\n  version: \"0.1.0\";\n}\n",
        )
        .unwrap();

        let prior_home = std::env::var_os("XIOM_HOME");
        // SAFETY: this module's tests serialize env mutation via ENV_LOCK.
        unsafe { std::env::set_var("XIOM_HOME", &home) };

        let consumer = ProjectManifest {
            project: ProjectMeta {
                name: "consumer".to_string(),
                version: "0.1.0".to_string(),
                ..Default::default()
            },
            dependencies: vec![DependencySpec {
                name: "xiom.fontraster".to_string(),
                version: ">=0.1.0 <1.0.0".to_string(),
                path: None,
                git: None,
            }],
            compiler: CompilerConfig::default(),
            manifest_dir: root.join("consumer"),
        };

        let mut config = empty_config();
        let applied = apply_dependency_port_args(&mut config, &consumer);
        let fr_src = fr.join("src").join("fontraster.c").to_string_lossy().to_string();
        let hb_src = hb
            .join("vendor")
            .join("harfbuzz")
            .join("src")
            .join("harfbuzz-subset.cc")
            .to_string_lossy()
            .to_string();
        assert!(
            config.dependency_c_sources.contains(&fr_src),
            "direct dependency source missing: {:?}",
            config.dependency_c_sources
        );
        assert!(
            config.dependency_c_sources.contains(&hb_src),
            "TRANSITIVE harfbuzz source missing: {:?}",
            config.dependency_c_sources
        );
        assert_eq!(config.cxx_standard.as_deref(), Some("17"));
        assert_eq!(dependency_timeout_floor(&consumer), Some(900));
        assert!(applied >= 2, "both levels contribute flags; applied={applied}");

        // Cycle: harfbuzz now depends back on fontraster. The closure must
        // terminate and must not re-add the same sources.
        std::fs::write(
            hb.join("package.xi"),
            "package xiom_harfbuzz {\n  name: \"xiom.harfbuzz\";\n  version: \"0.1.0\";\n  deps: { \"xiom.fontraster\": \">=0.1.0 <1.0.0\" };\n}\n",
        )
        .unwrap();
        let before = config.dependency_c_sources.len();
        let _ = apply_dependency_port_args(&mut config, &consumer);
        assert_eq!(
            config.dependency_c_sources.len(),
            before,
            "cycle guard: no duplicate sources on a cyclic graph"
        );

        // SAFETY: this test serializes env mutation via ENV_LOCK; restore the
        // prior process env before releasing it.
        unsafe {
            match prior_home {
                Some(v) => std::env::set_var("XIOM_HOME", v),
                None => std::env::remove_var("XIOM_HOME"),
            }
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_compile_timeout_precedence() {
        // m340: explicit consumer choices (CLI then manifest) always win;
        // with neither set, the dependency floor raises the default 300 but
        // can never LOWER it.
        assert_eq!(resolve_compile_timeout(Some(120), Some(600), Some(1800)), (120, "explicit --timeout"));
        assert_eq!(resolve_compile_timeout(Some(0), None, Some(1800)), (0, "explicit --timeout"));
        assert_eq!(resolve_compile_timeout(None, Some(900), Some(1800)), (900, "[compiler] timeout-secs"));
        assert_eq!(
            resolve_compile_timeout(None, None, Some(600)),
            (600, "dependency port.args.json --timeout floor")
        );
        assert_eq!(
            resolve_compile_timeout(None, None, Some(1800)),
            (1800, "dependency port.args.json --timeout floor")
        );
        // A floor below/at the default leaves the default in place.
        assert_eq!(resolve_compile_timeout(None, None, Some(120)), (300, "default"));
        assert_eq!(resolve_compile_timeout(None, None, Some(300)), (300, "default"));
        assert_eq!(resolve_compile_timeout(None, None, None), (300, "default"));
    }
}
