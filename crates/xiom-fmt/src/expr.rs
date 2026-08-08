// XIOM — Canonical Formatter (expression formatting)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// M14.1: Extracted from lib.rs — expression formatting.

use xiom_ast::*;

impl crate::Formatter {
    pub(super) fn format_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int(n, _) => self.buf.push_str(&n.to_string()),
            Expr::BigInt(n, _) => self.buf.push_str(&n.to_string()),
            Expr::Float(f, _) => self.buf.push_str(&crate::format_float(*f)),
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
                    UnaryOp::BitNot => self.buf.push('~'),
                    UnaryOp::Deref => self.buf.push('*'),
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
                    BinOp::BitAnd => "&",
                    BinOp::BitOr => "|",
                    BinOp::BitXor => "^",
                    BinOp::Shl => "<<",
                    BinOp::Shr => ">>",
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
            Expr::GenericCall(func, types, args, _) => {
                self.format_expr(func);
                self.buf.push_str("::<");
                for (i, t) in types.iter().enumerate() {
                    if i > 0 { self.buf.push_str(", "); }
                    self.format_type(t);
                }
                self.buf.push_str(">(");
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
            Expr::Struct(name, fields, _spread, _) => {
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
            Expr::Unsafe(block, _) => {
                self.buf.push_str("unsafe {\n");
                self.indent += 1;
                self.format_block(block);
                self.indent -= 1;
                self.push_indent();
                self.buf.push('}');
            }
            Expr::Match(scrutinee, arms, _) => {
                self.buf.push_str("match ");
                self.format_expr(scrutinee);
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
                self.buf.push('}');
            }
            Expr::BlockExpr(block, _) => {
                self.buf.push_str("{\n");
                self.indent += 1;
                self.format_block(block);
                self.indent -= 1;
                self.push_indent();
                self.buf.push('}');
            }
            Expr::ConstBlock(inner, _) => {
                self.buf.push_str("const { ");
                self.format_expr(inner);
                self.buf.push_str(" }");
            }
            Expr::Error(_, _) => self.buf.push_str("<error>"),
        }
    }
}
