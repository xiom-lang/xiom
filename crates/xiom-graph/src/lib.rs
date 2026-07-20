//! XIOM dependency graph, module system, and project manifest.
//!
//! Phase 7A — Foundation for the module system:
//! - `xiom.toml` project manifest parsing
//! - Transitive .xi file discovery from source roots
//! - Dependency graph construction from `use` declarations
//! - Topological sort for compilation order
//! - Cycle detection with actionable diagnostics
//!
//! This crate is the single source of truth for "what modules exist in the
//! project and in what order should they be compiled."

mod discover;
mod graph;
pub mod manifest;
mod sort;

pub use discover::{discover_modules, discover_sources, ModuleDiscovery};
pub use graph::{DependencyGraph, ModuleNode};
pub use manifest::{CompilerConfig, DependencySpec, ProjectManifest};
pub use sort::{topological_sort, CycleError};

use std::path::PathBuf;

/// Errors that can occur during module graph construction.
#[derive(Debug, Clone)]
pub enum GraphError {
    /// A `xiom.toml` was found but could not be parsed.
    ManifestParse(String),
    /// No `xiom.toml` or `package.xi` found; no source roots configured.
    NoProjectFound(PathBuf),
    /// A module declares `use mod;` but mod's file cannot be located.
    ModuleNotFound {
        importer: String,
        imported: String,
        searched: Vec<PathBuf>,
    },
    /// Circular dependency detected.
    CycleDetected(Vec<String>),
    /// I/O error during file discovery.
    IoError(String),
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::ManifestParse(msg) => write!(f, "failed to parse xiom.toml: {}", msg),
            GraphError::NoProjectFound(dir) => {
                write!(f, "no xiom.toml or package.xi found in '{}' or its ancestors", dir.display())
            }
            GraphError::ModuleNotFound { importer, imported, searched } => {
                write!(
                    f,
                    "module '{}' imports '{}' but its source file was not found (searched {} directories)",
                    importer,
                    imported,
                    searched.len()
                )
            }
            GraphError::CycleDetected(cycle) => {
                write!(f, "circular dependency detected: {}", cycle.join(" → "))
            }
            GraphError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for GraphError {}

/// Build a complete dependency graph for a project.
///
/// Walks up from `entry_file` to find `xiom.toml` (or `package.xi` as fallback),
/// discovers all `.xi` source files under the configured source roots,
/// parses their `module` headers and `use` declarations, and constructs
/// a directed acyclic graph of module dependencies.
///
/// # Errors
/// Returns `GraphError` if the manifest is missing/invalid, a module
/// cannot be found, or a circular dependency is detected.
pub fn build_project_graph(entry_file: &std::path::Path) -> Result<DependencyGraph, GraphError> {
    // 1. Find and parse the project manifest (xiom.toml or package.xi)
    let manifest = manifest::find_and_parse_manifest(entry_file)?;

    // 2. Resolve source roots relative to the manifest directory
    let manifest_dir = manifest.manifest_dir.clone();
    let source_roots = manifest::resolve_source_roots(&manifest, &manifest_dir);

    // 3. Discover all .xi files under source roots
    let discovery = discover::discover_modules(&source_roots)?;

    // 4. Build the dependency graph
    let mut graph = graph::DependencyGraph::new(manifest.project.name.clone(), source_roots);

    for module in &discovery.modules {
        graph.add_node(module.clone());
    }

    // 5. Resolve edges: for each `use` declaration, find which module node provides it
    graph.resolve_edges(&discovery)?;

    Ok(graph)
}

/// Quick helper: find the project root directory for a given source file.
///
/// Walks up the directory tree looking for `xiom.toml` or `package.xi`.
/// Returns `None` if no project marker is found within 8 parent levels.
pub fn find_project_root(file: &std::path::Path) -> Option<PathBuf> {
    manifest::find_project_root(file)
}
