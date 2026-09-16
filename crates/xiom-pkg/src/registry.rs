// XIOM -- Package Manager (registry client)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// M14.1: Extracted from main.rs -- registry download, search, install.

use std::collections::HashMap;

use std::path::{Path, PathBuf};
use std::process;

const DEFAULT_REGISTRY: &str = "https://registry.xiom-lang.org";

pub(crate) fn registry_url() -> String {
    std::env::var("XIOM_REGISTRY").unwrap_or_else(|_| DEFAULT_REGISTRY.to_string())
}

// ============================================================================
// HTTP transport -- ureq-only, TLS-verified, size/timeout-bounded (audit #4/#5)
// ============================================================================

/// AUDIT #5 FIX: the old transport ladder was curl -> PowerShell (URL
/// interpolated into `-Command` -- command injection via --registry /
/// XIOM_REGISTRY) -> raw-TCP plaintext HTTP. HTTP is now ureq-ONLY:
/// native Rust, TLS-verified, timeout-bounded. Plain http:// is REJECTED
/// unless XIOM_PKG_ALLOW_HTTP=1 is explicitly set (local dev registries).
pub(crate) fn http_get(url: &str) -> Result<String, String> {
    ensure_https(url)?;
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(30))
        .call()
        .map_err(|e| format!("GET {url}: {e}"))?;
    let mut body = String::new();
    use std::io::Read;
    resp.into_reader().take(16 * 1024 * 1024).read_to_string(&mut body)
        .map_err(|e| format!("read: {e}"))?;
    Ok(body)
}

/// AUDIT #4/#5 FIX: binary downloads are ureq-only as well; response is
/// size-capped so a hostile mirror cannot OOM the client.
pub(crate) fn http_get_binary(url: &str) -> Result<Vec<u8>, String> {
    ensure_https(url)?;
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(120))
        .call()
        .map_err(|e| format!("GET {url}: {e}"))?;
    const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024; // 256 MiB
    let mut data = Vec::new();
    use std::io::Read;
    resp.into_reader().take(MAX_ARCHIVE_BYTES).read_to_end(&mut data)
        .map_err(|e| format!("read: {e}"))?;
    Ok(data)
}

fn ensure_https(url: &str) -> Result<(), String> {
    if url.starts_with("https://") {
        return Ok(());
    }
    if std::env::var("XIOM_PKG_ALLOW_HTTP").as_deref() == Ok("1")
        && url.starts_with("http://localhost")
    {
        return Ok(());
    }
    Err(format!(
        "refusing insecure registry URL '{url}': package downloads require HTTPS \
         (set XIOM_PKG_ALLOW_HTTP=1 to allow http://localhost for local dev)"
    ))
}
// 5e.7b: Remote Registry Client
// ============================================================================

/// Cached registry index (lazy-loaded, refreshed every 5 min)
static REGISTRY_CACHE: std::sync::Mutex<Option<(String, u64)>> = std::sync::Mutex::new(None);

#[derive(Debug, serde::Deserialize)]
pub(crate) struct RegistryIndex {
    #[allow(dead_code)]
    pub(crate) registry: String,
    #[allow(dead_code)]
    pub(crate) version: String,
    pub(crate) packages: HashMap<String, RegistryPackage>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct RegistryPackage {
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) repository: String,
    pub(crate) latest: String,
    /// AUDIT #4 FIX: the server publishes per-version metadata INCLUDING
    /// the tarball sha256 (registry/server.js writes `versions[version] =
    /// { sha256, size, ... }`). The old client modeled versions as a plain
    /// string array and DISCARDED the hashes -- installs were never
    /// verified. Both shapes now deserialize; hashes are mandatory at
    /// install time (see verify step in install_from_registry).
    #[serde(default, deserialize_with = "deserialize_versions")]
    pub(crate) versions: Vec<RegistryVersion>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct RegistryVersion {
    pub(crate) version: String,
    #[serde(default)]
    pub(crate) sha256: String,
}

/// Accept either the server's object map (`"1.0": {sha256,...}`) or the
/// legacy string array (`["1.0", "0.9"]`) so both registry generations work.
fn deserialize_versions<'de, D>(de: D) -> Result<Vec<RegistryVersion>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Shape {
        Map(std::collections::HashMap<String, VersionMeta>),
        List(Vec<String>),
    }
    #[derive(serde::Deserialize)]
    struct VersionMeta {
        #[serde(default)]
        sha256: String,
        #[serde(default)]
        version: Option<String>,
    }
    match Shape::deserialize(de)? {
        Shape::Map(map) => Ok(map.into_iter()
            .map(|(ver, meta)| RegistryVersion {
                version: meta.version.unwrap_or(ver.clone()),
                sha256: meta.sha256,
            })
            .collect()),
        Shape::List(list) => Ok(list.into_iter()
            .map(|ver| RegistryVersion { version: ver, sha256: String::new() })
            .collect()),
    }
}

pub(crate) fn fetch_registry_index(registry: &str) -> Result<RegistryIndex, String> {
    // Check cache (5 min TTL)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    {
        let cache = REGISTRY_CACHE.lock().map_err(|e| format!("cache lock: {e}"))?;
        if let Some((ref cached, ts)) = *cache {
            if now - ts < 300 {
                if let Ok(idx) = serde_json::from_str::<RegistryIndex>(cached) {
                    return Ok(idx);
                }
            }
        }
    }

    let url = format!("{registry}/index.json");
    let body = http_get(&url)?;
    let index: RegistryIndex = serde_json::from_str(&body)
        .map_err(|e| format!("Invalid registry index: {e}"))?;

    let mut cache = REGISTRY_CACHE.lock().map_err(|e| format!("cache lock: {e}"))?;
    *cache = Some((body, now));
    Ok(index)
}

pub(crate) fn search_registry(query: &str, registry: &str) -> Result<(), String> {
    let index = fetch_registry_index(registry)?;
    let q = query.to_lowercase();
    let mut found = 0;
    println!("Searching '{}' in {}...", query, registry);
    for (name, pkg) in &index.packages {
        if q.is_empty() || name.to_lowercase().contains(&q) || pkg.description.to_lowercase().contains(&q) {
            println!("  {} v{}", name, pkg.latest);
            println!("    {}", pkg.description);
            if !pkg.repository.is_empty() {
                println!("    repo: {}", pkg.repository);
            }
            println!();
            found += 1;
        }
    }
    println!("{} package(s) found.", found);
    Ok(())
}

pub(crate) fn install_from_registry(package: &str, version: Option<&str>, registry: &str) -> Result<(), String> {
    let index = fetch_registry_index(registry)?;

    // Resolve package name
    let pkg_info = index.packages.get(package)
        .ok_or_else(|| format!("Package '{}' not found in registry. Try: xiom pkg search {}", package, package))?;

    let ver = version.unwrap_or(&pkg_info.latest);
    let ver_meta = pkg_info.versions.iter().find(|v| &v.version == ver)
        .ok_or_else(|| format!("Version '{}' not found for '{}'. Available: {:?}",
            ver, package,
            pkg_info.versions.iter().map(|v| v.version.as_str()).collect::<Vec<_>>()))?;

    // Download package archive
    let dl_url = format!("{}/packages/{}/{}/package.tar.gz", registry, package, ver);
    println!("Downloading {} v{} from {}...", package, ver, registry);

    let archive = http_get_binary(&dl_url)?;
    if archive.is_empty() {
        return Err(format!("Empty archive from {dl_url}"));
    }

    // AUDIT #4 FIX: CLIENT-SIDE SHA-256 VERIFICATION. The server has always
    // published the digest; the client now enforces it. Unhashed entries are
    // refused unless XIOM_PKG_ALLOW_UNHASHED=1 (legacy local indexes only).
    if ver_meta.sha256.is_empty() {
        let allow = std::env::var("XIOM_PKG_ALLOW_UNHASHED").as_deref() == Ok("1");
        if !allow {
            return Err(format!(
                "registry index has NO sha256 for {} v{} -- refusing to install \
                 unverified artifacts (set XIOM_PKG_ALLOW_UNHASHED=1 to override)",
                package, ver));
        }
        eprintln!("  WARNING: installing UNHASHED artifact {} v{} (override active)", package, ver);
    } else {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&archive);
        let actual = format!("{:x}", hasher.finalize());
        let expected = ver_meta.sha256.trim().to_lowercase();
        if actual != expected {
            return Err(format!(
                "CHECKSUM MISMATCH for {} v{}: expected sha256 {}, got {} -- \
                 the download is corrupted or tampered with",
                package, ver, expected, actual));
        }
        println!("  checksum verified (sha256:{actual})");
    }

    // Stage 5: LOCKFILE v2 ENFORCEMENT. When a xiom.lock is found walking up
    // from the CWD, the artifact must match the LOCKED digest and version --
    // the server index alone cannot protect against an artifact swapped after
    // locking. XIOM_PKG_LOCKED=0 bypasses (explicit opt-out), =1 requires a
    // lock to exist.
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let locked_env = std::env::var("XIOM_PKG_LOCKED").ok();
    match crate::lockfile::find_lockfile(&cwd) {
        Some((lock_path, lock)) if locked_env.as_deref() != Some("0") => {
            crate::lockfile::verify_locked_archive(&lock, package, ver, &archive)
                .map_err(|e| format!("{e}\n  (lockfile: {})", lock_path.display()))?;
            println!("  lockfile verified ({})", lock_path.display());
        }
        None if locked_env.as_deref() == Some("1") => {
            return Err("XIOM_PKG_LOCKED=1 but no xiom.lock was found in this directory or any parent".to_string());
        }
        _ => {}
    }

    // Extract to local package cache
    let cache_dir = package_cache_dir();
    let pkg_dir = cache_dir.join(format!("{}-{}", package.replace('.', "-"), ver));
    if pkg_dir.exists() {
        std::fs::remove_dir_all(&pkg_dir).map_err(|e| format!("Cannot clean cache: {e}"))?;
    }
    extract_tar_gz(&archive, &pkg_dir)?;

    println!("Installed {} v{} to {}", package, ver, pkg_dir.display());
    println!("  Add to your package.xi dependencies:");
    println!("    dependencies = {{ {} = \"{}\" }}", package, ver);
    Ok(())
}

pub(crate) fn package_cache_dir() -> PathBuf {
    let base = if cfg!(windows) {
        PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string()))
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
    };
    base.join("xiom").join("packages")
}

/// Minimal tar.gz extractor (handles basic .tar.gz files without external tools).
///
/// AUDIT #19 FIXES:
/// - Unique RANDOM temp file (pid-only names allowed pre-planting).
/// - MEMBER-PATH VALIDATION: `tar -tzf` lists every entry BEFORE extraction;
///   absolute paths, drive letters, `..` components and Windows-style
///   backslash escapes are rejected (symlink/hardlink members are refused
///   outright -- a vetted in-process reader remains future work, logged in
///   the readiness plan Stage 5).
pub(crate) fn extract_tar_gz(data: &[u8], dest: &Path) -> Result<(), String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos()).unwrap_or(0);
    let rnd: u32 = (nanos as u32) ^ (((nanos >> 32) as u32).wrapping_mul(0x9E37_79B9));
    let tmp = std::env::temp_dir()
        .join(format!("xiom_pkg_{}_{:08x}.tar.gz", std::process::id(), rnd));
    std::fs::write(&tmp, data).map_err(|e| format!("Write temp: {e}"))?;

    std::fs::create_dir_all(dest).map_err(|e| format!("Create dir: {e}"))?;

    let cleanup = || { let _ = std::fs::remove_file(&tmp); };

    // --- Pre-extraction member validation ---
    let listing = process::Command::new("tar")
        .args(["-tzf", &tmp.to_string_lossy()])
        .output();
    if let Ok(out) = &listing {
        if out.status.success() {
            let listed = String::from_utf8_lossy(&out.stdout);
            for entry in listed.lines() {
                let entry = entry.trim();
                if entry.is_empty() { continue; }
                let evil = entry.starts_with('/')
                    || entry.starts_with('\\')
                    || entry.contains(":\\")
                    || entry.split('/').any(|seg| seg == "..")
                    || entry.split('\\').any(|seg| seg == "..");
                if evil {
                    cleanup();
                    return Err(format!(
                        "refusing malicious archive: member '{entry}' escapes the package directory"));
                }
            }
        }
        // Listing failures fall through to the extract attempt's own error.
    }

    let result = process::Command::new("tar")
        .args(["-xzf", &tmp.to_string_lossy(), "-C", &dest.to_string_lossy()])
        .status();

    cleanup();

    match result {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("tar exited with code {}", s.code().unwrap_or(-1))),
        Err(e) => Err(format!("tar not found: {e}. Install tar to extract packages.")),
    }
}
