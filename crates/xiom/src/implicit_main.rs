// XIOM — Implicit main wrapping for scripting mode (M10)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

/// Wrap top-level code in an implicit `fn main()` if no explicit main exists.
/// Declarations (type, enum, interface, module, const, use, fn) stay at top level.
/// Only executable statements go inside `fn main()`.
/// Also injects default stdlib imports and strips shebangs.
pub fn wrap_implicit_main(source: &str) -> String {
    // M10: Strip shebang line before any processing
    let source = if source.starts_with("#!") {
        if let Some(newline) = source.find('\n') {
            &source[newline + 1..]
        } else { source }
    } else { source };

    // Already has an explicit fn main — don't wrap
    if source.contains("fn main") {
        return add_default_imports(source);
    }

    let trimmed = source.trim();
    if trimmed.is_empty() {
        return source.to_string();
    }

    // Separate declarations from code
    let decl_keywords = ["type ", "enum ", "interface ", "module ", "const ", "use ", "fn ", "pub "];
    let mut declarations = Vec::new();
    let mut code_lines = Vec::new();

    for line in trimmed.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() { continue; }
        let is_decl = decl_keywords.iter().any(|kw| trimmed_line.starts_with(kw));
        if is_decl {
            declarations.push(trimmed_line.to_string());
        } else {
            code_lines.push(trimmed_line.to_string());
        }
    }

    let mut result = String::new();

    // Add default imports
    for import in default_imports() {
        if !declarations.iter().any(|d| d.contains(import)) {
            result.push_str(import);
            result.push('\n');
        }
    }

    // Add declarations at top level
    for decl in &declarations {
        result.push_str(decl);
        result.push('\n');
    }

    // Wrap code in main
    if !code_lines.is_empty() {
        result.push_str("fn main() {\n");
        for line in &code_lines {
            result.push_str(line);
            result.push('\n');
        }
        result.push_str("}\n");
    }

    result
}

/// Default imports for scripting mode.
fn default_imports() -> &'static [&'static str] {
    &["use xiom.io;"]
}

/// Add default stdlib imports for scripting convenience.
fn add_default_imports(source: &str) -> String {
    // Only add imports if they're not already present
    let mut result = String::new();
    let imports_needed: &[&str] = if source.contains("use xiom.io") {
        &[]
    } else {
        &["use xiom.io;"]
    };

    for import in imports_needed {
        if !source.contains(import) {
            result.push_str(import);
            result.push('\n');
        }
    }
    result.push_str(source);
    result
}

/// Default imports for scripting mode.
fn default_import_block() -> String {
    "use xiom.io;\n".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_wrap_when_main_exists() {
        let src = "fn main() -> Int { return 42; }";
        let result = wrap_implicit_main(src);
        assert!(result.contains("fn main()"), "should contain main");
    }

    #[test]
    fn test_wrap_simple_expression() {
        let src = "io.println(\"hello\");";
        let result = wrap_implicit_main(src);
        assert!(result.contains("fn main() {"));
        assert!(result.contains("io.println"));
        assert!(result.contains("use xiom.io;"), "should auto-import io");
    }

    #[test]
    fn test_wrap_multiple_statements() {
        let src = "var x = 1;\nio.println(x.to_str());";
        let result = wrap_implicit_main(src);
        assert!(result.contains("fn main() {"));
        assert!(result.contains("use xiom.io;"));
    }

    #[test]
    fn test_module_with_body() {
        let src = "module test\nio.println(\"in module\");";
        let result = wrap_implicit_main(src);
        assert!(result.contains("module test"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_module_only_declarations() {
        // Module with only declarations (no executable code) stays at top level
        let src = "module math\npub fn add(a: Int, b: Int) -> Int { return a + b; }";
        let result = wrap_implicit_main(src);
        assert!(result.contains("module math"));
        assert!(result.contains("pub fn add"));
    }

    #[test]
    fn test_already_has_main_after_module() {
        let src = "module test\nfn main() -> Int { return 0; }";
        let result = wrap_implicit_main(src);
        assert!(result.contains("fn main()"), "should still have main");
        assert!(result.contains("module test"));
    }
}
