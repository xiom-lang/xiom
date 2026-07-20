// xiom-display — Shared type/expression display utilities
// Sprint 6B.2: Deduplicates type_to_string/format_fn_signature/op_to_str
// across xiom-lsp, xiom-doc, xiom-fmt, xiom-mcp, xiom-ffigen.

use xiom_ast::{BinOp, Pattern, Type, UnaryOp};

pub fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Named(ident, generics) => {
            if generics.is_empty() { ident.name.clone() }
            else { let g: Vec<String> = generics.iter().map(|t| type_to_string(t)).collect();
                   format!("{}[{}]", ident.name, g.join(", ")) }
        }
        Type::Ptr(inner) => format!("*{}", type_to_string(inner)),
        Type::Ref(inner) => format!("&{}", type_to_string(inner)),
        Type::MutRef(inner) => format!("&mut {}", type_to_string(inner)),
        Type::Array(_, inner) => format!("[N]{}", type_to_string(inner)),
        Type::Vec(inner) => format!("Vec[{}]", type_to_string(inner)),
        Type::Slice(inner) => format!("Slice[{}]", type_to_string(inner)),
        Type::Map(k, v) => format!("Map[{}, {}]", type_to_string(k), type_to_string(v)),
        Type::Set(inner) => format!("Set[{}]", type_to_string(inner)),
        Type::Result(ok, err) => format!("Result[{}, {}]", type_to_string(ok), type_to_string(err)),
        Type::Option(inner) => format!("Option[{}]", type_to_string(inner)),
        Type::Fn(params, ret) => {
            let p: Vec<String> = params.iter().map(|t| type_to_string(t)).collect();
            format!("fn({}) -> {}", p.join(", "), type_to_string(ret))
        }
        Type::Tuple(types) => {
            let inner: Vec<String> = types.iter().map(|t| type_to_string(t)).collect();
            format!("({})", inner.join(", "))
        }
    }
}

pub fn format_fn_signature(name: &str, params: &[(String, Type)], ret: &Option<Box<Type>>) -> String {
    let mut sig = format!("fn {name}(");
    let p: Vec<String> = params.iter().map(|(n, t)| format!("{n}: {}", type_to_string(t))).collect();
    sig.push_str(&p.join(", "));
    sig.push(')');
    if let Some(r) = ret { sig.push_str(&format!(" -> {}", type_to_string(r))); }
    sig
}

pub fn op_to_str(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+", BinOp::Sub => "-", BinOp::Mul => "*", BinOp::Div => "/",
        BinOp::Rem => "%", BinOp::Eq => "==", BinOp::Neq => "!=",
        BinOp::Lt => "<", BinOp::Gt => ">", BinOp::Le => "<=", BinOp::Ge => ">=",
        BinOp::And => "&&", BinOp::Or => "||", BinOp::Assign => "=",
        BinOp::BitAnd => "&", BinOp::BitOr => "|", BinOp::BitXor => "^",
        BinOp::Shl => "<<", BinOp::Shr => ">>",
    }
}

pub fn unary_op_to_str(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-", UnaryOp::Not => "!",
        UnaryOp::Ref => "&", UnaryOp::MutRef => "&mut ",
        UnaryOp::BitNot => "~", UnaryOp::Deref => "*",
    }
}

pub fn pattern_to_string(pat: &Pattern) -> String {
    match pat {
        Pattern::Wildcard(_) => "_".to_string(),
        Pattern::Ident(ident) => ident.name.clone(),
        Pattern::Lit(_) => "<lit>".to_string(),
        Pattern::Variant(name, fields, _) => {
            if fields.is_empty() { name.name.clone() }
            else { format!("{}({})", name.name, fields.iter().map(|i| i.name.clone()).collect::<Vec<_>>().join(", ")) }
        }
        Pattern::Some(inner, _) => format!("Some({})", pattern_to_string(inner)),
        Pattern::None(_) => "None".to_string(),
        Pattern::Ok(inner, _) => format!("Ok({})", pattern_to_string(inner)),
        Pattern::Err(inner, _) => format!("Err({})", pattern_to_string(inner)),
        _ => format!("<pat:{pat:?}>"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xiom_ast::{Ident, Span};

    fn id(name: &str) -> Ident { Ident { name: name.to_string(), span: Span::new(0, 0) } }

    #[test]
    fn test_type_builtins() {
        assert_eq!(type_to_string(&Type::Named(id("Int"), vec![])), "Int");
        assert_eq!(type_to_string(&Type::Named(id("Str"), vec![])), "Str");
    }

    #[test]
    fn test_type_option() {
        assert_eq!(type_to_string(&Type::Option(Box::new(Type::Named(id("Int"), vec![])))), "Option[Int]");
    }

    #[test]
    fn test_type_vec_generic() {
        assert_eq!(type_to_string(&Type::Named(id("Vec"), vec![Type::Named(id("Int"), vec![])])), "Vec[Int]");
    }

    #[test]
    fn test_op_to_str() {
        assert_eq!(op_to_str(&BinOp::Add), "+");
        assert_eq!(op_to_str(&BinOp::Eq), "==");
        assert_eq!(op_to_str(&BinOp::And), "&&");
    }

    #[test]
    fn test_fn_signature() {
        let sig = format_fn_signature("add",
            &[("x".into(), Type::Named(id("Int"), vec![])), ("y".into(), Type::Named(id("Int"), vec![]))],
            &Some(Box::new(Type::Named(id("Int"), vec![]))));
        assert_eq!(sig, "fn add(x: Int, y: Int) -> Int");
    }
}
