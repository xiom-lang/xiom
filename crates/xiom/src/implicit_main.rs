// XIOM — Implicit main wrapping for scripting mode (M10)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

/// Wrap top-level code in an implicit `fn main()` if no explicit main exists.
/// Also injects default stdlib imports for scripting ergonomics.
pub fn wrap_implicit_main(source: &str) -> String {
    // Already has an explicit main — don't wrap, but add imports if needed
    if source.contains("fn main") {
        return add_default_imports(source);
    }

    let trimmed = source.trim();
    if trimmed.is_empty() {
        return source.to_string();
    }

    // Check if first line is a module declaration
    if let Some(rest) = trimmed.strip_prefix("module ") {
        if let Some(newline) = rest.find('\n') {
            let module_line_len = "module ".len() + rest[..newline].len();
            let module_line = &trimmed[..module_line_len];
            let body = &trimmed[module_line_len..];
            let body_trimmed = body.trim();
            if body_trimmed.is_empty() {
                return source.to_string();
            }
            return format!("{module_line}\nfn main() {{\n{body}\n}}\n");
        }
    }

    // Wrap entire source in main, with default imports
    let imports = default_import_block();
    format!("{imports}\nfn main() {{\n{trimmed}\n}}\n")
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
    fn test_module_only_no_wrap() {
        let src = "module math\npub fn add(a: Int, b: Int) -> Int { return a + b; }";
        // This has fn main in the source? No. But it has `fn add` which contains `fn`
        // The check is `contains("fn main")` — this should be wrapped.
        let result = wrap_implicit_main(src);
        // Since there's no explicit fn main, it should be wrapped
        assert!(result.contains("fn main()"), "module-only should still get implicit main");
    }

    #[test]
    fn test_already_has_main_after_module() {
        let src = "module test\nfn main() -> Int { return 0; }";
        let result = wrap_implicit_main(src);
        assert!(result.contains("fn main()"), "should still have main");
        assert!(result.contains("module test"));
    }
}
