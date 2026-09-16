// XIOM -- Canonical Formatter
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// M14.1: format_stmt -> stmt.rs, format_expr -> expr.rs

pub mod stmt;
pub mod expr;

use xiom_ast::*;

pub struct Formatter {
    buf: String,
    indent: usize,
}

impl Formatter {
    pub fn new() -> Self {
        Self { buf: String::new(), indent: 0 }
    }

    pub fn format(&mut self, program: &Program) -> String {
        self.format_program(program);
        std::mem::take(&mut self.buf)
    }

    fn push_indent(&mut self) {
        for _ in 0..self.indent {
            self.buf.push(' ');
            self.buf.push(' ');
        }
    }

    fn format_program(&mut self, program: &Program) {
        for (i, item) in program.items.iter().enumerate() {
            if i > 0 { self.buf.push('\n'); }
            self.format_top_decl(item);
        }
        self.buf.push('\n');
    }

    fn format_top_decl(&mut self, decl: &TopDecl) {
        match decl {
            TopDecl::Fn(fn_decl) => self.format_fn_decl(fn_decl),
            TopDecl::Type(type_decl) => self.format_type_decl(type_decl),
            TopDecl::Enum(enum_decl) => self.format_enum_decl(enum_decl),
            TopDecl::Interface(iface_decl) => self.format_interface_decl(iface_decl),
            TopDecl::Module(module) => self.format_module(module),
            TopDecl::Use(use_decl) => self.format_use_decl(use_decl),
            TopDecl::Const(const_decl) => self.format_const_decl(const_decl),
            TopDecl::Extern(extern_block) => self.format_extern(extern_block),
            TopDecl::Impl(_) => {} // expanded at registration time
            TopDecl::Spawn(block, _, is_move) => {
                if *is_move {
                    self.buf.push_str("spawn move {\n");
                } else {
                    self.buf.push_str("spawn {\n");
                }
                self.indent += 1;
                self.format_block(block);
                self.indent -= 1;
                self.push_indent();
                self.buf.push('}');
            }
        }
    }

    fn format_fn_decl(&mut self, f: &FnDecl) {
        if f.is_pub { self.buf.push_str("pub "); }
        if f.is_async { self.buf.push_str("async "); }
        self.buf.push_str("fn ");
        if let Some(receiver) = &f.receiver {
            self.buf.push_str(&receiver.name);
            self.buf.push('.');
        }
        self.buf.push_str(&f.name.name);
        if !f.generics.is_empty() {
            self.buf.push('[');
            for (i, g) in f.generics.iter().enumerate() {
                if i > 0 { self.buf.push_str(", "); }
                self.buf.push_str(&g.name.name);
                if !g.bounds.is_empty() {
                    self.buf.push_str(": ");
                    for (j, b) in g.bounds.iter().enumerate() {
                        if j > 0 { self.buf.push_str(" + "); }
                        self.buf.push_str(&b.name);
                    }
                }
            }
            self.buf.push(']');
        }
        self.buf.push('(');
        for (i, p) in f.params.iter().enumerate() {
            if i > 0 { self.buf.push_str(", "); }
            self.buf.push_str(&p.name.name);
            self.buf.push_str(": ");
            self.format_type(&p.ty);
        }
        self.buf.push(')');
        if let Some(ret) = &f.return_type {
            self.buf.push_str(" -> ");
            self.format_type(ret);
        }
        if !f.contracts.is_empty() {
            for c in &f.contracts {
                self.buf.push('\n');
                self.push_indent();
                match c {
                    ContractClause::Requires(expr, _) => {
                        self.buf.push_str("  requires: ");
                        self.format_expr(expr);
                    }
                    ContractClause::Ensures(expr, _) => {
                        self.buf.push_str("  ensures: ");
                        self.format_expr(expr);
                    }
                }
            }
        }
        if let Some(body) = &f.body {
            self.buf.push_str(" {\n");
            self.indent += 1;
            self.format_block(body);
            self.indent -= 1;
            self.push_indent();
            self.buf.push('}');
        }
    }

    fn format_extern(&mut self, eb: &ExternBlock) {
        self.buf.push_str("extern \"");
        self.buf.push_str(&eb.linkage);
        self.buf.push_str("\" {\n");
        self.indent += 1;
        for func in &eb.functions {
            self.push_indent();
            self.format_fn_decl(func);
            self.buf.push_str(";\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.buf.push('}');
    }

    fn format_block(&mut self, block: &Block) {
        for item in &block.stmts {
            match item {
                StmtOrExpr::Stmt(stmt) => self.format_stmt(stmt),
                StmtOrExpr::Expr(expr) => {
                    self.push_indent();
                    self.format_expr(expr);
                    self.buf.push_str(";\n");
                }
            }
        }
    }

    fn format_pattern(&mut self, pat: &Pattern) {
        match pat {
            Pattern::Wildcard(_) => self.buf.push('_'),
            Pattern::Ident(ident) => self.buf.push_str(&ident.name),
            Pattern::Variant(name, fields, _) => {
                self.buf.push_str(&name.name);
                self.buf.push('(');
                for (i, f) in fields.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.buf.push_str(&f.name);
                }
                self.buf.push(')');
            }
            Pattern::Lit(lit) => match lit {
                Literal::Int(n, _) => self.buf.push_str(&n.to_string()),
                Literal::Float(f, _) => self.buf.push_str(&format_float(*f)),
                Literal::Str(s, _) => { self.buf.push('"'); self.buf.push_str(s); self.buf.push('"'); }
                Literal::Char(c, _) => { self.buf.push('\''); self.buf.push(*c); self.buf.push('\''); }
                Literal::Bool(b, _) => self.buf.push_str(if *b { "true" } else { "false" }),
            },
            Pattern::Some(inner, _) => {
                self.buf.push_str("Some(");
                self.format_pattern(inner);
                self.buf.push(')');
            }
            Pattern::None(_) => self.buf.push_str("None"),
            Pattern::Ok(inner, _) => {
                self.buf.push_str("Ok(");
                self.format_pattern(inner);
                self.buf.push(')');
            }
            Pattern::Err(inner, _) => {
                self.buf.push_str("Err(");
                self.format_pattern(inner);
                self.buf.push(')');
            }
            Pattern::Or(alts, _) => {
                for (i, alt) in alts.iter().enumerate() {
                    if i > 0 { self.buf.push_str(" | "); }
                    self.format_pattern(alt);
                }
            }
            Pattern::Struct(name, fields, _) => {
                self.buf.push_str(&name.name);
                self.buf.push_str(" { ");
                for (i, (field, sub_pat)) in fields.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.buf.push_str(&field.name);
                    self.buf.push_str(": ");
                    self.format_pattern(sub_pat);
                }
                self.buf.push_str(" }");
            }
            Pattern::Tuple(patterns, _) => {
                self.buf.push('(');
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.format_pattern(p);
                }
                self.buf.push(')');
            }
        }
    }

    fn format_type(&mut self, ty: &Type) {
        match ty {
            Type::Named(ident, args) => {
                self.buf.push_str(&ident.name);
                if !args.is_empty() {
                    self.buf.push('[');
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { self.buf.push_str(", "); }
                        self.format_type(arg);
                    }
                    self.buf.push(']');
                }
            }
            Type::Ref(inner) => {
                self.buf.push('&');
                self.format_type(inner);
            }
            Type::MutRef(inner) => {
                self.buf.push_str("&mut ");
                self.format_type(inner);
            }
            Type::Option(inner) => {
                self.buf.push_str("Option[");
                self.format_type(inner);
                self.buf.push(']');
            }
            Type::Result(ok, err) => {
                self.buf.push_str("Result[");
                self.format_type(ok);
                self.buf.push_str(", ");
                self.format_type(err);
                self.buf.push(']');
            }
            Type::Vec(inner) => {
                self.buf.push_str("Vec[");
                self.format_type(inner);
                self.buf.push(']');
            }
            Type::Slice(inner) => {
                self.buf.push_str("Slice[");
                self.format_type(inner);
                self.buf.push(']');
            }
            Type::Map(k, v) => {
                self.buf.push_str("Map[");
                self.format_type(k);
                self.buf.push_str(", ");
                self.format_type(v);
                self.buf.push(']');
            }
            Type::Set(inner) => {
                self.buf.push_str("Set[");
                self.format_type(inner);
                self.buf.push(']');
            }
            Type::Tuple(types) => {
                self.buf.push('(');
                for (i, t) in types.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.format_type(t);
                }
                self.buf.push(')');
            }
            Type::Ptr(inner) => {
                self.buf.push('*');
                self.format_type(inner);
            }
            Type::Array(size, inner) => {
                self.buf.push('[');
                self.format_expr(size);
                self.buf.push(']');
                self.format_type(inner);
            }
            Type::Fn(params, ret) => {
                self.buf.push_str("fn(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.format_type(p);
                }
                self.buf.push_str(") -> ");
                self.format_type(ret);
            }
            Type::ImplTrait(traits) => {
                self.buf.push_str("impl ");
                for (i, t) in traits.iter().enumerate() {
                    if i > 0 { self.buf.push_str(" + "); }
                    self.buf.push_str(&t.name);
                }
            }
            Type::AnonStruct(fields) => {
                self.buf.push('{');
                for (i, f) in fields.iter().enumerate() {
                    if i > 0 { self.buf.push(' '); }
                    self.buf.push_str(&f.name.name);
                    self.buf.push_str(": ");
                    self.format_type(&f.ty);
                    self.buf.push(';');
                }
                self.buf.push_str(" }");
            }
            Type::Never => {
                self.buf.push('!');
            }
        }
    }

    fn format_type_decl(&mut self, td: &TypeDecl) {
        if td.is_pub { self.buf.push_str("pub "); }
        self.buf.push_str("type ");
        self.buf.push_str(&td.name.name);
        if !td.generics.is_empty() {
            self.buf.push('[');
            for (i, g) in td.generics.iter().enumerate() {
                if i > 0 { self.buf.push_str(", "); }
                self.buf.push_str(&g.name.name);
                if !g.bounds.is_empty() {
                    self.buf.push_str(": ");
                    for (j, b) in g.bounds.iter().enumerate() {
                        if j > 0 { self.buf.push_str(" + "); }
                        self.buf.push_str(&b.name);
                    }
                }
            }
            self.buf.push(']');
        }
        if let Some(alias) = &td.alias {
            self.buf.push_str(" = ");
            self.format_type(alias);
            self.buf.push_str(";\n");
            return;
        }
        self.buf.push_str(" = {\n");
        self.indent += 1;
        for field in &td.fields {
            self.push_indent();
            self.buf.push_str(&field.name.name);
            self.buf.push_str(": ");
            self.format_type(&field.ty);
            self.buf.push_str(";\n");
        }
        for (name, ty, expr) in &td.derived_fields {
            self.push_indent();
            self.buf.push_str(&name.name);
            self.buf.push_str(": ");
            self.format_type(ty);
            self.buf.push_str(" derived(");
            self.format_expr(expr);
            self.buf.push_str(");\n");
        }
        for inv in &td.invariants {
            self.push_indent();
            self.buf.push_str("invariant: ");
            self.format_expr(inv);
            self.buf.push_str(";\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.buf.push('}');
        if !td.derives.is_empty() {
            self.buf.push_str(" derive[");
            for (i, d) in td.derives.iter().enumerate() {
                if i > 0 { self.buf.push_str(", "); }
                self.buf.push_str(match d {
                    DeriveTrait::Eq => "Eq",
                    DeriveTrait::Clone => "Clone",
                    DeriveTrait::Display => "Display",
                    DeriveTrait::Hash => "Hash",
                    DeriveTrait::Ord => "Ord",
                    DeriveTrait::Debug => "Debug",
                });
            }
            self.buf.push(']');
        }
    }

    fn format_enum_decl(&mut self, ed: &EnumDecl) {
        if ed.is_pub { self.buf.push_str("pub "); }
        self.buf.push_str("enum ");
        self.buf.push_str(&ed.name.name);
        if !ed.generics.is_empty() {
            self.buf.push('[');
            for (i, g) in ed.generics.iter().enumerate() {
                if i > 0 { self.buf.push_str(", "); }
                self.buf.push_str(&g.name.name);
                if !g.bounds.is_empty() {
                    self.buf.push_str(": ");
                    for (j, b) in g.bounds.iter().enumerate() {
                        if j > 0 { self.buf.push_str(" + "); }
                        self.buf.push_str(&b.name);
                    }
                }
            }
            self.buf.push(']');
        }
        self.buf.push_str(" {\n");
        self.indent += 1;
        for (i, variant) in ed.variants.iter().enumerate() {
            self.push_indent();
            self.buf.push_str(&variant.name.name);
            if !variant.fields.is_empty() {
                self.buf.push('(');
                for (j, f) in variant.fields.iter().enumerate() {
                    if j > 0 { self.buf.push_str(", "); }
                    self.buf.push_str(&f.name.name);
                    self.buf.push_str(": ");
                    self.format_type(&f.ty);
                }
                self.buf.push(')');
            }
            if i < ed.variants.len() - 1 { self.buf.push(','); }
            self.buf.push('\n');
        }
        self.indent -= 1;
        self.push_indent();
        self.buf.push('}');
        if !ed.derives.is_empty() {
            self.buf.push_str(" derive[");
            for (i, d) in ed.derives.iter().enumerate() {
                if i > 0 { self.buf.push_str(", "); }
                self.buf.push_str(match d {
                    DeriveTrait::Eq => "Eq",
                    DeriveTrait::Clone => "Clone",
                    DeriveTrait::Display => "Display",
                    DeriveTrait::Hash => "Hash",
                    DeriveTrait::Ord => "Ord",
                    DeriveTrait::Debug => "Debug",
                });
            }
            self.buf.push(']');
        }
    }

    fn format_interface_decl(&mut self, id: &InterfaceDecl) {
        if id.is_pub { self.buf.push_str("pub "); }
        self.buf.push_str("interface ");
        self.buf.push_str(&id.name.name);
        if !id.generics.is_empty() {
            self.buf.push('[');
            for (i, g) in id.generics.iter().enumerate() {
                if i > 0 { self.buf.push_str(", "); }
                self.buf.push_str(&g.name.name);
                if !g.bounds.is_empty() {
                    self.buf.push_str(": ");
                    for (j, b) in g.bounds.iter().enumerate() {
                        if j > 0 { self.buf.push_str(" + "); }
                        self.buf.push_str(&b.name);
                    }
                }
            }
            self.buf.push(']');
        }
        self.buf.push_str(" {\n");
        self.indent += 1;
        for member in &id.members {
            match member {
                InterfaceMember::Field(fd) => {
                    self.push_indent();
                    self.buf.push_str(&fd.name.name);
                    self.buf.push_str(": ");
                    self.format_type(&fd.ty);
                    self.buf.push_str(";\n");
                }
                InterfaceMember::FnSignature(fd) => {
                    self.format_fn_decl(fd);
                    self.buf.push_str(";\n");
                }
            }
        }
        self.indent -= 1;
        self.push_indent();
        self.buf.push_str("}\n");
    }

    fn format_module(&mut self, m: &ModuleDecl) {
        self.buf.push_str("module ");
        self.buf.push_str(&m.name.name);
        self.buf.push_str(" {\n");
        self.indent += 1;
        for item in &m.items {
            self.format_top_decl(item);
        }
        self.indent -= 1;
        self.push_indent();
        self.buf.push_str("}\n");
    }

    fn format_use_decl(&mut self, u: &UseDecl) {
        self.buf.push_str("use ");
        for (i, seg) in u.path.iter().enumerate() {
            if i > 0 { self.buf.push('.'); }
            self.buf.push_str(&seg.name);
        }
        if u.glob {
            self.buf.push_str(".*");
        }
        if let Some(ref alias) = u.alias {
            self.buf.push_str(" as ");
            self.buf.push_str(&alias.name);
        }
        self.buf.push_str(";\n");
    }

    fn format_const_decl(&mut self, c: &ConstDecl) {
        self.buf.push_str(if c.is_mut { "var " } else { "const " });
        self.buf.push_str(&c.name.name);
        self.buf.push_str(": ");
        self.format_type(&c.ty);
        self.buf.push_str(" = ");
        self.format_expr(&c.value);
        self.buf.push_str(";\n");
    }
}

pub(crate) fn format_float(f: f64) -> String {
    if f == f.floor() && f.is_finite() {
        format!("{}.0", f as u64)
    } else {
        f.to_string()
    }
}

/// Format a source file while PRESERVING the shebang line and the leading
/// comment/blank block (license headers). The AST pretty-printer used to
/// drop both, so `xiom fmt -i` was destructive on every headered file.
/// Comments INSIDE the program body are still dropped (trivia attachment to
/// AST nodes is the follow-on slice; the leading block is the damaging case).
pub fn format_source_text(source: &str) -> Result<String, String> {
    let (prefix, body) = split_leading_trivia(source);
    let mut lexer = xiom_lexer::Lexer::new(body);
    let tokens = lexer.tokenize();
    let mut parser = xiom_parser::Parser::new(tokens);
    let program = parser.parse_program().map_err(|e| format!("parse error: {e:?}"))?;
    let mut formatter = Formatter::new();
    let formatted = formatter.format(&program);
    if prefix.is_empty() {
        Ok(formatted)
    } else {
        Ok(format!("{prefix}{formatted}"))
    }
}

/// Split the shebang and the leading comment/blank block off `source`.
/// Returns `(prefix including its trailing newline, remaining source)`.
fn split_leading_trivia(source: &str) -> (String, &str) {
    let mut cut = 0usize;
    if source.starts_with("#!") {
        match source.find('\n') {
            Some(nl) => cut = nl + 1,
            None => return (source.to_string(), ""),
        }
    }
    let mut line_start = cut;
    let mut in_block = false;
    loop {
        if line_start >= source.len() {
            break;
        }
        let rest = &source[line_start..];
        let line_end = rest.find('\n').map(|i| line_start + i + 1).unwrap_or(source.len());
        let line = &source[line_start..line_end];
        let trimmed = line.trim_start();
        if in_block {
            cut = line_end;
            line_start = line_end;
            if line.contains("*/") {
                in_block = false;
            }
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            cut = line_end;
            line_start = line_end;
            continue;
        }
        if trimmed.starts_with("/*") {
            cut = line_end;
            line_start = line_end;
            if !trimmed.contains("*/") {
                in_block = true;
            }
            continue;
        }
        break;
    }
    (source[..cut].to_string(), &source[cut..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use xiom_lexer::Lexer;
    use xiom_parser::Parser;

    fn format_source(source: &str) -> String {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        let mut formatter = Formatter::new();
        formatter.format(&program)
    }

    #[test]
    fn test_format_empty() {
        let formatted = format_source("");
        assert_eq!(formatted, "\n");
    }

    #[test]
    fn test_format_simple_fn() {
        let src = "fn main()->Int{return 42;}";
        let formatted = format_source(src);
        assert!(formatted.contains("fn main() -> Int {"));
        assert!(formatted.contains("  return 42;"));
    }

    #[test]
    fn test_format_struct() {
        let src = "type Point={x:Float64;y:Float64;}derive[Eq,Clone]";
        let formatted = format_source(src);
        assert!(formatted.contains("type Point"));
        assert!(formatted.contains("x: Float64"));
        assert!(formatted.contains("derive[Eq, Clone]"));
    }

    #[test]
    fn test_idempotent() {
        let src = "fn main() -> Int {\n  return 42;\n}\n";
        let pass1 = format_source(src);
        let pass2 = format_source(&pass1);
        assert_eq!(pass1, pass2, "formatter must be idempotent");
    }

    #[test]
    fn test_format_demo_float() {
        let src = "fn add(a: Int, b: Int) -> Int {\n  return a + b;\n}\nfn sq(x: Float64) -> Float64 {\n  return x * x;\n}\nfn main() -> Int {\n  let f = sq(3.0);\n  return add(10, 20);\n}\n";
        let formatted = format_source(src);
        assert_eq!(formatted, src, "demo_float must be unchanged");
    }

    #[test]
    fn test_format_with_contracts() {
        let src = "fn divide(a: Float64, b: Float64) -> Float64\n  requires: b != 0.0\n  ensures: result * b == a\n{\n  return a / b;\n}\n";
        let formatted = format_source(src);
        assert!(formatted.contains("requires:"));
        assert!(formatted.contains("ensures:"));
        assert!(formatted.contains("fn divide"));
    }

    #[test]
    fn test_format_enum() {
        let src = "enum Option{Some(value:Int),None}derive[Eq,Clone]";
        let formatted = format_source(src);
        assert!(formatted.contains("enum Option {"));
        assert!(formatted.contains("Some("));
        assert!(formatted.contains("derive[Eq, Clone]"));
    }

    #[test]
    fn test_format_module() {
        let src = "module math{fn add(a:Int,b:Int)->Int{return a+b;}}";
        let formatted = format_source(src);
        assert!(formatted.contains("module math {"));
        assert!(formatted.contains("fn add("));
    }

    #[test]
    fn test_format_use() {
        let src = "use math.vector.Vec3 as V3;";
        let formatted = format_source(src);
        assert!(formatted.contains("use math.vector.Vec3 as V3;"));
    }

    #[test]
    fn test_format_const() {
        let src = "const MAX: Int = 100;";
        let formatted = format_source(src);
        assert!(formatted.contains("const MAX: Int = 100;"));
    }

    #[test]
    fn test_format_generic_fn() {
        let src = "fn max[T:Comparable](a:T,b:T)->T{if a>b{return a;}return b;}";
        let formatted = format_source(src);
        assert!(formatted.contains("fn max[T: Comparable]"));
    }

    #[test]
    fn test_format_method() {
        let src = "pub fn Vec3.dot(other: &Vec3) -> Float32 { return x * other.x + y * other.y; }";
        let formatted = format_source(src);
        assert!(formatted.contains("fn Vec3.dot("));
    }

    #[test]
    fn test_format_binary_expr() {
        let src = "fn main(){let x:Int=1+2*3;}";
        let formatted = format_source(src);
        assert!(formatted.contains("1 + 2 * 3"));
    }

    #[test]
    fn test_format_if_elif_else() {
        let src = "fn test(x: Int) -> Int { if x > 0 { return 1; } elif x < 0 { return -1; } else { return 0; } }";
        let formatted = format_source(src);
        assert!(formatted.contains("if "));
        assert!(formatted.contains("elif "));
        assert!(formatted.contains("else {"));
    }

    #[test]
    fn test_format_pub_fn() {
        let src = "pub fn add(a: Int, b: Int) -> Int { return a + b; }";
        let formatted = format_source(src);
        assert!(formatted.starts_with("pub fn add"));
    }

    #[test]
    fn test_format_async_fn() {
        let src = "async fn fetch(url: Str) -> Str;";
        let formatted = format_source(src);
        assert!(formatted.starts_with("async fn fetch("));
    }

    #[test]
    fn test_format_while() {
        let src = "fn main() { while x > 0 { x = x - 1; } }";
        let formatted = format_source(src);
        assert!(formatted.contains("while x > 0 {"));
    }

    #[test]
    fn test_format_for() {
        let src = "fn main() { for i in [0, 1, 2] { print(i); } }";
        let formatted = format_source(src);
        assert!(formatted.contains("for i in [0, 1, 2] {"));
    }

    // -- M3.4: AST round-trip tests --------------------------------------

    fn parse_program(source: &str) -> xiom_ast::Program {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse_program().expect("parse failed")
    }

    fn assert_round_trip(source: &str) {
        let prog1 = parse_program(source);
        let mut f = super::Formatter::new();
        let formatted = f.format(&prog1);
        let prog2 = parse_program(&formatted);
        assert_eq!(prog1.items.len(), prog2.items.len(),
            "round-trip should preserve item count: {} -> {} -> {}",
            source.len(), formatted.len(), prog2.items.len());
        for (a, b) in prog1.items.iter().zip(prog2.items.iter()) {
            assert_eq!(std::mem::discriminant(a), std::mem::discriminant(b),
                "item type changed after round-trip");
        }
    }

    #[test] fn test_rt_simple_fn() { assert_round_trip("fn add(a: Int, b: Int) -> Int { return a + b; }"); }
    #[test] fn test_rt_struct() { assert_round_trip("type Point = { x: Float64; y: Float64; } derive[Eq, Clone]"); }
    #[test] fn test_rt_enum() { assert_round_trip("enum Color { Red, Green, Blue }"); }
    #[test] fn test_rt_interface() { assert_round_trip("interface Comparable { fn compare(other: &Self) -> Int; }"); }
    #[test] fn test_rt_module() { assert_round_trip("module math { pub fn add(a: Int, b: Int) -> Int { return a + b; } }"); }
    #[test] fn test_rt_generic() { assert_round_trip("fn max[T: Comparable](a: T, b: T) -> T { if a > b { return a; } return b; }"); }
    #[test] fn test_rt_method() { assert_round_trip("pub fn Vec3.dot(other: &Vec3) -> Float32 { return x * other.x + y * other.y; }"); }
    #[test] fn test_rt_impl_trait() { assert_round_trip("fn get_display() -> impl Display { return 42; }"); }
    #[test] fn test_rt_const() { assert_round_trip("pub const MAX: Int = 1024;"); }
    #[test] fn test_rt_contracts() {
        assert_round_trip("fn divide(a: Float64, b: Float64) -> Float64\n  requires: b != 0.0\n  ensures: result * b == a\n{ return a / b; }");
    }

    // -- Edge cases ------------------------------------------------------

    #[test] fn test_rt_match_expr() {
        assert_round_trip("fn check(x: Option[Int]) -> Int { match x { Some(v) => v, None => 0, } }");
    }

    #[test] fn test_rt_for_loop() {
        assert_round_trip("fn main() { for i in [0, 1, 2] { print(i); } }");
    }

    #[test] fn test_rt_while_loop() {
        assert_round_trip("fn main() { var x = 5; while x > 0 { x = x - 1; } }");
    }

    #[test] fn test_rt_if_elif_else() {
        assert_round_trip("fn test(x: Int) -> Int { if x > 0 { return 1; } elif x < 0 { return -1; } else { return 0; } }");
    }

    #[test] fn test_rt_closure() {
        assert_round_trip("fn foo() { var d = fn(x: Int) -> Int { return x * 2; }; }");
    }

    #[test] fn test_rt_async_fn() {
        assert_round_trip("async fn fetch(url: Str) -> Str;");
    }

    #[test] fn test_rt_borrow_mut() {
        assert_round_trip("fn swap(a: &mut Int, b: &mut Int) { let tmp = a; a = b; b = tmp; }");
    }

    // M8: extern and unsafe blocks are now round-trippable
    #[test] fn test_rt_extern_block() {
        assert_round_trip("extern \"C\" { fn printf(format: *UInt8, ...) -> Int32; fn malloc(size: UInt) -> *UInt8; }");
    }
    #[test] fn test_rt_unsafe_block() {
        assert_round_trip("fn foo() { unsafe { let ptr = malloc(8); free(ptr); } }");
    }
    #[test] fn test_rt_unsafe_expr() {
        assert_round_trip("fn get_ptr() -> *UInt8 { unsafe { return malloc(16); } }");
    }
    // NOTE: extern blocks and unsafe blocks are now round-trippable
    // by the formatter. This is a known formatter limitation, not a parser bug.
    // Tracked as fmt/extern-unsafe-roundtrip.

    #[test] fn test_rt_if_let() {
        assert_round_trip("fn main() { if let Some(v) = maybe_val { use(v); } }");
    }

    #[test] fn test_rt_while_let() {
        assert_round_trip("fn main() { while let Some(v) = next() { process(v); } }");
    }

    #[test] fn test_rt_nested_module() {
        assert_round_trip("module outer { module inner { pub fn foo() -> Int { return 1; } } }");
    }

    #[test] fn test_rt_array_literal() {
        assert_round_trip("fn main() -> Int { var arr = [1, 2, 3]; return arr[0]; }");
    }

    #[test] fn test_rt_spawn() {
        assert_round_trip("fn main() { spawn { io.println(\"async\"); } }");
    }

    #[test] fn test_rt_error_propagation() {
        assert_round_trip("fn load(path: Str) -> Result[Config, AppError] { let file = io.read_file(path)?; return Ok(file); }");
    }

    // -- M21-1: Formatter edge cases -------------------------------------

    // Nested type definitions with 5+ levels of indentation
    #[test] fn test_rt_nested_type_deep() {
        let src = "type A = { b: B; } type B = { c: C; } type C = { d: D; } type D = { e: E; } type E = { f: Int; }";
        let formatted = format_source(src);
        assert!(formatted.contains("type A = {"));
        assert!(formatted.contains("type B = {"));
        assert!(formatted.contains("type C = {"));
        assert!(formatted.contains("type D = {"));
        assert!(formatted.contains("type E = {"));
    }

    // Long lines (200+ chars)
    #[test] fn test_format_long_function_signature() {
        let src = "fn very_long_function_name_with_many_params(a: Int, b: Int, c: Int, d: Int, e: Int, f: Int, g: Int, h: Int, i: Int, j: Int, k: Int, l: Int, m: Int, n: Int, o: Int) -> Int { return a + b + c + d + e + f + g + h + i + j + k + l + m + n + o; }";
        let formatted = format_source(src);
        assert!(formatted.contains("fn very_long_function_name_with_many_params("));
        assert!(formatted.len() > src.len(), "long line should be properly formatted");
    }

    #[test] fn test_format_long_return_type_chain() {
        let src = "fn deep() -> Option[Result[Option[Result[Option[Result[Int, Str]], Str]], Str]] { return None; }";
        let formatted = format_source(src);
        assert!(formatted.contains("fn deep()"));
    }

    // Comments in every position
    #[test] fn test_format_comment_after_expr() {
        let src = "fn main() -> Int {\n  return 42; // the answer\n}\n";
        let formatted = format_source(src);
        assert!(formatted.contains("return 42;"));
    }

    #[test] fn test_format_comment_between_params() {
        let src = "fn add(\n  a: Int, // first param\n  b: Int // second param\n) -> Int { return a + b; }";
        let formatted = format_source(src);
        assert!(formatted.contains("fn add("));
    }

    #[test] fn test_format_multiline_comment() {
        let src = "/* this is a multiline\n   comment block */\nfn main() -> Int { return 0; }";
        let formatted = format_source(src);
        assert!(formatted.contains("fn main()"));
    }

    // Impl blocks formatting
    #[test] fn test_format_impl_block() {
        let src = "impl Display for Point { fn fmt(p: Point) -> Str { return \"\"; } }";
        let formatted = format_source(src);
        assert!(!formatted.is_empty(), "impl block formatter should not crash");
    }

    // Interface blocks formatting
    #[test] fn test_format_interface_with_methods() {
        let src = "interface Comparable { fn compare(other: &Self) -> Int; fn equals(other: &Self) -> Bool; }";
        let formatted = format_source(src);
        assert!(formatted.contains("interface Comparable {"));
        assert!(formatted.contains("fn compare("));
        assert!(formatted.contains("fn equals("));
    }

    #[test] fn test_format_interface_derive() {
        let src = "interface Eq { fn eq(a: &Self, b: &Self) -> Bool; } derive[Ord]";
        let formatted = format_source(src);
        assert!(!formatted.is_empty(), "interface with derive should not crash formatter");
    }

    // Trailing commas
    #[test] fn test_format_trailing_comma_enum() {
        let src = "enum Color { Red, Green, Blue, }";
        let formatted = format_source(src);
        assert!(formatted.contains("Color {"));
    }

    #[test] fn test_format_trailing_comma_struct() {
        let src = "type Point = { x: Float64; y: Float64; }";
        let formatted = format_source(src);
        assert!(formatted.contains("Point"));
    }

    #[test] fn test_format_trailing_semicolons() {
        let src = "fn main() -> Int { return 0;;; }";
        let formatted = format_source(src);
        assert!(formatted.contains("fn main"));
    }

    // Empty blocks and files
    #[test] fn test_format_empty_fn_body() {
        let src = "fn nop() { }";
        let formatted = format_source(src);
        assert!(formatted.contains("fn nop()"));
    }

    #[test] fn test_format_empty_module() {
        let src = "module empty { }";
        let formatted = format_source(src);
        assert!(formatted.contains("module empty"));
    }

    #[test] fn test_format_empty_enum() {
        let src = "enum Void { }";
        let formatted = format_source(src);
        assert!(!formatted.is_empty(), "empty enum should not panic formatter");
    }

    // Shebang line preservation
    #[test] fn test_format_shebang() {
        let src = "#!/usr/bin/env xiom\nfn main() -> Int { return 42; }";
        let formatted = format_source_text(src).unwrap();
        assert!(formatted.starts_with("#!/usr/bin/env xiom\n"),
            "shebang must survive formatting: {formatted}");
        assert!(formatted.contains("fn main()"));
    }

    #[test] fn test_leading_license_comment_preserved() {
        let src = "// Copyright (c) 2026 Example\n// Licensed under MIT\n\nfn main() -> Int { return 0; }";
        let formatted = format_source_text(src).unwrap();
        assert!(formatted.starts_with("// Copyright (c) 2026 Example\n// Licensed under MIT"),
            "leading comment block must survive: {formatted}");
        assert!(formatted.contains("fn main()"));
    }

    #[test] fn test_block_header_comment_preserved() {
        let src = "/* header\n   block */\nfn main() -> Int { return 0; }";
        let formatted = format_source_text(src).unwrap();
        assert!(formatted.starts_with("/* header\n   block */"),
            "leading block comment must survive: {formatted}");
    }

    #[test] fn test_string_literal_escapes_round_trip() {
        let src = "fn main() -> Int { let s = \"quote \\\" back \\\\ nl \\n tab \\t\"; return 0; }";
        let formatted = format_source_text(src).unwrap();
        let tokens = xiom_lexer::Lexer::new(&formatted).tokenize();
        let value = tokens.iter().find_map(|t| match &t.kind {
            xiom_lexer::TokenKind::Str(s) => Some(s.clone()),
            _ => None,
        }).expect("string literal should re-lex");
        assert_eq!(value, "quote \" back \\ nl \n tab \t",
            "escaped literal must round-trip through format+re-lex: {formatted}");
    }

    #[test] fn test_char_literal_escapes_round_trip() {
        let src = "fn main() -> Int { let c = '\\''; let n = '\\n'; return 0; }";
        let formatted = format_source_text(src).unwrap();
        let tokens = xiom_lexer::Lexer::new(&formatted).tokenize();
        let chars: Vec<char> = tokens.iter().filter_map(|t| match &t.kind {
            xiom_lexer::TokenKind::Char(c) => Some(*c),
            _ => None,
        }).collect();
        assert_eq!(chars, vec!['\'', '\n'],
            "char literals must round-trip: {formatted}");
    }

    // Foreign/extern formatting
    #[test] fn test_format_extern_block_with_variadic() {
        let src = "extern \"C\" { fn printf(fmt: *UInt8, ...) -> Int32; fn malloc(size: Int) -> *UInt8; }";
        let formatted = format_source(src);
        assert!(formatted.contains("extern \"C\""));
        assert!(formatted.contains("printf("));
    }

    // Unsafe block formatting
    #[test] fn test_format_unsafe_block() {
        let src = "fn main() { unsafe { let ptr = malloc(8); free(ptr); } }";
        let formatted = format_source(src);
        assert!(formatted.contains("unsafe {"));
    }

    // Generics edge cases
    #[test] fn test_format_multi_generic_bounded() {
        let src = "fn complex[K: Eq + Hash, V: Clone + Display](map: BTreeMap[K, V]) -> Option[V] { return None; }";
        let formatted = format_source(src);
        assert!(formatted.contains("fn complex["));
        assert!(formatted.contains("Eq + Hash"));
    }

    #[test] fn test_format_generic_struct() {
        let src = "type Pair[T, U] = { first: T; second: U; } derive[Clone]";
        let formatted = format_source(src);
        assert!(formatted.contains("type Pair[T, U]"));
    }

    // Array and slice types
    #[test] fn test_format_array_type() {
        let src = "fn sum(items: [3]Int) -> Int { return items[0] + items[1] + items[2]; }";
        let formatted = format_source(src);
        assert!(formatted.contains("[3]Int"));
    }

    #[test] fn test_format_slice_type() {
        let src = "fn process(data: &Slice[Float64]) { }";
        let formatted = format_source(src);
        assert!(formatted.contains("Slice[Float64]"));
    }

    // Pointer types
    #[test] fn test_format_ptr_types() {
        let src = "fn alloc() -> *UInt8 { return null; } fn free_all(ptr: *UInt8) { } fn deref(ptr: *Int) -> Int { return ptr; }";
        let formatted = format_source(src);
        assert!(formatted.contains("*UInt8"));
        assert!(formatted.contains("*Int"));
    }

    // Option/Result types
    #[test] fn test_format_option_type() {
        let src = "fn div(a: Int, b: Int) -> Option[Int] { if b == 0 { return None; } return Some(a / b); }";
        let formatted = format_source(src);
        assert!(formatted.contains("Option[Int]"));
    }

    #[test] fn test_format_result_type() {
        let src = "fn parse_int(s: Str) -> Result[Int, Str] { return Ok(42); }";
        let formatted = format_source(src);
        assert!(formatted.contains("Result[Int, Str]"));
    }

    // Spawn expression
    #[test] fn test_format_spawn() {
        let src = "fn main() { spawn { io.println(\"bg\"); } }";
        let formatted = format_source(src);
        assert!(formatted.contains("spawn {"));
    }

    // `use` with multiple aliases
    #[test] fn test_format_use_multi_alias() {
        let src = "use math.vector.Vec3 as V3;\nuse math.vector.Vec4 as V4;";
        let formatted = format_source(src);
        assert!(formatted.contains("Vec3 as V3"));
        assert!(formatted.contains("Vec4 as V4"));
    }

    // Nested match with complex arms
    #[test] fn test_format_nested_match() {
        let src = "fn f(a: Option[Int], b: Option[Int]) -> Int { match a { Some(x) => match b { Some(y) => x + y, None => x, }, None => 0, } }";
        let formatted = format_source(src);
        assert!(formatted.contains("match a"));
        assert!(formatted.contains("match b"));
    }

    // Pattern match with guard
    #[test] fn test_format_match_guard() {
        let src = "fn abs(n: Int) -> Int { match n { 0 => 0, n => if n > 0 { n } else { -n }, } }";
        let formatted = format_source(src);
        assert!(formatted.contains("match n"));
    }

    // Contract formatting with multiple clauses
    #[test] fn test_format_multi_contract() {
        let src = "fn sqrt(x: Float64) -> Float64\n  requires: x >= 0.0\n  ensures: result >= 0.0\n  ensures: result * result >= x\n{ return 0.0; }";
        let formatted = format_source(src);
        assert!(formatted.contains("requires:"));
        assert!(formatted.contains("ensures:"));
    }

    // Boolean operators
    #[test] fn test_format_bool_ops() {
        let src = "fn complex(a: Bool, b: Bool, c: Bool) -> Bool { return a && b || c && !a; }";
        let formatted = format_source(src);
        assert!(formatted.contains("&&"));
        assert!(formatted.contains("||"));
    }

    // Comparison operators
    #[test] fn test_format_cmp_ops() {
        let src = "fn range(x: Int, lo: Int, hi: Int) -> Bool { return lo <= x && x <= hi; }";
        let formatted = format_source(src);
        assert!(formatted.contains("<="));
    }

    // Field access chaining
    #[test] fn test_format_field_chain() {
        let src = "fn f(p: Point) -> Float64 { return p.nested.inner.deep.value; }";
        let formatted = format_source(src);
        assert!(formatted.contains("p.nested.inner.deep.value"));
    }

    // Method chaining
    #[test] fn test_format_method_chain() {
        let src = "fn process() -> Int { return vec.iter().filter().map().sum(); }";
        let formatted = format_source(src);
        assert!(formatted.contains("vec.iter()"));
    }

    // idempotency: formatted output == doubly-formatted output
    #[test] fn test_format_idempotent_complex() {
        let src = "fn complex[T: Eq + Clone](input: T, count: Int) -> Option[Vec[T]]\n  requires: count > 0\n{\n  var result = Vec[T].new();\n  var i = 0;\n  while i < count {\n    result.push(input.clone());\n    i = i + 1;\n  }\n  return Some(result);\n}\n";
        let pass1 = format_source(src);
        let pass2 = format_source(&pass1);
        assert_eq!(pass1, pass2, "complex formatting must be idempotent");
    }
}
