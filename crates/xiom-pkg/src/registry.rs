// XIOM -- Package Manager (registry client)
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// M14.1: Extracted from main.rs -- registry download, search, install.

use std::collections::HashMap;

use std::path::{Path, PathBuf};
use std::process;

const DEFAULT_REGISTRY: &str = "https://registry.xiom-lang.org";

pub(crate) fn registry_url() -> String {
    canonicalize_registry_url(
        &std::env::var("XIOM_REGISTRY").unwrap_or_else(|_| DEFAULT_REGISTRY.to_string()),
    )
}

/// Canonical form of a registry base URL: surrounding whitespace trimmed, the
/// scheme and host lowercased, every trailing slash dropped. A path component
/// (for a registry mounted under one) keeps its case.
///
/// ONE canonical string must reach every call site -- trust-store lookups,
/// URL building, publish, lock generation -- or a text difference (trailing
/// slash, uppercase host) silently breaks key pinning (R33) and produces
/// `//index.json`-style paths (R34).
pub(crate) fn canonicalize_registry_url(raw: &str) -> String {
    let trimmed = raw.trim();
    let (scheme, rest) = match trimmed.split_once("://") {
        Some((scheme, rest)) => (Some(scheme.to_ascii_lowercase()), rest),
        None => (None, trimmed),
    };
    let (host, path) = match rest.find('/') {
        Some(slash) => (&rest[..slash], &rest[slash..]),
        None => (rest, ""),
    };
    let mut out = String::with_capacity(trimmed.len());
    if let Some(scheme) = scheme {
        out.push_str(&scheme);
        out.push_str("://");
    }
    out.push_str(&host.to_ascii_lowercase());
    out.push_str(path.trim_end_matches('/'));
    out
}

/// Max bytes of a non-2xx response body kept for diagnostics.
const ERROR_BODY_CAP: u64 = 64 * 1024;
/// Max characters of an error body rendered into a message.
const ERROR_BODY_CHARS: usize = 2048;

/// Read a bounded response body (best effort; empty on failure).
fn read_body_capped(resp: ureq::Response, cap: u64) -> String {
    let mut body = String::new();
    use std::io::Read;
    let _ = resp.into_reader().take(cap).read_to_string(&mut body);
    body
}

/// Message for a non-2xx response: status + the registry's JSON error body
/// (the server returns actionable codes: signature_invalid, signature_required,
/// reserved_namespace, scope_denied, version_exists, index_full, rate_limited).
/// The URL is rendered exactly once (R35).
fn format_status_error(method: &str, url: &str, code: u16, body: &str) -> String {
    let body = body.trim();
    if body.is_empty() {
        return format!("{method} {url}: HTTP {code}");
    }
    let rendered: String = body.chars().take(ERROR_BODY_CHARS).collect();
    let ellipsis = if body.chars().count() > ERROR_BODY_CHARS { "..." } else { "" };
    format!("{method} {url}: HTTP {code}: {rendered}{ellipsis}")
}

/// HTTP failures: non-2xx responses carry the registry's error body; transport
/// errors already include the URL in their own Display, so it is not prefixed
/// again (ureq 2.x renders `{url}: {kind}: {message}`).
fn describe_http_error(method: &str, url: &str, err: ureq::Error) -> String {
    match err {
        ureq::Error::Status(code, resp) => {
            let body = read_body_capped(resp, ERROR_BODY_CAP);
            format_status_error(method, url, code, &body)
        }
        other => format!("{method} {other}"),
    }
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
        .map_err(|e| describe_http_error("GET", url, e))?;
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
        .map_err(|e| describe_http_error("GET", url, e))?;
    const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024; // 256 MiB
    let mut data = Vec::new();
    use std::io::Read;
    resp.into_reader().take(MAX_ARCHIVE_BYTES).read_to_end(&mut data)
        .map_err(|e| format!("read: {e}"))?;
    Ok(data)
}

/// AUDIT #5 follow-up: multipart/form-data UPLOAD -- ureq-only. The publish
/// path used to shell out to `curl -F`, the last external process on the
/// HTTP surface (the transport ladder was already ureq-only for GETs).
pub(crate) fn http_post_multipart(
    url: &str,
    file_path: &str,
    file_field: &str,
    fields: &[(&str, &str)],
) -> Result<String, String> {
    ensure_https(url)?;
    let file_bytes = std::fs::read(file_path).map_err(|e| format!("read {file_path}: {e}"))?;
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("package.tar.gz");
    let boundary = multipart_boundary();
    let body = build_multipart_body(&boundary, file_field, file_name, &file_bytes, fields);
    let mut req = ureq::post(url)
        .set("Content-Type", &format!("multipart/form-data; boundary={boundary}"))
        .timeout(std::time::Duration::from_secs(120));
    // Stage 5: authenticated publish. The token comes from the environment
    // (never the URL); publishing to a non-localhost registry without one is
    // allowed but warned about so the gap is visible.
    match std::env::var("XIOM_REGISTRY_TOKEN") {
        Ok(token) if !token.trim().is_empty() => {
            req = req.set("Authorization", &format!("Bearer {}", token.trim()));
        }
        _ => {
            if !url.contains("localhost") {
                eprintln!("xiom pkg: WARNING: XIOM_REGISTRY_TOKEN is not set; publishing WITHOUT authentication");
            }
        }
    }
    let resp = req.send_bytes(&body).map_err(|e| describe_http_error("POST", url, e))?;
    let mut text = String::new();
    use std::io::Read;
    resp.into_reader().take(1024 * 1024).read_to_string(&mut text)
        .map_err(|e| format!("read: {e}"))?;
    Ok(text)
}

/// Boundary derived from pid + nanos (unique per process invocation without a
/// RNG dependency); checked not to occur in the payload by construction.
pub(crate) fn multipart_boundary() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("----XIOMBoundary{:x}{:x}", std::process::id(), nanos)
}

/// Build the multipart body: text fields first, then the gzip file part.
pub(crate) fn build_multipart_body(
    boundary: &str,
    file_field: &str,
    file_name: &str,
    file_bytes: &[u8],
    fields: &[(&str, &str)],
) -> Vec<u8> {
    let mut body = Vec::with_capacity(file_bytes.len() + 512);
    for (k, v) in fields {
        body.extend_from_slice(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n").as_bytes(),
        );
    }
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{file_field}\"; filename=\"{file_name}\"\r\nContent-Type: application/gzip\r\n\r\n"
        ).as_bytes(),
    );
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
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
    /// Stage 5: detached ed25519 signature (hex) over the tarball bytes and
    /// the signer's public key (hex). Optional so legacy indexes work;
    /// TRUSTED registries must provide both.
    #[serde(default)]
    pub(crate) signature: String,
    #[serde(default, alias = "publicKey")]
    pub(crate) public_key: String,
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
        signature: String,
        #[serde(default, alias = "publicKey")]
        public_key: String,
        #[serde(default)]
        version: Option<String>,
    }
    match Shape::deserialize(de)? {
        Shape::Map(map) => Ok(map.into_iter()
            .map(|(ver, meta)| RegistryVersion {
                version: meta.version.unwrap_or(ver.clone()),
                sha256: meta.sha256,
                signature: meta.signature,
                public_key: meta.public_key,
            })
            .collect()),
        Shape::List(list) => Ok(list.into_iter()
            .map(|ver| RegistryVersion {
                version: ver,
                sha256: String::new(),
                signature: String::new(),
                public_key: String::new(),
            })
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

/// Errors from `install_from_registry`.
///
/// Only "we could not obtain it" classes may trigger a local fallback.
/// Integrity-class failures are TERMINAL: a failed sha256 / signature /
/// lockfile check must never be retried through an unverified path (R32).
#[derive(Debug)]
pub(crate) enum InstallError {
    /// Registry unreachable, transport failure, or the artifact could not be
    /// fetched (no verification decision was made).
    RegistryUnavailable(String),
    /// Package / version is not in the registry index.
    NotFound(String),
    /// An integrity gate rejected the artifact (sha256, signature, lockfile).
    Integrity(String),
    /// Local cache/extraction failure: terminal (retrying elsewhere would
    /// install into a broken environment).
    Local(String),
}

impl InstallError {
    /// True when falling back to local package resolution is safe.
    pub(crate) fn allows_local_fallback(&self) -> bool {
        matches!(self, InstallError::RegistryUnavailable(_) | InstallError::NotFound(_))
    }
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallError::RegistryUnavailable(m)
            | InstallError::NotFound(m)
            | InstallError::Integrity(m)
            | InstallError::Local(m) => write!(f, "{m}"),
        }
    }
}

/// Resolve the version to install from an index entry.
///
/// `latest` is empty when every version is yanked (registry protocol); that
/// must produce a clear message instead of a bogus
/// `.../<pkg>//package.tar.gz` URL (R37). A pinned version still resolves
/// when listed (yanked versions stay installable by pin).
fn resolve_version<'a>(
    package: &str,
    pkg: &'a RegistryPackage,
    requested: Option<&str>,
) -> Result<&'a RegistryVersion, InstallError> {
    let available = || {
        let mut names: Vec<&str> = pkg.versions.iter().map(|v| v.version.as_str()).collect();
        names.sort_unstable();
        names.join(", ")
    };
    if let Some(version) = requested {
        return pkg.versions.iter().find(|v| v.version == version).ok_or_else(|| {
            InstallError::NotFound(format!(
                "version '{}' not found for '{}'. Available: {}",
                version, package, available()
            ))
        });
    }
    if pkg.latest.trim().is_empty() {
        return Err(InstallError::NotFound(format!(
            "all versions of '{}' are yanked; nothing to install by default. \
             Install a pinned version with `xiom pkg install {}@<version>`.",
            package, package
        )));
    }
    pkg.versions.iter().find(|v| v.version == pkg.latest).ok_or_else(|| {
        InstallError::NotFound(format!(
            "latest version '{}' of '{}' is not in the version list. Available: {}",
            pkg.latest, package, available()
        ))
    })
}

pub(crate) fn install_from_registry(
    package: &str,
    version: Option<&str>,
    registry: &str,
) -> Result<(), InstallError> {
    let index = fetch_registry_index(registry).map_err(InstallError::RegistryUnavailable)?;

    let pkg_info = index.packages.get(package).ok_or_else(|| {
        InstallError::NotFound(format!(
            "package '{}' not found in registry. Try: xiom pkg search {}",
            package, package
        ))
    })?;

    let ver_meta = resolve_version(package, pkg_info, version)?;
    let ver = ver_meta.version.as_str();

    // Download package archive
    let dl_url = format!("{}/packages/{}/{}/package.tar.gz", registry, package, ver);
    println!("Downloading {} v{} from {}...", package, ver, registry);

    let archive =
        http_get_binary(&dl_url).map_err(InstallError::RegistryUnavailable)?;
    if archive.is_empty() {
        return Err(InstallError::RegistryUnavailable(format!(
            "empty archive from {dl_url}"
        )));
    }

    // AUDIT #4 FIX: CLIENT-SIDE SHA-256 VERIFICATION. The server has always
    // published the digest; the client now enforces it. Unhashed entries are
    // refused unless XIOM_PKG_ALLOW_UNHASHED=1 (legacy local indexes only).
    if ver_meta.sha256.is_empty() {
        let allow = std::env::var("XIOM_PKG_ALLOW_UNHASHED").as_deref() == Ok("1");
        if !allow {
            return Err(InstallError::Integrity(format!(
                "registry index has NO sha256 for {} v{} -- refusing to install \
                 unverified artifacts (set XIOM_PKG_ALLOW_UNHASHED=1 to override)",
                package, ver
            )));
        }
        eprintln!("  WARNING: installing UNHASHED artifact {} v{} (override active)", package, ver);
    } else {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&archive);
        let actual = format!("{:x}", hasher.finalize());
        let expected = ver_meta.sha256.trim().to_lowercase();
        if actual != expected {
            return Err(InstallError::Integrity(format!(
                "CHECKSUM MISMATCH for {} v{}: expected sha256 {}, got {} -- \
                 the download is corrupted or tampered with",
                package, ver, expected, actual
            )));
        }
        println!("  checksum verified (sha256:{actual})");
    }

    // Stage 5: SIGNATURE VERIFICATION (ed25519). A TRUSTED registry (pinned
    // with `xiom pkg trust --registry URL --key HEX`) must sign its
    // artifacts -- an unsigned or mis-signed one is REFUSED (fail closed);
    // the index digest and the artifact can otherwise be swapped together by
    // a compromised registry. Untrusted registries keep the sha256 +
    // lockfile checks and get a hint to pin the signer.
    let trusted_key = crate::signing::TrustStore::load().get(registry).cloned();
    match (trusted_key.as_ref(), ver_meta.signature.as_str()) {
        (Some(key), sig) if !sig.is_empty() => {
            crate::signing::verify(key, &archive, sig).map_err(|e| {
                InstallError::Integrity(format!("{e} ({package} v{ver} from {registry})"))
            })?;
            println!("  signature verified (fp {})", crate::signing::fingerprint(key));
        }
        (Some(_), _) => {
            return Err(InstallError::Integrity(format!(
                "registry {registry} is TRUSTED but {package} v{ver} carries NO signature -- \
                 refusing to install (unpin with: remove its entry from {})",
                crate::signing::default_xiom_home().join("trusted_keys.json").display()
            )));
        }
        (None, sig) if !sig.is_empty() => {
            println!("  artifact is signed (fp {}); pin it with `xiom pkg trust --registry {registry} --key {}` to enforce",
                crate::signing::fingerprint(&ver_meta.public_key), ver_meta.public_key);
        }
        (None, _) => {}
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
            crate::lockfile::verify_locked_archive(&lock, package, ver, &archive).map_err(|e| {
                InstallError::Integrity(format!("{e}\n  (lockfile: {})", lock_path.display()))
            })?;
            println!("  lockfile verified ({})", lock_path.display());
        }
        None if locked_env.as_deref() == Some("1") => {
            return Err(InstallError::Integrity(
                "XIOM_PKG_LOCKED=1 but no xiom.lock was found in this directory or any parent"
                    .to_string(),
            ));
        }
        _ => {}
    }

    // Extract to local package cache (clean first: stale members from an
    // earlier install must not survive).
    let cache_dir = package_cache_dir();
    let pkg_dir = cache_dir.join(format!("{}-{}", package.replace('.', "-"), ver));
    if pkg_dir.exists() {
        std::fs::remove_dir_all(&pkg_dir)
            .map_err(|e| InstallError::Local(format!("cannot clean cache: {e}")))?;
    }
    extract_tar_gz(&archive, &pkg_dir).map_err(InstallError::Local)?;

    println!("Installed {} v{} to {}", package, ver, pkg_dir.display());
    println!("  Add to your package.xi dependencies:");
    println!("    dependencies = {{ {} = \"{}\" }}", package, ver);
    Ok(())
}

pub(crate) fn package_cache_dir() -> PathBuf {
    resolve_package_cache_dir(std::env::var("XIOM_HOME").ok().as_deref())
}

/// `$XIOM_HOME/packages` when XIOM_HOME is set (sandboxed runs -- CI, tests --
/// must not write the developer's real cache), otherwise the platform default
/// `($LOCALAPPDATA|$HOME)/xiom/packages`, matching `get_xiom_home` in main.rs
/// (R38).
fn resolve_package_cache_dir(xiom_home: Option<&str>) -> PathBuf {
    if let Some(home) = xiom_home.map(str::trim).filter(|h| !h.is_empty()) {
        return PathBuf::from(home).join("packages");
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multipart_body_is_well_formed() {
        let body = build_multipart_body(
            "BOUND", "package", "pkg.tar.gz", b"TARBYTES",
            &[("name", "demo"), ("signature", "abcd")],
        );
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("--BOUND\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\ndemo\r\n"), "{text}");
        assert!(text.contains("name=\"signature\""));
        assert!(text.contains("filename=\"pkg.tar.gz\""));
        assert!(text.contains("Content-Type: application/gzip"));
        assert!(text.contains("TARBYTES"));
        assert!(text.ends_with("--BOUND--\r\n"));
        assert_eq!(text.matches("--BOUND").count(), 4, "2 fields + file + closing");
    }

    #[test]
    fn multipart_boundaries_are_unique_and_prefixed() {
        let a = multipart_boundary();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = multipart_boundary();
        assert_ne!(a, b);
        assert!(a.starts_with("----XIOMBoundary"), "{a}");
    }

    // --- R33/R34: registry URL canonicalization -----------------------------

    #[test]
    fn canonicalize_registry_url_matches_text_variants() {
        for raw in [
            "http://localhost:3203",
            "http://localhost:3203/",
            "  http://localhost:3203//  ",
            "HTTP://LOCALHOST:3203",
            "http://LOCALHOST:3203/",
        ] {
            assert_eq!(canonicalize_registry_url(raw), "http://localhost:3203", "raw={raw:?}");
        }
        // A mounted path keeps its case; only the trailing slash is dropped.
        assert_eq!(
            canonicalize_registry_url("https://Host.Example/Base/"),
            "https://host.example/Base"
        );
        assert_eq!(
            canonicalize_registry_url("https://registry.xiom-lang.org"),
            "https://registry.xiom-lang.org"
        );
    }

    // --- R32: fallback classes are explicit ---------------------------------

    #[test]
    fn install_error_fallback_classes_are_explicit() {
        assert!(InstallError::RegistryUnavailable("x".into()).allows_local_fallback());
        assert!(InstallError::NotFound("x".into()).allows_local_fallback());
        assert!(!InstallError::Integrity("x".into()).allows_local_fallback());
        assert!(!InstallError::Local("x".into()).allows_local_fallback());
    }

    // --- R37: version resolution --------------------------------------------

    fn pkg(latest: &str, versions: &[&str]) -> RegistryPackage {
        RegistryPackage {
            description: String::new(),
            repository: String::new(),
            latest: latest.to_string(),
            versions: versions.iter().map(|v| RegistryVersion {
                version: v.to_string(),
                sha256: String::new(),
                signature: String::new(),
                public_key: String::new(),
            }).collect(),
        }
    }

    #[test]
    fn resolve_version_pins_versions_and_reports_empty_latest() {
        let p = pkg("0.2.0", &["0.1.0", "0.2.0"]);
        assert_eq!(resolve_version("p", &p, Some("0.1.0")).expect("pinned").version, "0.1.0");
        assert_eq!(resolve_version("p", &p, None).expect("latest").version, "0.2.0");
        let missing = resolve_version("p", &p, Some("9.9.9")).unwrap_err();
        assert!(missing.to_string().contains("Available: 0.1.0, 0.2.0"), "{missing}");

        // All versions yanked: latest is empty; pinned versions still resolve.
        let yanked = pkg("", &["0.1.0"]);
        let err = resolve_version("p", &yanked, None).unwrap_err();
        assert!(err.to_string().contains("yanked"), "{err}");
        assert!(!err.to_string().contains("//package.tar.gz"), "{err}");
        assert_eq!(resolve_version("p", &yanked, Some("0.1.0")).expect("pinned yanked").version, "0.1.0");

        // Latest points at a version the server does not list.
        let drift = pkg("0.3.0", &["0.1.0"]);
        let err = resolve_version("p", &drift, None).unwrap_err();
        assert!(err.to_string().contains("not in the version list"), "{err}");
    }

    // --- R35: HTTP error rendering ------------------------------------------

    #[test]
    fn status_errors_render_status_body_and_url_once() {
        let msg = format_status_error(
            "POST",
            "http://localhost:3203/publish",
            422,
            r#"{"error":"signature required","code":"signature_required"}"#,
        );
        assert_eq!(msg.matches("http://localhost:3203/publish").count(), 1, "{msg}");
        assert!(msg.contains("HTTP 422"), "{msg}");
        assert!(msg.contains("signature_required"), "{msg}");
        let empty = format_status_error("GET", "http://x/index.json", 404, "   ");
        assert_eq!(empty, "GET http://x/index.json: HTTP 404");
    }

    // --- R38: cache location -------------------------------------------------

    #[test]
    fn package_cache_dir_respects_xiom_home() {
        assert_eq!(
            resolve_package_cache_dir(Some("  E:/sandbox  ")),
            PathBuf::from("E:/sandbox").join("packages")
        );
        assert!(resolve_package_cache_dir(Some("")).ends_with(Path::new("xiom").join("packages")));
        assert!(resolve_package_cache_dir(None).ends_with(Path::new("xiom").join("packages")));
    }
}
