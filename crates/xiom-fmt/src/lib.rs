// XIOM — Canonical Formatter
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

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
        }
    }

    fn format_fn_decl(&mut self, f: &FnDecl) {
        if f.is_pub { self.buf.push_str("pub "); }
        if f.is_async { self.buf.push_str("async "); }
        self.buf.push_str("fn ");
        if let Some(ref receiver) = f.receiver {
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
        if let Some(ref ret) = f.return_type {
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
        if let Some(ref body) = f.body {
            self.buf.push_str(" {\n");
            self.indent += 1;
            self.format_block(body);
            self.indent -= 1;
            self.push_indent();
            self.buf.push('}');
        }
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

    fn format_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(ident, ty, expr, _) => {
                self.push_indent();
                self.buf.push_str("let ");
                self.buf.push_str(&ident.name);
                if let Some(ref t) = ty {
                    self.buf.push_str(": ");
                    self.format_type(t);
                }
                self.buf.push_str(" = ");
                self.format_expr(expr);
                self.buf.push_str(";\n");
            }
            Stmt::Var(ident, ty, expr, _) => {
                self.push_indent();
                self.buf.push_str("var ");
                self.buf.push_str(&ident.name);
                if let Some(ref t) = ty {
                    self.buf.push_str(": ");
                    self.format_type(t);
                }
                self.buf.push_str(" = ");
                self.format_expr(expr);
                self.buf.push_str(";\n");
            }
            Stmt::Assign(place, expr, _) => {
                self.push_indent();
                self.format_expr(place);
                self.buf.push_str(" = ");
                self.format_expr(expr);
                self.buf.push_str(";\n");
            }
            Stmt::Return(expr, _) => {
                self.push_indent();
                self.buf.push_str("return");
                if let Some(ref e) = expr {
                    self.buf.push(' ');
                    self.format_expr(e);
                }
                self.buf.push_str(";\n");
            }
            Stmt::Expr(expr, _) => {
                self.push_indent();
                self.format_expr(expr);
                self.buf.push_str(";\n");
            }
            Stmt::If(cond, then_block, elifs, else_block, _) => {
                self.push_indent();
                self.buf.push_str("if ");
                self.format_expr(cond);
                self.buf.push_str(" {\n");
                self.indent += 1;
                self.format_block(then_block);
                self.indent -= 1;
                self.push_indent();
                self.buf.push('}');
                for (econd, eblock) in elifs {
                    self.buf.push_str(" elif ");
                    self.format_expr(econd);
                    self.buf.push_str(" {\n");
                    self.indent += 1;
                    self.format_block(eblock);
                    self.indent -= 1;
                    self.push_indent();
                    self.buf.push('}');
                }
                if let Some(ref else_b) = else_block {
                    self.buf.push_str(" else {\n");
                    self.indent += 1;
                    self.format_block(else_b);
                    self.indent -= 1;
                    self.push_indent();
                    self.buf.push('}');
                }
                self.buf.push('\n');
            }
            Stmt::Match(expr, arms, _) => {
                self.push_indent();
                self.buf.push_str("match ");
                self.format_expr(expr);
                self.buf.push_str(" {\n");
                self.indent += 1;
                for arm in arms {
                    self.push_indent();
                    self.format_pattern(&arm.pattern);
                    self.buf.push_str(" => ");
                    match &arm.body {
                        MatchBody::Block(block) => {
                            self.buf.push_str("{\n");
                            self.indent += 1;
                            self.format_block(block);
                            self.indent -= 1;
                            self.push_indent();
                            self.buf.push_str("},\n");
                        }
                        MatchBody::Expr(e) => {
                            self.format_expr(e);
                            self.buf.push_str(",\n");
                        }
                    }
                }
                self.indent -= 1;
                self.push_indent();
                self.buf.push_str("}\n");
            }
            Stmt::While(cond, body, _) => {
                self.push_indent();
                self.buf.push_str("while ");
                self.format_expr(cond);
                self.buf.push_str(" {\n");
                self.indent += 1;
                self.format_block(body);
                self.indent -= 1;
                self.push_indent();
                self.buf.push_str("}\n");
            }
            Stmt::For(ident, iter, body, _) => {
                self.push_indent();
                self.buf.push_str("for ");
                self.buf.push_str(&ident.name);
                self.buf.push_str(" in ");
                self.format_expr(iter);
                self.buf.push_str(" {\n");
                self.indent += 1;
                self.format_block(body);
                self.indent -= 1;
                self.push_indent();
                self.buf.push_str("}\n");
            }
            Stmt::Spawn(body, _) => {
                self.push_indent();
                self.buf.push_str("spawn {\n");
                self.indent += 1;
                self.format_block(body);
                self.indent -= 1;
                self.push_indent();
                self.buf.push_str("}\n");
            }
            Stmt::Destructure(names, val, _) => {
                self.buf.push_str("var (");
                for (i, n) in names.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.buf.push_str(&n.name);
                }
                self.buf.push_str(") = ");
                self.format_expr(val);
                self.buf.push_str(";\n");
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
        }
    }

    fn format_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int(n, _) => self.buf.push_str(&n.to_string()),
            Expr::Float(f, _) => self.buf.push_str(&format_float(*f)),
            Expr::Bool(b, _) => self.buf.push_str(if *b { "true" } else { "false" }),
            Expr::Str(s, _) => {
                self.buf.push('"');
                self.buf.push_str(s);
                self.buf.push('"');
            }
            Expr::Char(c, _) => {
                self.buf.push('\'');
                self.buf.push(*c);
                self.buf.push('\'');
            }
            Expr::Ident(ident) => self.buf.push_str(&ident.name),
            Expr::Paren(inner, _) => {
                self.buf.push('(');
                self.format_expr(inner);
                self.buf.push(')');
            }
            Expr::Tuple(items, _) => {
                self.buf.push('(');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.format_expr(item);
                }
                self.buf.push(')');
            }
            Expr::Unary(op, inner, _) => {
                match op {
                    UnaryOp::Neg => self.buf.push('-'),
                    UnaryOp::Not => self.buf.push('!'),
                    UnaryOp::Ref => self.buf.push('&'),
                    UnaryOp::MutRef => self.buf.push_str("&mut "),
                }
                self.format_expr(inner);
            }
            Expr::Binary(left, op, right, _) => {
                self.format_expr(left);
                self.buf.push(' ');
                self.buf.push_str(match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Rem => "%",
                    BinOp::Eq => "==",
                    BinOp::Neq => "!=",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::Le => "<=",
                    BinOp::Ge => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                    BinOp::Assign => "=",
                });
                self.buf.push(' ');
                self.format_expr(right);
            }
            Expr::Call(func, args, _) => {
                self.format_expr(func);
                self.buf.push('(');
                for (i, a) in args.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.format_expr(a);
                }
                self.buf.push(')');
            }
            Expr::Field(obj, field, _) => {
                self.format_expr(obj);
                self.buf.push('.');
                self.buf.push_str(&field.name);
            }
            Expr::Index(obj, index, _) => {
                self.format_expr(obj);
                self.buf.push('[');
                self.format_expr(index);
                self.buf.push(']');
            }
            Expr::Struct(name, fields, _) => {
                self.buf.push_str(&name.name);
                self.buf.push_str("{ ");
                for (i, (ident, val)) in fields.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.buf.push_str(&ident.name);
                    self.buf.push_str(": ");
                    self.format_expr(val);
                }
                self.buf.push_str(" }");
            }
            Expr::Array(items, _) => {
                self.buf.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.format_expr(item);
                }
                self.buf.push(']');
            }
            Expr::Try(inner, _) => {
                self.format_expr(inner);
                self.buf.push('?');
            }
            Expr::Imply(left, right, _) => {
                self.format_expr(left);
                self.buf.push_str(" => ");
                self.format_expr(right);
            }
            Expr::Is(expr, pat, _) => {
                self.format_expr(expr);
                self.buf.push_str(" is ");
                self.format_pattern(pat);
            }
            Expr::AtPre(inner, _) => {
                self.format_expr(inner);
                self.buf.push_str("@pre");
            }
            Expr::Ref(inner, _) => {
                self.buf.push('&');
                self.format_expr(inner);
            }
            Expr::MutRef(inner, _) => {
                self.buf.push_str("&mut ");
                self.format_expr(inner);
            }
            Expr::Some(inner, _) => {
                self.buf.push_str("Some(");
                self.format_expr(inner);
                self.buf.push(')');
            }
            Expr::None(_) => self.buf.push_str("None"),
            Expr::Ok(inner, _) => {
                self.buf.push_str("Ok(");
                self.format_expr(inner);
                self.buf.push(')');
            }
            Expr::Err(inner, _) => {
                self.buf.push_str("Err(");
                self.format_expr(inner);
                self.buf.push(')');
            }
            Expr::Closure(params, ret_ty, body, _) => {
                self.buf.push_str("fn(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.buf.push_str(&p.name.name);
                    self.buf.push_str(": ");
                    self.format_type(&p.ty);
                }
                self.buf.push(')');
                if let Some(rt) = ret_ty {
                    self.buf.push_str(" -> ");
                    self.format_type(rt);
                }
                self.buf.push_str(" {\n");
                self.indent += 1;
                self.format_block(body);
                self.indent -= 1;
                self.push_indent();
                self.buf.push('}');
            }
            Expr::PipeClosure(params, body, _) => {
                self.buf.push('|');
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.buf.push_str(&p.name);
                }
                self.buf.push_str("| ");
                self.format_expr(body);
            }
            Expr::Await(inner, _) => {
                self.buf.push_str("await ");
                self.format_expr(inner);
            }
            Expr::Comptime(inner, _) => {
                self.buf.push_str("comptime ");
                self.format_expr(inner);
            }
            Expr::As(inner, ty, _) => {
                self.format_expr(inner);
                self.buf.push_str(" as ");
                self.format_type(ty);
            }
            Expr::If(cond, then_block, elifs, else_block, _) => {
                self.buf.push_str("if ");
                self.format_expr(cond);
                self.buf.push(' ');
                self.format_block(then_block);
                for (econd, eblock) in elifs {
                    self.buf.push_str(" elif ");
                    self.format_expr(econd);
                    self.buf.push(' ');
                    self.format_block(eblock);
                }
                if let Some(eblock) = else_block {
                    self.buf.push_str(" else ");
                    self.format_block(eblock);
                }
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
        self.buf.push_str("const ");
        self.buf.push_str(&c.name.name);
        self.buf.push_str(": ");
        self.format_type(&c.ty);
        self.buf.push_str(" = ");
        self.format_expr(&c.value);
        self.buf.push_str(";\n");
    }
}

fn format_float(f: f64) -> String {
    if f == f.floor() && f.is_finite() {
        format!("{}.0", f as u64)
    } else {
        f.to_string()
    }
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
}
