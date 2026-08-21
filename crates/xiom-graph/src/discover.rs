//! Module discovery: walk source roots to find all `.xi` files and extract
//! their `module` headers and `use` declarations.
//!
//! Uses quick lexer-only parsing to extract module identity and dependencies
//! without invoking the full AST parser.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::GraphError;
use super::graph::ModuleNode;

/// The result of scanning source roots for .xi files.
#[derive(Debug, Clone)]
pub struct ModuleDiscovery {
    /// All discovered modules, keyed by their dotted module path (e.g. "xiom.math").
    pub modules: Vec<ModuleNode>,
    /// Fast lookup: dotted module path -> index into `modules`.
    pub index: HashMap<String, usize>,
    /// All source file paths discovered (for backward compat with CLI file lists).
    pub source_files: Vec<PathBuf>,
}

/// Discover all modules under the given source roots.
///
/// Walks each root directory recursively, finds all `.xi` files,
/// and quick-parses them to extract:
/// - The `module a.b.c;` header (identity)
/// - All `use x.y.z;` declarations (dependencies)
///
/// Files without a `module` header are assigned an identity based on their
/// relative path within the source root.
pub fn discover_modules(source_roots: &[PathBuf]) -> Result<ModuleDiscovery, GraphError> {
    let mut modules = Vec::new();
    let mut index = HashMap::new();
    let mut source_files = Vec::new();

    for root in source_roots {
        discover_in_dir(root, root, &mut modules, &mut index, &mut source_files)?;
    }

    Ok(ModuleDiscovery {
        modules,
        index,
        source_files,
    })
}

/// Convenience: discover source files only (no module metadata).
/// Used for backward compatibility with CLI file list compilation.
pub fn discover_sources(source_roots: &[PathBuf]) -> Result<Vec<PathBuf>, GraphError> {
    let discovery = discover_modules(source_roots)?;
    Ok(discovery.source_files)
}

/// Walk a directory recursively, discovering .xi files.
fn discover_in_dir(
    dir: &Path,
    source_root: &Path,
    modules: &mut Vec<ModuleNode>,
    index: &mut HashMap<String, usize>,
    source_files: &mut Vec<PathBuf>,
) -> Result<(), GraphError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            // Permission errors are non-fatal for discovery
            eprintln!("xiom-graph: warning: cannot read {}: {}", dir.display(), e);
            return Ok(());
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden dirs and build artifacts
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "target" || name == "build" {
                    continue;
                }
            }
            discover_in_dir(&path, source_root, modules, index, source_files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("xi") {
            // Skip package.xi (manifest files)
            if path.file_name().and_then(|n| n.to_str()) == Some("package.xi") {
                continue;
            }

            source_files.push(path.clone());

            // Quick-parse the file
            if let Some(node) = quick_parse_module(&path, source_root) {
                let key = node.module_path.clone();
                if !index.contains_key(&key) {
                    let idx = modules.len();
                    index.insert(key, idx);
                    modules.push(node);
                }
            }
        }
    }

    Ok(())
}

/// Quick-parse a .xi file to extract the `module` header and `use` dependencies.
///
/// Uses the `xiom-lexer` to tokenize and scans for keyword tokens.
/// Returns `None` if the file cannot be read (non-fatal for discovery).
fn quick_parse_module(file_path: &Path, source_root: &Path) -> Option<ModuleNode> {
    let source = std::fs::read_to_string(file_path).ok()?;
    let tokens = xiom_lexer::Lexer::new(&source).tokenize();

    let mut module_path = infer_module_path(file_path, source_root);
    let mut dependencies = Vec::new();
    let mut saw_module_kw = false;

    let mut i = 0;
    while i < tokens.len() {
        let kind = &tokens[i].kind;

        match kind {
            xiom_lexer::TokenKind::Module => {
                // Parse `module a.b.c;` -- collect dot-separated idents
                if !saw_module_kw {
                    let path_parts = parse_dotted_path(&tokens, i + 1);
                    if !path_parts.is_empty() {
                        module_path = path_parts.join(".");
                        saw_module_kw = true;
                    }
                }
            }
            xiom_lexer::TokenKind::Use => {
                // Parse `use a.b.c;`, `use a.b.*;`, `use a.b as c;`
                let path_parts = parse_dotted_path(&tokens, i + 1);
                if !path_parts.is_empty() {
                    dependencies.push(path_parts.join("."));
                }
            }
            _ => {}
        }

        i += 1;
    }

    Some(ModuleNode {
        module_path,
        file_path: file_path.to_path_buf(),
        dependencies,
        source_hash: None,
    })
}

/// Parse a dot-separated identifier path from tokens starting at position `start`.
fn parse_dotted_path(tokens: &[xiom_lexer::Token], start: usize) -> Vec<String> {
    let mut parts = Vec::new();
    let mut i = start;

    while i < tokens.len() {
        match &tokens[i].kind {
            xiom_lexer::TokenKind::Ident(_name) => {
                parts.push(tokens[i].lexeme.clone());
            }
            xiom_lexer::TokenKind::Dot => {
                // Continue after dot (expect another ident)
            }
            xiom_lexer::TokenKind::Star => {
                // `use a.b.*;` -- star terminates the path
                break;
            }
            xiom_lexer::TokenKind::Semicolon => {
                // End of declaration
                break;
            }
            _ => {
                // Unknown token terminates the path (e.g. `use a.b as c`, where `as` stops us)
                break;
            }
        }
        i += 1;
    }

    parts
}

/// Infer a module path from a file path relative to the source root.
///
/// Example: `src/xiom/math/trig.xi` with root `src/` -> `xiom.math.trig`
fn infer_module_path(file_path: &Path, source_root: &Path) -> String {
    let relative = file_path
        .strip_prefix(source_root)
        .unwrap_or(file_path);

    let stem_path = if relative.extension().is_some() {
        relative.with_extension("")
    } else {
        relative.to_path_buf()
    };

    stem_path
        .components()
        .filter_map(|c| {
            match c {
                std::path::Component::Normal(os_str) => os_str.to_str(),
                _ => None,
            }
        })
        .collect::<Vec<&str>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_module_path_from_src() {
        let root = Path::new("src");
        let file = Path::new("src/xiom/math/trig.xi");
        assert_eq!(infer_module_path(file, root), "xiom.math.trig");
    }

    #[test]
    fn infer_module_path_no_subdir() {
        let root = Path::new("src");
        let file = Path::new("src/main.xi");
        assert_eq!(infer_module_path(file, root), "main");
    }

    #[test]
    fn infer_module_deeply_nested() {
        let root = Path::new(".");
        let file = Path::new("./a/b/c/d/util.xi");
        assert_eq!(infer_module_path(file, root), "a.b.c.d.util");
    }
}
