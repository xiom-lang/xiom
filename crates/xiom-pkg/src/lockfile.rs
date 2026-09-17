// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// XIOM Package Manager -- lockfile v2 (Stage 5 supply chain).
//
// Audit/plan requirement: "lockfile v2 pinning {name, version, integrity,
// source} transitively". v1 (a bare {name: version} map) pinned versions
// only; an artifact could be swapped at the registry after locking. v2
// records the sha256 digest that `xiom pkg install` then ENFORCES, so the
// bytes installed are exactly the bytes that were locked.
//
// The format is deliberately JSON (no yaml dependency) and deterministic
// (BTreeMap ordering) so the file diffs cleanly in review.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LockedPackage {
    pub version: String,
    /// Where the artifact comes from: "registry" (default) or "git:<url>@<rev>".
    pub source: String,
    /// "sha256-<lowercase hex>"; empty when the digest could not be resolved
    /// at lock time (install refuses it unless the override is set).
    pub integrity: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Lockfile {
    #[serde(rename = "lockfileVersion")]
    pub lockfile_version: u32,
    /// Root package name/version the lock was generated for.
    pub package: String,
    pub version: String,
    pub packages: BTreeMap<String, LockedPackage>,
}

/// Lowercase hex sha256 of an archive.
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Format an integrity string from a digest.
pub fn integrity_for(data: &[u8]) -> String {
    format!("sha256-{}", sha256_hex(data))
}

/// Normalize a digest: bare hex gets the "sha256-" prefix; prefixed values
/// are lowercased. Empty input stays empty.
pub fn normalize_integrity(digest: &str) -> String {
    let d = digest.trim();
    if d.is_empty() {
        return String::new();
    }
    let hex = d.strip_prefix("sha256-").or_else(|| d.strip_prefix("sha256:")).unwrap_or(d);
    format!("sha256-{}", hex.to_lowercase())
}

impl Lockfile {
    /// Build a v2 lock from a manifest's direct dependencies.
    /// `resolve` maps (name, version-req) -> resolved version; `integrity`
    /// maps (name, resolved version) -> digest (bare hex or "sha256-<hex>").
    /// Digests normally come from the registry index (the server publishes
    /// them); a lock made while offline records an empty digest, which
    /// `install` refuses until it is regenerated with the registry up.
    pub fn build(
        package: &str,
        version: &str,
        deps: &[(String, String)],
        resolve: &dyn Fn(&str, &str) -> Option<String>,
        integrity: &dyn Fn(&str, &str) -> Option<String>,
    ) -> Lockfile {
        let mut packages = BTreeMap::new();
        for (name, req) in deps {
            let resolved = resolve(name, req).unwrap_or_else(|| req.clone());
            let digest = integrity(name, &resolved)
                .map(|d| normalize_integrity(&d))
                .unwrap_or_default();
            packages.insert(name.clone(), LockedPackage {
                version: resolved,
                source: "registry".to_string(),
                integrity: digest,
            });
        }
        Lockfile {
            lockfile_version: 2,
            package: package.to_string(),
            version: version.to_string(),
            packages,
        }
    }

    /// Build a v2 lock from an already-resolved closure
    /// `(name, version, source, integrity)`.
    ///
    /// Stage 5 transitive locking: the caller walks the registry index and
    /// passes every package in the closure, so the lock pins
    /// `{name, version, integrity, source}` for INDIRECT dependencies too.
    pub fn from_resolved(
        package: &str,
        version: &str,
        resolved: Vec<(String, String, String, String)>,
    ) -> Lockfile {
        let mut packages = BTreeMap::new();
        for (name, ver, source, integrity) in resolved {
            packages.insert(name, LockedPackage {
                version: ver,
                source,
                integrity: normalize_integrity(&integrity),
            });
        }
        Lockfile {
            lockfile_version: 2,
            package: package.to_string(),
            version: version.to_string(),
            packages,
        }
    }

    pub fn to_json(&self) -> String {
        // Pretty + trailing newline: reviewable diffs.
        let mut out = serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string());
        out.push('\n');
        out
    }

    /// Parse a lockfile. v1 files (no lockfileVersion) are rejected so callers
    /// can fall back with a "regenerate" warning instead of mis-reading them.
    pub fn parse(text: &str) -> Result<Lockfile, String> {
        let value: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| format!("xiom.lock is not valid JSON: {e}"))?;
        match value.get("lockfileVersion").and_then(|v| v.as_u64()) {
            Some(2) => serde_json::from_value(value)
                .map_err(|e| format!("xiom.lock v2 is malformed: {e}")),
            Some(other) => Err(format!("unsupported xiom.lock version {other} (expected 2)")),
            None => Err("xiom.lock is v1 (no lockfileVersion); run `xiom pkg lock` to regenerate".to_string()),
        }
    }
}

/// Walk up from `start` looking for `xiom.lock`; returns the path + parsed
/// lock. Malformed/v1 locks are reported to stderr and treated as absent.
pub fn find_lockfile(start: &Path) -> Option<(PathBuf, Lockfile)> {
    let mut dir = Some(start.to_path_buf());
    while let Some(d) = dir {
        let candidate = d.join("xiom.lock");
        if candidate.is_file() {
            match std::fs::read_to_string(&candidate) {
                Ok(text) => match Lockfile::parse(&text) {
                    Ok(lock) => return Some((candidate, lock)),
                    Err(e) => {
                        eprintln!("xiom pkg: ignoring {}: {e}", candidate.display());
                        return None;
                    }
                },
                Err(e) => {
                    eprintln!("xiom pkg: cannot read {}: {e}", candidate.display());
                    return None;
                }
            }
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    None
}

/// Enforce the lock for one install: the package must be listed, the resolved
/// version must match, and the archive bytes must hash to the locked digest.
pub fn verify_locked_archive(
    lock: &Lockfile,
    name: &str,
    version: &str,
    archive: &[u8],
) -> Result<(), String> {
    let entry = lock.packages.get(name).ok_or_else(|| format!(
        "package '{name}' is not in xiom.lock -- run `xiom pkg lock` after adding it, \
         or install with XIOM_PKG_LOCKED=0 to bypass the lock"
    ))?;
    if entry.version != version {
        return Err(format!(
            "xiom.lock pins '{name}' at version {}, but {version} was requested",
            entry.version
        ));
    }
    if entry.integrity.is_empty() {
        return Err(format!(
            "xiom.lock has NO integrity for '{name}' v{version} (locked without a digest); \
             regenerate with `xiom pkg lock` while the registry is reachable"
        ));
    }
    let expected = entry.integrity.strip_prefix("sha256-").unwrap_or(&entry.integrity);
    let actual = sha256_hex(archive);
    if !expected.eq_ignore_ascii_case(&actual) {
        return Err(format!(
            "xiom.lock INTEGRITY MISMATCH for '{name}' v{version}: locked sha256-{expected}, \
             got sha256-{actual} -- the artifact changed after locking"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_deps() -> Vec<(String, String)> {
        vec![("alpha".to_string(), "1.2.3".to_string()), ("beta".to_string(), "0.4.0".to_string())]
    }

    #[test]
    fn build_round_trips_and_is_deterministic() {
        let deps = sample_deps();
        let resolve = |_n: &str, v: &str| Some(v.to_string());
        let digest = |n: &str, _v: &str| Some(format!("{:064x}", n.len()));
        let lock = Lockfile::build("root", "9.9.9", &deps, &resolve, &digest);
        assert_eq!(lock.lockfile_version, 2);
        assert_eq!(lock.packages.len(), 2);
        assert!(lock.packages["alpha"].integrity.starts_with("sha256-"));
        let json = lock.to_json();
        let parsed = Lockfile::parse(&json).expect("round trip");
        assert_eq!(parsed, lock);
        // Byte-identical output for identical inputs (deterministic ordering).
        let again = Lockfile::build("root", "9.9.9", &deps, &resolve, &digest);
        assert_eq!(again.to_json(), json);
    }

    #[test]
    fn integrity_normalization() {
        assert_eq!(normalize_integrity("ABCDEF"), "sha256-abcdef");
        assert_eq!(normalize_integrity("sha256-ABCDEF"), "sha256-abcdef");
        assert_eq!(normalize_integrity("sha256:ABCDEF"), "sha256-abcdef");
        assert_eq!(normalize_integrity("  "), "");
    }

    #[test]
    fn v1_and_malformed_locks_are_rejected() {
        assert!(Lockfile::parse("{\"package\":\"x\",\"version\":\"1\",\"dependencies\":{}}")
            .unwrap_err().contains("v1"));
        assert!(Lockfile::parse("not json").is_err());
        assert!(Lockfile::parse("{\"lockfileVersion\": 3}").unwrap_err().contains("version 3"));
    }

    #[test]
    fn verify_accepts_locked_bytes_and_rejects_swaps() {
        let archive = b"the-real-archive".to_vec();
        let integrity = integrity_for(&archive);
        let mut packages = BTreeMap::new();
        packages.insert("alpha".to_string(), LockedPackage {
            version: "1.2.3".to_string(),
            source: "registry".to_string(),
            integrity: integrity.clone(),
        });
        let lock = Lockfile {
            lockfile_version: 2,
            package: "root".to_string(),
            version: "1.0.0".to_string(),
            packages,
        };
        assert!(verify_locked_archive(&lock, "alpha", "1.2.3", &archive).is_ok());
        // Swapped artifact
        let err = verify_locked_archive(&lock, "alpha", "1.2.3", b"evil").unwrap_err();
        assert!(err.contains("INTEGRITY MISMATCH"), "{err}");
        // Version moved under the lock
        let err = verify_locked_archive(&lock, "alpha", "1.2.4", &archive).unwrap_err();
        assert!(err.contains("pins 'alpha' at version"), "{err}");
        // Not locked at all
        let err = verify_locked_archive(&lock, "gamma", "1.0.0", &archive).unwrap_err();
        assert!(err.contains("not in xiom.lock"), "{err}");
        // Locked without an integrity digest
        let mut no_digest = lock.clone();
        no_digest.packages.get_mut("alpha").unwrap().integrity = String::new();
        let err = verify_locked_archive(&no_digest, "alpha", "1.2.3", &archive).unwrap_err();
        assert!(err.contains("NO integrity"), "{err}");
    }

    #[test]
    fn find_lockfile_walks_up() {
        let base = std::env::temp_dir().join(format!(
            "xiom_lockprobe_{}_{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
        ));
        let nested = base.join("a").join("b");
        std::fs::create_dir_all(&nested).unwrap();
        let lock = Lockfile {
            lockfile_version: 2,
            package: "root".to_string(),
            version: "1.0.0".to_string(),
            packages: BTreeMap::new(),
        };
        std::fs::write(base.join("xiom.lock"), lock.to_json()).unwrap();
        let found = find_lockfile(&nested).expect("walk up");
        assert_eq!(found.0, base.join("xiom.lock"));
        let _ = std::fs::remove_dir_all(&base);
    }
}
