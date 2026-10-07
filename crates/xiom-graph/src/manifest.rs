// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `xiom.toml` project manifest parsing and source root resolution.
//!
//! Supports two formats:
//! - `xiom.toml` (TOML) -- the canonical Phase 7 manifest
//! - `package.xi` (legacy) -- fallback for backward compatibility

use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::GraphError;

// ---------------------------------------------------------------------------
// xiom.toml schema
// ---------------------------------------------------------------------------

/// A parsed `xiom.toml` project manifest.
#[derive(Debug, Clone)]
pub struct ProjectManifest {
    pub project: ProjectMeta,
    pub dependencies: Vec<DependencySpec>,
    pub compiler: CompilerConfig,
    /// The directory containing `xiom.toml` (or `package.xi`).
    pub manifest_dir: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectMeta {
    pub name: String,
    pub version: String,
    pub root: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub extra_source_roots: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DependencySpec {
    pub name: String,
    pub version: String,
    pub path: Option<String>,
    pub git: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CompilerConfig {
    pub target: Option<String>,
    pub release: bool,
    pub incremental: bool,
    pub max_depth: Option<u32>,
    pub timeout_secs: Option<u32>,
}

// TOML deserialization structures (serde)
#[derive(Debug, Deserialize)]
struct ManifestToml {
    project: Option<ProjectToml>,
    dependencies: Option<std::collections::BTreeMap<String, DependencyToml>>,
    compiler: Option<CompilerToml>,
}

#[derive(Debug, Deserialize)]
struct ProjectToml {
    name: Option<String>,
    version: Option<String>,
    root: Option<String>,
    description: Option<String>,
    authors: Option<Vec<String>>,
    #[serde(rename = "source-roots")]
    source_roots: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DependencyToml {
    /// `dependency = "1.0"`
    Short(String),
    /// `dependency = { version = "1.0", path = "../lib" }`
    Full {
        version: Option<String>,
        path: Option<String>,
        git: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct CompilerToml {
    target: Option<String>,
    release: Option<bool>,
    incremental: Option<bool>,
    #[serde(rename = "max-depth")]
    max_depth: Option<u32>,
    #[serde(rename = "timeout-secs")]
    timeout_secs: Option<u32>,
}

// ---------------------------------------------------------------------------
// Manifest discovery and parsing
// ---------------------------------------------------------------------------

/// Walk up from `entry_file` to find the nearest `xiom.toml` or `package.xi`.
/// Returns the parsed manifest and the directory containing it.
pub fn find_and_parse_manifest(entry_file: &Path) -> Result<ProjectManifest, GraphError> {
    let entry_abs = entry_file
        .canonicalize()
        .unwrap_or_else(|_| entry_file.to_path_buf());

    let start_dir = if entry_abs.is_dir() {
        entry_abs
    } else {
        entry_abs
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };

    // Walk up looking for xiom.toml or package.xi
    let mut current = Some(start_dir.as_path());
    let mut tried = Vec::new();

    while let Some(dir) = current {
        let toml_path = dir.join("xiom.toml");
        if toml_path.exists() {
            return parse_xiom_toml(&toml_path);
        }

        let pkg_path = dir.join("package.xi");
        if pkg_path.exists() {
            return parse_package_xi(&pkg_path);
        }

        tried.push(dir.to_path_buf());
        current = dir.parent();
    }

    Err(GraphError::NoProjectFound(start_dir))
}

/// Find the project root directory (containing `xiom.toml` or `package.xi`).
pub fn find_project_root(file: &Path) -> Option<PathBuf> {
    let start_dir = if file.is_dir() {
        file.to_path_buf()
    } else {
        file.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."))
    };

    let mut current = Some(start_dir.as_path());
    let mut depth = 0;

    while let Some(dir) = current {
        if dir.join("xiom.toml").exists() || dir.join("package.xi").exists() {
            return Some(dir.to_path_buf());
        }
        depth += 1;
        if depth > 8 {
            break;
        }
        current = dir.parent();
    }

    None
}

/// Parse a `xiom.toml` file.
fn parse_xiom_toml(path: &Path) -> Result<ProjectManifest, GraphError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        GraphError::ManifestParse(format!("cannot read {}: {}", path.display(), e))
    })?;

    let manifest: ManifestToml = toml::from_str(&content).map_err(|e| {
        GraphError::ManifestParse(format!("{}: {}", path.display(), e))
    })?;

    let manifest_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let project = match manifest.project {
        Some(p) => ProjectMeta {
            name: p.name.unwrap_or_else(|| "unnamed".to_string()),
            version: p.version.unwrap_or_else(|| "0.1.0".to_string()),
            root: p.root,
            description: p.description,
            authors: p.authors.unwrap_or_default(),
            extra_source_roots: p.source_roots.unwrap_or_default(),
        },
        None => ProjectMeta {
            name: "unnamed".to_string(),
            version: "0.1.0".to_string(),
            ..Default::default()
        },
    };

    let dependencies = manifest
        .dependencies
        .map(|deps| {
            deps.into_iter()
                .map(|(name, spec)| match spec {
                    DependencyToml::Short(version) => DependencySpec {
                        name,
                        version,
                        path: None,
                        git: None,
                    },
                    DependencyToml::Full { version, path, git } => DependencySpec {
                        name,
                        version: version.unwrap_or_else(|| "*".to_string()),
                        path,
                        git,
                    },
                })
                .collect()
        })
        .unwrap_or_default();

    let compiler = match manifest.compiler {
        Some(c) => CompilerConfig {
            target: c.target,
            release: c.release.unwrap_or(false),
            incremental: c.incremental.unwrap_or(true),
            max_depth: c.max_depth,
            timeout_secs: c.timeout_secs,
        },
        None => CompilerConfig::default(),
    };

    Ok(ProjectManifest {
        project,
        dependencies,
        compiler,
        manifest_dir,
    })
}

/// Parse a legacy `package.xi` file.
fn parse_package_xi(path: &Path) -> Result<ProjectManifest, GraphError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        GraphError::ManifestParse(format!("cannot read {}: {}", path.display(), e))
    })?;

    let manifest_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    // Extract name
    let name = extract_field(&content, "name").unwrap_or_else(|| "unnamed".to_string());
    // Extract version
    let version = extract_field(&content, "version").unwrap_or_else(|| "0.1.0".to_string());
    // Extract root (from modules: ["src/main"] -- take first segment as root)
    let root = extract_modules_root(&content);

    // Extract dependencies
    let dependencies = extract_deps_legacy(&content);

    Ok(ProjectManifest {
        project: ProjectMeta {
            name,
            version,
            root,
            ..Default::default()
        },
        dependencies,
        compiler: CompilerConfig::default(),
        manifest_dir,
    })
}

/// Extract a simple `field: "value"` from package.xi content.
fn extract_field(content: &str, field_name: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&format!("{}:", field_name)) {
            let value = rest.trim().trim_matches('"').trim().to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
        // Also handle `"field": "value"` format
        if let Some(rest) = trimmed.strip_prefix(&format!("\"{}\":", field_name)) {
            let value = rest.trim().trim_matches('"').trim().to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

/// Extract the root directory from a legacy `modules: ["src/main", ...]` list.
fn extract_modules_root(content: &str) -> Option<String> {
    // Find the modules array
    let modules_start = content.find("modules:")?;
    let after_modules = &content[modules_start..];
    let bracket_start = after_modules.find('[')?;
    let after_bracket = &after_modules[bracket_start..];
    let bracket_end = after_bracket.find(']')?;
    let modules_str = &after_bracket[1..bracket_end];

    // Extract first module path and take its first segment as root
    for part in modules_str.split(',') {
        let cleaned = part.trim().trim_matches('"').trim();
        if !cleaned.is_empty() {
            // "src/main" -> "src"
            if let Some(slash_pos) = cleaned.find('/') {
                return Some(cleaned[..slash_pos].to_string());
            }
            if let Some(dot_pos) = cleaned.find('.') {
                return Some(cleaned[..dot_pos].to_string());
            }
            return None; // just a single file, use cwd
        }
    }
    None
}

/// Extract dependencies from legacy `package.xi` format.
fn extract_deps_legacy(content: &str) -> Vec<DependencySpec> {
    let mut deps = Vec::new();

    // Look for `dependencies: { ... }` or `dependencies: [...]`
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("dependencies:") || trimmed.starts_with("\"dependencies\":") {
            // Try to find a TOML/JSON-like object
            let after = trimmed
                .splitn(2, ':')
                .nth(1)
                .unwrap_or("")
                .trim();
            if after.starts_with('{') || after.starts_with('[') {
                // Simple key-value extraction from within braces
                let inner = after.trim_matches(|c| c == '{' || c == '}' || c == '[' || c == ']');
                for kv in inner.split(',') {
                    let kv = kv.trim();
                    if let Some((name, version)) = kv.split_once(':') {
                        let name = name.trim().trim_matches('"').trim();
                        let version = version.trim().trim_matches('"').trim();
                        if !name.is_empty() {
                            deps.push(DependencySpec {
                                name: name.to_string(),
                                version: version.to_string(),
                                path: None,
                                git: None,
                            });
                        }
                    }
                }
            }
        }
    }

    deps
}

// ---------------------------------------------------------------------------
// Source root resolution
// ---------------------------------------------------------------------------

/// C-PULSE-02 (m212): source roots contributed by `[dependencies]`.
///
/// - `path = "..."` form: the directory relative to the manifest dir, plus
///   its `src/` subdirectory when present.
/// - Registry form (`dep = "0.2"` / `{ version = "0.2" }`): the installed
///   package under `<xiom_home>/packages/<name>-<version>/` (newest match
///   wins when several versions are installed). Both the package ROOT and
///   its `src/` are added: the packages layout keeps root modules outside
///   `src/` (e.g. `xiom-graphql/graphql.xi`), and the old behavior left such
///   dependencies off the catalog path entirely, so `use xiom.rate;`
///   type-checked against nothing unless the project hard-coded the
///   directory in `source-roots` (the Pulse workaround).
fn dependency_roots_under(
    manifest: &ProjectManifest,
    manifest_dir: &Path,
    xiom_home: &Path,
) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    for dep in &manifest.dependencies {
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(ref p) = dep.path {
            let base = manifest_dir.join(p);
            candidates.push(base.clone());
            candidates.push(base.join("src"));
        } else {
            let packages = xiom_home.join("packages");
            let prefix = format!("{}-", dep.name);
            let requested = if dep.version.trim().is_empty() || dep.version.trim() == "*" {
                None
            } else {
                Some(dep.version.trim())
            };
            let mut matches: Vec<PathBuf> = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&packages) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    let Some(ver) = dir_name.strip_prefix(&prefix) else { continue };
                    if let Some(req) = requested {
                        if !ver.starts_with(req) {
                            continue;
                        }
                    }
                    matches.push(path);
                }
            }
            // Deterministic "newest": lexicographic max of the version dirs.
            matches.sort();
            if let Some(latest) = matches.pop() {
                let named = latest.join(&dep.name);
                let base = if named.is_dir() { named } else { latest };
                candidates.push(base.clone());
                candidates.push(base.join("src"));
            }
        }
        for c in candidates {
            if c.is_dir() && !roots.contains(&c) {
                roots.push(c);
            }
        }
    }
    roots
}

/// Resolve source root directories from a manifest.
///
/// Priority:
/// 1. `[project].root` field (relative to manifest_dir)
/// 2. Legacy `modules:` first path segment
/// 3. `src/` subdirectory (if it exists)
/// 4. `[project].source-roots` extra entries
/// 5. `[dependencies]` -> local path or installed `$XIOM_HOME/packages`
///    package roots (C-PULSE-02 / m212)
/// 6. Manifest directory itself (fallback)
pub fn resolve_source_roots(manifest: &ProjectManifest, manifest_dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();

    // Primary root from `[project].root`
    if let Some(ref root) = manifest.project.root {
        let candidate = manifest_dir.join(root);
        if candidate.is_dir() {
            roots.push(candidate);
        }
    }

    // Extra source roots from `[project].source-roots`
    for extra in &manifest.project.extra_source_roots {
        let candidate = manifest_dir.join(extra);
        if candidate.is_dir() {
            if !roots.contains(&candidate) {
                roots.push(candidate);
            }
        }
    }

    // If no explicit root, try src/
    if roots.is_empty() {
        let src_dir = manifest_dir.join("src");
        if src_dir.is_dir() {
            roots.push(src_dir);
        }
    }

    // C-PULSE-02: dependency roots (path or installed registry package).
    for dep_root in dependency_roots_under(manifest, manifest_dir, &crate::paths::xiom_home()) {
        if !roots.contains(&dep_root) {
            roots.push(dep_root);
        }
    }

    // Fallback: manifest directory itself
    if roots.is_empty() {
        roots.push(manifest_dir.to_path_buf());
    }

    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_xiom_toml() {
        let toml_content = r#"
[project]
name = "test_app"
version = "1.0.0"

[dependencies]
xiom-vulkan = "0.5"
"#;
        let manifest: ManifestToml = toml::from_str(toml_content).unwrap();
        assert_eq!(manifest.project.as_ref().unwrap().name.as_deref(), Some("test_app"));
        assert!(manifest.dependencies.as_ref().unwrap().contains_key("xiom-vulkan"));
    }

    #[test]
    fn parse_toml_with_source_roots() {
        let toml_content = r#"
[project]
name = "multi_src"
root = "lib/"
source-roots = ["vendor/", "generated/"]

[compiler]
release = true
target = "native"
"#;
        let manifest: ManifestToml = toml::from_str(toml_content).unwrap();
        let proj = manifest.project.unwrap();
        assert_eq!(proj.root.as_deref(), Some("lib/"));
        assert_eq!(proj.source_roots.as_ref().unwrap().len(), 2);
        assert!(manifest.compiler.unwrap().release.unwrap());
    }

    #[test]
    fn extract_name_from_package_xi() {
        let content = r#"
name: "xiom.stdlib"
version: "0.48.9"
modules: ["core", "string", "math"]
"#;
        assert_eq!(extract_field(content, "name"), Some("xiom.stdlib".to_string()));
        assert_eq!(extract_field(content, "version"), Some("0.48.9".to_string()));
    }

    // C-PULSE-02 (m212): `[dependencies]` contribute catalog source roots.
    #[test]
    fn dependency_path_roots_are_added() {
        let dir = std::env::temp_dir().join(format!("xiom_m212_{}", std::process::id()));
        let lib = dir.join("libs/core");
        std::fs::create_dir_all(lib.join("src")).unwrap();
        let manifest = ProjectManifest {
            project: ProjectMeta { name: "app".into(), ..Default::default() },
            dependencies: vec![DependencySpec {
                name: "core".into(),
                version: "*".into(),
                path: Some("libs/core".into()),
                git: None,
            }],
            compiler: CompilerConfig::default(),
            manifest_dir: dir.clone(),
        };
        let roots = resolve_source_roots(&manifest, &dir);
        assert!(roots.contains(&lib), "package root missing: {roots:?}");
        assert!(roots.contains(&lib.join("src")), "src root missing: {roots:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn installed_dependency_roots_are_added() {
        let dir = std::env::temp_dir().join(format!("xiom_m212i_{}", std::process::id()));
        let home = dir.join("home");
        let pkg = home.join("packages").join("xiom-rate-0.2.0").join("xiom-rate");
        std::fs::create_dir_all(pkg.join("src")).unwrap();
        // An older installed version must lose to the newest version match.
        std::fs::create_dir_all(home.join("packages/xiom-rate-0.1.0/xiom-rate/src")).unwrap();
        let manifest = ProjectManifest {
            project: ProjectMeta { name: "app".into(), ..Default::default() },
            dependencies: vec![DependencySpec {
                name: "xiom-rate".into(),
                version: "0.2".into(),
                path: None,
                git: None,
            }],
            compiler: CompilerConfig::default(),
            manifest_dir: dir.clone(),
        };
        let roots = dependency_roots_under(&manifest, &dir, &home);
        assert!(roots.contains(&pkg), "installed root missing: {roots:?}");
        assert!(roots.contains(&pkg.join("src")), "installed src missing: {roots:?}");
        assert!(
            !roots.iter().any(|r| r.to_string_lossy().contains("0.1.0")),
            "older version must not win: {roots:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_dependency_short_form() {
        let toml_content = r#"
[project]
name = "test"

[dependencies]
dep_a = "1.0"
dep_b = { version = "2.0", path = "../lib" }
"#;
        let manifest: ManifestToml = toml::from_str(toml_content).unwrap();
        let deps = manifest.dependencies.unwrap();
        assert_eq!(deps.len(), 2);

        match &deps["dep_a"] {
            DependencyToml::Short(v) => assert_eq!(v, "1.0"),
            _ => panic!("expected short form"),
        }
    }
}
