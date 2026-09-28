// XIOM shared toolchain probe (front-end audit FE-1/FE-3).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// ONE probe shared by `xiom doctor` AND the compiler driver. Before this,
// doctor ran only `Command::new("clang").arg("--version")` through PATH while
// the driver carried its own fallback list (including a developer's personal
// NASM path). The two could disagree: the report that triggered the front-end
// audit had LLVM installed at the standard Windows location -- so the compiler
// worked -- while doctor said "clang NOT FOUND".
//
// Order is the audit contract: PATH first (the binary the user's shell would
// run), then the per-OS known locations (Program Files / winget / Homebrew /
// distro LLVM dirs). The version is probed by EXECUTING the resolved path, so
// doctor and the driver report and use the same binary.
//
// `tool_candidates_with` takes the PATH string and an env getter so the
// ordering and the per-OS lists are unit-testable on any host.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A resolved external tool (clang, opt, nasm).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolInfo {
    /// Requested tool name ("clang", "opt", "nasm").
    pub name: String,
    /// The resolved path. For a PATH hit this is the absolute PATH entry
    /// (not the bare name), so callers can report where it came from.
    pub path: PathBuf,
    /// First non-empty line of `--version`, when the tool answers one.
    pub version: Option<String>,
    /// True when the path came from the PATH environment variable.
    pub from_path: bool,
}

/// A probe candidate: a concrete path plus where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub path: PathBuf,
    pub from_path: bool,
}

/// Probe order for `name` on this machine (PATH first, then known locations).
pub fn tool_candidates(name: &str) -> Vec<Candidate> {
    tool_candidates_with(
        name,
        std::env::var("PATH").ok().as_deref(),
        &|key| std::env::var(key).ok(),
        cfg!(windows),
    )
}

/// Pure candidate builder (FE-1 ordering): every PATH directory first, in
/// PATH order, then the per-OS known locations. Wildcards are expanded by
/// scanning the directory that holds them (winget package dirs carry a
/// version/hash suffix, `/usr/lib/llvm-*` carries the major version).
pub fn tool_candidates_with(
    name: &str,
    path_env: Option<&str>,
    env: &dyn Fn(&str) -> Option<String>,
    windows: bool,
) -> Vec<Candidate> {
    let mut out: Vec<Candidate> = Vec::new();
    let mut push = |path: PathBuf, from_path: bool| {
        if !out.iter().any(|c| c.path == path) {
            out.push(Candidate { path, from_path });
        }
    };
    if let Some(path_env) = path_env {
        for dir in std::env::split_paths(path_env) {
            if dir.as_os_str().is_empty() {
                continue;
            }
            for file in exe_names(name, windows) {
                push(dir.join(file), true);
            }
        }
    }
    for pattern in known_locations(name, env, windows) {
        for path in expand_wildcards(&pattern) {
            push(path, false);
        }
    }
    out
}

/// Resolve `name` on this machine: first candidate that exists AND answers
/// `--version` (a file that cannot be spawned is skipped, not returned).
pub fn probe(name: &str) -> Option<ToolInfo> {
    probe_candidates(name, &tool_candidates(name))
}

/// Resolve from an explicit candidate list (testable without the machine).
pub fn probe_candidates(name: &str, candidates: &[Candidate]) -> Option<ToolInfo> {
    for c in candidates {
        if !c.path.is_file() {
            continue;
        }
        match run_version(&c.path) {
            Err(()) => continue,
            Ok(version) => {
                return Some(ToolInfo {
                    name: name.to_string(),
                    path: c.path.clone(),
                    version,
                    from_path: c.from_path,
                });
            }
        }
    }
    None
}

/// The compiler's required C toolchain.
pub fn probe_clang() -> Option<ToolInfo> {
    probe("clang")
}

/// LLVM `opt` (optional: IR verification + `--opt-level` passes).
pub fn probe_opt() -> Option<ToolInfo> {
    probe("opt")
}

/// NASM (optional: stdlib hardware-accelerated asm; C fallbacks exist).
pub fn probe_nasm() -> Option<ToolInfo> {
    probe("nasm")
}

/// Resolve a sibling LLVM tool from the resolved clang directory
/// (`<llvm>/bin/opt` next to `<llvm>/bin/clang`). Covers custom LLVM
/// installs the known-location list does not carry.
pub fn sibling_tool(clang: &Path, name: &str) -> Option<ToolInfo> {
    let dir = clang.parent()?;
    let candidate = dir.join(exe_names(name, cfg!(windows)).into_iter().next()?);
    if !candidate.is_file() {
        return None;
    }
    match run_version(&candidate) {
        Err(()) => None,
        Ok(version) => Some(ToolInfo {
            name: name.to_string(),
            path: candidate,
            version,
            from_path: false,
        }),
    }
}

/// Bare-name lookup on PATH, resolved to the absolute directory entry (the
/// doctor identity block must never print a bare `z3.exe`).
pub fn path_lookup(name: &str) -> Option<PathBuf> {
    let path_env = std::env::var("PATH").ok()?;
    for dir in std::env::split_paths(&path_env) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// First line of `--version` output (stdout, else stderr), trimmed.
pub fn version_of(path: &Path) -> Option<String> {
    run_version(path).ok().flatten()
}

/// Platform executable spelling(s) for a tool name.
fn exe_names(name: &str, windows: bool) -> Vec<String> {
    if windows {
        vec![format!("{name}.exe")]
    } else {
        vec![name.to_string()]
    }
}

/// Run `<path> --version`. `Err(())` means the file could not be spawned
/// (broken binary / no exec permission) -- the caller moves on.
fn run_version(path: &Path) -> Result<Option<String>, ()> {
    let output = Command::new(path).arg("--version").output().map_err(|_| ())?;
    let bytes = if output.stdout.is_empty() {
        output.stderr
    } else {
        output.stdout
    };
    Ok(parse_first_version_line(&String::from_utf8_lossy(&bytes)))
}

/// First non-empty, trimmed line of a version banner.
fn parse_first_version_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.to_string())
}

/// Per-OS known locations for `name` (FE-1/FE-3). Windows mirrors the list
/// `install_deps.ps1` already probes; the personal `C:\Users\<dev>\...`
/// NASM path the driver used to carry is deliberately NOT here.
fn known_locations(name: &str, env: &dyn Fn(&str) -> Option<String>, windows: bool) -> Vec<PathBuf> {
    if windows {
        let exe = exe_names(name, true).remove(0);
        match name {
            "clang" | "opt" => {
                let mut out = Vec::new();
                let mut push_in = |base: Option<String>, parts: &[&str]| {
                    if let Some(base) = base {
                        if !base.trim().is_empty() {
                            let mut p = PathBuf::from(base.trim());
                            for part in parts {
                                p.push(part);
                            }
                            p.push(&exe);
                            out.push(p);
                        }
                    }
                };
                push_in(env("ProgramFiles"), &["LLVM", "bin"]);
                push_in(env("LOCALAPPDATA"), &["Programs", "LLVM", "bin"]);
                for base in ["LOCALAPPDATA", "ProgramData"] {
                    push_in(env(base), &["Microsoft", "WinGet", "Packages", "LLVM.LLVM_*", "bin"]);
                }
                if let Some(la) = env("LOCALAPPDATA") {
                    if !la.trim().is_empty() {
                        let mut p = PathBuf::from(la.trim());
                        for part in ["Microsoft", "WinGet", "Links"] {
                            p.push(part);
                        }
                        p.push(&exe);
                        out.push(p);
                    }
                }
                out
            }
            "nasm" => {
                let mut out = Vec::new();
                let mut push_in = |base: Option<String>, parts: &[&str]| {
                    if let Some(base) = base {
                        if !base.trim().is_empty() {
                            let mut p = PathBuf::from(base.trim());
                            for part in parts {
                                p.push(part);
                            }
                            p.push(&exe);
                            out.push(p);
                        }
                    }
                };
                push_in(env("LOCALAPPDATA"), &["bin", "NASM"]);
                push_in(env("ProgramFiles"), &["NASM"]);
                push_in(env("ProgramFiles(x86)"), &["NASM"]);
                // winget installs NASM under its Packages tree (and Links).
                push_in(env("LOCALAPPDATA"), &["Microsoft", "WinGet", "Packages", "NASM.NASM_*"]);
                if let Some(la) = env("LOCALAPPDATA") {
                    if !la.trim().is_empty() {
                        let mut p = PathBuf::from(la.trim());
                        for part in ["Microsoft", "WinGet", "Links"] {
                            p.push(part);
                        }
                        p.push(&exe);
                        out.push(p);
                    }
                }
                out
            }
            _ => Vec::new(),
        }
    } else {
        match name {
            "clang" | "opt" => vec![
                PathBuf::from("/usr/bin").join(name),
                PathBuf::from("/usr/local/bin").join(name),
                PathBuf::from("/opt/homebrew/opt/llvm/bin").join(name),
                PathBuf::from("/usr/lib/llvm-*/bin").join(name),
            ],
            "nasm" => vec![
                PathBuf::from("/usr/local/bin/nasm"),
                PathBuf::from("/usr/bin/nasm"),
                PathBuf::from("/opt/homebrew/bin/nasm"),
            ],
            _ => Vec::new(),
        }
    }
}

/// Expand a candidate containing at most one `*` component by scanning its
/// parent directory. Sorted for deterministic order; a pattern with more
/// than one `*` is unsupported and yields nothing.
fn expand_wildcards(pattern: &Path) -> Vec<PathBuf> {
    let s = pattern.to_string_lossy();
    if !s.contains('*') {
        return vec![pattern.to_path_buf()];
    }
    let (before, after) = match s.split_once('*') {
        Some(parts) => parts,
        None => return vec![pattern.to_path_buf()],
    };
    if after.contains('*') {
        return Vec::new();
    }
    let sep = |c: char| c == '/' || c == '\\';
    let dir = match before.rfind(sep) {
        Some(i) => PathBuf::from(&before[..i]),
        None => PathBuf::from("."),
    };
    let stem = match before.rfind(sep) {
        Some(i) => &before[i + 1..],
        None => before,
    };
    let (tail, rest) = match after.find(sep) {
        Some(i) => (&after[..i], &after[i..]),
        None => (after, ""),
    };
    let mut out: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(stem) && name.ends_with(tail) {
                let mut path = entry.path();
                for part in rest.split(sep).filter(|p| !p.is_empty()) {
                    path.push(part);
                }
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_from<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key: &str| {
            pairs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v.to_string())
        }
    }

    // CI hygiene: these assert WINDOWS path semantics (separators, WinGet
    // package dirs); on ubuntu-latest they ran and failed. The windows-latest
    // leg still exercises them.
    #[cfg(windows)]
    #[test]
    fn candidates_are_path_first_then_known_locations() {
        let env = env_from(&[("ProgramFiles", "C:\\Program Files")]);
        let got = tool_candidates_with(
            "clang",
            Some("C:\\first-bin;C:\\second-bin"),
            &env,
            true,
        );
        assert!(got.len() >= 3, "PATH + Program Files candidates: {got:?}");
        assert_eq!(got[0].path, PathBuf::from("C:\\first-bin").join("clang.exe"));
        assert_eq!(got[1].path, PathBuf::from("C:\\second-bin").join("clang.exe"));
        assert!(got[0].from_path && got[1].from_path);
        assert!(
            got[2..].iter().all(|c| !c.from_path),
            "known locations must come after PATH entries: {got:?}"
        );
        assert!(got
            .iter()
            .any(|c| c.path == PathBuf::from("C:\\Program Files\\LLVM\\bin\\clang.exe")));
    }

    #[cfg(windows)]
    #[test]
    fn windows_llvm_locations_cover_winget_and_local_programs() {
        let env = env_from(&[
            ("LOCALAPPDATA", "C:\\Users\\alice\\AppData\\Local"),
            ("ProgramData", "C:\\ProgramData"),
            ("ProgramFiles", "C:\\Program Files"),
        ]);
        let got = known_locations("clang", &env, true);
        let as_str: Vec<String> = got.iter().map(|p| p.display().to_string()).collect();
        assert!(as_str.iter().any(|p| p.contains("WinGet\\Packages\\LLVM.LLVM_*")));
        assert!(as_str.iter().any(|p| p.ends_with("Programs\\LLVM\\bin\\clang.exe")));
        assert!(as_str.iter().any(|p| p.ends_with("WinGet\\Links\\clang.exe")));
    }

    #[test]
    fn windows_nasm_locations_do_not_hardcode_a_user_profile() {
        let env = env_from(&[
            ("LOCALAPPDATA", "C:\\Users\\alice\\AppData\\Local"),
            ("ProgramFiles", "C:\\Program Files"),
            ("ProgramFiles(x86)", "C:\\Program Files (x86)"),
        ]);
        let got = known_locations("nasm", &env, true);
        assert!(!got.is_empty());
        for path in &got {
            let text = path.display().to_string();
            assert!(!text.contains("lefte"), "personal path leaked: {text}");
        }
        // At least the LOCALAPPDATA-derived candidate must follow the user
        // profile; Program Files entries are machine-wide by design.
        assert!(
            got.iter()
                .any(|p| p.display().to_string().contains("alice")),
            "LOCALAPPDATA-relative path expected: {got:?}"
        );
        assert!(got.iter().any(|p| p.display().to_string().replace('\\', "/").ends_with("bin/NASM/nasm.exe")));
        assert!(got.iter().any(|p| p.display().to_string().replace('\\', "/").ends_with("Program Files/NASM/nasm.exe")));
    }

    #[test]
    fn unix_locations_include_homebrew_and_distro_llvm() {
        let env = env_from(&[]);
        let got = known_locations("clang", &env, false);
        // Compare by path components (display separators differ on Windows).
        let normalized: Vec<String> = got
            .iter()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .collect();
        assert!(normalized.contains(&"/usr/bin/clang".to_string()), "{normalized:?}");
        assert!(normalized.contains(&"/usr/local/bin/clang".to_string()));
        assert!(normalized.contains(&"/opt/homebrew/opt/llvm/bin/clang".to_string()));
        assert!(normalized.contains(&"/usr/lib/llvm-*/bin/clang".to_string()));
    }

    #[test]
    fn wildcard_expansion_matches_stem_and_keeps_the_tail() {
        let base = std::env::temp_dir().join(format!(
            "xiom_toolchain_glob_{}_{}",
            std::process::id(),
            std::line!()
        ));
        let a = base.join("llvm-14").join("bin");
        let b = base.join("llvm-18").join("bin");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(a.join("clang"), "x").unwrap();
        std::fs::write(b.join("clang"), "x").unwrap();
        std::fs::write(b.join("opt"), "x").unwrap();

        let pattern = base.join("llvm-*").join("bin").join("clang");
        let got = expand_wildcards(&pattern);
        assert_eq!(got.len(), 2, "expected two matches: {got:?}");
        assert!(got[0].ends_with("llvm-14/bin/clang") || got[0].ends_with("llvm-14\\bin\\clang"));
        assert!(got[1].ends_with("llvm-18/bin/clang") || got[1].ends_with("llvm-18\\bin\\clang"));

        // Two stars are unsupported: no matches, no panic.
        assert!(expand_wildcards(&base.join("llvm-*").join("*")).is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn version_line_parsing_skips_blanks() {
        assert_eq!(
            parse_first_version_line("\n  clang version 22.1.0\nTarget: x86_64"),
            Some("clang version 22.1.0".to_string())
        );
        assert_eq!(parse_first_version_line("   \n\t"), None);
        assert_eq!(
            parse_first_version_line("NASM version 2.16.01 compiled on Dec 21 2022"),
            Some("NASM version 2.16.01 compiled on Dec 21 2022".to_string())
        );
    }

    #[test]
    fn probe_skips_unextractable_files_and_returns_the_first_runnable() {
        let base = std::env::temp_dir().join(format!(
            "xiom_toolchain_probe_{}_{}",
            std::process::id(),
            std::line!()
        ));
        std::fs::create_dir_all(&base).unwrap();
        let missing = Candidate {
            path: base.join("missing.exe"),
            from_path: true,
        };
        let not_runnable = base.join("not-runnable.exe");
        std::fs::write(&not_runnable, b"not a program").unwrap();
        let candidates = vec![
            missing.clone(),
            Candidate {
                path: not_runnable.clone(),
                from_path: true,
            },
        ];
        assert!(probe_candidates("clang", &candidates).is_none());
        let _ = std::fs::remove_dir_all(&base);
    }
}
