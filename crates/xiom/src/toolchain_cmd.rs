// XIOM toolchain updater (docs/POST_RELEASE_PLAN.md section 1, D-2 step 4).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `xiom toolchain check` reports `current vs latest` for the installed
// toolchain. Source is the GitHub Releases API for `xiom-lang/xiom` ONLY
// (spec rule 1); no git channel, no mirrors. The command is read-only:
// it never downloads or writes anything.
//
// Exit codes (spec rule 6): 0 ok/up-to-date, 1 update available,
// 2 verification failed (network/API/parse), 3 permission/install-kind.
//
// `install_kind` detects package-manager installs (spec rule 5) from the exe
// path and an optional marker file so a future `update` can refuse politely;
// `check` merely reports it.
//
// `parse_latest_release` is split from the HTTP call so it is unit-testable
// against a canned payload (no network in tests).

use std::path::Path;

/// Where the toolchain came from, as far as we can tell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallKind {
    /// Installed by install.ps1/install.sh (or a release archive).
    Standalone,
    /// Running from a cargo build directory (developer build).
    Dev,
    /// Owned by a package manager (spec rule 5: refuse to update).
    PackageManager,
}

impl InstallKind {
    pub fn as_str(self) -> &'static str {
        match self {
            InstallKind::Standalone => "standalone",
            InstallKind::Dev => "dev",
            InstallKind::PackageManager => "package-manager",
        }
    }
}

/// The result of `xiom toolchain check`.
#[derive(Debug, Clone)]
pub struct CheckReport {
    pub current: String,
    pub latest: String,
    pub platform: String,
    pub asset: Option<String>,
    pub up_to_date: bool,
    pub notes: String,
    pub install_kind: InstallKind,
    pub exe: Option<String>,
}

/// `x86_64-pc-windows-msvc`-style platform id used for the release assets.
pub fn platform_id() -> String {
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "unknown"
    };
    let os = if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unknown"
    };
    format!("{os}-{arch}")
}

/// Release asset name for a platform/version, matching release.yml staging:
/// `xiom-<ver>-windows-x64.zip`, `xiom-<ver>-linux-x64.tar.gz`.
pub fn asset_name(version: &str, platform: &str) -> Option<String> {
    match platform {
        "windows-x64" => Some(format!("xiom-{version}-windows-x64.zip")),
        "linux-x64" => Some(format!("xiom-{version}-linux-x64.tar.gz")),
        _ => None,
    }
}

/// SHA256SUMS asset name published next to the archive.
pub fn sums_name(platform: &str) -> Option<String> {
    match platform {
        "windows-x64" => Some("SHA256SUMS-windows-x64".to_string()),
        "linux-x64" => Some("SHA256SUMS-linux-x64".to_string()),
        _ => None,
    }
}

/// Semver-ish comparison: true when `latest` is strictly newer than
/// `current`. Non-numeric suffixes after '-' are ignored; a build with
/// version `0.61.3` against release `0.61.3` is up to date.
pub fn is_newer(latest: &str, current: &str) -> bool {
    fn parse(v: &str) -> Vec<u64> {
        v.trim()
            .trim_start_matches('v')
            .split(['-', '+'])
            .next()
            .unwrap_or("")
            .split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    }
    let (a, b) = (parse(latest), parse(current));
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        if x != y {
            return x > y;
        }
    }
    false
}

/// Detect the install kind from the exe path (spec rule 5) plus an optional
/// `<root>/.xiom-package-manager` marker written by distro packagers.
pub fn install_kind(exe: Option<&Path>) -> InstallKind {
    let Some(exe) = exe else {
        return InstallKind::Standalone;
    };
    let text = exe.to_string_lossy().to_ascii_lowercase();
    // Package-manager trees (spec rule 5 path heuristic).
    for marker in [
        "/usr/bin/",
        "/usr/local/bin/",
        "/opt/homebrew/",
        "/nix/store/",
        "\\chocolatey\\",
        "\\scoop\\",
        "\\winget\\",
        "\\program files\\xiom",
    ] {
        if text.contains(marker) {
            return InstallKind::PackageManager;
        }
    }
    // A marker file next to the install root wins over heuristics.
    if let Some(root) = super::doctor::install_root_of(exe) {
        if root.join(".xiom-package-manager").exists() {
            return InstallKind::PackageManager;
        }
    }
    if text.contains("\\target\\debug\\")
        || text.contains("\\target\\release\\")
        || text.contains("/target/debug/")
        || text.contains("/target/release/")
    {
        return InstallKind::Dev;
    }
    InstallKind::Standalone
}

/// Parse the GitHub "latest release" payload into (tag, html_url).
pub fn parse_latest_release(body: &str) -> Result<(String, String), String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("invalid GitHub response: {e}"))?;
    let tag = value
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "GitHub response has no tag_name".to_string())?
        .to_string();
    let url = value
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok((tag, url))
}

/// GitHub Releases API endpoint (spec rule 1: this repository only).
pub const RELEASES_API: &str = "https://api.github.com/repos/xiom-lang/xiom/releases/latest";

/// Test/mirror override; empty/unset means the official API. Documented as
/// toolchain-mirror support only -- it never changes WHICH repository the
/// release is expected to come from.
pub fn releases_api() -> String {
    std::env::var("XIOM_TOOLCHAIN_API").unwrap_or_else(|_| RELEASES_API.to_string())
}

/// Fetch the latest release tag + page (network).
pub fn fetch_latest() -> Result<(String, String), String> {
    let response = ureq::get(&releases_api())
        .set("User-Agent", "xiom-toolchain-check")
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("GitHub Releases API unreachable: {e}"))?;
    let body = response
        .into_string()
        .map_err(|e| format!("cannot read GitHub response: {e}"))?;
    parse_latest_release(&body)
}

/// Build the report (one network call).
pub fn check() -> Result<CheckReport, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let (tag, url) = fetch_latest()?;
    let latest = tag.trim_start_matches('v').to_string();
    let platform = platform_id();
    let exe = std::env::current_exe().ok();
    Ok(CheckReport {
        up_to_date: !is_newer(&latest, &current),
        asset: asset_name(&latest, &platform),
        notes: url,
        install_kind: install_kind(exe.as_deref()),
        exe: exe.map(|p| p.display().to_string()),
        current,
        latest,
        platform,
    })
}

/// Exit code for a check result (spec rule 6).
pub fn check_exit_code(report: &CheckReport) -> i32 {
    if report.up_to_date {
        0
    } else {
        1
    }
}

/// Human-readable check output.
pub fn render_check_text(report: &CheckReport) -> String {
    let mut out = String::new();
    out.push_str("XIOM toolchain check\n");
    out.push_str(&format!("  current:      v{}\n", report.current));
    out.push_str(&format!("  latest:       v{}\n", report.latest));
    out.push_str(&format!("  platform:     {}\n", report.platform));
    out.push_str(&format!("  install kind: {}\n", report.install_kind.as_str()));
    if report.up_to_date {
        out.push_str("  status:       up to date\n");
    } else {
        out.push_str("  status:       UPDATE AVAILABLE\n");
        if !report.notes.is_empty() {
            out.push_str(&format!("  notes:        {}\n", report.notes));
        }
        if report.install_kind == InstallKind::PackageManager {
            out.push_str("  note:         installed by a package manager -- update through it\n");
        } else if let Some(asset) = &report.asset {
            out.push_str(&format!("  asset:        {asset}\n"));
        }
    }
    out
}

/// `--json` check output (POST_RELEASE_PLAN shape: current, latest,
/// platform, up_to_date, notes -- plus install_kind/asset/exe).
pub fn render_check_json(report: &CheckReport) -> String {
    let value = serde_json::json!({
        "schema": 1,
        "current": report.current,
        "latest": report.latest,
        "platform": report.platform,
        "up_to_date": report.up_to_date,
        "notes": report.notes,
        "install_kind": report.install_kind.as_str(),
        "asset": report.asset,
        "exe": report.exe,
    });
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison_is_semver_ish() {
        assert!(is_newer("0.62.0", "0.61.3"));
        assert!(is_newer("0.61.10", "0.61.9"));
        assert!(is_newer("1.0.0", "0.99.99"));
        assert!(!is_newer("0.61.3", "0.61.3"));
        assert!(!is_newer("0.61.2", "0.61.3"));
        assert!(!is_newer("v0.61.3", "0.61.3"));
        assert!(is_newer("0.62.0-rc.1", "0.61.3"));
    }

    #[test]
    fn asset_names_match_release_staging() {
        assert_eq!(
            asset_name("0.62.0", "windows-x64").as_deref(),
            Some("xiom-0.62.0-windows-x64.zip")
        );
        assert_eq!(
            asset_name("0.62.0", "linux-x64").as_deref(),
            Some("xiom-0.62.0-linux-x64.tar.gz")
        );
        assert_eq!(asset_name("0.62.0", "macos-arm64"), None);
        assert_eq!(
            sums_name("windows-x64").as_deref(),
            Some("SHA256SUMS-windows-x64")
        );
        assert_eq!(sums_name("linux-x64").as_deref(), Some("SHA256SUMS-linux-x64"));
    }

    #[test]
    fn latest_release_parsing() {
        let body = r#"{"tag_name":"v0.62.0","html_url":"https://github.com/xiom-lang/xiom/releases/tag/v0.62.0"}"#;
        let (tag, url) = parse_latest_release(body).expect("parse");
        assert_eq!(tag, "v0.62.0");
        assert!(url.ends_with("v0.62.0"));
        assert!(parse_latest_release("{}").is_err());
        assert!(parse_latest_release("not json").is_err());
    }

    #[test]
    fn install_kind_heuristics() {
        assert_eq!(
            install_kind(Some(Path::new("/usr/bin/xiom"))),
            InstallKind::PackageManager
        );
        assert_eq!(
            install_kind(Some(Path::new("/opt/homebrew/bin/xiom"))),
            InstallKind::PackageManager
        );
        assert_eq!(
            install_kind(Some(Path::new("C:\\ProgramData\\chocolatey\\bin\\xiom.exe"))),
            InstallKind::PackageManager
        );
        assert_eq!(
            install_kind(Some(Path::new("/home/u/xiom/bin/xiom"))),
            InstallKind::Standalone
        );
        assert_eq!(
            install_kind(Some(Path::new("E:\\repo\\target\\debug\\xiom.exe"))),
            InstallKind::Dev
        );
        assert_eq!(install_kind(None), InstallKind::Standalone);
    }

    #[test]
    fn check_json_has_the_spec_keys() {
        let report = CheckReport {
            current: "0.61.3".into(),
            latest: "0.62.0".into(),
            platform: "windows-x64".into(),
            asset: asset_name("0.62.0", "windows-x64"),
            up_to_date: false,
            notes: "https://example.invalid".into(),
            install_kind: InstallKind::Standalone,
            exe: Some("C:\\xiom\\bin\\xiom.exe".into()),
        };
        let parsed: serde_json::Value = serde_json::from_str(&render_check_json(&report)).unwrap();
        for key in [
            "schema",
            "current",
            "latest",
            "platform",
            "up_to_date",
            "notes",
            "install_kind",
        ] {
            assert!(parsed.get(key).is_some(), "missing {key}: {parsed}");
        }
        assert_eq!(parsed["up_to_date"], serde_json::Value::Bool(false));
        assert_eq!(check_exit_code(&report), 1);
    }
}
