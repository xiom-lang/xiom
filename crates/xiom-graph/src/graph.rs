//! Dependency graph: nodes, edges, and resolution.
//!
//! Each node represents a single module (one .xi file). Edges represent
//! `use` dependencies: if module A has `use B.C;`, then A depends on B.C.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::discover::ModuleDiscovery;
use super::GraphError;

/// A single module node in the dependency graph.
///
/// Represents one `.xi` source file with its module identity and `use` dependencies.
#[derive(Debug, Clone)]
pub struct ModuleNode {
    /// Fully-qualified dotted module path (e.g. "xiom.math.trig").
    pub module_path: String,
    /// Absolute path to the `.xi` source file.
    pub file_path: PathBuf,
    /// Dotted module paths this file imports via `use`.
    /// These are the raw paths from `use` declarations — not yet resolved
    /// to specific graph nodes.
    pub dependencies: Vec<String>,
    /// Content hash for incremental compilation (SHA-256 of source).
    /// Populated on demand.
    pub source_hash: Option<String>,
}

/// The complete dependency graph for a project.
///
/// Nodes are modules; edges represent `use` dependencies.
/// The graph is directed and must be acyclic.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Project name from manifest.
    pub project_name: String,
    /// All module nodes, in discovery order.
    pub nodes: Vec<ModuleNode>,
    /// Source root directories.
    pub source_roots: Vec<PathBuf>,
    /// Adjacency list: `node_index → [dep_node_indices]`.
    pub edges: Vec<Vec<usize>>,
    /// Reverse adjacency: `node_index → [dependent_node_indices]`.
    pub reverse_edges: Vec<Vec<usize>>,
    /// Fast lookup: dotted module path → node index.
    pub path_to_idx: HashMap<String, usize>,
    /// Fast lookup: file path → node index.
    pub file_to_idx: HashMap<PathBuf, usize>,
}

impl DependencyGraph {
    /// Create an empty dependency graph.
    pub fn new(project_name: String, source_roots: Vec<PathBuf>) -> Self {
        DependencyGraph {
            project_name,
            nodes: Vec::new(),
            source_roots,
            edges: Vec::new(),
            reverse_edges: Vec::new(),
            path_to_idx: HashMap::new(),
            file_to_idx: HashMap::new(),
        }
    }

    /// Add a module node to the graph.
    /// Returns the node's index.
    pub fn add_node(&mut self, node: ModuleNode) -> usize {
        let idx = self.nodes.len();
        self.path_to_idx
            .insert(node.module_path.clone(), idx);
        self.file_to_idx
            .insert(node.file_path.clone(), idx);
        self.nodes.push(node);
        self.edges.push(Vec::new());
        self.reverse_edges.push(Vec::new());
        idx
    }

    /// Resolve `use` declarations to graph edges.
    ///
    /// For each node's `dependencies` list, tries to find which graph node
    /// corresponds to each imported module path. Uses prefix matching:
    /// `use xiom.math.trig` matches `xiom.math.trig` exactly, and also
    /// `xiom.math` if no exact match exists (for module-level imports).
    pub fn resolve_edges(
        &mut self,
        _discovery: &ModuleDiscovery,
    ) -> Result<(), GraphError> {
        for i in 0..self.nodes.len() {
            let deps = self.nodes[i].dependencies.clone();

            for dep_path in &deps {
                // Try exact match first
                if let Some(&dep_idx) = self.path_to_idx.get(dep_path.as_str()) {
                    self.add_edge(i, dep_idx);
                    continue;
                }

                // Try prefix match: `use xiom.math` matches any `xiom.math.*` module
                let prefix = format!("{}.", dep_path);
                let matching: Vec<usize> = self
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| n.module_path.starts_with(&prefix))
                    .map(|(j, _)| j)
                    .collect();

                if matching.len() == 1 {
                    self.add_edge(i, matching[0]);
                } else if !matching.is_empty() {
                    // Multiple match — prefer the most specific (longest path)
                    let best = matching
                        .into_iter()
                        .max_by_key(|&j| self.nodes[j].module_path.len())
                        .unwrap();
                    self.add_edge(i, best);
                } else if dep_path.contains('.') {
                    // Try loading from external catalog (stdlib, registry packages)
                    // For now, external packages from the registry/stdlib are resolved
                    // at check-time by the Checker. We only track internal deps.
                    // External deps are not added as edges — they'll be resolved by
                    // the Checker's catalog at compile time.
                }
                // If dep_path has no dots (single-segment), it's likely a local module
                // or a builtin. Don't error — the Checker will handle resolution.
            }
        }

        Ok(())
    }

    /// Add a directed edge: `from` depends on `to`.
    fn add_edge(&mut self, from: usize, to: usize) {
        if from != to && !self.edges[from].contains(&to) {
            self.edges[from].push(to);
            self.reverse_edges[to].push(from);
        }
    }

    /// Get the number of nodes in the graph.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get a node by index.
    pub fn get_node(&self, idx: usize) -> Option<&ModuleNode> {
        self.nodes.get(idx)
    }

    /// Get a node by its module path.
    pub fn get_node_by_path(&self, path: &str) -> Option<&ModuleNode> {
        self.path_to_idx.get(path).and_then(|&idx| self.nodes.get(idx))
    }

    /// Get all source file paths in dependency order (topological sort).
    /// Returns `Err` if a cycle is detected.
    pub fn compilation_order(&self) -> Result<Vec<PathBuf>, GraphError> {
        let order = super::sort::topological_sort(self)?;
        Ok(order
            .into_iter()
            .map(|idx| self.nodes[idx].file_path.clone())
            .collect())
    }

    /// Get the immediate dependencies of a node.
    pub fn dependencies_of(&self, idx: usize) -> &[usize] {
        if idx < self.edges.len() {
            &self.edges[idx]
        } else {
            &[]
        }
    }

    /// Get the immediate dependents of a node (who depends on this node).
    pub fn dependents_of(&self, idx: usize) -> &[usize] {
        if idx < self.reverse_edges.len() {
            &self.reverse_edges[idx]
        } else {
            &[]
        }
    }

    /// Get all transitive dependencies of a node (everything it needs).
    pub fn transitive_deps(&self, idx: usize) -> HashSet<usize> {
        let mut visited = HashSet::new();
        let mut stack = vec![idx];
        while let Some(current) = stack.pop() {
            for &dep in &self.edges[current] {
                if visited.insert(dep) {
                    stack.push(dep);
                }
            }
        }
        visited
    }
}
