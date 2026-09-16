// XIOM -- shared repository/path resolution (R27 + R31).
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// R31 (release/infra, pre-split): ONE helper for every cross-repo test and
// tool. The compiler repo and the stdlib repo are separate after the split;
// the stdlib lives in a pinned checkout at `<repo>/stdlib/` (gitignored,
// fetched by `scripts/fetch-stdlib.ps1|.sh` from `STDLIB_VERSION`), and the
// smoke corpus is resolved through `stdlib_smoke_dir()`:
//   XIOM_STDLIB_SMOKES -> <stdlib>/tests/smoke/ -> legacy
//   <repo>/examples/stdlib_smoke/ (transition only).
// Missing checkout = loud SKIP locally; hard FAIL when XIOM_REQUIRE_STDLIB=1
// (CI sets it) -- see `skip_if_missing`.

use std::path::{Path, PathBuf};

/// The compiler repo root, inferred from this crate's compile-time path
/// (`crates/xiom-graph` -> repo root). Stable across build machines only for
/// the machine that built the binary; use it as a last-resort fallback.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// R27 (release/infra R0): ordered, filesystem-free stdlib root candidates.
///
/// The compiler used to only look for a literal `stdlib/` directory above the
/// executable, so an installed binary (`bin/xiom.exe` next to `lib/xiom/**`,
/// `lib/package.xi`, `lib/runtime/**`) found nothing unless the user set
/// `XIOM_STDLIB` or ran from a checkout. Candidate order:
/// `XIOM_STDLIB` -> exe ancestors (`{dir}/stdlib`, `{dir}/lib`,
/// `{dir}/share/xiom`, nearest ancestor first) -> CWD `stdlib/` ->
/// `XIOM_HOME/{lib,stdlib}` -> baked repo checkout (`CARGO_MANIFEST_DIR`).
/// `XIOM_HOME` is deliberately a FALLBACK, not an override: a version-pinned
/// sibling `lib/` (dev checkout or install) must win over a stale global home
/// -- a dev machine with an old install exported as XIOM_HOME otherwise mixed
/// the old stdlib into checkout builds (JIT/catalog-body failures).
/// Existence/content filtering happens in [`existing_stdlib_roots`].
pub fn stdlib_candidates(
    exe_dir: Option<&Path>,
    xiom_stdlib: Option<&str>,
    xiom_home: Option<&str>,
    manifest_dir: Option<&str>,
) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut push = |p: PathBuf| {
        if !out.contains(&p) {
            out.push(p);
        }
    };
    if let Some(s) = xiom_stdlib {
        if !s.trim().is_empty() {
            push(PathBuf::from(s.trim()));
        }
    }
    if let Some(dir) = exe_dir {
        let mut cur = Some(dir);
        for _ in 0..=8 {
            if let Some(d) = cur {
                push(d.join("stdlib"));
                push(d.join("lib"));
                push(d.join("share").join("xiom"));
                cur = d.parent();
            } else {
                break;
            }
        }
    }
    push(PathBuf::from("stdlib"));
    if let Some(home) = xiom_home {
        if !home.trim().is_empty() {
            let home = PathBuf::from(home.trim());
            push(home.join("lib"));
            push(home.join("stdlib"));
        }
    }
    if let Some(m) = manifest_dir {
        if let Some(repo) = Path::new(m).parent().and_then(|p| p.parent()) {
            push(repo.join("stdlib"));
        }
    }
    out
}

/// R27: a usable stdlib root carries the module tree (`xiom/`) or the stdlib
/// manifest (`package.xi`). Content validation keeps an unrelated `lib/`,
/// `share/xiom/`, or a stale baked checkout path from being selected.
pub fn is_stdlib_root(dir: &Path) -> bool {
    dir.join("xiom").is_dir() || dir.join("package.xi").is_file()
}

/// R27: the existing, content-valid stdlib roots in candidate order (deduped).
pub fn existing_stdlib_roots(candidates: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    for c in candidates {
        if c.is_dir() && is_stdlib_root(c) && !roots.contains(c) {
            roots.push(c.clone());
        }
    }
    roots
}

/// R27: candidate list for the RUNNING process (exe path + env vars).
pub fn current_stdlib_candidates() -> Vec<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()));
    stdlib_candidates(
        exe_dir.as_deref(),
        std::env::var("XIOM_STDLIB").ok().as_deref(),
        std::env::var("XIOM_HOME").ok().as_deref(),
        Some(env!("CARGO_MANIFEST_DIR")),
    )
}

/// R31: the FIRST existing, content-valid stdlib root for this process.
/// This is the single resolver every cross-repo test and tool uses.
pub fn stdlib_root() -> Option<PathBuf> {
    existing_stdlib_roots(&current_stdlib_candidates()).into_iter().next()
}

/// R31: the smoke-corpus directory, in contract order:
/// `XIOM_STDLIB_SMOKES` -> `<stdlib_root>/tests/smoke/` ->
/// legacy `<repo>/examples/stdlib_smoke/` (transition only; the stdlib repo
/// normalizes its corpus to `tests/smoke/`).
pub fn stdlib_smoke_dir() -> Option<PathBuf> {
    if let Ok(env_dir) = std::env::var("XIOM_STDLIB_SMOKES") {
        let p = PathBuf::from(env_dir.trim());
        if !env_dir.trim().is_empty() && p.is_dir() {
            return Some(p);
        }
    }
    if let Some(root) = stdlib_root() {
        let tests_smoke = root.join("tests").join("smoke");
        if tests_smoke.is_dir() {
            return Some(tests_smoke);
        }
    }
    let legacy = repo_root().join("examples").join("stdlib_smoke");
    if legacy.is_dir() {
        return Some(legacy);
    }
    None
}

/// R31: the resolved stdlib root, or a loud SKIP (hard FAIL under
/// `XIOM_REQUIRE_STDLIB=1`). Test callers early-return on `None`.
pub fn stdlib_or_skip() -> Option<PathBuf> {
    match stdlib_root() {
        Some(root) => Some(root),
        None => {
            let msg = "SKIP: stdlib checkout not found (XIOM_STDLIB, repo stdlib/, \
                       installed lib/ -- run scripts/fetch-stdlib.ps1|.sh)".to_string();
            if require_stdlib() {
                panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
            }
            eprintln!("{msg}");
            None
        }
    }
}

/// R31: true when missing checkouts must fail instead of skipping (CI sets
/// `XIOM_REQUIRE_STDLIB=1`).
pub fn require_stdlib() -> bool {
    std::env::var("XIOM_REQUIRE_STDLIB").map_or(false, |v| v.trim() == "1")
}

/// R31: guard for a missing cross-repo path. Returns `true` when the caller
/// must SKIP: prints a loud SKIP line naming `what` and the path, or panics
/// when `XIOM_REQUIRE_STDLIB=1`. Returns `false` when `path` exists.
pub fn skip_if_missing(what: &str, path: &Path) -> bool {
    if path.exists() {
        return false;
    }
    let msg = format!(
        "SKIP: {what} not found at '{}' -- stdlib checkout missing (run scripts/fetch-stdlib.ps1|.sh)",
        path.display()
    );
    if require_stdlib() {
        panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
    }
    eprintln!("{msg}");
    true
}

/// R31: same as [`skip_if_missing`] but for the resolved smoke directory.
pub fn skip_smokes_if_missing() -> Option<PathBuf> {
    match stdlib_smoke_dir() {
        Some(dir) => Some(dir),
        None => {
            let msg = "SKIP: stdlib smoke corpus not found -- set XIOM_STDLIB_SMOKES or \
                       fetch the stdlib checkout (scripts/fetch-stdlib.ps1|.sh)".to_string();
            if require_stdlib() {
                panic!("{msg} -- XIOM_REQUIRE_STDLIB=1 forbids skipping");
            }
            eprintln!("{msg}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- R27/R31: exe-relative stdlib discovery --------------------------

    fn r31_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("xiom_r31_{}_{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn r31_make_stdlib_root(root: &Path) {
        std::fs::create_dir_all(root.join("xiom")).unwrap();
        std::fs::write(root.join("package.xi"), "name = \"std\"").unwrap();
    }

    fn r31_strs(cands: &[PathBuf]) -> Vec<String> {
        cands.iter().map(|p| p.to_string_lossy().replace('\\', "/")).collect()
    }

    #[test]
    fn r31_repo_layout_prefers_exe_ancestor_stdlib() {
        let repo = r31_dir("repo");
        r31_make_stdlib_root(&repo.join("stdlib"));
        let exe_dir = repo.join("target").join("debug");
        std::fs::create_dir_all(&exe_dir).unwrap();
        let roots = existing_stdlib_roots(&stdlib_candidates(Some(&exe_dir), None, None, None));
        assert_eq!(
            roots.first().map(|p| p.as_path()),
            Some(repo.join("stdlib").as_path()),
            "repo checkout layout must resolve <repo>/stdlib via exe ancestors"
        );
    }

    #[test]
    fn r31_install_layout_bin_and_lib() {
        let install = r31_dir("install");
        r31_make_stdlib_root(&install.join("lib"));
        std::fs::create_dir_all(install.join("lib").join("runtime")).unwrap();
        std::fs::write(install.join("lib").join("runtime").join("xiom_runtime.c"), "// rt").unwrap();
        let exe_dir = install.join("bin");
        std::fs::create_dir_all(&exe_dir).unwrap();
        let roots = existing_stdlib_roots(&stdlib_candidates(Some(&exe_dir), None, None, None));
        assert_eq!(
            roots.first().map(|p| p.as_path()),
            Some(install.join("lib").as_path()),
            "installed layout (bin/ + lib/) must resolve <install>/lib"
        );
        let rt = stdlib_candidates(Some(&exe_dir), None, None, None)
            .into_iter()
            .map(|root| root.join("runtime").join("xiom_runtime.c"))
            .find(|c| c.is_file());
        assert_eq!(rt, Some(install.join("lib").join("runtime").join("xiom_runtime.c")));
    }

    #[test]
    fn r31_xiom_home_fallback_only_when_no_sibling_lib() {
        let home = r31_dir("home");
        r31_make_stdlib_root(&home.join("lib"));
        let exe_dir = r31_dir("elsewhere").join("bin");
        std::fs::create_dir_all(&exe_dir).unwrap();
        let roots = existing_stdlib_roots(&stdlib_candidates(
            Some(&exe_dir), None, Some(home.to_str().unwrap()), None));
        assert_eq!(
            roots.first().map(|p| p.as_path()),
            Some(home.join("lib").as_path()),
            "XIOM_HOME/lib must serve a relocated binary"
        );
        let repo = r31_dir("repo2");
        r31_make_stdlib_root(&repo.join("stdlib"));
        let exe_dir2 = repo.join("target").join("debug");
        std::fs::create_dir_all(&exe_dir2).unwrap();
        let roots2 = existing_stdlib_roots(&stdlib_candidates(
            Some(&exe_dir2), None, Some(home.to_str().unwrap()), None));
        assert_eq!(
            roots2.first().map(|p| p.as_path()),
            Some(repo.join("stdlib").as_path()),
            "sibling checkout must outrank a stale XIOM_HOME"
        );
    }

    #[test]
    fn r31_stale_baked_path_and_empty_lib_are_ignored() {
        let tmp = r31_dir("stale");
        let install = tmp.join("install");
        std::fs::create_dir_all(install.join("lib")).unwrap();
        let exe_dir = install.join("bin");
        std::fs::create_dir_all(&exe_dir).unwrap();
        let manifest = tmp.join("gone").join("crates").join("xiom-graph");
        let roots = existing_stdlib_roots(&stdlib_candidates(
            Some(&exe_dir), None, None, Some(manifest.to_str().unwrap())));
        assert!(
            roots.is_empty(),
            "empty lib/ and a stale baked path must not validate: {roots:?}"
        );
    }

    #[test]
    fn r31_candidate_order_stdlib_exe_cwd_home_baked() {
        let exe_dir = PathBuf::from("X").join("bin");
        let strs = r31_strs(&stdlib_candidates(
            Some(&exe_dir), Some("S"), Some("H"), Some("M/crates/xiom-graph")));
        let pos = |needle: &str| strs.iter().position(|s| s == needle).unwrap_or(usize::MAX);
        assert_eq!(strs.first().map(|s| s.as_str()), Some("S"),
            "XIOM_STDLIB must be the first candidate");
        assert!(pos("X/bin/stdlib") < pos("X/stdlib"),
            "nearest exe ancestor must come first");
        assert!(pos("X/stdlib") < pos("X/lib") && pos("X/lib") < pos("X/share/xiom"));
        assert!(pos("X/share/xiom") < pos("stdlib"),
            "exe-relative candidates outrank CWD");
        assert!(pos("stdlib") < pos("H/lib"),
            "XIOM_HOME is a fallback, not an override");
        assert!(pos("H/lib") < pos("H/stdlib"));
        assert!(pos("H/stdlib") < pos("M/stdlib"),
            "baked checkout path is the last fallback");
    }

    // ---- R31: skip/fail guard -------------------------------------------

    #[test]
    fn r31_skip_guard_existing_and_missing() {
        let dir = r31_dir("guard");
        assert!(!skip_if_missing("test path", &dir), "existing path must not skip");
        assert!(skip_if_missing("test path", &dir.join("nope")),
            "missing path must skip (no XIOM_REQUIRE_STDLIB in this test process)");
        assert!(!require_stdlib(), "XIOM_REQUIRE_STDLIB must be unset in unit tests");
    }

    #[test]
    fn r31_repo_root_has_workspace_manifest() {
        assert!(repo_root().join("Cargo.toml").is_file(),
            "repo_root() must point at the workspace root");
    }
}
