// XIOM doctor v2 (front-end audit FE-1..FE-5, FE-7, FE-16).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `xiom doctor` is the first command a new user runs. The audit found it
// could not see an LLVM that was not on PATH, pointed at a dead
// `xiom install llvm` command, ignored NASM, used its own stdlib path instead
// of the shared resolver, and had no machine-readable mode. This module is
// the report: it uses the SHARED toolchain probe (crate::toolchain) and the
// SHARED stdlib resolver (xiom_graph::paths::stdlib_root) so doctor and the
// compiler cannot disagree, emits an identity block, and maps status to exit
// codes (0 OK, 1 warnings, 2 errors) for installers/CI.
//
// `evaluate` is pure over a `DoctorInput` snapshot so every status/warning
// rule is unit-tested without the host machine.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::toolchain;

/// Operating-system family for remediation text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    MacOs,
    Linux,
    Other,
}

/// A resolved tool with its version banner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSnapshot {
    pub path: PathBuf,
    pub version: Option<String>,
}

/// Everything `evaluate` reasons about, captured from the machine.
#[derive(Debug, Clone)]
pub struct DoctorInput {
    pub compiler_version: String,
    pub exe: Option<PathBuf>,
    pub install_root: Option<PathBuf>,
    pub xiom_home: PathBuf,
    pub xiom_home_explicit: bool,
    pub stdlib_root: Option<PathBuf>,
    pub stdlib_version: Option<String>,
    /// The stdlib tag this compiler was built against (`STDLIB_VERSION`,
    /// embedded by build.rs); None on builds without it.
    pub stdlib_pin: Option<String>,
    pub stdlib_candidates: Vec<PathBuf>,
    pub clang: Option<ToolSnapshot>,
    pub opt: Option<ToolSnapshot>,
    pub nasm: Option<ToolSnapshot>,
    pub z3: Option<ToolSnapshot>,
    pub runtime_c: Option<PathBuf>,
    pub packages_dir: Option<PathBuf>,
    /// Every `xiom` binary found on PATH, in PATH order.
    pub xiom_on_path: Vec<PathBuf>,
    pub os: Os,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    Info,
    Warn,
    Error,
}

impl Status {
    fn tag(self) -> &'static str {
        match self {
            Status::Ok => "[OK]",
            Status::Info => "[--]",
            Status::Warn => "[!!]",
            Status::Error => "[!!]",
        }
    }

    fn json(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Info => "info",
            Status::Warn => "warn",
            Status::Error => "error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Check {
    pub name: &'static str,
    pub status: Status,
    pub detail: String,
    /// Follow-up lines (install commands, searched paths) shown when the
    /// check is not OK.
    pub remediation: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub input: DoctorInput,
    pub checks: Vec<Check>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Gather the machine state and evaluate it.
pub fn build_report() -> DoctorReport {
    evaluate(gather())
}

/// Capture the host state for one doctor run.
pub fn gather() -> DoctorInput {
    let compiler_version = env!("CARGO_PKG_VERSION").to_string();
    let exe = std::env::current_exe().ok();
    let install_root = exe.as_deref().and_then(install_root_of);
    let xiom_home_explicit = std::env::var("XIOM_HOME")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let xiom_home = xiom_graph::paths::xiom_home();
    let stdlib_root = xiom_graph::paths::stdlib_root();
    let stdlib_version = stdlib_root
        .as_deref()
        .and_then(|root| manifest_version(&root.join("package.xi")));
    let stdlib_pin = option_env!("XIOM_STDLIB_PIN").map(pin_version);
    let stdlib_candidates = xiom_graph::paths::current_stdlib_candidates();
    let clang = toolchain::probe_clang().map(snapshot);
    let opt = toolchain::probe_opt().map(snapshot);
    let nasm = toolchain::probe_nasm().map(snapshot);
    let z3 = xiom_verify::Z3Runner::find_z3().map(|path| {
        // The verifier may return a bare `z3.exe` (PATH hit); report the
        // absolute entry like the clang/nasm probe does.
        let resolved = if Path::new(&path).components().count() > 1 {
            PathBuf::from(&path)
        } else {
            toolchain::path_lookup(&path).unwrap_or_else(|| PathBuf::from(&path))
        };
        ToolSnapshot {
            version: toolchain::version_of(&resolved),
            path: resolved,
        }
    });
    let runtime_c = crate::find_runtime_c().map(PathBuf::from);
    let packages_dir = Some(xiom_home.join("packages"));
    let xiom_on_path = xiom_path_bins(std::env::var("PATH").ok().as_deref());

    DoctorInput {
        compiler_version,
        exe,
        install_root,
        xiom_home,
        xiom_home_explicit,
        stdlib_root,
        stdlib_version,
        stdlib_pin,
        stdlib_candidates,
        clang,
        opt,
        nasm,
        z3,
        runtime_c,
        packages_dir,
        xiom_on_path,
        os: host_os(),
    }
}

/// Pure evaluation: checks, warnings, errors. Unit-tested over synthetic
/// inputs (see the tests at the bottom).
pub fn evaluate(input: DoctorInput) -> DoctorReport {
    let mut checks: Vec<Check> = Vec::new();

    // Required: clang (FE-1/FE-2). Missing clang is an error with the exact
    // per-OS install command -- never the retired `xiom install llvm`.
    match &input.clang {
        Some(t) => checks.push(Check {
            name: "clang",
            status: Status::Ok,
            detail: clang_detail(t),
            remediation: Vec::new(),
        }),
        None => checks.push(Check {
            name: "clang",
            status: Status::Error,
            detail: "clang not found".to_string(),
            remediation: clang_remediation(input.os),
        }),
    }

    // Optional: NASM (FE-3). The driver uses it for stdlib asm acceleration;
    // C fallbacks keep everything working without it.
    match &input.nasm {
        Some(t) => checks.push(Check {
            name: "nasm",
            status: Status::Ok,
            detail: format!("{} at {}", tool_version_text(t), t.path.display()),
            remediation: Vec::new(),
        }),
        None => checks.push(Check {
            name: "nasm",
            status: Status::Info,
            detail: "nasm not found (optional -- C fallbacks in use)".to_string(),
            remediation: nasm_remediation(input.os),
        }),
    }

    // Optional: z3 (contract verification; release archives bundle it).
    match &input.z3 {
        Some(t) => checks.push(Check {
            name: "z3",
            status: Status::Ok,
            detail: format!("{} at {}", tool_version_text(t), t.path.display()),
            remediation: Vec::new(),
        }),
        None => checks.push(Check {
            name: "z3",
            status: Status::Info,
            detail: "z3 not found (release archives bundle bin/z3)".to_string(),
            remediation: Vec::new(),
        }),
    }

    // Stdlib (FE-4/FE-5/FE-16): the SHARED resolver's root, plus version
    // parity against the compiler and against the embedded pin.
    match &input.stdlib_root {
        Some(root) => {
            let version = input.stdlib_version.as_deref();
            checks.push(Check {
                name: "stdlib",
                status: Status::Ok,
                detail: format!(
                    "stdlib {} at {}",
                    version.map(|v| format!("v{v}")).unwrap_or_else(|| "(version unknown)".to_string()),
                    root.display()
                ),
                remediation: Vec::new(),
            });
            if let Some(version) = version {
                if version != input.compiler_version {
                    checks.push(Check {
                        name: "stdlib-version",
                        status: Status::Warn,
                        detail: format!(
                            "stdlib version {version} does not match compiler {} -- re-run the installer so bin/ and lib/ ship together",
                            input.compiler_version
                        ),
                        remediation: Vec::new(),
                    });
                }
                if let Some(pin) = input.stdlib_pin.as_deref() {
                    if version != pin {
                        checks.push(Check {
                            name: "stdlib-pin",
                            status: Status::Warn,
                            detail: format!(
                                "stdlib version {version} does not match the compiler's pinned stdlib {pin} -- this install mixes versions"
                            ),
                            remediation: Vec::new(),
                        });
                    }
                }
            }
        }
        None => {
            let mut remediation =
                vec!["re-run the installer for your platform (it ships bin/ + lib/ together)".to_string()];
            remediation.extend(searched_stdlib_lines(&input.stdlib_candidates));
            checks.push(Check {
                name: "stdlib",
                status: Status::Error,
                detail: "stdlib not found".to_string(),
                remediation,
            });
        }
    }

    // Runtime C source (FE-4): required for native linking.
    match &input.runtime_c {
        Some(path) => checks.push(Check {
            name: "runtime",
            status: Status::Ok,
            detail: format!("runtime C source at {}", path.display()),
            remediation: Vec::new(),
        }),
        None => checks.push(Check {
            name: "runtime",
            status: Status::Warn,
            detail: "runtime C source (xiom_runtime.c) not found -- native linking may fail"
                .to_string(),
            remediation: vec![
                "re-run the installer for your platform (lib/runtime ships with the toolchain)"
                    .to_string(),
            ],
        }),
    }

    // Identity-only lines.
    checks.push(Check {
        name: "XIOM_HOME",
        status: Status::Info,
        detail: format!("XIOM_HOME={}", input.xiom_home.display()),
        remediation: Vec::new(),
    });
    let packages = input
        .packages_dir
        .as_deref()
        .filter(|p| p.is_dir())
        .map(|p| format!("packages directory at {}", p.display()))
        .unwrap_or_else(|| "No packages (use: xiom pkg install <name>)".to_string());
    let packages_status = if input
        .packages_dir
        .as_deref()
        .map(|p| p.is_dir())
        .unwrap_or(false)
    {
        Status::Ok
    } else {
        Status::Info
    };
    checks.push(Check {
        name: "packages",
        status: packages_status,
        detail: packages,
        remediation: Vec::new(),
    });

    // Duplicate installs (FE-4/FE-13): the launcher/precedence trap.
    if input.xiom_on_path.len() > 1 {
        let list = input
            .xiom_on_path
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        checks.push(Check {
            name: "install-precedence",
            status: Status::Warn,
            detail: format!("multiple xiom binaries on PATH (the first one wins): {list}"),
            remediation: vec!["keep one install, or reorder PATH so the intended install is first"
                .to_string()],
        });
    } else if let (Some(first), Some(exe)) = (input.xiom_on_path.first(), input.exe.as_ref()) {
        if !same_path(first, exe) {
            checks.push(Check {
                name: "install-precedence",
                status: Status::Warn,
                detail: format!(
                    "the running xiom ({}) is not the first on PATH; 'xiom' resolves to {}",
                    exe.display(),
                    first.display()
                ),
                remediation: vec!["reorder PATH or remove the stale install".to_string()],
            });
        }
    }

    let warnings: Vec<String> = checks
        .iter()
        .filter(|c| c.status == Status::Warn)
        .map(|c| c.detail.clone())
        .collect();
    let errors: Vec<String> = checks
        .iter()
        .filter(|c| c.status == Status::Error)
        .map(|c| c.detail.clone())
        .collect();

    DoctorReport {
        input,
        checks,
        warnings,
        errors,
    }
}

/// 0 = all OK, 1 = warnings, 2 = errors (FE-7).
pub fn exit_code(report: &DoctorReport) -> i32 {
    if !report.errors.is_empty() {
        2
    } else if !report.warnings.is_empty() {
        1
    } else {
        0
    }
}

/// Human-readable report (the historical doctor format, plus the identity
/// block and WARNINGS/ERRORS sections).
pub fn render_text(report: &DoctorReport) -> String {
    let input = &report.input;
    let mut out = String::new();
    let _ = writeln!(out, "XIOM Doctor v{}", input.compiler_version);
    let _ = writeln!(out, "====================");
    let _ = writeln!(out);
    let _ = writeln!(out, "IDENTITY");
    let _ = writeln!(
        out,
        "  {:<13} {}",
        "xiom.exe",
        input
            .exe
            .as_deref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(unknown)".to_string())
    );
    let _ = writeln!(
        out,
        "  {:<13} {}",
        "install root",
        input
            .install_root
            .as_deref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(unknown)".to_string())
    );
    let _ = writeln!(out, "  {:<13} {}", "XIOM_HOME", input.xiom_home.display());
    let _ = writeln!(
        out,
        "  {:<13} {}",
        "stdlib root",
        match (&input.stdlib_root, &input.stdlib_version) {
            (Some(root), Some(v)) => format!("{} (v{v})", root.display()),
            (Some(root), None) => format!("{} (version unknown)", root.display()),
            (None, _) => "(not found)".to_string(),
        }
    );
    let _ = writeln!(
        out,
        "  {:<13} {}",
        "clang",
        input
            .clang
            .as_ref()
            .map(clang_detail)
            .unwrap_or_else(|| "(not found)".to_string())
    );
    let _ = writeln!(
        out,
        "  {:<13} {}",
        "nasm",
        input
            .nasm
            .as_ref()
            .map(|t| format!("{} at {}", tool_version_text(t), t.path.display()))
            .unwrap_or_else(|| "(not found -- optional)".to_string())
    );
    let _ = writeln!(
        out,
        "  {:<13} {}",
        "z3",
        input
            .z3
            .as_ref()
            .map(|t| format!("{} at {}", tool_version_text(t), t.path.display()))
            .unwrap_or_else(|| "(not found -- optional)".to_string())
    );
    let _ = writeln!(
        out,
        "  {:<13} {}",
        "runtime C",
        input
            .runtime_c
            .as_deref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(not found)".to_string())
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "CHECKS");
    for check in &report.checks {
        let _ = writeln!(out, "  {} {}", check.status.tag(), check.detail);
        if check.status != Status::Ok {
            for line in &check.remediation {
                let _ = writeln!(out, "      {line}");
            }
        }
    }
    if !report.warnings.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "WARNINGS ({})", report.warnings.len());
        for w in &report.warnings {
            let _ = writeln!(out, "  - {w}");
        }
    }
    if !report.errors.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "ERRORS ({})", report.errors.len());
        for e in &report.errors {
            let _ = writeln!(out, "  - {e}");
        }
    }
    out
}

/// Machine-readable report (FE-7). `{ ok, warnings[], errors[] }` plus the
/// identity the installers/CI wanted to assert on.
pub fn render_json(report: &DoctorReport) -> String {
    let input = &report.input;
    let tool_json = |t: &Option<ToolSnapshot>| match t {
        Some(t) => json!({
            "path": t.path.display().to_string(),
            "version": t.version,
        }),
        None => serde_json::Value::Null,
    };
    let checks: Vec<serde_json::Value> = report
        .checks
        .iter()
        .map(|c| {
            json!({
                "name": c.name,
                "status": c.status.json(),
                "detail": c.detail,
                "remediation": c.remediation,
            })
        })
        .collect();
    let value = json!({
        "schema": 1,
        "ok": report.errors.is_empty() && report.warnings.is_empty(),
        "compiler": {
            "version": input.compiler_version,
            "exe": input.exe.as_ref().map(|p| p.display().to_string()),
            "install_root": input.install_root.as_ref().map(|p| p.display().to_string()),
        },
        "identity": {
            "exe": input.exe.as_ref().map(|p| p.display().to_string()),
            "install_root": input.install_root.as_ref().map(|p| p.display().to_string()),
            "xiom_home": input.xiom_home.display().to_string(),
            "stdlib_root": input.stdlib_root.as_ref().map(|p| p.display().to_string()),
            "stdlib_version": input.stdlib_version,
            "stdlib_pin": input.stdlib_pin,
            "clang": tool_json(&input.clang),
            "opt": tool_json(&input.opt),
            "nasm": tool_json(&input.nasm),
            "z3": tool_json(&input.z3),
            "runtime_c": input.runtime_c.as_ref().map(|p| p.display().to_string()),
            "packages_dir": input.packages_dir.as_ref().map(|p| p.display().to_string()),
        },
        "checks": checks,
        "warnings": report.warnings,
        "errors": report.errors,
    });
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
}

/// Current OS family.
pub fn host_os() -> Os {
    if cfg!(windows) {
        Os::Windows
    } else if cfg!(target_os = "macos") {
        Os::MacOs
    } else if cfg!(target_os = "linux") {
        Os::Linux
    } else {
        Os::Other
    }
}

/// Exact per-OS commands to install clang/LLVM (FE-2). Deliberately contains
/// no `xiom install` reference: that command is retired and there is no
/// `llvm` package in the registry.
pub fn clang_remediation(os: Os) -> Vec<String> {
    match os {
        Os::Windows => vec![
            "install LLVM: winget install LLVM.LLVM".to_string(),
            "  or (Chocolatey): choco install llvm -y".to_string(),
            "then open a NEW terminal and re-run 'xiom doctor'".to_string(),
            "releases: https://github.com/xiom-lang/xiom/releases/latest".to_string(),
        ],
        Os::MacOs => vec![
            "install LLVM: brew install llvm".to_string(),
            "add it to PATH: export PATH=\"/opt/homebrew/opt/llvm/bin:$PATH\"".to_string(),
            "then re-run 'xiom doctor'".to_string(),
        ],
        Os::Linux => vec![
            linux_clang_install_line().to_string(),
            "then re-run 'xiom doctor'".to_string(),
        ],
        Os::Other => vec![
            "install clang/LLVM with your package manager, then re-run 'xiom doctor'".to_string(),
        ],
    }
}

/// Optional NASM install commands (FE-3).
pub fn nasm_remediation(os: Os) -> Vec<String> {
    match os {
        Os::Windows => vec![
            "optional: winget install NASM.NASM".to_string(),
            "  or (Chocolatey): choco install nasm -y".to_string(),
        ],
        Os::MacOs => vec!["optional: brew install nasm".to_string()],
        Os::Linux => vec![linux_nasm_install_line().to_string()],
        Os::Other => vec!["optional: install nasm with your package manager".to_string()],
    }
}

fn linux_clang_install_line() -> &'static str {
    let id = linux_distro_id();
    if id.contains("debian") || id.contains("ubuntu") || id.contains("mint") {
        "install clang: sudo apt install clang"
    } else if id.contains("fedora") || id.contains("rhel") || id.contains("centos") {
        "install clang: sudo dnf install clang"
    } else if id.contains("arch") || id.contains("manjaro") {
        "install clang: sudo pacman -S clang"
    } else if id.contains("suse") {
        "install clang: sudo zypper install clang"
    } else {
        "install clang with your package manager (apt/dnf/pacman/zypper)"
    }
}

fn linux_nasm_install_line() -> String {
    let id = linux_distro_id();
    if id.contains("debian") || id.contains("ubuntu") || id.contains("mint") {
        "optional: sudo apt install nasm".to_string()
    } else if id.contains("fedora") || id.contains("rhel") || id.contains("centos") {
        "optional: sudo dnf install nasm".to_string()
    } else if id.contains("arch") || id.contains("manjaro") {
        "optional: sudo pacman -S nasm".to_string()
    } else if id.contains("suse") {
        "optional: sudo zypper install nasm".to_string()
    } else {
        "optional: install nasm with your package manager".to_string()
    }
}

/// Lowercased `/etc/os-release` contents (empty elsewhere).
fn linux_distro_id() -> String {
    std::fs::read_to_string("/etc/os-release")
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// `clang version 22.1.0 at C:\...` (never a duplicate "clang clang").
fn clang_detail(t: &ToolSnapshot) -> String {
    match &t.version {
        Some(v) if v.to_ascii_lowercase().contains("clang") => {
            format!("{v} at {}", t.path.display())
        }
        Some(v) => format!("clang {v} at {}", t.path.display()),
        None => format!("clang (version unknown) at {}", t.path.display()),
    }
}

fn tool_version_text(t: &ToolSnapshot) -> String {
    t.version
        .clone()
        .unwrap_or_else(|| "(version unknown)".to_string())
}

fn snapshot(t: toolchain::ToolInfo) -> ToolSnapshot {
    ToolSnapshot {
        path: t.path,
        version: t.version,
    }
}

/// Install root: the parent of `bin/` for a release layout, else the exe dir.
fn install_root_of(exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?;
    let is_bin = dir
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.eq_ignore_ascii_case("bin"))
        .unwrap_or(false);
    if is_bin {
        Some(dir.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| dir.to_path_buf()))
    } else {
        Some(dir.to_path_buf())
    }
}

/// `version: "0.61.3";` -> `0.61.3` (first version field in a package.xi).
pub fn manifest_version(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    text.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("version:"))
        .map(|v| v.trim().trim_end_matches(';').trim().trim_matches('"').to_string())
        .filter(|v| !v.is_empty())
}

/// `stdlib-v0.61.3` -> `0.61.3`.
fn pin_version(pin: &str) -> String {
    pin.trim()
        .strip_prefix("stdlib-v")
        .unwrap_or(pin.trim())
        .to_string()
}

fn searched_stdlib_lines(candidates: &[PathBuf]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for candidate in candidates.iter().take(6) {
        lines.push(format!("searched: {}", candidate.display()));
    }
    if candidates.len() > 6 {
        lines.push(format!("... and {} more candidate locations", candidates.len() - 6));
    }
    lines
}

/// Every `xiom` executable on PATH, in PATH order (FE-4/FE-13).
fn xiom_path_bins(path_env: Option<&str>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let names: &[&str] = if cfg!(windows) { &["xiom.exe"] } else { &["xiom"] };
    if let Some(path_env) = path_env {
        for dir in std::env::split_paths(path_env) {
            if dir.as_os_str().is_empty() {
                continue;
            }
            for name in names {
                let candidate = dir.join(name);
                if candidate.is_file() && !out.iter().any(|p| same_path(p, &candidate)) {
                    out.push(candidate);
                }
            }
        }
    }
    out
}

fn same_path(a: &Path, b: &Path) -> bool {
    let ca = std::fs::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    let cb = std::fs::canonicalize(b).unwrap_or_else(|_| b.to_path_buf());
    if cfg!(windows) {
        ca.to_string_lossy()
            .eq_ignore_ascii_case(&cb.to_string_lossy())
    } else {
        ca == cb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(path: &str, version: &str) -> ToolSnapshot {
        ToolSnapshot {
            path: PathBuf::from(path),
            version: Some(version.to_string()),
        }
    }

    fn base_input() -> DoctorInput {
        DoctorInput {
            compiler_version: "0.61.3".to_string(),
            exe: Some(PathBuf::from("C:\\xiom\\bin\\xiom.exe")),
            install_root: Some(PathBuf::from("C:\\xiom")),
            xiom_home: PathBuf::from("C:\\Users\\alice\\AppData\\Local\\xiom"),
            xiom_home_explicit: false,
            stdlib_root: Some(PathBuf::from("C:\\xiom\\lib")),
            stdlib_version: Some("0.61.3".to_string()),
            stdlib_pin: Some("0.61.3".to_string()),
            stdlib_candidates: vec![PathBuf::from("C:\\xiom\\lib")],
            clang: Some(tool(
                "C:\\Program Files\\LLVM\\bin\\clang.exe",
                "clang version 22.1.0",
            )),
            opt: None,
            nasm: None,
            z3: None,
            runtime_c: Some(PathBuf::from("C:\\xiom\\lib\\runtime\\xiom_runtime.c")),
            packages_dir: Some(PathBuf::from("C:\\Users\\alice\\AppData\\Local\\xiom\\packages")),
            xiom_on_path: Vec::new(),
            os: Os::Windows,
        }
    }

    #[test]
    fn clean_install_is_ok_and_exit_zero() {
        let report = evaluate(base_input());
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(exit_code(&report), 0);
    }

    #[test]
    fn missing_clang_is_an_error_with_os_specific_commands() {
        let mut input = base_input();
        input.clang = None;
        let report = evaluate(input);
        assert_eq!(exit_code(&report), 2);
        assert!(report.errors.iter().any(|e| e.contains("clang not found")));
        let check = report.checks.iter().find(|c| c.name == "clang").unwrap();
        assert!(check
            .remediation
            .iter()
            .any(|r| r.contains("winget install LLVM.LLVM")));
        // FE-2: never the retired package channel.
        let text = render_text(&report);
        assert!(!text.contains("xiom install"), "dead command leaked: {text}");
    }

    #[test]
    fn stdlib_version_mismatch_is_a_warning_with_exit_one() {
        let mut input = base_input();
        input.stdlib_version = Some("0.0.1".to_string());
        let report = evaluate(input);
        assert_eq!(exit_code(&report), 1);
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("stdlib version 0.0.1 does not match compiler 0.61.3")));
    }

    #[test]
    fn stdlib_pin_mismatch_is_a_warning() {
        let mut input = base_input();
        input.stdlib_version = Some("0.61.2".to_string());
        input.stdlib_pin = Some("0.61.3".to_string());
        let report = evaluate(input);
        assert_eq!(exit_code(&report), 1);
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("does not match the compiler's pinned stdlib 0.61.3")));
    }

    #[test]
    fn missing_stdlib_lists_searched_candidates() {
        let mut input = base_input();
        input.stdlib_root = None;
        input.stdlib_version = None;
        input.stdlib_candidates = vec![
            PathBuf::from("C:\\xiom\\stdlib"),
            PathBuf::from("C:\\xiom\\lib"),
        ];
        let report = evaluate(input);
        assert_eq!(exit_code(&report), 2);
        let check = report.checks.iter().find(|c| c.name == "stdlib").unwrap();
        assert!(check.remediation.iter().any(|r| r.contains("C:\\xiom\\lib")));
    }

    #[test]
    fn multiple_installs_warn_and_name_both_paths() {
        let mut input = base_input();
        input.exe = Some(PathBuf::from("D:\\fresh\\bin\\xiom.exe"));
        input.xiom_on_path = vec![
            PathBuf::from("C:\\Users\\alice\\AppData\\Local\\xiom\\bin\\xiom.exe"),
            PathBuf::from("D:\\fresh\\bin\\xiom.exe"),
        ];
        let report = evaluate(input);
        assert_eq!(exit_code(&report), 1);
        let warning = report
            .warnings
            .iter()
            .find(|w| w.contains("multiple xiom binaries on PATH"))
            .expect("multi-install warning");
        assert!(warning.contains("C:\\Users\\alice\\AppData\\Local\\xiom\\bin\\xiom.exe"));
        assert!(warning.contains("D:\\fresh\\bin\\xiom.exe"));
    }

    #[test]
    fn running_binary_shadowed_by_path_install_warns() {
        let mut input = base_input();
        input.exe = Some(PathBuf::from("D:\\fresh\\bin\\xiom.exe"));
        input.xiom_on_path = vec![PathBuf::from(
            "C:\\Users\\alice\\AppData\\Local\\xiom\\bin\\xiom.exe",
        )];
        let report = evaluate(input);
        assert_eq!(exit_code(&report), 1);
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("not the first on PATH")));
    }

    #[test]
    fn json_has_the_frozen_top_level_keys() {
        let report = evaluate(base_input());
        let parsed: serde_json::Value = serde_json::from_str(&render_json(&report)).unwrap();
        for key in ["schema", "ok", "compiler", "identity", "checks", "warnings", "errors"] {
            assert!(parsed.get(key).is_some(), "missing key {key}: {parsed}");
        }
        assert_eq!(parsed["ok"], serde_json::Value::Bool(true));
        assert_eq!(
            parsed["identity"]["clang"]["version"],
            serde_json::Value::String("clang version 22.1.0".to_string())
        );
    }

    #[test]
    fn manifest_version_reads_the_package_xi_field() {
        let base = std::env::temp_dir().join(format!(
            "xiom_doctor_manifest_{}_{}",
            std::process::id(),
            line!()
        ));
        std::fs::create_dir_all(&base).unwrap();
        let manifest = base.join("package.xi");
        std::fs::write(
            &manifest,
            "package xiom_std {\n  name: \"xiom-std\";\n  version: \"0.61.3\";\n}\n",
        )
        .unwrap();
        assert_eq!(manifest_version(&manifest).as_deref(), Some("0.61.3"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn text_report_contains_the_ok_lines_for_present_tools() {
        let report = evaluate(base_input());
        let text = render_text(&report);
        assert!(text.contains("[OK] clang version 22.1.0 at C:\\Program Files\\LLVM\\bin\\clang.exe"));
        assert!(text.contains("IDENTITY"));
        assert!(text.contains("install root"));
    }
}
