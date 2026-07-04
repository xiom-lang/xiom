// XIOM — Package Manager
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help") {
        print_usage();
        return;
    }

    if let Some(cmd) = args.get(1) {
        if cmd == "publish" {
            publish_package(&args);
            return;
        }
        if cmd == "install" {
            install_package(&args);
            return;
        }
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

    let body = format!(
        r#"{{"name":"{}","version":"{}","description":"{}"}}"#,
        pkg.name, pkg.version, pkg.description
    );

    let output = process::Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "http://localhost:8080/publish",
            "-H",
            "Content-Type: application/json",
            "-d",
            &body,
        ])
        .output()
        .unwrap_or_else(|e| {
            eprintln!("xiom pkg: failed to run curl: {}", e);
            process::exit(1);
        });

    if output.status.success() {
        println!("Published {} v{}", pkg.name, pkg.version);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("xiom pkg: publish failed: {}", stderr);
        process::exit(1);
    }
}

fn install_package(args: &[String]) {
    let pkg_name = match args.get(2) {
        Some(n) => n,
        None => {
            eprintln!("Usage: xiom pkg install <package>");
            process::exit(1);
        }
    };

    let output = process::Command::new("curl")
        .args(["-s", "http://localhost:8080/index.json"])
        .output()
        .unwrap_or_else(|e| {
            eprintln!("xiom pkg: failed to run curl: {}", e);
            process::exit(1);
        });

    if !output.status.success() {
        eprintln!("xiom pkg: failed to fetch registry");
        process::exit(1);
    }

    let body = String::from_utf8_lossy(&output.stdout);

    let search = format!("\"name\":\"{}\",\"version\":\"", pkg_name);
    if let Some(pos) = body.find(&search) {
        let rest = &body[pos + search.len()..];
        let version = rest.split('"').next().unwrap_or("?");
        println!("Found: {} v{}", pkg_name, version);
        println!("Install directory: <project>/vendor/{}", pkg_name);
    } else {
        println!("Package '{}' not found in registry.", pkg_name);
    }
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

fn print_usage() {
    eprintln!("XIOM Package v0.10.1 -- Package Manager");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  xiom pkg [OPTIONS] --root <dir>");
    eprintln!("  xiom pkg publish                   Publish package to registry");
    eprintln!("  xiom pkg install <name>            Install package from registry");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --help        Show this help message");
    eprintln!("  --list        List package modules");
    eprintln!("  --resolve     Show resolved dependency tree");
    eprintln!("  --root <dir>  Package root directory");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  xiom pkg --list --root stdlib");
    eprintln!("  xiom pkg --root myproject");
}
