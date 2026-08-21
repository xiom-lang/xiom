// XIOM -- Package Manager (registry client)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// M14.1: Extracted from main.rs -- registry download, search, install.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process;

const DEFAULT_REGISTRY: &str = "https://registry.xiom-lang.org";

pub(crate) fn registry_url() -> String {
    std::env::var("XIOM_REGISTRY").unwrap_or_else(|_| DEFAULT_REGISTRY.to_string())
}

// ============================================================================
// Native HTTP client -- falls back from curl -> PowerShell -> built-in TCP
// ============================================================================

pub(crate) fn http_get(url: &str) -> Result<String, String> {
    // Strategy 0: ureq (native Rust, TLS built-in, no external deps)
    match ureq::get(url).call() {
        Ok(resp) => return resp.into_string().map_err(|e| format!("ureq read: {e}")),
        Err(_) => {} // fall through to curl
    }
    // Strategy 1: curl (most portable, handles HTTPS)
    if let Ok(output) = process::Command::new("curl").args(["-s", "-L", url]).output() {
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
    }
    // Strategy 2: PowerShell on Windows
    #[cfg(windows)]
    {
        if let Ok(output) = process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &format!("(Invoke-WebRequest -Uri '{url}' -UseBasicParsing).Content")])
            .output()
        {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }
    // Strategy 3: built-in TCP for plain HTTP (no TLS)
    if url.starts_with("http://") {
        return http_get_tcp(url);
    }
    Err(format!("Cannot fetch {url}: no curl, no powershell, and URL requires HTTPS"))
}

fn http_get_tcp(url: &str) -> Result<String, String> {
    let url = url.strip_prefix("http://").ok_or("Invalid HTTP URL")?;
    let (host, path) = url.split_once('/').unwrap_or((url, ""));
    let host_port = if host.contains(':') { host.to_string() } else { format!("{host}:80") };
    let path = format!("/{path}");

    let mut stream = TcpStream::connect(&host_port).map_err(|e| format!("TCP connect: {e}"))?;
    let request = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).map_err(|e| format!("TCP write: {e}"))?;

    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| format!("TCP read: {e}"))?;

    // Strip HTTP headers
    if let Some(body_start) = response.find("\r\n\r\n") {
        Ok(response[body_start + 4..].to_string())
    } else {
        Ok(response)
    }
}

// ============================================================================
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
    pub(crate) description: String,
    pub(crate) repository: String,
    pub(crate) latest: String,
    pub(crate) versions: Vec<String>,
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
    if !pkg_info.versions.contains(&ver.to_string()) {
        return Err(format!("Version '{}' not found for '{}'. Available: {:?}", ver, package, pkg_info.versions));
    }

    // Download package archive
    let dl_url = format!("{}/packages/{}/{}/package.tar.gz", registry, package, ver);
    println!("Downloading {} v{} from {}...", package, ver, registry);

    let archive = http_get_binary(&dl_url)?;
    if archive.is_empty() {
        return Err(format!("Empty archive from {dl_url}"));
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

pub(crate) fn http_get_binary(url: &str) -> Result<Vec<u8>, String> {
    // Strategy 0: ureq (native Rust, TLS built-in)
    match ureq::get(url).call() {
        Ok(resp) => {
            let mut data = Vec::new();
            resp.into_reader().read_to_end(&mut data).map_err(|e| format!("read: {e}"))?;
            return Ok(data);
        }
        Err(_) => {}
    }
    // Strategy 1: curl (binary downloads, handles HTTPS)
    if let Ok(output) = process::Command::new("curl").args(["-s", "-L", url]).output() {
        if output.status.success() {
            return Ok(output.stdout);
        }
    }
    // 6C.1: Fixed PowerShell fallback -- use -OutFile for binary, then read file
    #[cfg(windows)]
    {
        let tmp = std::env::temp_dir().join(format!("xiom_pkg_dl_{}", std::process::id()));
        if let Ok(output) = process::Command::new("powershell")
            .args(["-NoProfile", "-Command",
                   &format!("Invoke-WebRequest -Uri '{url}' -OutFile '{}' -UseBasicParsing",
                            tmp.to_string_lossy().replace('\'', "''"))])
            .output()
        {
            if output.status.success() {
                if let Ok(data) = std::fs::read(&tmp) {
                    let _ = std::fs::remove_file(&tmp);
                    return Ok(data);
                }
                let _ = std::fs::remove_file(&tmp);
            }
        }
    }
    Err(format!("Cannot download binary from {url}"))
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
pub(crate) fn extract_tar_gz(data: &[u8], dest: &Path) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!("xiom_pkg_{}.tar.gz", std::process::id()));
    std::fs::write(&tmp, data).map_err(|e| format!("Write temp: {e}"))?;

    std::fs::create_dir_all(dest).map_err(|e| format!("Create dir: {e}"))?;

    let result = if cfg!(windows) {
        process::Command::new("tar").args(["-xzf", &tmp.to_string_lossy(), "-C", &dest.to_string_lossy()]).status()
    } else {
        process::Command::new("tar").args(["-xzf", &tmp.to_string_lossy(), "-C", &dest.to_string_lossy()]).status()
    };

    let _ = std::fs::remove_file(&tmp);

    match result {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("tar exited with code {}", s.code().unwrap_or(-1))),
        Err(e) => Err(format!("tar not found: {e}. Install tar to extract packages.")),
    }
}
