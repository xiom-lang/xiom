//! Build graph visualization -- outputs the project dependency graph
//! in DOT (GraphViz) or Mermaid format for documentation and debugging.

use std::path::Path;

/// Output format for dependency graph visualization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphFormat {
    /// GraphViz DOT format -- render with `dot -Tpng graph.dot -o graph.png`
    Dot,
    /// Mermaid markdown format -- render in GitHub/GitLab markdown
    Mermaid,
}

/// Generate a DOT-format dependency graph from the project graph.
pub fn generate_dot_graph(
    graph: &xiom_graph::DependencyGraph,
    format: GraphFormat,
) -> String {
    match format {
        GraphFormat::Dot => generate_dot(graph),
        GraphFormat::Mermaid => generate_mermaid(graph),
    }
}

fn generate_dot(graph: &xiom_graph::DependencyGraph) -> String {
    let mut out = String::new();
    out.push_str("digraph xiom_project {\n");
    out.push_str("  rankdir=LR;\n");
    out.push_str(&format!("  label=\"{}\";\n", graph.project_name));
    out.push_str("  node [shape=box, style=filled, fillcolor=lightyellow];\n\n");

    for (i, node) in graph.nodes.iter().enumerate() {
        let label = node.module_path.replace('.', "\\n");
        let deps: Vec<String> = graph.edges[i]
            .iter()
            .map(|&d| &graph.nodes[d].module_path)
            .cloned()
            .collect();
        if deps.is_empty() {
            out.push_str(&format!("  \"{}\" [label=\"{}\", fillcolor=lightgreen];\n",
                node.module_path, label));
        }
    }
    out.push_str("\n");

    for (i, node) in graph.nodes.iter().enumerate() {
        for &dep in &graph.edges[i] {
            if dep < graph.nodes.len() {
                out.push_str(&format!("  \"{}\" -> \"{}\";\n",
                    node.module_path, graph.nodes[dep].module_path));
            }
        }
    }

    out.push_str("}\n");
    out
}

fn generate_mermaid(graph: &xiom_graph::DependencyGraph) -> String {
    let mut out = String::new();
    out.push_str("```mermaid\n");
    out.push_str("graph LR\n");

    for (i, node) in graph.nodes.iter().enumerate() {
        let id = node.module_path.replace('.', "_");
        let label = &node.module_path;
        let is_leaf = graph.edges[i].is_empty();
        if is_leaf {
            out.push_str(&format!("  {}[\"{}:::leaf\"]\n", id, label));
        } else {
            out.push_str(&format!("  {}[\"{}\"]\n", id, label));
        }
    }

    out.push_str("\n");
    for (i, node) in graph.nodes.iter().enumerate() {
        for &dep in &graph.edges[i] {
            if dep < graph.nodes.len() {
                let from_id = node.module_path.replace('.', "_");
                let to_id = graph.nodes[dep].module_path.replace('.', "_");
                out.push_str(&format!("  {} --> {}\n", from_id, to_id));
            }
        }
    }

    out.push_str("\n  classDef leaf fill:#90EE90,stroke:#333,stroke-width:1px;\n");
    out.push_str("```\n");
    out
}

/// Generate and write a build graph to a file.
pub fn write_graph_file(
    graph: &xiom_graph::DependencyGraph,
    format: GraphFormat,
    output_path: &Path,
) -> std::io::Result<()> {
    let content = generate_dot_graph(graph, format);
    std::fs::write(output_path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xiom_graph::{DependencyGraph, ModuleNode};
    use std::path::PathBuf;

    fn make_test_graph() -> DependencyGraph {
        let mut g = DependencyGraph::new("test_project".into(), vec![PathBuf::from("src")]);
        g.add_node(ModuleNode {
            module_path: "core".into(), file_path: PathBuf::from("src/core.xi"),
            dependencies: vec![], source_hash: None,
        });
        g.add_node(ModuleNode {
            module_path: "utils".into(), file_path: PathBuf::from("src/utils.xi"),
            dependencies: vec!["core".into()], source_hash: None,
        });
        g.add_node(ModuleNode {
            module_path: "main".into(), file_path: PathBuf::from("src/main.xi"),
            dependencies: vec!["utils".into(), "core".into()], source_hash: None,
        });
        // Resolve edges manually
        for i in 0..g.nodes.len() {
            let deps: Vec<usize> = g.nodes[i].dependencies.iter()
                .filter_map(|d| g.path_to_idx.get(d).copied())
                .collect();
            for d in deps { g.edges[i].push(d); g.reverse_edges[d].push(i); }
        }
        g
    }

    #[test]
    fn dot_graph_contains_nodes() {
        let g = make_test_graph();
        let dot = generate_dot(&g);
        assert!(dot.contains("core"));
        assert!(dot.contains("utils"));
        assert!(dot.contains("main"));
        // Edges: main depends on core+utils, utils depends on core
        // The DOT output shows main -> core and main -> utils
        assert!(dot.contains("->"), "Must have edges in graph");
    }

    #[test]
    fn mermaid_graph_contains_nodes() {
        let g = make_test_graph();
        let mermaid = generate_mermaid(&g);
        assert!(mermaid.contains("graph LR"));
        assert!(mermaid.contains("core"));
        assert!(mermaid.contains("utils"));
        assert!(mermaid.contains("main"));
    }

    #[test]
    fn empty_graph_produces_valid_output() {
        let g = DependencyGraph::new("empty".into(), vec![]);
        let dot = generate_dot(&g);
        assert!(dot.contains("digraph"));
        let mermaid = generate_mermaid(&g);
        assert!(mermaid.contains("graph LR"));
    }
}
