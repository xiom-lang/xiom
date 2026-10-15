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

use std::path::{Path, PathBuf};

use xiom_graph::manifest::ProjectManifest;

use crate::CompileConfig;

/// Escape hatch: set to `1`/`true` to ignore dependency port.args.json for a
/// build (diagnosing a package whose port args misbehave).
pub const NO_DEP_PORT_ARGS_ENV: &str = "XIOM_NO_DEP_PORT_ARGS";

/// Apply every dependency's shipped `port.args.json` to the consumer build.
///
/// Returns the number of flags applied. Never fails the build itself: a
/// missing/malformed/misplaced file is reported and skipped (the consumer
/// can still pass flags manually, which keeps the historical recipe alive).
pub fn apply_dependency_port_args(
    config: &mut CompileConfig,
    manifest: &ProjectManifest,
) -> usize {
    if std::env::var(NO_DEP_PORT_ARGS_ENV)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return 0;
    }
    let mut applied = 0usize;
    for dir in xiom_graph::manifest::dependency_package_dirs(manifest) {
        applied += apply_one(config, &dir);
    }
    applied
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
    let Some(args) = body.as_array() else {
        eprintln!("warning: {}: expected a JSON array of strings; ignored", path.display());
        return 0;
    };

    let mut applied = 0usize;
    let mut i = 0;
    while i < args.len() {
        let Some(raw_flag) = args[i].as_str() else {
            eprintln!("warning: {}: entry {} is not a string; ignored", path.display(), i);
            i += 1;
            continue;
        };
        // Accept `--flag value` and `--flag=value`.
        let (flag, inline) = match raw_flag.split_once('=') {
            Some((f, v)) => (f, Some(v.to_string())),
            None => (raw_flag, None),
        };
        let value = match &inline {
            Some(v) => Some(v.clone()),
            None => args.get(i + 1).and_then(|v| v.as_str()).map(|s| s.to_string()),
        };
        // One element for `--flag=value`, two for `--flag value` (a lone
        // flag with no following value consumes itself only).
        let consumed = if inline.is_some() || value.is_none() { 1 } else { 2 };
        match flag {
            "--c-source" => {
                if let Some(v) = value {
                    let p = resolve_path(&v, dir);
                    if !config.c_sources.contains(&p) {
                        config.c_sources.push(p);
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
            other => {
                eprintln!(
                    "note: dependency {}: port.args.json: ignoring '{}' (not applied at consumer build)",
                    dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                    other
                );
            }
        }
        i += consumed;
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
            r#"["--c-source", "${PACKAGE_DIR}/vendor/x.c", "--link=curl", "--opt-level", "default"]"#,
        )
        .unwrap();
        let mut config = empty_config();
        let applied = apply_one(&mut config, &dir);
        let src = dir.join("vendor").join("x.c").to_string_lossy().to_string();
        assert!(config.c_sources.contains(&src), "c-source applied: {:?}", config.c_sources);
        assert!(config.link_libs.contains(&"curl".to_string()), "link applied");
        assert_eq!(applied, 2, "unknown flags are not counted as applied");
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
        let _ = std::fs::remove_dir_all(&dir);
    }
}
