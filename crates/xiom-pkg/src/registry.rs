// XIOM -- Package Manager (registry client)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// M14.1: Extracted from main.rs -- registry download, search, install.

use std::collections::{BTreeSet, HashMap};

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

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct RegistryPackage {
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) repository: String,
    /// R53 (registry relay): server-extracted metadata from package.xi.
    /// Field names match the index wire shape exactly.
    #[serde(default)]
    pub(crate) license: String,
    #[serde(default)]
    pub(crate) categories: Vec<String>,
    #[serde(default)]
    pub(crate) keywords: Vec<String>,
    #[serde(default)]
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
    /// Stage 5: dependencies declared by this version's manifest
    /// (`{ "name": ">=1.0,<2.0" }`), as published by the registry. Used for
    /// transitive resolution; the verified tarball's own `package.xi` is
    /// authoritative once extracted.
    #[serde(default)]
    pub(crate) dependencies: HashMap<String, String>,
    /// Yanked versions stay installable by exact pin but are skipped when a
    /// range/latest resolves.
    #[serde(default)]
    pub(crate) yanked: bool,
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
        /// Stage 5: dependency name -> version spec, published from the
        /// tarball's package.xi at publish time (registry/src/app.js T3).
        #[serde(default)]
        dependencies: std::collections::HashMap<String, String>,
        #[serde(default)]
        yanked: bool,
    }
    match Shape::deserialize(de)? {
        Shape::Map(map) => Ok(map.into_iter()
            .map(|(ver, meta)| RegistryVersion {
                version: meta.version.unwrap_or(ver.clone()),
                sha256: meta.sha256,
                signature: meta.signature,
                public_key: meta.public_key,
                dependencies: meta.dependencies,
                yanked: meta.yanked,
            })
            .collect()),
        Shape::List(list) => Ok(list.into_iter()
            .map(|ver| RegistryVersion {
                version: ver,
                sha256: String::new(),
                signature: String::new(),
                public_key: String::new(),
                dependencies: std::collections::HashMap::new(),
                yanked: false,
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

/// R53: pure filter used by `search_registry` (unit-testable offline).
/// `category` filters by exact category name (case-insensitive); the text
/// query matches name, description, keywords and categories.
pub(crate) fn filter_packages<'a>(
    packages: &'a HashMap<String, RegistryPackage>,
    query: &str,
    category: Option<&str>,
) -> Vec<(&'a String, &'a RegistryPackage)> {
    let q = query.to_lowercase();
    let cat = category.map(|c| c.to_lowercase());
    let mut matches: Vec<(&String, &RegistryPackage)> = packages.iter()
        .filter(|(name, pkg)| {
            if let Some(ref c) = cat {
                if !pkg.categories.iter().any(|pc| pc.to_lowercase() == *c) {
                    return false;
                }
            }
            q.is_empty()
                || name.to_lowercase().contains(&q)
                || pkg.description.to_lowercase().contains(&q)
                || pkg.keywords.iter().any(|k| k.to_lowercase().contains(&q))
                || pkg.categories.iter().any(|c| c.to_lowercase().contains(&q))
        })
        .collect();
    matches.sort_by(|a, b| a.0.cmp(b.0));
    matches
}

/// R53 (registry relay): `xiom pkg search` -- keyword/category aware listing.
/// `category` filters the locally fetched index (no server change needed);
/// keywords and categories participate in the text match.
pub(crate) fn search_registry(query: &str, registry: &str, category: Option<&str>, json: bool) -> Result<(), String> {
    let index = fetch_registry_index(registry)?;
    let matches = filter_packages(&index.packages, query, category);
    if json {
        let packages: Vec<serde_json::Value> = matches.iter()
            .map(|(name, pkg)| package_summary_json(name, pkg))
            .collect();
        let out = serde_json::json!({
            "query": query,
            "category": category.unwrap_or(""),
            "packages": packages,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        return Ok(());
    }
    println!("Searching '{}' in {}...", query, registry);
    for (name, pkg) in &matches {
        println!("  {} v{}", name, pkg.latest);
        if !pkg.description.is_empty() {
            println!("    {}", pkg.description);
        }
        if !pkg.categories.is_empty() {
            println!("    categories: {}", pkg.categories.join(", "));
        }
        if !pkg.keywords.is_empty() {
            println!("    keywords: {}", pkg.keywords.join(", "));
        }
        if !pkg.license.is_empty() {
            println!("    license: {}", pkg.license);
        }
        if !pkg.repository.is_empty() {
            println!("    repo: {}", pkg.repository);
        }
        println!();
    }
    println!("{} package(s) found.", matches.len());
    Ok(())
}

/// Wire-shaped package summary (field names mirror the registry index).
fn package_summary_json(name: &str, pkg: &RegistryPackage) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "description": pkg.description,
        "categories": pkg.categories,
        "keywords": pkg.keywords,
        "latest": pkg.latest,
        "license": pkg.license,
        "repository": pkg.repository,
    })
}

/// R53: fetch `/packages/:name` -- tolerates a bare package object or a
/// `{"package": {...}}` wrapper.
pub(crate) fn fetch_package_metadata(registry: &str, name: &str) -> Result<RegistryPackage, String> {
    let url = format!("{registry}/packages/{name}");
    let body = http_get(&url)?;
    let root: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Invalid package metadata: {e}"))?;
    let obj = match root.get("package") {
        Some(p) if p.is_object() => p.clone(),
        _ => root,
    };
    serde_json::from_value::<RegistryPackage>(obj)
        .map_err(|e| format!("Invalid package metadata for '{name}': {e}"))
}

/// R53: `xiom pkg info <name>[@version]` (+ `--json` for the MCP tool).
pub(crate) fn package_info(name: &str, version: Option<&str>, registry: &str, json: bool) -> Result<(), String> {
    let trimmed = name.trim_end_matches('/');
    let pkg = fetch_package_metadata(registry, trimmed)?;
    if json {
        let versions: Vec<serde_json::Value> = pkg.versions.iter().map(|v| serde_json::json!({
            "version": v.version,
            "sha256": v.sha256,
            "signature": v.signature,
            "publicKey": v.public_key,
            "yanked": v.yanked,
            "dependencies": v.dependencies,
        })).collect();
        let out = serde_json::json!({
            "name": trimmed,
            "description": pkg.description,
            "categories": pkg.categories,
            "keywords": pkg.keywords,
            "license": pkg.license,
            "repository": pkg.repository,
            "latest": pkg.latest,
            "versions": versions,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        return Ok(());
    }
    println!("{} v{}", trimmed, if pkg.latest.is_empty() { "?" } else { &pkg.latest });
    if !pkg.description.is_empty() {
        println!("  {}", pkg.description);
    }
    if !pkg.categories.is_empty() {
        println!("  categories: {}", pkg.categories.join(", "));
    }
    if !pkg.keywords.is_empty() {
        println!("  keywords: {}", pkg.keywords.join(", "));
    }
    if !pkg.license.is_empty() {
        println!("  license: {}", pkg.license);
    }
    if !pkg.repository.is_empty() {
        println!("  repository: {}", pkg.repository);
    }
    if let Some(v) = version {
        match pkg.versions.iter().find(|rv| rv.version == v) {
            Some(rv) => {
                println!("  version {}:", rv.version);
                println!("    sha256: {}", if rv.sha256.is_empty() { "(none)" } else { &rv.sha256 });
                println!("    signature: {}", if rv.signature.is_empty() { "(unsigned)" } else { &rv.signature });
            }
            None => eprintln!("xiom pkg info: version '{v}' not found for '{trimmed}'"),
        }
    }
    println!("  versions:");
    let mut vs: Vec<&RegistryVersion> = pkg.versions.iter().collect();
    vs.sort_by(|a, b| a.version.cmp(&b.version));
    for rv in vs {
        let yanked = if rv.yanked { " (yanked)" } else { "" };
        let digest = if rv.sha256.is_empty() {
            "no digest".to_string()
        } else {
            format!("sha256 {}", &rv.sha256[..rv.sha256.len().min(12)])
        };
        let sig = if rv.signature.is_empty() { "unsigned" } else { "signed" };
        println!("    {} -- {}, {}{}", rv.version, digest, sig, yanked);
    }
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

// ============================================================================
// Stage 5: version-range matching + transitive dependency closure
// ============================================================================

/// Parse a dotted version: numeric core segments plus an optional
/// pre-release/build tail kept verbatim ("1.2.3-rc1" -> ([1,2,3], "-rc1")).
fn parse_version(v: &str) -> (Vec<u64>, String) {
    let v = v.trim();
    let (core, tail) = match v.find(['-', '+']) {
        Some(i) => (&v[..i], &v[i..]),
        None => (v, ""),
    };
    let nums = core
        .split('.')
        .map(|s| s.trim().parse::<u64>().unwrap_or(0))
        .collect();
    (nums, tail.to_string())
}

/// Numeric-aware version ordering: missing segments are 0 and a release sorts
/// ABOVE its pre-releases (`1.0.0-rc1 < 1.0.0`).
pub(crate) fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let (an, at) = parse_version(a);
    let (bn, bt) = parse_version(b);
    for i in 0..an.len().max(bn.len()) {
        let x = an.get(i).copied().unwrap_or(0);
        let y = bn.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    match (at.is_empty(), bt.is_empty()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => at.cmp(&bt),
    }
}

/// True when `version` satisfies a comma-separated constraint list.
/// Operators: `=` (or bare), `>=`, `<=`, `>`, `<`, `^`, `~`; `*`/empty = any.
pub(crate) fn version_satisfies(version: &str, req: &str) -> bool {
    use std::cmp::Ordering;
    let req = req.trim();
    if req.is_empty() || req == "*" {
        return true;
    }
    for part in req.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let (op, rest) = if let Some(r) = p.strip_prefix(">=") {
            (">=", r)
        } else if let Some(r) = p.strip_prefix("<=") {
            ("<=", r)
        } else if let Some(r) = p.strip_prefix('>') {
            (">", r)
        } else if let Some(r) = p.strip_prefix('<') {
            ("<", r)
        } else if let Some(r) = p.strip_prefix('=') {
            ("=", r)
        } else if let Some(r) = p.strip_prefix('^') {
            ("^", r)
        } else if let Some(r) = p.strip_prefix('~') {
            ("~", r)
        } else {
            ("=", p)
        };
        let rest = rest.trim();
        let (vn, _) = parse_version(version);
        let (rn, _) = parse_version(rest);
        let ord = version_cmp(version, rest);
        let ok = match op {
            ">=" => ord != Ordering::Less,
            "<=" => ord != Ordering::Greater,
            ">" => ord == Ordering::Greater,
            "<" => ord == Ordering::Less,
            "=" => ord == Ordering::Equal,
            "^" => vn.first() == rn.first() && ord != Ordering::Less,
            "~" => {
                vn.first() == rn.first()
                    && vn.get(1).copied().unwrap_or(0) == rn.get(1).copied().unwrap_or(0)
                    && ord != Ordering::Less
            }
            _ => false,
        };
        if !ok {
            return false;
        }
    }
    true
}

/// An EXACT spec ("1.2.3" / "=1.2.3") still resolves a yanked version --
/// yanked releases stay installable by pin; ranges skip them.
fn spec_is_exact(req: &str) -> bool {
    let req = req.trim();
    if req.is_empty() || req == "*" {
        return false;
    }
    let bare = req.strip_prefix('=').unwrap_or(req);
    !bare.contains(|c: char| c == '<' || c == '>' || c == '^' || c == '~' || c == ',' || c == '*')
}

/// Highest version satisfying `req`; empty/"*" prefers `latest` (non-yanked).
pub(crate) fn select_version(pkg: &RegistryPackage, req: &str) -> Option<String> {
    let req = req.trim();
    let exact = spec_is_exact(req);
    if (req.is_empty() || req == "*") && !pkg.latest.trim().is_empty() {
        if let Some(v) = pkg.versions.iter().find(|v| v.version == pkg.latest) {
            if !v.yanked {
                return Some(v.version.clone());
            }
        }
    }
    let mut candidates: Vec<&RegistryVersion> = pkg
        .versions
        .iter()
        .filter(|v| (exact || !v.yanked) && version_satisfies(&v.version, req))
        .collect();
    candidates.sort_by(|a, b| version_cmp(&b.version, &a.version));
    candidates.first().map(|v| v.version.clone())
}

/// A resolved registry artifact in closure order.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClosureEntry {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) sha256: String,
}

/// True when a dependency spec is NOT a registry version: path/git/URL deps
/// are not registry artifacts and stay out of the registry closure.
pub(crate) fn is_non_registry_spec(spec: &str) -> bool {
    let s = spec.trim();
    s.starts_with("path:")
        || s.starts_with("git:")
        || s.starts_with("file:")
        || s.starts_with("http:")
        || s.starts_with("https:")
}

/// R52 (packages relay): the standard library is a PLATFORM package, not a
/// registry artifact. A manifest may declare `xiom.std: "0.1.0"`; the
/// closure must treat it as locally satisfied (`resolve_dependencies` maps it
/// to the checkout) instead of failing "not in the registry". Legacy
/// hyphen spelling stays accepted.
pub(crate) fn is_platform_dep(name: &str) -> bool {
    matches!(name, "xiom.std" | "xiom-std")
}

/// True when a dependency (name + spec) stays out of the registry closure.
pub(crate) fn is_non_registry_dep(name: &str, spec: &str) -> bool {
    is_platform_dep(name) || is_non_registry_spec(spec)
}

/// Deterministic transitive closure from already-resolved roots.
///
/// Cycle-safe (keyed `name@version`), dependency order sorted at every level,
/// and `deps_of` is the authoritative dependency source (the verified
/// tarball's manifest, falling back to index metadata).
fn resolve_closure(
    index: &RegistryIndex,
    roots: Vec<(String, String)>,
    deps_of: &dyn Fn(&str, &str) -> Vec<(String, String)>,
) -> Result<Vec<ClosureEntry>, InstallError> {
    let mut order: Vec<ClosureEntry> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<(String, String)> = roots;
    while let Some((name, version)) = stack.pop() {
        let key = format!("{name}@{version}");
        if !seen.insert(key) {
            continue;
        }
        let sha256 = index
            .packages
            .get(&name)
            .and_then(|p| p.versions.iter().find(|v| v.version == version))
            .map(|v| v.sha256.clone())
            .unwrap_or_default();
        order.push(ClosureEntry { name: name.clone(), version: version.clone(), sha256 });
        let mut deps: Vec<(String, String)> = deps_of(&name, &version)
            .into_iter()
            .filter(|(dep, spec)| !is_non_registry_dep(dep, spec))
            .collect();
        deps.sort();
        for (dep, spec) in deps.into_iter().rev() {
            let Some(dep_info) = index.packages.get(&dep) else {
                return Err(InstallError::NotFound(format!(
                    "dependency '{}' of {} v{} is not in the registry",
                    dep, name, version
                )));
            };
            let Some(resolved) = select_version(dep_info, &spec) else {
                return Err(InstallError::NotFound(format!(
                    "no version of '{}' satisfies '{}' (required by {} v{})",
                    dep, spec, name, version
                )));
            };
            stack.push((dep, resolved));
        }
    }
    Ok(order)
}

/// Full install closure for a resolved root package version.
pub(crate) fn install_closure(
    index: &RegistryIndex,
    package: &str,
    version: &str,
    deps_of: &dyn Fn(&str, &str) -> Vec<(String, String)>,
) -> Result<Vec<ClosureEntry>, InstallError> {
    resolve_closure(index, vec![(package.to_string(), version.to_string())], deps_of)
}

/// Lock closure for direct manifest dependencies (registry specs only;
/// path/git specs are recorded by the caller). Dependency metadata comes from
/// the index -- locking does not download artifacts.
pub(crate) fn lock_closure(
    index: &RegistryIndex,
    roots: &[(String, String)],
) -> Result<Vec<ClosureEntry>, InstallError> {
    let mut resolved_roots: Vec<(String, String)> = Vec::new();
    for (name, req) in roots {
        let Some(info) = index.packages.get(name) else {
            return Err(InstallError::NotFound(format!(
                "dependency '{name}' is not in the registry"
            )));
        };
        let Some(version) = select_version(info, req) else {
            return Err(InstallError::NotFound(format!(
                "no version of '{name}' satisfies '{req}'"
            )));
        };
        resolved_roots.push((name.clone(), version));
    }
    let deps_of = |name: &str, version: &str| -> Vec<(String, String)> {
        index
            .packages
            .get(name)
            .and_then(|p| p.versions.iter().find(|v| v.version == version))
            .map(|v| v.dependencies.iter().map(|(k, s)| (k.clone(), s.clone())).collect())
            .unwrap_or_default()
    };
    resolve_closure(index, resolved_roots, &deps_of)
}

/// Read the dependency list from an installed package's OWN manifest
/// (`package.xi`): the authoritative source, because those bytes were
/// verified. None when no manifest is present (caller falls back to index
/// metadata).
fn installed_manifest_deps(dir: &Path) -> Option<Vec<(String, String)>> {
    let path = dir.join("package.xi");
    if !path.is_file() {
        return None;
    }
    let text = std::fs::read_to_string(&path).ok()?;
    let pkg = crate::parse_manifest(&text);
    Some(pkg.deps.into_iter().collect())
}

/// Download, verify (sha256 + signature + lockfile) and extract ONE registry
/// artifact. Returns the installed directory.
fn install_verified(
    index: &RegistryIndex,
    package: &str,
    ver: &str,
    registry: &str,
) -> Result<PathBuf, InstallError> {
    let ver_meta = index
        .packages
        .get(package)
        .and_then(|p| p.versions.iter().find(|v| v.version == ver))
        .ok_or_else(|| {
            InstallError::NotFound(format!(
                "version '{}' of '{}' disappeared from the registry index \
                 between resolution and download",
                ver, package
            ))
        })?;

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
    Ok(pkg_dir)
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
    let root_version = ver_meta.version.clone();

    // Root artifact first, then its transitive closure; every artifact goes
    // through the full verification path (sha256 + signature + lockfile).
    let root_dir = install_verified(&index, package, &root_version, registry)?;

    // Dependency source: the VERIFIED tarball's own package.xi (authoritative),
    // falling back to index metadata when the package ships no manifest.
    let deps_of = |name: &str, ver: &str| -> Vec<(String, String)> {
        let dir = package_cache_dir().join(format!("{}-{}", name.replace('.', "-"), ver));
        installed_manifest_deps(&dir).unwrap_or_else(|| {
            index
                .packages
                .get(name)
                .and_then(|p| p.versions.iter().find(|x| x.version == ver))
                .map(|v| v.dependencies.iter().map(|(k, s)| (k.clone(), s.clone())).collect())
                .unwrap_or_default()
        })
    };
    let closure = install_closure(&index, package, &root_version, &deps_of)?;
    let mut installed_deps = 0usize;
    for entry in &closure {
        if entry.name == package && entry.version == root_version {
            continue;
        }
        println!("Installing dependency {} v{}...", entry.name, entry.version);
        install_verified(&index, &entry.name, &entry.version, registry)?;
        installed_deps += 1;
    }

    println!("Installed {} v{} to {}", package, root_version, root_dir.display());
    if installed_deps > 0 {
        println!(
            "  {} transitive dependenc{} installed",
            installed_deps,
            if installed_deps == 1 { "y" } else { "ies" }
        );
    }
    println!("  Add to your package.xi dependencies:");
    println!("    dependencies = {{ {} = \"{}\" }}", package, root_version);
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
            license: String::new(),
            categories: Vec::new(),
            keywords: Vec::new(),
            latest: latest.to_string(),
            versions: versions.iter().map(|v| RegistryVersion {
                version: v.to_string(),
                sha256: String::new(),
                signature: String::new(),
                public_key: String::new(),
                dependencies: HashMap::new(),
                yanked: false,
            }).collect(),
        }
    }

    fn version(ver: &str, deps: &[(&str, &str)], yanked: bool) -> RegistryVersion {
        RegistryVersion {
            version: ver.to_string(),
            sha256: format!("{ver}-digest"),
            signature: String::new(),
            public_key: String::new(),
            dependencies: deps.iter().map(|(n, s)| (n.to_string(), s.to_string())).collect(),
            yanked,
        }
    }

    fn index_with(entries: &[(&str, &str, Vec<RegistryVersion>)]) -> RegistryIndex {
        RegistryIndex {
            registry: "https://registry.test".to_string(),
            version: "1".to_string(),
            packages: entries.iter().map(|(name, latest, versions)| {
                (name.to_string(), RegistryPackage {
                    description: String::new(),
                    repository: String::new(),
                    license: String::new(),
                    categories: Vec::new(),
                    keywords: Vec::new(),
                    latest: latest.to_string(),
                    versions: versions.clone(),
                })
            }).collect(),
        }
    }

    #[test]
    fn search_filters_by_category_and_keywords() {
        let mut packages = HashMap::new();
        packages.insert("render".to_string(), RegistryPackage {
            description: "Rendering helpers".to_string(),
            repository: String::new(),
            license: "MIT".to_string(),
            categories: vec!["graphics".to_string()],
            keywords: vec!["gpu".to_string(), "shader".to_string()],
            latest: "1.0.0".to_string(),
            versions: vec![],
        });
        packages.insert("cli".to_string(), RegistryPackage {
            description: "Command line tools".to_string(),
            repository: String::new(),
            license: String::new(),
            categories: vec!["tools".to_string()],
            keywords: vec!["terminal".to_string()],
            latest: "0.1.0".to_string(),
            versions: vec![],
        });
        // text match on a KEYWORD
        let hits = filter_packages(&packages, "gpu", None);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, "render");
        // text match on a CATEGORY
        let hits = filter_packages(&packages, "tools", None);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, "cli");
        // category filter is case-insensitive and combines with an empty query
        let hits = filter_packages(&packages, "", Some("GRAPHICS"));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, "render");
        assert!(filter_packages(&packages, "", Some("nope")).is_empty());
    }

    #[test]
    fn version_ordering_and_constraints() {
        use std::cmp::Ordering;
        assert_eq!(version_cmp("1.2.3", "1.2.10"), Ordering::Less);
        assert_eq!(version_cmp("1.0", "1.0.0"), Ordering::Equal);
        assert_eq!(version_cmp("1.0.0-rc1", "1.0.0"), Ordering::Less);
        assert_eq!(version_cmp("2.0.0", "1.9.9"), Ordering::Greater);

        assert!(version_satisfies("0.9.9", ">=0.5.0,<1.0.0"));
        assert!(!version_satisfies("1.0.0", ">=0.5.0,<1.0.0"));
        assert!(version_satisfies("1.9.0", "^1.2.0"));
        assert!(!version_satisfies("2.0.0", "^1.2.0"));
        assert!(version_satisfies("1.2.9", "~1.2.0"));
        assert!(!version_satisfies("1.3.0", "~1.2.0"));
        assert!(version_satisfies("1.2.3", "1.2.3"));
        assert!(version_satisfies("9.9.9", "*"));
    }

    #[test]
    fn select_version_prefers_latest_skips_yanked_and_allows_pinned_yanked() {
        let p = RegistryPackage {
            description: String::new(),
            repository: String::new(),
            license: String::new(),
            categories: Vec::new(),
            keywords: Vec::new(),
            latest: "1.2.0".to_string(),
            versions: vec![
                version("1.0.0", &[], false),
                version("1.1.0", &[], true),
                version("1.2.0", &[], false),
            ],
        };
        assert_eq!(select_version(&p, "").as_deref(), Some("1.2.0"));
        assert_eq!(select_version(&p, ">=1.0.0,<2.0.0").as_deref(), Some("1.2.0"));
        // Exact pin may resolve a yanked version; ranges skip it.
        assert_eq!(select_version(&p, "1.1.0").as_deref(), Some("1.1.0"));
        assert_ne!(select_version(&p, ">=1.0.0,<1.2.0").as_deref(), Some("1.1.0"));

        // Yanked latest falls back to the highest remaining.
        let mut q = p.clone();
        q.versions[2].yanked = true;
        assert_eq!(select_version(&q, "").as_deref(), Some("1.0.0"));
    }

    #[test]
    fn install_closure_is_transitive_deterministic_and_cycle_safe() {
        let index = index_with(&[
            ("a", "1.0.0", vec![version("1.0.0", &[("c", ">=1.0.0"), ("b", ">=1.0.0")], false)]),
            ("b", "1.1.0", vec![version("1.1.0", &[("c", "1.0.0")], false), version("1.0.0", &[], false)]),
            ("c", "1.0.0", vec![version("1.0.0", &[("a", ">=1.0.0")], false)]),
        ]);
        let deps_of = |name: &str, ver: &str| -> Vec<(String, String)> {
            index.packages.get(name)
                .and_then(|p| p.versions.iter().find(|v| v.version == ver))
                .map(|v| v.dependencies.iter().map(|(k, s)| (k.clone(), s.clone())).collect())
                .unwrap_or_default()
        };
        let closure = install_closure(&index, "a", "1.0.0", &deps_of).expect("closure");
        let names: Vec<(&str, &str)> = closure.iter().map(|e| (e.name.as_str(), e.version.as_str())).collect();
        // Root first, dependencies in sorted order; the c -> a cycle edge is
        // already seen and stops.
        assert_eq!(names, vec![("a", "1.0.0"), ("b", "1.1.0"), ("c", "1.0.0")]);
        assert_eq!(closure[0].sha256, "1.0.0-digest");
    }

    #[test]
    fn closure_reports_missing_and_unsatisfiable_dependencies() {
        let index = index_with(&[
            ("a", "1.0.0", vec![version("1.0.0", &[("ghost", "1.0.0")], false)]),
            ("b", "1.0.0", vec![version("1.0.0", &[("c", ">=9.0.0")], false)]),
            ("c", "1.0.0", vec![version("1.0.0", &[], false)]),
        ]);
        let deps_of = |name: &str, ver: &str| -> Vec<(String, String)> {
            index.packages.get(name)
                .and_then(|p| p.versions.iter().find(|v| v.version == ver))
                .map(|v| v.dependencies.iter().map(|(k, s)| (k.clone(), s.clone())).collect())
                .unwrap_or_default()
        };
        let err = install_closure(&index, "a", "1.0.0", &deps_of).unwrap_err();
        assert!(err.to_string().contains("ghost"), "{err}");
        let err = install_closure(&index, "b", "1.0.0", &deps_of).unwrap_err();
        assert!(err.to_string().contains("satisfies"), "{err}");
    }

    #[test]
    fn lock_closure_resolves_roots_and_transitives() {
        let index = index_with(&[
            ("a", "1.2.0", vec![version("1.2.0", &[("b", "^1.0.0")], false), version("1.0.0", &[], false)]),
            ("b", "1.1.0", vec![version("1.1.0", &[], false)]),
        ]);
        let closure = lock_closure(&index, &[("a".to_string(), ">=1.0.0,<2.0.0".to_string())])
            .expect("lock closure");
        let names: Vec<(&str, &str)> = closure.iter().map(|e| (e.name.as_str(), e.version.as_str())).collect();
        assert_eq!(names, vec![("a", "1.2.0"), ("b", "1.1.0")]);
        // Non-registry specs never enter the registry closure.
        assert!(is_non_registry_spec("path:../lib"));
        assert!(is_non_registry_spec("git:https://x/y@0123456789abcdef0123456789abcdef01234567"));
        assert!(!is_non_registry_spec(">=1.0.0"));
    }

    #[test]
    fn platform_deps_stay_out_of_the_registry_closure() {
        // R52: a manifest may declare `xiom.std: "0.1.0"`; the closure must
        // not demand it from the registry (it is resolved locally).
        assert!(is_platform_dep("xiom.std"));
        assert!(is_platform_dep("xiom-std"));
        assert!(!is_platform_dep("xiom.hello"));
        assert!(is_non_registry_dep("xiom.std", "0.1.0"));
        assert!(is_non_registry_dep("lib", "path:../lib"));
        assert!(!is_non_registry_dep("lib", "^1.0.0"));

        let index = index_with(&[
            ("hello", "0.1.0", vec![version("0.1.0", &[("xiom.std", "0.1.0")], false)]),
        ]);
        let deps_of = |name: &str, ver: &str| -> Vec<(String, String)> {
            index.packages.get(name)
                .and_then(|p| p.versions.iter().find(|v| v.version == ver))
                .map(|v| v.dependencies.iter().map(|(k, s)| (k.clone(), s.clone())).collect())
                .unwrap_or_default()
        };
        let closure = install_closure(&index, "hello", "0.1.0", &deps_of)
            .expect("a stdlib dep must not fail the registry closure");
        assert_eq!(closure.len(), 1);
        assert_eq!(closure[0].name, "hello");
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
