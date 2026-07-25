// XIOM — Implicit main wrapping for scripting mode (M10)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

/// Wrap top-level code in an implicit `fn main()` if no explicit main exists.
/// Declarations (type, enum, interface, module, const, use, fn) stay at top level.
/// Only executable statements go inside `fn main()`.
/// Also injects default stdlib imports and strips shebangs.
///
/// M12.2: Uses brace-depth tracking to correctly handle multi-line declarations.
/// A line starting with a decl keyword at depth 0 begins a declaration block;
/// all subsequent lines (including nested braces) are part of the declaration
/// until depth returns to 0.
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

    // Separate declarations from code using brace-depth tracking.
    // Declaration keywords that start multi-line blocks at depth 0.
    let decl_keywords = ["type ", "enum ", "interface ", "module ", "const ", "use ", "fn ", "pub "];
    let mut declarations = Vec::new();
    let mut code_lines = Vec::new();
    let mut in_decl = false;
    let mut depth: i32 = 0;

    for line in trimmed.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() { continue; }

        if !in_decl {
            // Check if this line starts a new declaration at depth 0
            let is_decl = decl_keywords.iter().any(|kw| trimmed_line.starts_with(kw));
                if is_decl {
                    in_decl = true;
                    depth = count_brace_delta(trimmed_line);
                    // Single-line declaration (e.g. `const X: Int = 5;` or `use foo;`)
                    if depth <= 0 {
                        in_decl = false;
                        depth = 0;
                        // M17: if the line has code after a `use` or `const`
                        // declaration, split at the semicolon suffix.
                        // e.g. `use xiom.io; io.println("hi")` → decl + code.
                        // Only applies to `use`/`const` — functions/types
                        // may contain semicolons in their bodies.
                        let is_splittable = trimmed_line.starts_with("use ")
                            || trimmed_line.starts_with("const ");
                        if is_splittable {
                            if let Some(semi_pos) = trimmed_line.find(';') {
                                declarations.push(trimmed_line[..=semi_pos].to_string());
                                let rest = trimmed_line[semi_pos + 1..].trim();
                                if !rest.is_empty() {
                                    code_lines.push(rest.to_string());
                                }
                                continue; // already added to declarations
                            }
                        }
                        declarations.push(trimmed_line.to_string());
                    } else {
                        declarations.push(trimmed_line.to_string());
                    }
            } else {
                code_lines.push(trimmed_line.to_string());
            }
        } else {
            // Inside a declaration — track brace depth
            depth += count_brace_delta(trimmed_line);
            declarations.push(trimmed_line.to_string());
            if depth <= 0 {
                in_decl = false;
                depth = 0;
            }
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

/// Count the net change in brace depth from a single line.
/// `{` increments, `}` decrements. Semantically: returns
/// (number of `{`) - (number of `}`).
fn count_brace_delta(line: &str) -> i32 {
    let opens = line.chars().filter(|&c| c == '{').count() as i32;
    let closes = line.chars().filter(|&c| c == '}').count() as i32;
    opens - closes
}

/// Default imports for scripting mode.
fn default_imports() -> &'static [&'static str] {
    &["use xiom.io;", "use xiom.convert;"]
}

/// Add default stdlib imports for scripting convenience.
fn add_default_imports(source: &str) -> String {
    let mut result = String::new();
    let imports = default_imports();
    for import in imports {
        if !source.contains(import) {
            result.push_str(import);
            result.push('\n');
        }
    }
    result.push_str(source);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_simple_code() {
        let result = wrap_implicit_main("io.println(\"hello\");");
        assert!(result.contains("fn main()"), "should wrap code in fn main");
        assert!(result.contains("io.println"), "should keep the code");
    }

    #[test]
    fn test_wrap_with_type_decl() {
        let src = "type Point = {\n  x: Float64;\n  y: Float64;\n}\nio.println(\"hi\");";
        let result = wrap_implicit_main(src);
        assert!(result.contains("type Point"), "type decl should be at top level");
        assert!(result.contains("fn main()"), "code should be wrapped");
        // type decl must appear BEFORE fn main
        let type_pos = result.find("type Point").unwrap();
        let main_pos = result.find("fn main()").unwrap();
        assert!(type_pos < main_pos, "type decl must precede fn main");
    }

    #[test]
    fn test_wrap_with_enum_decl() {
        let src = "enum Color { Red, Green, Blue }\nio.println(\"color\");";
        let result = wrap_implicit_main(src);
        let enum_pos = result.find("enum Color").unwrap();
        let main_pos = result.find("fn main()").unwrap();
        assert!(enum_pos < main_pos, "enum decl must precede fn main");
    }

    #[test]
    fn test_no_wrap_when_main_exists() {
        let src = "fn main() { io.println(\"hi\"); }";
        let result = wrap_implicit_main(src);
        assert!(!result.contains("fn main() {\n    io.println"), "should not double-wrap");
    }

    #[test]
    fn test_wrap_with_interface_decl() {
        let src = "interface Drawable {\n  fn draw(self);\n}\nio.println(\"test\");";
        let result = wrap_implicit_main(src);
        let iface_pos = result.find("interface Drawable").unwrap();
        let main_pos = result.find("fn main()").unwrap();
        assert!(iface_pos < main_pos, "interface decl must precede fn main");
    }
}
