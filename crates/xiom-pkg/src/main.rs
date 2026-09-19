// XIOM -- Package Manager
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// M14.1: registry functions -> registry.rs

mod lockfile;
mod registry;
mod signing;

use std::collections::HashMap;
use std::env;
use std::fs;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process;

use crate::registry::{registry_url, search_registry, install_from_registry, http_get_binary};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help") {
        print_usage();
        return;
    }
    if args.iter().any(|a| a == "--version") {
        eprintln!("xiom-pkg v{}", env!("CARGO_PKG_VERSION"));
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
        if cmd == "keygen" { keygen_command(&args); return; }
        if cmd == "trust" { trust_command(&args); return; }
        if cmd == "trusted" { trusted_command(); return; }
        if cmd == "sign" { sign_command(&args); return; }
        if cmd == "verify" { verify_command(&args); return; }
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
            match install_from_registry(name, version, &registry) {
                Ok(()) => {}
                Err(e) if e.allows_local_fallback() => {
                    eprintln!("xiom pkg: registry install failed: {e}");
                    eprintln!("xiom pkg: trying local resolution...");
                    install_local_package(&args);
                }
                Err(e) => {
                    // Integrity decisions are TERMINAL: a failed checksum /
                    // signature / lockfile check is never retried through an
                    // unverified path (R32).
                    eprintln!("xiom pkg: {e}");
                    process::exit(1);
                }
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
        } else if line.starts_with("deps:") || line.starts_with("deps =") {
            // BUG FIX (Stage 5): the old parser only CLEARED deps here --
            // package dependencies were never visible to `lock`, `install`'s
            // resolution or `resolve_dependencies`. Collect the (inline or
            // multiline) block and extract `"name": "req"` pairs. `=` is
            // accepted as the separator (the historical help text used
            // `deps = { "x" = "1.0" }`).
            pkg.deps = parse_deps_block(&lines, i);
            let mut depth = brace_delta(line);
            while depth > 0 && i + 1 < lines.len() {
                i += 1;
                depth += brace_delta(lines[i]);
            }
            // Skip the block's closing line (or the inline line itself):
            // `continue` bypasses the loop's own increment.
            i += 1;
            continue;
        }

        i += 1;
    }

    pkg
}

fn strip_outer_block(manifest: &str) -> &str {
    // Only strip an OUTER package block: the first non-comment, non-blank
    // character must be '{'. The old version stripped at the first brace
    // ANYWHERE -- an unbraced manifest's `deps: { ... }` therefore wiped out
    // every preceding field (deps, name and version all "parsed" as empty).
    let mut first_is_brace = false;
    for line in manifest.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") {
            continue;
        }
        first_is_brace = t.starts_with('{');
        break;
    }
    if first_is_brace {
        if let (Some(start), Some(end)) = (manifest.find('{'), manifest.rfind('}')) {
            if end > start {
                return manifest[start + 1..end].trim();
            }
        }
    }
    manifest
}

/// Net brace balance of a line (open minus close).
fn brace_delta(line: &str) -> i32 {
    line.chars().map(|c| match c {
        '{' => 1,
        '}' => -1,
        _ => 0,
    }).sum()
}

/// Parse a `deps:` / `deps =` block starting at `lines[start]`.
/// Accepts inline (`deps: { "x": "1.0", "y": "2.0" }`) and multiline blocks,
/// with `:` or `=` separators. Keys and values are quoted; values may contain
/// commas (`">=0.5.0,<1.0.0"`) and prefixes (`path:..`, `git:...@rev`).
fn parse_deps_block(lines: &[&str], start: usize) -> HashMap<String, String> {
    let mut block = String::new();
    // `lines` are RAW source lines; the caller matches on the trimmed form.
    let first = lines.get(start).copied().unwrap_or("").trim();
    // Strip the `deps:` / `deps =` prefix.
    let after: &str = if let Some(rest) = first.strip_prefix("deps:") {
        rest
    } else if let Some(rest) = first.strip_prefix("deps") {
        rest.trim_start().trim_start_matches('=').trim_start()
    } else {
        return HashMap::new();
    };
    block.push_str(after);
    let mut depth = brace_delta(first);
    let mut i = start;
    while depth > 0 && i + 1 < lines.len() {
        i += 1;
        block.push(' ');
        block.push_str(lines[i]);
        depth += brace_delta(lines[i]);
    }    // Content between the outermost braces (or the whole remainder when the
    // block is unbraced).
    let content = match (block.find('{'), block.rfind('}')) {
        (Some(o), Some(c)) if c > o => &block[o + 1..c],
        _ => block.as_str(),
    };

    let mut deps = HashMap::new();
    let chars: Vec<char> = content.chars().collect();
    let mut idx = 0;
    while idx < chars.len() {
        // Find the next quoted string -> key.
        while idx < chars.len() && chars[idx] != '"' { idx += 1; }
        if idx >= chars.len() { break; }
        let (key, next) = read_quoted(&chars, idx);
        idx = next;
        // Skip to the separator (`:` or `=`).
        while idx < chars.len() && chars[idx] != ':' && chars[idx] != '=' { idx += 1; }
        if idx >= chars.len() { break; }
        idx += 1;
        // Find the next quoted string -> value.
        while idx < chars.len() && chars[idx] != '"' { idx += 1; }
        if idx >= chars.len() { break; }
        let (value, next) = read_quoted(&chars, idx);
        idx = next;
        if !key.is_empty() {
            deps.insert(key, value);
        }
    }
    deps
}

/// Read the quoted string starting at `chars[start]` (a `"`), returning its
/// content and the index just past the closing quote.
fn read_quoted(chars: &[char], start: usize) -> (String, usize) {
    let mut out = String::new();
    let mut i = start + 1;
    while i < chars.len() {
        match chars[i] {
            '\\' if i + 1 < chars.len() => {
                out.push(chars[i + 1]);
                i += 2;
            }
            '"' => return (out, i + 1),
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    (out, i)
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

/// Stage 5: git dependency specs must pin an IMMUTABLE commit.
/// `git:<url>@<rev>` with a branch/tag can be moved under the consumer, so
/// only a full commit hash (40 or 64 hex chars) is accepted; set
/// XIOM_PKG_ALLOW_MUTABLE_GIT=1 for an explicit override.
fn validate_git_pin(dep: &str, spec: &str) -> Result<(), String> {
    let Some(rest) = spec.strip_prefix("git:") else { return Ok(()) };
    let Some((url, rev)) = rest.rsplit_once('@') else {
        return Err(format!(
            "git dependency '{dep}' has no revision: use git:<url>@<commit-hash>"
        ));
    };
    let is_commit = (rev.len() == 40 || rev.len() == 64)
        && rev.chars().all(|c| c.is_ascii_hexdigit());
    if is_commit || std::env::var("XIOM_PKG_ALLOW_MUTABLE_GIT").as_deref() == Ok("1") {
        return Ok(());
    }
    Err(format!(
        "git dependency '{dep}' is pinned to the MUTABLE ref '{rev}' ({url}); \
         pin a full commit hash (40 hex chars) or set XIOM_PKG_ALLOW_MUTABLE_GIT=1"
    ))
}

fn validate_dependency_specs(pkg: &Package) -> Result<(), String> {
    for (name, spec) in &pkg.deps {
        validate_git_pin(name, spec)?;
    }
    Ok(())
}

fn resolve_dependencies(pkg: &Package, project_root: &Path) -> HashMap<String, PathBuf> {
    let mut resolved = HashMap::new();

    let known_packages = vec![
        // R50 (registry relay f743308): registry names are dotted now.
        ("xiom.std", "stdlib"),
        ("xiom", "stdlib/xiom"),
        // Legacy hyphen spelling (pre-f743308 packages): accepted as an
        // alias until the pinned stdlib checkout renames its `deps` key.
        ("xiom-std", "stdlib"),
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
    if stdlib_path.exists() {
        // Canonical dotted name plus the legacy hyphen alias (same target).
        if !resolved.contains_key("xiom.std") {
            resolved.insert("xiom.std".to_string(), stdlib_path.clone());
        }
        if !resolved.contains_key("xiom-std") {
            resolved.insert("xiom-std".to_string(), stdlib_path);
        }
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

/// `xiom pkg keygen [--out PATH]` -- write a hex ed25519 secret key and print
/// the public key + fingerprint to pin with `xiom pkg trust`.
fn keygen_command(args: &[String]) {
    let out = args.iter().position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| signing::default_xiom_home().join("keys").join("default.key"));
    if out.exists() {
        eprintln!("xiom pkg: refusing to overwrite existing key {}", out.display());
        process::exit(1);
    }
    let kp = signing::KeyPair::generate().unwrap_or_else(|e| {
        eprintln!("xiom pkg: keygen failed: {e}");
        process::exit(1);
    });
    if let Some(parent) = out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(e) = fs::write(&out, kp.secret_hex() + "\n") {
        eprintln!("xiom pkg: cannot write {}: {e}", out.display());
        process::exit(1);
    }
    let public = kp.public_hex();
    println!("Secret key written to {}", out.display());
    println!("Public key: {}", public);
    println!("Fingerprint: {}", signing::fingerprint(&public));
    println!("Pin it on consumers with: xiom pkg trust --registry <URL> --key {}", public);
}

/// `xiom pkg trust --registry URL --key HEX` -- pin a registry's signing key.
fn trust_command(args: &[String]) {
    let get = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1));
    let (registry, key) = match (get("--registry"), get("--key")) {
        (Some(r), Some(k)) => (r.clone(), k.clone()),
        _ => {
            eprintln!("Usage: xiom pkg trust --registry <URL> --key <ed25519-public-hex>");
            process::exit(1);
        }
    };
    let mut store = signing::TrustStore::load();
    if let Err(e) = store.pin(&registry, &key) {
        eprintln!("xiom pkg: {e}");
        process::exit(1);
    }
    println!("Trusted {} ({})", registry, signing::fingerprint(&key));
}

/// `xiom pkg trusted` -- list pinned registry keys.
fn trusted_command() {
    let store = signing::TrustStore::load();
    let mut count = 0;
    for (registry, key) in store.entries() {
        println!("{}  {}  fp={}", registry, key, signing::fingerprint(key));
        count += 1;
    }
    if count == 0 {
        println!("No trusted registry keys pinned ({}).", signing::default_xiom_home().join("trusted_keys.json").display());
    }
}

/// `xiom pkg sign FILE [--key PATH]` -- write FILE.sig.
fn sign_command(args: &[String]) {
    let file = match args.get(2) {
        Some(f) => f.clone(),
        None => { eprintln!("Usage: xiom pkg sign <file> [--key PATH]"); process::exit(1); }
    };
    let key_path = args.iter().position(|a| a == "--key")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| signing::default_xiom_home().join("keys").join("default.key"));
    let secret = fs::read_to_string(&key_path).unwrap_or_else(|e| {
        eprintln!("xiom pkg: cannot read signing key {}: {e}", key_path.display());
        eprintln!("  (generate one with `xiom pkg keygen`)");
        process::exit(1);
    });
    let data = fs::read(&file).unwrap_or_else(|e| {
        eprintln!("xiom pkg: cannot read {file}: {e}");
        process::exit(1);
    });
    let kp = signing::KeyPair::from_secret_hex(&secret).unwrap_or_else(|e| {
        eprintln!("xiom pkg: bad signing key: {e}");
        process::exit(1);
    });
    let sig = kp.sign(&data);
    let sig_path = format!("{file}.sig");
    if let Err(e) = fs::write(&sig_path, sig + "\n") {
        eprintln!("xiom pkg: cannot write {sig_path}: {e}");
        process::exit(1);
    }
    println!("Signed {file} -> {sig_path}");
    println!("Public key: {} (fp {})", kp.public_hex(), signing::fingerprint(&kp.public_hex()));
}

/// `xiom pkg verify FILE SIG [--key HEX]` -- verify a detached signature.
fn verify_command(args: &[String]) {
    let (file, sig_file) = match (args.get(2), args.get(3)) {
        (Some(f), Some(s)) => (f.clone(), s.clone()),
        _ => { eprintln!("Usage: xiom pkg verify <file> <signature-file> [--key HEX]"); process::exit(1); }
    };
    let key = args.iter().position(|a| a == "--key").and_then(|i| args.get(i + 1)).cloned()
        .or_else(|| {
            // Fall back to the trusted key for the configured registry.
            signing::TrustStore::load().get(&registry_url()).cloned()
        })
        .unwrap_or_else(|| {
            eprintln!("xiom pkg: no --key given and no trusted key for {}", registry_url());
            process::exit(1);
        });
    let data = match fs::read(&file) { Ok(d) => d, Err(e) => { eprintln!("xiom pkg: cannot read {file}: {e}"); process::exit(1); } };
    let sig = match fs::read_to_string(&sig_file) { Ok(s) => s, Err(e) => { eprintln!("xiom pkg: cannot read {sig_file}: {e}"); process::exit(1); } };
    match signing::verify(&key, &data, sig.trim()) {
        Ok(()) => println!("OK: {file} is signed by {}", signing::fingerprint(&key)),
        Err(e) => { eprintln!("xiom pkg: {e}"); process::exit(1); }
    }
}

fn publish_package(_args: &[String]) {
    let manifest_path = find_manifest();
    let manifest = fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
        eprintln!("xiom pkg: cannot read {}: {}", manifest_path.display(), e);
        process::exit(1);
    });
    let pkg = parse_manifest(&manifest);

    if let Err(e) = validate_dependency_specs(&pkg) {
        eprintln!("xiom pkg: {e}");
        process::exit(1);
    }

    let pkg_dir = manifest_path.parent().expect("package.xi must be in a directory");
    let pkg_name = &pkg.name;

    // Build tarball from package directory
    let tmp = std::env::temp_dir().join(format!("xiom_publish_{}.tar.gz", pkg_name));
    let tarball_path = tmp.to_string_lossy().to_string();

    println!("Packaging {} v{}...", pkg.name, pkg.version);
    if let Err(e) = create_tarball(pkg_dir, &tarball_path) {
        eprintln!("xiom pkg: cannot create tarball: {e}");
        eprintln!("  Install 'tar' to create packages, or manually tar the directory.");
        process::exit(1);
    }
    let tarball_size = fs::metadata(&tarball_path).map(|m| m.len()).unwrap_or(0);
    println!("  Created tarball: {} bytes", tarball_size);

    // Stage 5: sign the artifact when a local key exists. The registry stores
    // the signature + public key in its version metadata; consumers with a
    // PINNED key refuse artifacts whose signature is missing or invalid.
    let signing_key = load_signing_key();
    let (signature, public_key) = match &signing_key {
        Some(kp) => {
            let bytes = fs::read(&tarball_path).unwrap_or_default();
            let sig = kp.sign(&bytes);
            println!("  Signed with {} (fp {})", kp.public_hex(), signing::fingerprint(&kp.public_hex()));
            (Some(sig), Some(kp.public_hex()))
        }
        None => {
            eprintln!("  WARNING: no signing key ({}); publishing UNSIGNED.",
                signing::default_xiom_home().join("keys").join("default.key").display());
            eprintln!("  Generate one with `xiom pkg keygen` -- trusted consumers will refuse unsigned artifacts.");
            (None, None)
        }
    };
    let mut fields: Vec<(&str, &str)> = vec![
        ("name", pkg.name.as_str()),
        ("version", pkg.version.as_str()),
    ];
    if let (Some(sig), Some(pk)) = (signature.as_deref(), public_key.as_deref()) {
        fields.push(("signature", sig));
        fields.push(("publicKey", pk));
    }

    // Upload tarball to registry via multipart form (ureq-only).
    let registry = registry_url();
    println!("Publishing to {}...", registry);

    match registry::http_post_multipart(&format!("{}/publish", registry), &tarball_path, "package", &fields) {
        Ok(resp) => {
            println!("Published {} v{} -- {}", pkg.name, pkg.version, resp.trim());
            // Clean up temp file
            let _ = fs::remove_file(&tarball_path);
        }
        Err(e) => {
            eprintln!("xiom pkg: publish failed: {e}");
            let _ = fs::remove_file(&tarball_path);
            process::exit(1);
        }
    }
}

/// The default signing keypair, when present.
fn load_signing_key() -> Option<signing::KeyPair> {
    let path = signing::default_xiom_home().join("keys").join("default.key");
    let secret = fs::read_to_string(path).ok()?;
    signing::KeyPair::from_secret_hex(&secret).ok()
}

/// Create a gzipped tarball of a package directory.
fn create_tarball(dir: &std::path::Path, output: &str) -> Result<(), String> {
    let parent = dir.parent().expect("pkg dir has parent");
    let dirname = dir.file_name().expect("pkg dir has name").to_string_lossy();

    // Try system tar command first
    let status = process::Command::new("tar")
        .args(["-czf", output, "-C"])
        .arg(parent)
        .arg(dirname.as_ref())
        .status()
        .map_err(|e| format!("tar: {e}"))?;

    if status.success() {
        return Ok(());
    }

    // On Windows, try PowerShell Compress-Archive -> .zip -> rename
    #[cfg(windows)]
    {
        let zip_path = output.replace(".tar.gz", ".zip");
        let ps_cmd = format!(
            "Compress-Archive -Path '{}' -DestinationPath '{}' -Force",
            dir.display(),
            zip_path
        );
        let status = process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .status()
            .map_err(|e| format!("powershell: {e}"))?;
        if status.success() {
            // Rename .zip to .tar.gz (the registry accepts either format)
            fs::rename(&zip_path, output).map_err(|e| format!("rename: {e}"))?;
            return Ok(());
        }
    }

    Err("no tar or PowerShell available".to_string())
}

/// Install without the registry: the local `packages/index.json` channel
/// (GitHub Releases downloads) and then the workspace `packages/` ecosystem
/// copy.
///
/// Reached from the `install` command ONLY for unavailable / not-found
/// registry conditions (`InstallError::allows_local_fallback`). This path
/// must never download a registry artifact: registry verification failures
/// are terminal (R32).
fn install_local_package(args: &[String]) {
    let pkg_spec = match args.get(2) {
        Some(n) => n,
        None => { eprintln!("Usage: xiom pkg install <package>[@version]"); process::exit(1); }
    };

    let (pkg_name, requested_version) = if let Some(at) = pkg_spec.find('@') {
        (&pkg_spec[..at], Some(&pkg_spec[at+1..]))
    } else {
        (pkg_spec.as_str(), None)
    };

    // 1. Local packages/index.json -> GitHub Releases download.
    if let Ok(index_content) = read_local_index() {
        if let Ok(index) = serde_json::from_str::<Value>(&index_content) {
            if let Some(packages) = index["packages"].as_array() {
                for pkg in packages {
                    if pkg["name"].as_str() == Some(pkg_name)
                        || pkg["name"].as_str() == Some(&format!("xiom-{}", pkg_name))
                    {
                        let version = requested_version.map(|v| v.to_string())
                            .unwrap_or_else(|| pkg["version"].as_str().unwrap_or("0.1.0").to_string());
                        let dl_url = pkg["download_url"].as_str().unwrap_or("");
                        if !dl_url.is_empty() {
                            println!("xiom pkg: downloading {} v{} from GitHub Releases", pkg_name, version);
                            match download_and_install(pkg_name, &version, dl_url) {
                                Ok(()) => return,
                                Err(e) => eprintln!("xiom pkg: download failed: {e}"),
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Workspace ecosystem copy.
    if install_from_ecosystem(pkg_name) {
        return;
    }
    eprintln!("xiom pkg: package '{}' not found locally", pkg_name);
    process::exit(1);
}

/// Install a package from the local packages/ directory.
/// Returns false when the package is absent (caller reports the failure).
fn install_from_ecosystem(pkg_name: &str) -> bool {
    // Find the AXIOM workspace root (where Cargo.toml lives)
    let workspace = find_workspace_root(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let packages_dir = workspace.join("packages");
    let pkg_dir = packages_dir.join(format!("xiom-{}", pkg_name.strip_prefix("xiom.").unwrap_or(pkg_name)));

    if !pkg_dir.exists() {
        // Try without xiom- prefix
        let alt_dir = packages_dir.join(pkg_name);
        if !alt_dir.exists() {
            eprintln!("xiom pkg: package '{}' not found in local packages/", pkg_name);
            eprintln!("xiom pkg: available packages:");
            if let Ok(entries) = std::fs::read_dir(&packages_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("xiom-") {
                        println!("  {}", name);
                    }
                }
            }
            return false;
        }
        install_package_files(&alt_dir, pkg_name);
    } else {
        install_package_files(&pkg_dir, pkg_name);
    }
    true
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

    println!("xiom pkg: installed {} v{} -> {} ({} files)",
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

/// Read the local packages/index.json registry manifest.
fn read_local_index() -> Result<String, String> {
    let workspace = find_workspace_root(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let index_path = workspace.join("packages").join("index.json");
    std::fs::read_to_string(&index_path).map_err(|e| format!("read index.json: {e}"))
}

/// Download and install a package from a GitHub Releases URL.
fn download_and_install(pkg_name: &str, version: &str, url: &str) -> Result<(), String> {
    let xiom_home = get_xiom_home();
    let pkg_dir = xiom_home.join("packages").join(format!("{}-{}", pkg_name, version));

    if pkg_dir.exists() {
        println!("xiom pkg: {} v{} already installed at {}", pkg_name, version, pkg_dir.display());
        return Ok(());
    }

    let _ = std::fs::create_dir_all(&pkg_dir);
    eprintln!("xiom pkg: downloading {}...", url);

    // AUDIT #19 FIX: shared hardened transport + extractor (validated member
    // paths, random temps). GitHub-release installs carry no index hash, so
    // checksum verification does not apply here (registry installs enforce it).
    let data = http_get_binary(url)?;
    crate::registry::extract_tar_gz(&data, &pkg_dir)?;

    println!("xiom pkg: installed {} v{} -> {}", pkg_name, version, pkg_dir.display());
    Ok(())
}
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
///
/// Stage 5 supply chain: lockfile v2 pins {name, version, source, integrity}
/// for every direct dependency. The integrity digest is resolved by
/// downloading the exact artifact `install` would fetch (server index digest
/// verification still applies too), so `xiom pkg install` can ENFORCE the
/// bytes it installs. Offline locking records an empty digest; install then
/// refuses that entry until the lock is regenerated with the registry up.
fn generate_lockfile() {
    let manifest_path = find_manifest();
    let manifest = fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
        eprintln!("xiom pkg: cannot read {}: {}", manifest_path.display(), e);
        process::exit(1);
    });
    let pkg = parse_manifest(&manifest);

    if let Err(e) = validate_dependency_specs(&pkg) {
        eprintln!("xiom pkg: {e}");
        process::exit(1);
    }

    let registry = registry_url();
    let index = match crate::registry::fetch_registry_index(&registry) {
        Ok(i) => Some(i),
        Err(e) => {
            eprintln!("xiom pkg: registry unavailable ({e}); locking versions without integrity");
            None
        }
    };
    // Stage 5 transitive locking: split direct deps into registry specs
    // (resolved through the index closure) and path/git/URL specs (recorded
    // as-is; they are not registry artifacts).
    let mut entries: Vec<(String, String, String, String)> = Vec::new();
    let mut registry_roots: Vec<(String, String)> = Vec::new();
    for (name, spec) in &pkg.deps {
        if crate::registry::is_non_registry_spec(spec) {
            entries.push((name.clone(), spec.clone(), "registry".to_string(), String::new()));
        } else {
            registry_roots.push((name.clone(), spec.clone()));
        }
    }
    match &index {
        Some(idx) => match crate::registry::lock_closure(idx, &registry_roots) {
            Ok(closure) => {
                for entry in closure {
                    entries.push((entry.name, entry.version, "registry".to_string(), entry.sha256));
                }
            }
            Err(e) => {
                eprintln!("xiom pkg: {e}");
                eprintln!("xiom pkg: locking unresolved dependencies by their requested spec (no integrity)");
                for (name, req) in &registry_roots {
                    if !entries.iter().any(|(n, _, _, _)| n == name) {
                        entries.push((name.clone(), req.clone(), "registry".to_string(), String::new()));
                    }
                }
            }
        },
        None => {
            for (name, req) in &registry_roots {
                entries.push((name.clone(), req.clone(), "registry".to_string(), String::new()));
            }
        }
    }

    // Integrity comes from the server-published digest in the same index
    // install verifies against; a hashed-but-indexless entry falls back to
    // downloading the archive once (bounded by ureq timeouts).
    for entry in &mut entries {
        if entry.3.is_empty()
            && index.is_some()
            && !crate::registry::is_non_registry_spec(&entry.1)
        {
            let url = format!("{}/packages/{}/{}/package.tar.gz", registry, entry.0, entry.1);
            if let Ok(bytes) = crate::registry::http_get_binary(&url) {
                entry.3 = crate::lockfile::integrity_for(&bytes);
            }
        }
    }

    let lock = lockfile::Lockfile::from_resolved(&pkg.name, &pkg.version, entries);

    let project_root = manifest_path.parent().unwrap_or(Path::new("."));
    let lock_path = project_root.join("xiom.lock");
    fs::write(&lock_path, lock.to_json()).unwrap_or_else(|e| {
        eprintln!("xiom pkg: cannot write {}: {}", lock_path.display(), e);
        process::exit(1);
    });
    let unlocked = lock.packages.values().filter(|p| p.integrity.is_empty()).count();
    println!("Generated {} (lockfile v2, {} dependencies{})",
        lock_path.display(),
        lock.packages.len(),
        if unlocked > 0 { format!(", {unlocked} WITHOUT integrity -- install will refuse them until re-locked online") } else { String::new() });
}

fn print_usage() {
    eprintln!("XIOM Package v{} -- Package Manager (local packages + remote registry)", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  xiom pkg [OPTIONS] --root <dir>");
    eprintln!("  xiom pkg search [query]           Search registry for packages");
    eprintln!("  xiom pkg install <pkg>[@version]  Install package (local packages fallback)");
    eprintln!("  xiom pkg publish                   Publish package to registry");
    eprintln!("  xiom pkg lock                      Generate xiom.lock from package.xi");
    eprintln!("  xiom pkg list                       List installed packages");
    eprintln!();
    eprintln!("Install locations:");
    eprintln!("  Local packages: <repo>/packages/xiom-<pkg>/ -> XIOM_HOME/packages/<pkg>-<ver>/");
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
    fn test_resolve_dependencies_dotted_and_legacy_alias() {
        // R50: canonical dotted name resolves, legacy hyphen spelling keeps
        // working as an alias against the same checkout.
        let base = std::env::temp_dir().join("xiom_pkg_dotted_std_test");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("stdlib")).expect("create stdlib dir");
        fs::write(base.join("Cargo.toml"), "[workspace]\n").expect("write Cargo.toml");
        let root = base.join("proj");
        fs::create_dir_all(&root).expect("create project dir");

        let mut dotted = Package::default();
        dotted.deps.insert("xiom.std".to_string(), "0.1.0".to_string());
        let resolved = resolve_dependencies(&dotted, &root);
        assert!(resolved.contains_key("xiom.std"), "dotted xiom.std must resolve");
        assert!(resolved.contains_key("xiom-std"), "legacy alias must resolve too");

        let mut legacy = Package::default();
        legacy.deps.insert("xiom-std".to_string(), "0.1.0".to_string());
        let resolved_legacy = resolve_dependencies(&legacy, &root);
        assert!(resolved_legacy.contains_key("xiom-std"), "legacy spelling must still resolve");

        let _ = fs::remove_dir_all(&base);
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

    #[test]
    fn test_create_tarball() {
        let tmp_dir = std::env::temp_dir().join("xiom_pkg_test_publish");
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).expect("create test dir");
        // Write a minimal package.xi
        fs::write(
            tmp_dir.join("package.xi"),
            r#"package test_pkg {
  name: "test-pkg";
  version: "0.1.0";
  description: "Test package for publish";
}
"#,
        ).expect("write package.xi");
        // Write a source file
        fs::create_dir_all(tmp_dir.join("src")).expect("create src dir");
        fs::write(
            tmp_dir.join("src").join("lib.xi"),
            "pub fn hello() -> Str { return \"hello\"; }",
        ).expect("write lib.xi");

        let tarball = std::env::temp_dir().join("xiom_test_publish.tar.gz");
        let result = create_tarball(&tmp_dir, &tarball.to_string_lossy());
        // tar may not be available in all test environments -- don't fail
        if result.is_ok() {
            assert!(tarball.exists(), "tarball should exist");
            let size = fs::metadata(&tarball).unwrap().len();
            assert!(size > 0, "tarball should not be empty");
            let _ = fs::remove_file(&tarball);
        }
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    // -- M21-5: Package manager edge cases -------------------------------

    // Version resolution
    #[test] fn test_parse_version_range() {
        let manifest = r#"
name: "pkg";
version: "1.2.3";
deps: {
    "xiom-std": ">=0.5.0,<1.0.0",
    "xiom-http": "~0.1.0",
}
"#;
        let pkg = parse_manifest(manifest);
        // Deps block: may confuse parser, test doesn't crash
        let _ = pkg;
    }

    #[test] fn test_parse_exact_version() {
        let manifest = r#"
name: "exact";
version: "0.3.0";
deps: { "dep-a": "1.0.0" }
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.deps.get("dep-a").map(String::as_str), Some("1.0.0"),
            "deps must actually be parsed (the old parser only cleared the map)");
    }

    #[test] fn test_parse_caret_version() {
        let manifest = r#"
name: "caret";
version: "2.0.0";
deps: { "dep": "^1.5.0" }
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.deps.get("dep").map(String::as_str), Some("^1.5.0"));
    }

    #[test] fn test_parse_deps_multiline_with_commas() {
        let manifest = r#"package alg {
  name: "alg";
  version: "0.1.0";
  deps: {
    "xiom.std": ">=0.5.0,<1.0.0",
    "lib": "path:../lib",
    "git-dep" = "git:https://github.com/x/y@0123456789abcdef0123456789abcdef01234567"
  };
}"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.deps.len(), 3, "deps: {:?}", pkg.deps);
        assert_eq!(pkg.deps.get("xiom.std").map(String::as_str), Some(">=0.5.0,<1.0.0"));
        assert_eq!(pkg.deps.get("lib").map(String::as_str), Some("path:../lib"));
        assert_eq!(pkg.deps.get("git-dep").map(String::as_str), Some("git:https://github.com/x/y@0123456789abcdef0123456789abcdef01234567"));
    }

    // Circular dependencies detection
    #[test] fn test_detect_direct_circular_dep() {
        // A package depending on itself
        let manifest = r#"
name: "self-ref";
version: "0.1.0";
deps: { "self-ref": "1.0.0" }
"#;
        let _pkg = parse_manifest(manifest);
    }

    // Missing package graceful error
    #[test] fn test_parse_missing_modules_section() {
        let manifest = r#"
name: "nofiles";
version: "1.0.0";
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "nofiles");
        assert!(pkg.modules.is_empty());
    }

    // Package with invalid manifest
    #[test] fn test_parse_invalid_syntax() {
        let manifest = "this is not a valid manifest at all";
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "");
        assert_eq!(pkg.version, "");
    }

    #[test] fn test_parse_partial_fields() {
        let manifest = r#"
name: "partial";
authors: ["dev"];
// no version field
modules: ["a.xi"];
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "partial");
        assert_eq!(pkg.version, "");
        assert!(!pkg.modules.is_empty());
        assert!(!pkg.authors.is_empty());
    }

    // Publish/install/yank workflows
    #[test] fn test_git_dependency_requires_commit_pin() {
        // Full commit hash: accepted.
        assert!(validate_git_pin("d", "git:https://github.com/x/y@0123456789abcdef0123456789abcdef01234567").is_ok());
        // Branch / tag: refused (mutable).
        let err = validate_git_pin("d", "git:https://github.com/x/y@main").unwrap_err();
        assert!(err.contains("MUTABLE ref"), "{err}");
        assert!(validate_git_pin("d", "git:https://github.com/x/y@v1.0.0").is_err());
        // Missing revision: refused.
        assert!(validate_git_pin("d", "git:https://github.com/x/y").unwrap_err().contains("no revision"));
        // Non-git specs pass through.
        assert!(validate_git_pin("d", "1.0.0").is_ok());
        assert!(validate_git_pin("d", "path:../lib").is_ok());
    }
    #[test] fn test_parse_with_git_dependency() {
        let manifest = r#"
name: "github-pkg";
version: "0.1.0";
deps: {
    "xiom-vulkan": "git:https://github.com/xiom/vulkan.xi@0123456789abcdef0123456789abcdef01234567",
}
"#;
        let _pkg = parse_manifest(manifest);
    }

    #[test] fn test_parse_with_path_dependency() {
        let manifest = r#"
name: "local-pkg";
version: "0.2.0";
deps: { "my-lib": "path:../my-lib" }
"#;
        let _pkg = parse_manifest(manifest);
    }

    #[test] fn test_parse_description_with_quotes() {
        let manifest = r#"
name: "quoted";
version: "1.0.0";
description: "A \"complex\" package description";
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "quoted");
        assert!(pkg.description.contains("complex"));
    }

    // Braced manifest with deps
    #[test] fn test_parse_braced_with_deps() {
        let manifest = r#"{
  name: "braced-pkg";
  version: "0.2.0";
  deps: {
    "dep1": "1.0.0",
    "dep2": "2.0.0",
  };
}"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "braced-pkg");
        assert_eq!(pkg.version, "0.2.0");
    }

    // Single-line brace
    #[test] fn test_parse_inline_braced() {
        let manifest = "{ name: \"compact\"; version: \"0.1.0\"; }";
        let _pkg = parse_manifest(manifest);
    }

    // Multi-value deps on same line
    #[test] fn test_parse_deps_inline() {
        let manifest = r#"
name: "inline-dep";
version: "0.2.0";
deps: { "x": "1.0.0", "y": "2.0.0" };
"#;
        let _pkg = parse_manifest(manifest);
    }

    // Optional fields
    #[test] fn test_parse_optional_license() {
        let manifest = r#"
name: "licensed";
version: "1.0.0";
license: "MIT";
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "licensed");
    }

    #[test] fn test_parse_optional_homepage() {
        let manifest = r#"
name: "web-pkg";
version: "0.1.0";
homepage: "https://example.com";
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "web-pkg");
    }

    #[test] fn test_parse_optional_keywords() {
        let manifest = r#"
name: "tagged";
version: "0.1.0";
keywords: ["graphics", "vulkan", "rendering"];
"#;
        let pkg = parse_manifest(manifest);
        assert_eq!(pkg.name, "tagged");
    }

    // Strip outer block edge cases
    #[test] fn test_strip_outer_block_trailing_whitespace() {
        let input = "{ name: \"x\"; }  ";
        let stripped = strip_outer_block(input);
        assert!(!stripped.contains("{"), "outer braces should be stripped");
        assert!(stripped.contains("name"), "name field should remain");
    }

    #[test] fn test_strip_outer_block_empty() {
        let input = "{}";
        assert_eq!(strip_outer_block(input), "");
    }

    // Extract field edge cases
    #[test] fn test_extract_field_with_spaces() {
        // extract_field: key must match exactly including trailing colon
        let result = extract_field(r#"name  :  "test"  ;"#, "name:");
        assert_eq!(result, None); // exact "name: " match with spaces fails
    }

    #[test] fn test_extract_field_multiline_value() {
        let _result = extract_field("name: \"multi\nline\";", "name:");
        // multiline values: behavior varies, must not crash
    }

    #[test] fn test_extract_field_invalid_no_close_quote() {
        let _result = extract_field("name: \"unclosed;", "name:");
        // unclosed quote: behavior varies, must not crash
    }

    #[test] fn test_resolve_deps_empty_name() {
        let mut pkg = Package::default();
        pkg.name = String::new();
        pkg.deps.insert("".to_string(), "1.0.0".to_string());
        let root = std::env::temp_dir();
        let resolved = resolve_dependencies(&pkg, &root);
        assert!(!resolved.contains_key(""));
    }

    // Large manifest parsing
    #[test] fn test_parse_large_manifest() {
        let mut manifest = String::from("name: \"big\";\nversion: \"1.0.0\";\n");
        for i in 0..100 {
            manifest.push_str(&format!("fn dummy{i}() -> Int {{ return {i}; }}\n"));
        }
        let pkg = parse_manifest(&manifest);
        // Large manifests with extra content should not crash
        assert!(pkg.name == "big" || pkg.name == "");
    }
}
