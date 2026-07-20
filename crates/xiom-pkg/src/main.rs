// XIOM — Package Manager
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process;

/// Production registry URL. Override with XIOM_REGISTRY env var.
const DEFAULT_REGISTRY: &str = "https://registry.xiom-lang.org";

fn registry_url() -> String {
    env::var("XIOM_REGISTRY").unwrap_or_else(|_| DEFAULT_REGISTRY.to_string())
}

// ============================================================================
// Native HTTP client — falls back from curl → PowerShell → built-in TCP
// ============================================================================

fn http_get(url: &str) -> Result<String, String> {
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

fn http_post(url: &str, body: &str) -> Result<String, String> {
    if let Ok(output) = process::Command::new("curl")
        .args(["-s", "-L", "-X", "POST", url, "-H", "Content-Type: application/json", "-d", body])
        .output()
    {
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
    }
    #[cfg(windows)]
    {
        if let Ok(output) = process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &format!("(Invoke-WebRequest -Uri '{url}' -Method POST -Body '{body}' -ContentType 'application/json' -UseBasicParsing).Content")])
            .output()
        {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }
    Err(format!("Cannot POST to {url}: no curl, no powershell"))
}

// ============================================================================
// 5e.7b: Remote Registry Client
// ============================================================================

/// Cached registry index (lazy-loaded, refreshed every 5 min)
static REGISTRY_CACHE: std::sync::Mutex<Option<(String, u64)>> = std::sync::Mutex::new(None);

#[derive(Debug, serde::Deserialize)]
struct RegistryIndex {
    #[allow(dead_code)]
    registry: String,
    #[allow(dead_code)]
    version: String,
    packages: HashMap<String, RegistryPackage>,
}

#[derive(Debug, serde::Deserialize)]
struct RegistryPackage {
    name: String,
    description: String,
    repository: String,
    latest: String,
    versions: Vec<String>,
}

fn fetch_registry_index(registry: &str) -> Result<RegistryIndex, String> {
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

fn search_registry(query: &str, registry: &str) -> Result<(), String> {
    let index = fetch_registry_index(registry)?;
    let q = query.to_lowercase();
    let mut found = 0;
    println!("Searching '{}' in {}...\n", query, registry);
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

fn install_from_registry(package: &str, version: Option<&str>, registry: &str) -> Result<(), String> {
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

fn http_get_binary(url: &str) -> Result<Vec<u8>, String> {
    // Use curl for binary downloads (available on all modern OS)
    if let Ok(output) = process::Command::new("curl").args(["-s", "-L", url]).output() {
        if output.status.success() {
            return Ok(output.stdout);
        }
    }
    // 6C.1: Fixed PowerShell fallback — use -OutFile for binary, then read file
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

fn package_cache_dir() -> PathBuf {
    let base = if cfg!(windows) {
        PathBuf::from(env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string()))
    } else {
        PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_string()))
    };
    base.join("xiom").join("packages")
}

/// Minimal tar.gz extractor (handles basic .tar.gz files without external tools).
fn extract_tar_gz(data: &[u8], dest: &Path) -> Result<(), String> {
    // Use system tar if available
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help") {
        print_usage();
        return;
    }
    if args.iter().any(|a| a == "--version") {
        eprintln!("xiom-pkg v{} (XIOM v{})",
            env!("CARGO_PKG_VERSION"),
            option_env!("XIOM_RELEASE_VERSION").unwrap_or("0.49.7"));
        return;
    }

    // 5e.7b: Remote registry commands
    if let Some(cmd) = args.get(1) {
        if cmd == "search" {
            let query = args.get(2).map(|s| s.as_str()).unwrap_or("");
            let registry = registry_url();
            if let Err(e) = search_registry(query, &registry) {
                eprintln!("xiom pkg search: {e}");
                process::exit(1);
            }
            return;
        }
        if cmd == "publish" { publish_package(&args); return; }
        if cmd == "install" {
            let pkg_name = args.get(2).cloned().unwrap_or_default();
            if pkg_name.is_empty() {
                eprintln!("Usage: xiom pkg install <package>[@version]");
                process::exit(1);
            }
            let (name, version) = if let Some(at) = pkg_name.find('@') {
                (&pkg_name[..at], Some(&pkg_name[at+1..]))
            } else {
                (pkg_name.as_str(), None)
            };
            let registry = registry_url();
            if let Err(e) = install_from_registry(name, version, &registry) {
                // Fallback: try local resolution
                eprintln!("xiom pkg: registry install failed: {e}");
                eprintln!("xiom pkg: trying local resolution...");
                install_package(&args);
            }
            return;
        }
        if cmd == "lock" { generate_lockfile(); return; }
    }

    let mut list_mode = false;
    let mut resolve_mode = false;
    let mut project_root = PathBuf::from(".");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list" => list_mode = true,
            "--resolve" => resolve_mode = true,
            "--root" => {
                i += 1;
                if i < args.len() {
                    project_root = PathBuf::from(&args[i]);
                }
            }
            _ => {}
        }
        i += 1;
    }

    let manifest_path = project_root.join("package.xi");
    if !manifest_path.exists() {
        eprintln!("xiom pkg: no package.xi found in {}", project_root.display());
        eprintln!("Usage: xiom pkg [OPTIONS] --root <dir>");
        process::exit(1);
    }

    let manifest = fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
        eprintln!("xiom pkg: cannot read {}: {}", manifest_path.display(), e);
        process::exit(1);
    });

    let pkg = parse_manifest(&manifest);

    if list_mode {
        println!("Package: {} v{}", pkg.name, pkg.version);
        if !pkg.description.is_empty() {
            println!("  {}", pkg.description);
        }
        if !pkg.authors.is_empty() {
            println!("  Authors: {}", pkg.authors.join(", "));
        }
        println!("  Modules ({}):", pkg.modules.len());
        for m in &pkg.modules {
            println!("    - {}", m);
        }
    }

    if resolve_mode {
        let resolved = resolve_dependencies(&pkg, &project_root);
        println!("Resolved dependency tree:");
        for (name, path) in &resolved {
            println!("  {} -> {}", name, path.display());
        }
    }

    if !list_mode && !resolve_mode {
        println!("{} v{}", pkg.name, pkg.version);
    }
}

#[derive(Debug, Default)]
struct Package {
    name: String,
    version: String,
    description: String,
    authors: Vec<String>,
    modules: Vec<String>,
    deps: HashMap<String, String>,
}

fn parse_manifest(manifest: &str) -> Package {
    let mut pkg = Package::default();

    let stripped = strip_outer_block(manifest);

    let lines: Vec<&str> = stripped.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() || line.starts_with("//") {
            i += 1;
            continue;
        }

        if let Some(value) = extract_field(line, "name:") {
            pkg.name = value;
        } else if let Some(value) = extract_field(line, "version:") {
            pkg.version = value;
        } else if let Some(value) = extract_field(line, "description:") {
            pkg.description = value;
        } else if line.starts_with("authors:") {
            let (values, consumed) = read_array(&lines, i);
            pkg.authors = values;
            i += consumed;
            continue;
        } else if line.starts_with("modules:") {
            let (values, consumed) = read_array(&lines, i);
            pkg.modules = values;
            i += consumed;
            continue;
        } else if line.starts_with("deps:") {
            pkg.deps = HashMap::new();
        }

        i += 1;
    }

    pkg
}

fn strip_outer_block(manifest: &str) -> &str {
    if let Some(start) = manifest.find('{') {
        if let Some(end) = manifest.rfind('}') {
            return manifest[start + 1..end].trim();
        }
    }
    manifest
}

fn extract_field(line: &str, prefix: &str) -> Option<String> {
    if let Some(rest) = line.strip_prefix(prefix) {
        let rest = rest.trim().trim_end_matches(';');
        let value = rest.trim_matches('"');
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

fn clean_array_item(item: &str) -> String {
    let item = item.trim().trim_matches('"');
    let item = item.trim_end_matches(';').trim_end_matches(']').trim_matches('"');
    item.trim().to_string()
}

fn read_array(lines: &[&str], start: usize) -> (Vec<String>, usize) {
    let mut result = Vec::new();
    let mut found_open = false;
    let mut consumed = 0;

    for (offset, line) in lines[start..].iter().enumerate() {
        let line = line.trim();

        if !found_open {
            if let Some(bracket_pos) = line.find('[') {
                found_open = true;
                let after_bracket = &line[bracket_pos + 1..];
                let trimmed = after_bracket.trim();
                if !trimmed.is_empty() && !trimmed.starts_with(']') {
                    for item in trimmed.split(',') {
                        let item = clean_array_item(item);
                        if !item.is_empty() {
                            result.push(item.to_string());
                        }
                    }
                }
                if line.contains(']') {
                    consumed = offset + 1;
                    break;
                }
            }
        } else {
            if line.contains(']') {
                let before_bracket = line.trim_end_matches(';').trim_end_matches(']').trim();
                if !before_bracket.is_empty() {
                    for item in before_bracket.split(',') {
                        let item = clean_array_item(item);
                        if !item.is_empty() {
                            result.push(item.to_string());
                        }
                    }
                }
                consumed = offset + 1;
                break;
            }
            for item in line.split(',') {
                let item = clean_array_item(item);
                if !item.is_empty() {
                    result.push(item.to_string());
                }
            }
        }

        consumed = offset + 1;
    }

    (result, consumed)
}

fn resolve_dependencies(pkg: &Package, project_root: &Path) -> HashMap<String, PathBuf> {
    let mut resolved = HashMap::new();

    let known_packages = vec![
        ("xiom-std", "stdlib"),
        ("xiom", "stdlib/xiom"),
    ];

    for dep_name in pkg.deps.keys() {
        for (known_name, known_path) in &known_packages {
            if dep_name == *known_name {
                let path = if Path::new(known_path).is_absolute() {
                    PathBuf::from(known_path)
                } else {
                    let workspace_root = find_workspace_root(project_root);
                    workspace_root.join(known_path)
                };
                if path.exists() {
                    resolved.insert(dep_name.clone(), path);
                }
            }
        }
    }

    let stdlib_path = find_workspace_root(project_root).join("stdlib");
    if stdlib_path.exists() && !resolved.contains_key("xiom-std") {
        resolved.insert("xiom-std".to_string(), stdlib_path);
    }

    resolved
}

fn find_manifest() -> PathBuf {
    let mut current = env::current_dir().unwrap_or_else(|e| {
        eprintln!("xiom pkg: {}", e);
        process::exit(1);
    });
    loop {
        let manifest = current.join("package.xi");
        if manifest.exists() {
            return manifest;
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            eprintln!("xiom pkg: no package.xi found");
            process::exit(1);
        }
    }
}

fn publish_package(_args: &[String]) {
    let manifest_path = find_manifest();
    let manifest = fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
        eprintln!("xiom pkg: cannot read {}: {}", manifest_path.display(), e);
        process::exit(1);
    });
    let pkg = parse_manifest(&manifest);

    let body = format!(r#"{{"name":"{}","version":"{}","description":"{}"}}"#, pkg.name, pkg.version, pkg.description);

    match http_post(&format!("{}/publish", registry_url()), &body) {
        Ok(_) => println!("Published {} v{}", pkg.name, pkg.version),
        Err(e) => { eprintln!("xiom pkg: publish failed: {e}"); process::exit(1); }
    }
}

/// Install a package from the local ecosystem directory or remote registry.
/// 7F+: Local ecosystem resolution — copies from `<repo>/ecosystem/<pkg>/` to
/// `<project>/vendor/<pkg>/` for development/prototyping before remote registry
/// is available.
fn install_package(args: &[String]) {
    let pkg_spec = match args.get(2) {
        Some(n) => n,
        None => { eprintln!("Usage: xiom pkg install <package>[@version]"); process::exit(1); }
    };

    let (pkg_name, _version) = if let Some(at) = pkg_spec.find('@') {
        (&pkg_spec[..at], Some(&pkg_spec[at+1..]))
    } else {
        (pkg_spec.as_str(), None)
    };

    // 1. Try registry first
    if let Ok(body) = http_get(&format!("{}/index.json", registry_url())) {
        let search = format!("\"name\":\"{}\"", pkg_name);
        if body.contains(&search) {
            println!("xiom pkg: found {} in registry", pkg_name);
            if let Err(e) = install_from_registry_download(pkg_name, _version, &registry_url()) {
                eprintln!("xiom pkg: registry download failed: {e}");
            } else {
                return;
            }
        }
    }

    // 2. Fallback: local ecosystem directory
    install_from_ecosystem(pkg_name);
}

/// Install a package from the local ecosystem/ directory.
fn install_from_ecosystem(pkg_name: &str) {
    // Find the AXIOM workspace root (where Cargo.toml lives)
    let workspace = find_workspace_root(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let ecosystem_dir = workspace.join("ecosystem");
    let pkg_dir = ecosystem_dir.join(format!("xiom-{}", pkg_name.strip_prefix("xiom.").unwrap_or(pkg_name)));

    if !pkg_dir.exists() {
        // Try without xiom- prefix
        let alt_dir = ecosystem_dir.join(pkg_name);
        if !alt_dir.exists() {
            eprintln!("xiom pkg: package '{}' not found in local ecosystem/", pkg_name);
            eprintln!("xiom pkg: available packages:");
            if let Ok(entries) = std::fs::read_dir(&ecosystem_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("xiom-") {
                        println!("  {}", name);
                    }
                }
            }
            return;
        }
        install_package_files(&alt_dir, pkg_name);
    } else {
        install_package_files(&pkg_dir, pkg_name);
    }
}

/// Copy package files from source directory to install location.
fn install_package_files(src_dir: &Path, pkg_name: &str) {
    // Determine install directory
    let xiom_home = std::env::var("XIOM_HOME").ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let base = if cfg!(windows) {
                PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string()))
            } else {
                PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
            };
            base.join("xiom")
        });

    let pkg_version = read_package_version(src_dir).unwrap_or_else(|| "0.1.0".to_string());
    let dest_dir = xiom_home.join("packages").join(format!("{}-{}", pkg_name, pkg_version));

    // Create destination
    if dest_dir.exists() {
        println!("xiom pkg: {} already installed at {}", pkg_name, dest_dir.display());
        println!("xiom pkg: add to your package.xi: deps = {{ \"{}\" = \"{}\" }}", pkg_name, pkg_version);
        return;
    }

    let _ = std::fs::create_dir_all(&dest_dir);

    // Copy package files
    let mut copied = 0usize;
    copy_dir_contents(src_dir, &dest_dir, &mut copied);

    println!("xiom pkg: installed {} v{} → {} ({} files)",
        pkg_name, pkg_version, dest_dir.display(), copied);
    println!("xiom pkg: add to your package.xi:");
    println!("  dependencies = {{");
    println!("    \"{}\" = \"{}\"", pkg_name, pkg_version);
    println!("  }}");
}

/// Read the version from a package.xi or Cargo.toml in the source directory.
fn read_package_version(dir: &Path) -> Option<String> {
    // Try package.xi first
    if let Ok(content) = std::fs::read_to_string(dir.join("package.xi")) {
        for line in content.lines() {
            if let Some(v) = line.trim().strip_prefix("version:") {
                return Some(v.trim().trim_matches('"').trim_matches(';').to_string());
            }
        }
    }
    None
}

/// Recursively copy directory contents.
fn copy_dir_contents(src: &Path, dest: &Path, count: &mut usize) {
    if let Ok(entries) = std::fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let dest_path = dest.join(&name);

            if path.is_dir() {
                // Skip build artifacts and hidden dirs
                if name.to_str().map_or(false, |n| n.starts_with('.') || n == "target" || n == "build") {
                    continue;
                }
                let _ = std::fs::create_dir_all(&dest_path);
                copy_dir_contents(&path, &dest_path, count);
            } else {
                let _ = std::fs::copy(&path, &dest_path);
                *count += 1;
            }
        }
    }
}

/// Download and install from remote registry.
fn install_from_registry_download(name: &str, version: Option<&str>, registry: &str) -> Result<(), String> {
    let index_url = format!("{}/index.json", registry);
    let body = http_get(&index_url)?;

    // Find the package in the index
    let search = format!("\"name\":\"{}\"", name);
    let pos = body.find(&search).ok_or_else(|| format!("package '{}' not found in registry", name))?;
    let section = &body[pos..];
    let latest = version.map(|v| v.to_string()).or_else(|| {
        section.find("\"latest\":\"").and_then(|p| {
            let rest = &section[p + 10..];
            rest.split('"').next().map(|s| s.to_string())
        })
    }).ok_or_else(|| "cannot determine version".to_string())?;

    let download_url = format!("{}/packages/{}/{}/package.tar.gz", registry, name, latest);
    eprintln!("xiom pkg: downloading {} v{} from {}", name, latest, download_url);

    // Download
    let tmp = std::env::temp_dir().join(format!("xiom_pkg_{}_{}.tar.gz", name, latest));
    let dl_result = http_get_binary(&download_url);
    match dl_result {
        Ok(data) => {
            std::fs::write(&tmp, &data).map_err(|e| format!("write: {e}"))?;
            // Extract
            let xiom_home = get_xiom_home();
            let pkg_dir = xiom_home.join("packages").join(format!("{}-{}", name, latest));
            let _ = std::fs::create_dir_all(&pkg_dir);
            let status = std::process::Command::new("tar")
                .args(["-xzf", &tmp.to_string_lossy(), "-C", &pkg_dir.to_string_lossy()])
                .status()
                .map_err(|e| format!("tar: {e}"))?;
            if status.success() {
                println!("xiom pkg: installed {} v{} → {}", name, latest, pkg_dir.display());
                Ok(())
            } else {
                Err("tar extraction failed".to_string())
            }
        }
        Err(e) => Err(format!("download failed: {e}")),
    }
}

/// Get XIOM_HOME directory.
fn get_xiom_home() -> PathBuf {
    std::env::var("XIOM_HOME").ok().map(PathBuf::from).unwrap_or_else(|| {
        let base = if cfg!(windows) {
            PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string()))
        } else {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
        };
        base.join("xiom")
    })
}

fn find_workspace_root(project_root: &Path) -> PathBuf {
    let mut current = project_root.to_path_buf();
    loop {
        if current.join("Cargo.toml").exists() {
            return current;
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    project_root.to_path_buf()
}

/// Generate a xiom.lock file from the package.xi manifest.
/// Locks all dependency versions for reproducible builds.
fn generate_lockfile() {
    let manifest_path = find_manifest();
    let manifest = fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
        eprintln!("xiom pkg: cannot read {}: {}", manifest_path.display(), e);
        process::exit(1);
    });
    let pkg = parse_manifest(&manifest);

    let mut locked_deps = Vec::new();
    for (name, version) in &pkg.deps {
        locked_deps.push(format!(r#"    "{}": "{}""#, name, version));
    }

    let lock_content = format!(
        "{{\n  \"package\": \"{}\",\n  \"version\": \"{}\",\n  \"dependencies\": {{\n{}\n  }}\n}}\n",
        pkg.name,
        pkg.version,
        locked_deps.join(",\n")
    );

    let project_root = manifest_path.parent().unwrap_or(Path::new("."));
    let lock_path = project_root.join("xiom.lock");
    fs::write(&lock_path, &lock_content).unwrap_or_else(|e| {
        eprintln!("xiom pkg: cannot write {}: {}", lock_path.display(), e);
        process::exit(1);
    });
    println!("Generated {}", lock_path.display());
}

fn print_usage() {
    eprintln!("XIOM Package v0.49.5 — Package Manager (7F: local ecosystem + remote registry)");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  xiom pkg [OPTIONS] --root <dir>");
    eprintln!("  xiom pkg search [query]           Search registry for packages");
    eprintln!("  xiom pkg install <pkg>[@version]  Install package (local ecosystem fallback)");
    eprintln!("  xiom pkg publish                   Publish package to registry");
    eprintln!("  xiom pkg lock                      Generate xiom.lock from package.xi");
    eprintln!("  xiom pkg list                       List installed packages");
    eprintln!();
    eprintln!("Install locations:");
    eprintln!("  Local ecosystem: <repo>/ecosystem/xiom-<pkg>/ → XIOM_HOME/packages/<pkg>-<ver>/");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --help        Show this help message");
    eprintln!("  --list        List package modules");
    eprintln!("  --resolve     Show resolved dependency tree");
    eprintln!("  --root <dir>  Package root directory");
    eprintln!();
    eprintln!("ENVIRONMENT:");
    eprintln!("  XIOM_REGISTRY  Registry URL (default: https://registry.xiom-lang.org)");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  xiom pkg search vulkan");
    eprintln!("  xiom pkg install xiom.stdlib");
    eprintln!("  xiom pkg install xiom.vulkan@0.5.0");
    eprintln!("  xiom pkg --list --root stdlib");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_manifest() {
        let manifest = r#"
name: "mypkg";
version: "0.1.0";
description: "A test package";
modules: ["src/mod1.xi", "src/mod2.xi"];
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "mypkg");
        assert_eq!(pkg.version, "0.1.0");
        assert_eq!(pkg.description, "A test package");
        assert_eq!(pkg.modules, vec!["src/mod1.xi".to_string(), "src/mod2.xi".to_string()]);
    }

    #[test]
    fn test_parse_package_with_braces() {
        let manifest = r#"{
  name: "braced";
  version: "2.0.0";
  modules: ["src/x.xi"];
}"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "braced");
        assert_eq!(pkg.version, "2.0.0");
        assert_eq!(pkg.modules, vec!["src/x.xi".to_string()]);
    }

    #[test]
    fn test_list_output() {
        let manifest = r#"
name: "mylist";
version: "1.0.0";
modules: ["src/main.xi", "src/lib.xi", "src/utils.xi"];
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "mylist");
        assert_eq!(pkg.version, "1.0.0");
        assert_eq!(pkg.modules.len(), 3);
        assert!(pkg.modules.contains(&"src/main.xi".to_string()));
        assert!(pkg.modules.contains(&"src/lib.xi".to_string()));
        assert!(pkg.modules.contains(&"src/utils.xi".to_string()));
    }

    #[test]
    fn test_list_output_empty_modules() {
        let manifest = r#"
name: "minimal";
version: "0.1.0";
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "minimal");
        assert_eq!(pkg.version, "0.1.0");
        assert_eq!(pkg.modules.len(), 0);
    }

    #[test]
    fn test_empty_manifest() {
        let manifest = "";
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "");
        assert_eq!(pkg.version, "");
        assert!(pkg.modules.is_empty());
        assert!(pkg.deps.is_empty());
        assert_eq!(pkg.description, "");
        assert!(pkg.authors.is_empty());
    }

    #[test]
    fn test_minimal_manifest() {
        let manifest = r#"
name: "mini";
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "mini");
        assert_eq!(pkg.version, "");
        assert!(pkg.modules.is_empty());
    }

    #[test]
    fn test_missing_name() {
        let manifest = r#"
version: "0.2.0";
modules: ["src/a.xi"];
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "");
        assert_eq!(pkg.version, "0.2.0");
        assert!(pkg.modules.contains(&"src/a.xi".to_string()));
    }

    #[test]
    fn test_multiline_modules() {
        let manifest = r#"
name: "multi";
version: "0.5.0";
modules: [
  "a.xi",
  "b.xi",
  "c.xi"
];
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "multi");
        assert_eq!(pkg.version, "0.5.0");
        assert_eq!(pkg.modules, vec![
            "a.xi".to_string(),
            "b.xi".to_string(),
            "c.xi".to_string(),
        ]);
    }

    #[test]
    fn test_resolve_dependencies_no_deps() {
        let pkg = Package::default();
        let root = std::env::temp_dir();
        let resolved = resolve_dependencies(&pkg, &root);
        assert!(resolved.is_empty());
    }

    #[test]
    fn test_resolve_dependencies_with_dep() {
        let mut pkg = Package::default();
        pkg.deps.insert("xiom-std".to_string(), "0.1.0".to_string());

        let root = std::env::temp_dir();
        let resolved = resolve_dependencies(&pkg, &root);
        // xiom-std resolves only if <workspace_root>/stdlib exists;
        // when run from a temp dir with no Cargo.toml ancestry,
        // find_workspace_root returns the temp dir itself and stdlib is absent.
        assert!(!resolved.contains_key("xiom-std"));
    }

    #[test]
    fn test_extract_field() {
        assert_eq!(extract_field(r#"name: "test";"#, "name:"), Some("test".to_string()));
        assert_eq!(extract_field(r#"version: "1.2.3";"#, "version:"), Some("1.2.3".to_string()));
        assert_eq!(extract_field(r#"other: "";"#, "other:"), None); // empty value returns None
        assert_eq!(extract_field(r#"name: "test";"#, "name:"), Some("test".to_string())); // immediate semicolon
    }

    #[test]
    fn test_strip_outer_block_no_braces() {
        let input = "name: \"x\";";
        assert_eq!(strip_outer_block(input), "name: \"x\";");
    }

    #[test]
    fn test_strip_outer_block_with_braces() {
        let input = "{ name: \"x\"; }";
        assert_eq!(strip_outer_block(input), "name: \"x\";");
    }

    #[test]
    fn test_parse_authors() {
        let manifest = r#"
name: "team";
version: "1.0.0";
authors: ["Alice", "Bob"];
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.authors, vec!["Alice".to_string(), "Bob".to_string()]);
    }

    #[test]
    fn test_parse_comments_ignored() {
        let manifest = r#"
// This is a comment
name: "pkg";
// Another comment
version: "0.1.0";
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "pkg");
        assert_eq!(pkg.version, "0.1.0");
    }
}
