// XIOM — Canonical Formatter (statement formatting)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// M14.1: Extracted from lib.rs — statement and pattern formatting.

use xiom_ast::*;

impl crate::Formatter {
    pub(super) fn format_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(ident, ty, expr, _) => {
                self.push_indent();
                self.buf.push_str("let ");
                self.buf.push_str(&ident.name);
                if let Some(t) = ty {
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
                if let Some(t) = ty {
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
                if let Some(e) = expr {
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
                if let Some(else_b) = else_block {
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
            Stmt::While(cond, body, _, _, _) => {
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
            Stmt::For(ident, iter, body, _, _) => {
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
            Stmt::Spawn(body, _, is_move) => {
                self.push_indent();
                if *is_move {
                    self.buf.push_str("spawn move {\n");
                } else {
                    self.buf.push_str("spawn {\n");
                }
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
            Stmt::Break(..) => {
                self.push_indent();
                self.buf.push_str("break;\n");
            }
            Stmt::Continue(..) => {
                self.push_indent();
                self.buf.push_str("continue;\n");
            }
            Stmt::Asm(ab) => {
                self.push_indent();
                self.buf.push_str("asm(\"");
                self.buf.push_str(&ab.template);
                self.buf.push_str("\");\n");
            }
            Stmt::Defer(_, _) => todo!(),
            Stmt::Assert(cond, msg, _) => {
                self.buf.push_str("assert(");
                self.format_expr(cond);
                if let Some(m) = msg {
                    self.buf.push_str(", ");
                    self.format_expr(m);
                }
                self.buf.push_str(");\n");
            }
            Stmt::Debugger(_) => {
                self.buf.push_str("debugger;\n");
            }
        }
    }
}
