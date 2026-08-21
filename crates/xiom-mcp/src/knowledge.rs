// XIOM MCP -- Knowledge Tools (Phase 5d.1 expansion)
// -----------------------------------------------------------------------
// Makes any MCP-capable agent an instant XIOM expert, regardless of LLM
// training data. Three tools:
//   xiom_stdlib_reference -- LIVE parsed from stdlib/*.xi (always in sync)
//   xiom_language_guide   -- deep language semantics by topic
//   xiom_workflow_guide   -- toolchain operations (build/test/debug/publish)

use xiom_ast::*;
use xiom_lexer::Lexer;
use xiom_parser::Parser;

// ============================================================================
// Live stdlib reference -- parses real source, never goes stale
// ============================================================================

/// List all stdlib modules, or describe one module's full public API.
pub fn stdlib_reference(module_filter: Option<&str>) -> Result<String, String> {
    let dirs = xiom::find_stdlib_dirs();
    if dirs.is_empty() {
        return Err("No stdlib directory found. Set XIOM_STDLIB or run from the XIOM repo/release.".into());
    }

    // Recursive scan of stdlib/xiom/**/*.xi (the 512-module layout uses
    // subdirectories: os/file.xi -> module xiom.os.file). Module names are
    // the dotted relative path ("memory.alloc"), and the LEGACY bare stem
    // ("alloc") resolves when unique -- keeps old tooling/agents working
    // against the frozen layout.
    let mut modules: Vec<(String, std::path::PathBuf)> = Vec::new(); // (dotted name, path)
    for dir in &dirs {
        let xiom_subdir = std::path::Path::new(dir).join("xiom");
        let scan_root = if xiom_subdir.is_dir() { xiom_subdir } else { std::path::PathBuf::from(dir) };
        let mut walk: Vec<std::path::PathBuf> = vec![scan_root.clone()];
        while let Some(d) = walk.pop() {
            if let Ok(entries) = std::fs::read_dir(&d) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        walk.push(p);
                    } else if p.extension().and_then(|e| e.to_str()) == Some("xi") {
                        let rel = p.strip_prefix(&scan_root).unwrap_or(&p);
                        let dotted = rel.with_extension("")
                            .components()
                            .filter_map(|c| c.as_os_str().to_str())
                            .collect::<Vec<_>>()
                            .join(".");
                        modules.push((dotted, p));
                    }
                }
            }
        }
        if !modules.is_empty() { break; } // first stdlib root wins (mirrors compiler)
    }
    modules.sort();

    if let Some(want) = module_filter {
        let want_norm = want.trim().trim_start_matches("xiom.").to_lowercase();
        // Exact dotted match first ("memory.alloc"), then bare-stem match
        // ("alloc") when unique.
        if let Some((_, f)) = modules.iter().find(|(n, _)| n == &want_norm) {
            return describe_module(f);
        }
        let stem_matches: Vec<&(String, std::path::PathBuf)> = modules.iter()
            .filter(|(n, _)| n.rsplit('.').next().map(|s| s == want_norm.as_str()).unwrap_or(false))
            .collect();
        if stem_matches.len() == 1 {
            return describe_module(&stem_matches[0].1);
        }
        let available: Vec<String> = modules.iter().map(|(n, _)| n.clone()).collect();
        return Err(format!("Module '{want}' not found. Available: {}", available.join(", ")));
    }

    // No filter: list all modules with their doc header + public item counts
    let mut out = String::from("# XIOM Standard Library Modules\n\nUse `xiom_stdlib_reference {module: \"<name>\"}` for full signatures.\n\n| Module | Description | Pub fns | Pub types |\n|--------|-------------|---------|-----------|\n");
    for (name, f) in &modules {
        let source = std::fs::read_to_string(f).unwrap_or_default();
        let desc = source.lines()
            .find(|l| l.starts_with("// XIOM") && l.contains("--"))
            .and_then(|l| l.split("--").nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let (fn_count, ty_count) = count_pub_items(&source);
        out.push_str(&format!("| {name} | {desc} | {fn_count} | {ty_count} |\n"));
    }
    Ok(out)
}

fn count_pub_items(source: &str) -> (usize, usize) {
    let tokens = Lexer::new(source).tokenize();
    match Parser::new(tokens).parse_program() {
        Ok(program) => {
            let mut fns = 0;
            let mut tys = 0;
            visit_items(&program.items, &mut |d| match d {
                TopDecl::Fn(f) if f.is_pub => fns += 1,
                TopDecl::Type(t) if t.is_pub => tys += 1,
                TopDecl::Enum(e) if e.is_pub => tys += 1,
                _ => {}
            });
            (fns, tys)
        }
        Err(_) => (0, 0),
    }
}

fn visit_items(items: &[TopDecl], f: &mut impl FnMut(&TopDecl)) {
    for item in items {
        f(item);
        if let TopDecl::Module(m) = item {
            visit_items(&m.items, f);
        }
    }
}

fn describe_module(path: &std::path::Path) -> Result<String, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
    // BUG 29 (new 512-module layout): derive the dotted module name from the
    // path ("stdlib/xiom/os/file.xi" -> "xiom.os.file"), falling back to the
    // bare stem for flat layouts.
    let dotted = {
        // Walk up from the file until the "xiom" root segment.
        let mut cur = path.parent();
        let mut parts: Vec<String> = Vec::new();
        while let Some(d) = cur {
            let name = d.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
            if name == "xiom" || name.is_empty() { break; }
            parts.insert(0, name);
            cur = d.parent();
        }
        parts.push(stem.to_string());
        parts.join(".")
    };
    let tokens = Lexer::new(&source).tokenize();
    let program = Parser::new(tokens).parse_program()
        .map_err(|e| format!("Parse error in {}: {}", path.display(), e.message))?;

    let mut out = format!("# Module xiom.{dotted}\n\nImport with: `use xiom.{dotted};`\n\n");

    let mut types_section = String::new();
    let mut fns_section = String::new();
    visit_items(&program.items, &mut |d| match d {
        TopDecl::Type(t) if t.is_pub => {
            types_section.push_str(&format!("### type {}\n```xiom\n{}\n```\n", t.name.name, render_type_decl(t)));
        }
        TopDecl::Enum(e) if e.is_pub => {
            let variants: Vec<String> = e.variants.iter().map(|v| v.name.name.clone()).collect();
            types_section.push_str(&format!("### enum {}\nVariants: {}\n\n", e.name.name, variants.join(", ")));
        }
        TopDecl::Interface(i) if i.is_pub => {
            types_section.push_str(&format!("### interface {}\n\n", i.name.name));
        }
        TopDecl::Fn(f) if f.is_pub => {
            fns_section.push_str(&format!("```xiom\n{}\n```\n", render_fn_signature(f)));
        }
        _ => {}
    });

    if !types_section.is_empty() {
        out.push_str("## Types\n\n");
        out.push_str(&types_section);
    }
    if !fns_section.is_empty() {
        out.push_str("## Functions\n\n");
        out.push_str(&fns_section);
    }
    Ok(out)
}

fn render_type_decl(t: &TypeDecl) -> String {
    let fields: Vec<String> = t.fields.iter()
        .map(|f| format!("{}: {}", f.name.name, type_to_str(&f.ty)))
        .collect();
    format!("pub type {} = {{ {} }}", t.name.name, fields.join("; "))
}

/// Render a function signature exactly as a user would write it,
/// including receiver, generics, params, return type, and contracts.
pub fn render_fn_signature(f: &FnDecl) -> String {
    let mut sig = String::new();
    if f.is_pub { sig.push_str("pub "); }
    if f.is_async { sig.push_str("async "); }
    sig.push_str("fn ");
    if let Some(recv) = &f.receiver {
        sig.push_str(&recv.name);
        sig.push('.');
    }
    sig.push_str(&f.name.name);
    if !f.generics.is_empty() {
        let gens: Vec<String> = f.generics.iter().map(|g| {
            if g.bounds.is_empty() { g.name.name.clone() }
            else { format!("{}: {}", g.name.name, g.bounds.iter().map(|b| b.name.clone()).collect::<Vec<_>>().join("+")) }
        }).collect();
        sig.push_str(&format!("[{}]", gens.join(", ")));
    }
    let params: Vec<String> = f.params.iter().map(|p| {
        if p.name.name == "self" {
            if p.is_mut_self { "&mut self".to_string() } else { "self".to_string() }
        } else {
            format!("{}: {}", p.name.name, type_to_str(&p.ty))
        }
    }).collect();
    sig.push_str(&format!("({})", params.join(", ")));
    if let Some(ret) = &f.return_type {
        sig.push_str(&format!(" -> {}", type_to_str(ret)));
    }
    for c in &f.contracts {
        match c {
            ContractClause::Requires(e, _) => sig.push_str(&format!("\n  requires: {}", expr_to_str(e))),
            ContractClause::Ensures(e, _) => sig.push_str(&format!("\n  ensures:  {}", expr_to_str(e))),
        }
    }
    sig
}

/// Compact type renderer (source syntax).
pub fn type_to_str(t: &Type) -> String {
    match t {
        Type::Named(id, args) => {
            if args.is_empty() { id.name.clone() }
            else { format!("{}[{}]", id.name, args.iter().map(type_to_str).collect::<Vec<_>>().join(", ")) }
        }
        Type::Ref(inner) => format!("&{}", type_to_str(inner)),
        Type::MutRef(inner) => format!("&mut {}", type_to_str(inner)),
        Type::Option(inner) => format!("Option[{}]", type_to_str(inner)),
        Type::Result(a, b) => format!("Result[{}, {}]", type_to_str(a), type_to_str(b)),
        Type::Vec(inner) => format!("Vec[{}]", type_to_str(inner)),
        Type::Slice(inner) => format!("Slice[{}]", type_to_str(inner)),
        Type::Map(k, v) => format!("Map[{}, {}]", type_to_str(k), type_to_str(v)),
        Type::Set(inner) => format!("Set[{}]", type_to_str(inner)),
        Type::Tuple(items) => format!("({})", items.iter().map(type_to_str).collect::<Vec<_>>().join(", ")),
        Type::Ptr(inner) => format!("*{}", type_to_str(inner)),
        Type::Array(_, inner) => format!("[N]{}", type_to_str(inner)),
        Type::Fn(params, ret) => format!("fn({}) -> {}", params.iter().map(type_to_str).collect::<Vec<_>>().join(", "), type_to_str(ret)),
        Type::ImplTrait(traits) => format!("impl {}", traits.iter().map(|t| t.name.clone()).collect::<Vec<_>>().join(" + ")),
        Type::AnonStruct(_fields) => "(anonymous struct)".to_string(),
        Type::Never => "!".to_string(),
    }
}

/// Compact expression renderer for contract clauses.
pub fn expr_to_str(e: &Expr) -> String {
    match e {
        Expr::Ident(id) => id.name.clone(),
        Expr::Int(v, _) => v.to_string(),
        Expr::Float(v, _) => v.to_string(),
        Expr::Str(s, _) => format!("\"{s}\""),
        Expr::Char(c, _) => format!("'{c}'"),
        Expr::Bool(b, _) => b.to_string(),
        Expr::None(_) => "null".into(),
        Expr::Paren(inner, _) => format!("({})", expr_to_str(inner)),
        Expr::Unary(op, inner, _) => {
            let op_str = match op {
                UnaryOp::Neg => "-", UnaryOp::Not => "!", UnaryOp::Ref => "&",
                UnaryOp::MutRef => "&mut ", UnaryOp::BitNot => "~", UnaryOp::Deref => "*",
            };
            format!("{op_str}{}", expr_to_str(inner))
        }
        Expr::Binary(l, op, r, _) => format!("{} {} {}", expr_to_str(l), op, expr_to_str(r)),
        Expr::Field(obj, field, _) => format!("{}.{}", expr_to_str(obj), field.name),
        Expr::Call(f, args, _) | Expr::GenericCall(f, _, args, _) => format!("{}({})", expr_to_str(f), args.iter().map(expr_to_str).collect::<Vec<_>>().join(", ")),
        Expr::Index(obj, idx, _) => format!("{}[{}]", expr_to_str(obj), expr_to_str(idx)),
        Expr::Is(inner, pat, _) => format!("{} is {}", expr_to_str(inner), pattern_to_str(pat)),
        Expr::Imply(l, r, _) => format!("{} => {}", expr_to_str(l), expr_to_str(r)),
        Expr::AtPre(inner, _) => format!("{}@pre", expr_to_str(inner)),
        _ => "<expr>".into(),
    }
}

fn pattern_to_str(p: &Pattern) -> String {
    match p {
        Pattern::Wildcard(_) => "_".into(),
        Pattern::Ident(id) => id.name.clone(),
        Pattern::Variant(name, fields, _) => {
            if fields.is_empty() { name.name.clone() }
            else { format!("{}({})", name.name, fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(", ")) }
        }
        Pattern::Lit(Literal::Int(v, _)) => v.to_string(),
        Pattern::Lit(Literal::Float(v, _)) => v.to_string(),
        Pattern::Lit(Literal::Str(s, _)) => format!("\"{s}\""),
        Pattern::Lit(Literal::Char(c, _)) => format!("'{c}'"),
        Pattern::Lit(Literal::Bool(b, _)) => b.to_string(),
        Pattern::Some(inner, _) => format!("Some({})", pattern_to_str(inner)),
        Pattern::None(_) => "None".into(),
        Pattern::Ok(inner, _) => format!("Ok({})", pattern_to_str(inner)),
        Pattern::Err(inner, _) => format!("Err({})", pattern_to_str(inner)),
        Pattern::Or(alts, _) => alts.iter().map(pattern_to_str).collect::<Vec<_>>().join(" | "),
        Pattern::Struct(name, fields, _) => {
            if fields.is_empty() { name.name.clone() }
            else { format!("{} {{{}}}", name.name, fields.iter().map(|(f, p)| format!("{}: {}", f.name, pattern_to_str(p))).collect::<Vec<_>>().join(", ")) }
        }
        Pattern::Tuple(patterns, _) => {
            format!("({})", patterns.iter().map(pattern_to_str).collect::<Vec<_>>().join(", "))
        }
    }
}
