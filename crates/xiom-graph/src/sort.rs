// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Topological sort for compilation order.
//!
//! Uses Kahn's algorithm (BFS-based) to produce a linear ordering of modules
//! such that every module appears after all its dependencies.
//! Detects and reports cycles with full path information.

use std::collections::VecDeque;

use super::DependencyGraph;
use super::GraphError;

/// Error details for a detected dependency cycle.
#[derive(Debug, Clone)]
pub struct CycleError {
    /// The module paths forming the cycle, in order.
    pub cycle: Vec<String>,
    /// All module paths involved in strongly connected components.
    pub involved: Vec<String>,
}

/// Perform topological sort on the dependency graph.
///
/// Returns module indices in compilation order (dependencies first, dependents last).
/// Uses Kahn's algorithm with remaining dependency counts.
///
/// # Errors
/// Returns `GraphError::CycleDetected` if the graph contains a cycle.
pub fn topological_sort(graph: &DependencyGraph) -> Result<Vec<usize>, GraphError> {
    let n = graph.nodes.len();

    // remaining_deps[i] = number of unmet dependencies for node i
    let mut remaining_deps: Vec<usize> = graph.edges.iter().map(|e| e.len()).collect();

    // Initialize queue with nodes that have no dependencies
    let mut queue = VecDeque::new();
    for i in 0..n {
        if remaining_deps[i] == 0 {
            queue.push_back(i);
        }
    }

    let mut order = Vec::with_capacity(n);
    let mut processed = 0;

    while let Some(node) = queue.pop_front() {
        order.push(node);
        processed += 1;

        // Node 'node' is now compiled. All nodes that depend on 'node'
        // have one fewer unmet dependency.
        for &dependent in &graph.reverse_edges[node] {
            if dependent < n {
                remaining_deps[dependent] = remaining_deps[dependent].saturating_sub(1);
                if remaining_deps[dependent] == 0 {
                    queue.push_back(dependent);
                }
            }
        }
    }

    if processed != n {
        // Cycle detected -- find the cycle for error reporting
        let remaining: Vec<usize> = (0..n).filter(|&i| remaining_deps[i] > 0).collect();
        let cycle = find_cycle(graph, &remaining);
        return Err(GraphError::CycleDetected(cycle));
    }

    Ok(order)
}

/// Find a cycle in the remaining (unresolved) nodes for error reporting.
fn find_cycle(graph: &DependencyGraph, remaining: &[usize]) -> Vec<String> {
    if remaining.is_empty() {
        return vec!["<unknown>".to_string()];
    }

    // DFS from the first remaining node to find a cycle
    let start = remaining[0];
    let mut visited = vec![false; graph.nodes.len()];
    let mut path = Vec::new();
    let mut path_set = vec![false; graph.nodes.len()];

    if dfs_cycle(graph, start, &mut visited, &mut path, &mut path_set) {
        return path
            .iter()
            .map(|&idx| graph.nodes[idx].module_path.clone())
            .collect();
    }

    // Fallback: report all remaining nodes
    remaining
        .iter()
        .map(|&idx| graph.nodes[idx].module_path.clone())
        .collect()
}

/// DFS to find a cycle. Returns `true` when a cycle is found.
fn dfs_cycle(
    graph: &DependencyGraph,
    node: usize,
    visited: &mut [bool],
    path: &mut Vec<usize>,
    path_set: &mut [bool],
) -> bool {
    if path_set[node] {
        // Found cycle -- trim path to only the cycle portion
        if let Some(pos) = path.iter().position(|&x| x == node) {
            path.drain(0..pos);
        }
        return true;
    }

    if visited[node] {
        return false;
    }

    visited[node] = true;
    path_set[node] = true;
    path.push(node);

    for &dep in &graph.edges[node] {
        if dep < graph.nodes.len() && dfs_cycle(graph, dep, visited, path, path_set) {
            return true;
        }
    }

    path.pop();
    path_set[node] = false;
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::ModuleNode;
    use std::path::PathBuf;

    fn make_graph(nodes: Vec<(&str, Vec<&str>)>) -> DependencyGraph {
        let mut graph = DependencyGraph::new("test".to_string(), vec![PathBuf::from(".")]);
        for (path, deps) in &nodes {
            graph.add_node(ModuleNode {
                module_path: path.to_string(),
                file_path: PathBuf::from(format!("{}.xi", path.replace('.', "/"))),
                dependencies: deps.iter().map(|d| d.to_string()).collect(),
                source_hash: None,
            });
        }
        graph.resolve_edges(&crate::discover::ModuleDiscovery {
            modules: Vec::new(),
            index: std::collections::HashMap::new(),
            source_files: Vec::new(),
        }).unwrap_or(());
        graph
    }

    #[test]
    fn linear_chain() {
        // main -> utils -> core
        let graph = make_graph(vec![
            ("main", vec!["utils"]),
            ("utils", vec!["core"]),
            ("core", vec![]),
        ]);

        let order = topological_sort(&graph).unwrap();
        // core should come first, then utils, then main
        let names: Vec<&str> = order.iter().map(|&i| graph.nodes[i].module_path.as_str()).collect();
        let core_pos = names.iter().position(|&n| n == "core").unwrap();
        let utils_pos = names.iter().position(|&n| n == "utils").unwrap();
        let main_pos = names.iter().position(|&n| n == "main").unwrap();
        assert!(core_pos < utils_pos);
        assert!(utils_pos < main_pos);
    }

    #[test]
    fn diamond_deps() {
        // app -> a, app -> b, a -> common, b -> common
        let graph = make_graph(vec![
            ("app", vec!["a", "b"]),
            ("a", vec!["common"]),
            ("b", vec!["common"]),
            ("common", vec![]),
        ]);

        let order = topological_sort(&graph).unwrap();
        let names: Vec<&str> = order.iter().map(|&i| graph.nodes[i].module_path.as_str()).collect();
        let common_pos = names.iter().position(|&n| n == "common").unwrap();
        let a_pos = names.iter().position(|&n| n == "a").unwrap();
        let b_pos = names.iter().position(|&n| n == "b").unwrap();
        let app_pos = names.iter().position(|&n| n == "app").unwrap();

        assert!(common_pos < a_pos);
        assert!(common_pos < b_pos);
        assert!(a_pos < app_pos);
        assert!(b_pos < app_pos);
    }

    #[test]
    fn no_deps() {
        let graph = make_graph(vec![
            ("a", vec![]),
            ("b", vec![]),
            ("c", vec![]),
        ]);

        let order = topological_sort(&graph).unwrap();
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn cycle_detection() {
        // a -> b -> c -> a
        let graph = make_graph(vec![
            ("a", vec!["b"]),
            ("b", vec!["c"]),
            ("c", vec!["a"]),
        ]);

        let result = topological_sort(&graph);
        assert!(result.is_err());
        if let Err(GraphError::CycleDetected(cycle)) = result {
            assert!(cycle.len() >= 3);
            assert!(cycle.contains(&"a".to_string()));
            assert!(cycle.contains(&"b".to_string()));
            assert!(cycle.contains(&"c".to_string()));
        }
    }

    #[test]
    fn self_loop_allowed() {
        // Self-referential use is weird but shouldn't crash
        let graph = make_graph(vec![
            ("a", vec!["a"]),
            ("b", vec![]),
        ]);

        // Self-loops are filtered out in add_edge
        let order = topological_sort(&graph).unwrap();
        assert_eq!(order.len(), 2);
    }
}
